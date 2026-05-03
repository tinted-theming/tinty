#![allow(clippy::module_name_repetitions)]

use crate::repo::RepositoryBackend;
use anyhow::{anyhow, bail, Context, Result};
use gix::bstr::ByteSlice;
use gix::refs::transaction::{Change, LogChange, PreviousValue, RefEdit, RefLog};
use gix::refs::Target;
use gix::remote::Direction;
use std::path::Path;
use std::sync::atomic::AtomicBool;

/// Repository backend implemented with the pure-Rust `gix` (gitoxide) library.
pub struct GixBackend;

impl RepositoryBackend for GixBackend {
    fn install(&self, url: &str, target: &Path, revision: Option<&str>) -> Result<()> {
        if target.exists() {
            return Err(anyhow!(
                "Error cloning {}. Target directory '{}' already exists",
                url,
                target.display()
            ));
        }

        let interrupt = AtomicBool::new(false);
        let mut prepare = gix::prepare_clone(url, target)
            .with_context(|| format!("Failed to set up clone of {url}"))?;

        // For tag- or branch-named revisions, point gix at the requested ref
        // up front so the clone fetches and checks it out in one step. SHA
        // revisions don't go through this path because they aren't a ref
        // name; they fall through to the post-fetch resolve-and-checkout.
        if let Some(rev) = revision {
            if !looks_like_sha(rev) {
                prepare = prepare
                    .with_ref_name(Some(rev))
                    .with_context(|| format!("Failed to target revision {rev} on remote {url}"))?;
            }
        }

        let fetch_result = prepare.fetch_then_checkout(gix::progress::Discard, &interrupt);
        let (mut checkout, _outcome) = match fetch_result {
            Ok(t) => t,
            Err(e) => {
                if let Some(rev) = revision {
                    if !looks_like_sha(rev) && format!("{e:#}").contains("didn't have any ref") {
                        bail!("cannot resolve {rev} into a Git SHA1");
                    }
                }
                return Err(e).with_context(|| format!("Failed to clone repository from {url}"));
            }
        };
        let (repo, _) = checkout
            .main_worktree(gix::progress::Discard, &interrupt)
            .with_context(|| format!("Failed to check out worktree at {}", target.display()))?;

        if let Some(rev) = revision {
            let post = if looks_like_sha(rev) {
                // SHA path: clone landed on the default branch; resolve and
                // detach onto the requested SHA.
                resolve_revision(&repo, "origin", rev)
                    .and_then(|resolved| checkout_revision(&repo, &resolved, rev))
            } else {
                // Tag/branch path: gix already checked out the ref. For
                // annotated tags we still need to detach HEAD onto the
                // peeled commit so `git rev-parse HEAD` returns the commit
                // SHA rather than the tag-object SHA.
                detach_if_tag(&repo)
            };
            if let Err(e) = post {
                let _ = std::fs::remove_dir_all(target);
                return Err(e);
            }
        }

        Ok(())
    }

    /// Update an installed repository to a (possibly new) URL and revision.
    ///
    /// Validate-first design:
    ///
    /// 1. ls-refs against the new URL via an in-memory anonymous remote.
    ///    This bails before any local mutation if the requested tag/branch
    ///    doesn't exist on the remote (and the revision isn't a SHA we can
    ///    verify post-fetch).
    /// 2. If the URL changed, write the new URL to `.git/config`. The
    ///    original URL is kept in memory for rollback.
    /// 3. Fetch from origin, resolve the revision against local refs, and
    ///    update HEAD + worktree.
    /// 4. On any failure during step 3, restore the original URL in
    ///    `.git/config` and propagate the error.
    fn update(&self, target: &Path, url: &str, revision: Option<&str>) -> Result<()> {
        if !target.is_dir() {
            return Err(anyhow!(
                "Error with updating. {} is not a directory",
                target.display()
            ));
        }

        let mut repo = gix::open(target)
            .with_context(|| format!("Failed to open git repository at {}", target.display()))?;
        let rev_str = revision.unwrap_or("main");

        // Phase 1: ls-refs against the new URL — no local mutation yet.
        let _kind_hint = probe_revision_kind_at(&repo, url, rev_str)?;

        // Phase 2: capture the current origin URL and switch to the new one
        // if it changed. The captured URL is used for rollback if the work
        // below fails.
        let old_url = read_origin_url(&repo)?;
        let url_changed = old_url.as_bstr() != url.as_bytes().as_bstr();
        if url_changed {
            write_origin_url(&repo, url)?;
            repo = gix::open(target).with_context(|| {
                format!(
                    "Failed to re-open repository after URL update at {}",
                    target.display()
                )
            })?;
        }

        // Phase 3: fetch + resolve + checkout. Encapsulated so we can run
        // rollback on any failure.
        let do_work = || -> Result<()> {
            fetch_origin(&repo)?;
            let resolved = resolve_revision(&repo, "origin", rev_str)?;
            checkout_revision(&repo, &resolved, rev_str)?;
            Ok(())
        };

        match do_work() {
            Ok(()) => Ok(()),
            Err(e) => {
                if url_changed {
                    let restored = std::str::from_utf8(old_url.as_ref())
                        .map(str::to_owned)
                        .ok();
                    if let Some(orig) = restored {
                        let _ = write_origin_url(&repo, &orig);
                    }
                }
                Err(e)
            }
        }
    }

    /// Reports whether the working directory is clean.
    ///
    /// Intentional divergence from the CLI backend: untracked files do **not**
    /// count as dirty here. The CLI backend uses `git status --porcelain` which
    /// reports untracked entries; this implementation excludes them so that
    /// artifacts written by `tinty generate-scheme` (and similar) into a cloned
    /// template repo don't block `tinty update` / `tinty sync`. See
    /// tinted-theming/tinty#130.
    ///
    /// Modified, deleted, renamed, or conflicted entries on tracked files all
    /// still count as dirty, matching the CLI backend.
    fn is_clean(&self, target: &Path) -> Result<bool> {
        let repo = gix::open(target)
            .with_context(|| format!("Failed to open git repository at {}", target.display()))?;
        let mut iter = repo
            .status(gix::progress::Discard)
            .with_context(|| format!("Failed to read status in {}", target.display()))?
            .untracked_files(gix::status::UntrackedFiles::None)
            .into_iter(None)
            .with_context(|| format!("Failed to iterate status in {}", target.display()))?;
        let any_change = iter.next().is_some();
        Ok(!any_change)
    }
}

#[derive(Clone, Copy)]
enum RevisionKind {
    Tag,
    Branch,
    Sha,
}

struct ResolvedRevision {
    /// The commit SHA that the revision resolves to. For annotated tags
    /// this is the *peeled* commit, not the tag object.
    commit: gix::ObjectId,
    kind: RevisionKind,
}

/// Probe a remote URL for whether it has the requested revision, without
/// touching any local state. Uses an anonymous in-memory remote and
/// `ref_map` (which performs an ls-refs handshake but doesn't fetch
/// objects). Returns the classified `RevisionKind` so callers can decide
/// whether to follow up with a fetch or jump straight to SHA verification.
fn probe_revision_kind_at(repo: &gix::Repository, url: &str, rev: &str) -> Result<RevisionKind> {
    // Anonymous remotes start with no refspecs; ref_map filters through
    // them, so without these the response is empty even if the remote has
    // matching refs. Configure the standard fetch refspecs explicitly.
    let probe = repo
        .remote_at(url)
        .with_context(|| format!("Failed to build in-memory remote at {url}"))?
        .with_refspecs(
            [
                "+refs/heads/*:refs/remotes/probe/*",
                "+refs/tags/*:refs/tags/*",
            ],
            Direction::Fetch,
        )
        .with_context(|| "Failed to set refspecs on probe remote")?;
    let connection = probe
        .connect(Direction::Fetch)
        .with_context(|| format!("Failed to connect to {url}"))?;
    let (ref_map, _handshake) = connection
        .ref_map(
            gix::progress::Discard,
            gix::remote::ref_map::Options::default(),
        )
        .with_context(|| format!("Failed to enumerate refs at {url}"))?;

    let tag_name = format!("refs/tags/{rev}");
    let head_name = format!("refs/heads/{rev}");
    let mut found_tag = false;
    let mut found_branch = false;
    for r in &ref_map.remote_refs {
        let (name, _target, _peeled) = r.unpack();
        if name.as_bstr() == tag_name.as_bytes() {
            found_tag = true;
        } else if name.as_bstr() == head_name.as_bytes() {
            found_branch = true;
        }
    }

    if found_tag {
        return Ok(RevisionKind::Tag);
    }
    if found_branch {
        return Ok(RevisionKind::Branch);
    }
    if !looks_like_sha(rev) {
        bail!("cannot resolve {rev} into a Git SHA1");
    }
    Ok(RevisionKind::Sha)
}

/// Read the fetch URL configured for `origin` in this repository.
fn read_origin_url(repo: &gix::Repository) -> Result<gix::bstr::BString> {
    let remote = repo
        .find_remote("origin")
        .with_context(|| "Failed to find origin remote")?;
    let url = remote
        .url(Direction::Fetch)
        .ok_or_else(|| anyhow!("origin has no fetch URL configured"))?;
    Ok(url.to_bstring())
}

/// Persist a new fetch URL for `origin` to `.git/config`. The caller is
/// expected to re-open the repository afterwards to pick up the change.
fn write_origin_url(repo: &gix::Repository, url: &str) -> Result<()> {
    let config_path = repo.git_dir().join("config");
    let mut config =
        gix::config::File::from_path_no_includes(config_path.clone(), gix::config::Source::Local)
            .with_context(|| format!("Failed to read {}", config_path.display()))?;
    config
        .set_raw_value("remote.origin.url", url.as_bytes())
        .with_context(|| format!("Failed to set remote.origin.url = {url}"))?;
    let mut file = std::fs::File::create(&config_path)
        .with_context(|| format!("Failed to open {} for writing", config_path.display()))?;
    config
        .write_to(&mut file)
        .with_context(|| format!("Failed to write to {}", config_path.display()))?;
    Ok(())
}

/// Fetch from the configured `origin` remote, updating `refs/remotes/origin/*`
/// and `refs/tags/*` per the existing refspecs in `.git/config`.
fn fetch_origin(repo: &gix::Repository) -> Result<()> {
    let interrupt = AtomicBool::new(false);
    let remote = repo
        .find_remote("origin")
        .with_context(|| "Failed to find origin remote")?;
    let connection = remote
        .connect(Direction::Fetch)
        .with_context(|| "Failed to connect to origin")?;
    let prepare = connection
        .prepare_fetch(
            gix::progress::Discard,
            gix::remote::ref_map::Options::default(),
        )
        .with_context(|| "Failed to prepare fetch from origin")?;
    prepare
        .receive(gix::progress::Discard, &interrupt)
        .with_context(|| "Failed to receive objects from origin")?;
    Ok(())
}

/// Resolve a revision string against an existing repository's local refs,
/// using the same precedence as the CLI backend: tag → branch → SHA. The
/// `remote_name` controls which `refs/remotes/<name>/*` namespace counts
/// for branch and SHA-reachability lookups.
fn resolve_revision(
    repo: &gix::Repository,
    remote_name: &str,
    rev: &str,
) -> Result<ResolvedRevision> {
    // Tag (peel through annotated tag objects to a commit).
    let tag_ref_name = format!("refs/tags/{rev}");
    if let Ok(mut reference) = repo.find_reference(tag_ref_name.as_str()) {
        let commit = reference
            .peel_to_id()
            .with_context(|| format!("Failed to peel {tag_ref_name}"))?
            .detach();
        return Ok(ResolvedRevision {
            commit,
            kind: RevisionKind::Tag,
        });
    }

    // Branch (remote-tracking ref).
    let branch_ref_name = format!("refs/remotes/{remote_name}/{rev}");
    if let Ok(mut reference) = repo.find_reference(branch_ref_name.as_str()) {
        let commit = reference
            .peel_to_id()
            .with_context(|| format!("Failed to peel {branch_ref_name}"))?
            .detach();
        return Ok(ResolvedRevision {
            commit,
            kind: RevisionKind::Branch,
        });
    }

    // SHA. Validate shape, then verify reachability from a remote-tracking
    // branch (matches the CLI backend's `git branch -a --contains <sha>`
    // semantic, filtered to the relevant remote prefix).
    if !looks_like_sha(rev) {
        bail!("cannot resolve {rev} into a Git SHA1");
    }
    let candidate = repo
        .rev_parse_single(rev)
        .map_err(|_| anyhow!("cannot find revision {rev} in remote {remote_name}"))?
        .detach();

    let prefix = format!("refs/remotes/{remote_name}/");
    let refs = repo
        .references()
        .with_context(|| "Failed to read references")?;
    let prefixed = refs
        .prefixed(prefix.as_str())
        .with_context(|| "Failed to filter references")?;
    for reference in prefixed {
        let Ok(mut reference) = reference else {
            continue;
        };
        let Ok(tip_id) = reference.peel_to_id() else {
            continue;
        };
        if let Ok(merge_base) = repo.merge_base(tip_id.detach(), candidate) {
            if merge_base.detach() == candidate {
                return Ok(ResolvedRevision {
                    commit: candidate,
                    kind: RevisionKind::Sha,
                });
            }
        }
    }

    bail!("cannot find revision {rev} in remote {remote_name}");
}

/// Apply a resolved revision: update HEAD (attached for branches, detached
/// for tags and SHAs) and reset the worktree to the resolved commit's tree.
fn checkout_revision(repo: &gix::Repository, resolved: &ResolvedRevision, rev: &str) -> Result<()> {
    match resolved.kind {
        RevisionKind::Branch => {
            // Create or reset the local branch to the resolved SHA, then
            // make HEAD point at the branch ref symbolically.
            let local_branch = format!("refs/heads/{rev}");
            let local_branch_name: gix::refs::FullName = local_branch
                .as_str()
                .try_into()
                .map_err(anyhow::Error::new)?;
            repo.edit_reference(RefEdit {
                change: Change::Update {
                    log: LogChange {
                        mode: RefLog::AndReference,
                        force_create_reflog: false,
                        message: format!("tinty: set {rev} to fetched tip").into(),
                    },
                    expected: PreviousValue::Any,
                    new: Target::Object(resolved.commit),
                },
                name: local_branch_name.clone(),
                deref: false,
            })
            .with_context(|| format!("Failed to update {local_branch}"))?;
            repo.edit_reference(RefEdit {
                change: Change::Update {
                    log: LogChange {
                        mode: RefLog::AndReference,
                        force_create_reflog: false,
                        message: format!("tinty: HEAD → {local_branch}").into(),
                    },
                    expected: PreviousValue::Any,
                    new: Target::Symbolic(local_branch_name),
                },
                name: "HEAD".try_into().map_err(anyhow::Error::new)?,
                deref: false,
            })
            .with_context(|| format!("Failed to attach HEAD to {local_branch}"))?;
        }
        RevisionKind::Tag | RevisionKind::Sha => {
            // Detach HEAD onto the (peeled) commit.
            repo.edit_reference(RefEdit {
                change: Change::Update {
                    log: LogChange {
                        mode: RefLog::AndReference,
                        force_create_reflog: false,
                        message: format!("tinty: detached HEAD onto {rev}").into(),
                    },
                    expected: PreviousValue::Any,
                    new: Target::Object(resolved.commit),
                },
                name: "HEAD".try_into().map_err(anyhow::Error::new)?,
                deref: false,
            })
            .with_context(|| format!("Failed to detach HEAD onto {rev}"))?;
        }
    }
    reset_worktree_to(repo, resolved.commit)
        .with_context(|| format!("Failed to check out worktree at {rev}"))?;
    Ok(())
}

/// After `with_ref_name(<tag>)` + clone, gix leaves HEAD symbolically attached
/// to the tag ref (`refs/tags/<tag>`). For annotated tags this means
/// `git rev-parse HEAD` resolves to the tag object's SHA rather than the
/// underlying commit's SHA, which differs from what `git checkout <tag>`
/// produces (detached HEAD at the commit). This helper detects that state
/// and detaches HEAD onto the peeled commit so behavior matches.
fn detach_if_tag(repo: &gix::Repository) -> Result<()> {
    let Ok(Some(mut reference)) = repo.head_ref() else {
        return Ok(());
    };
    if !reference.name().as_bstr().starts_with(b"refs/tags/") {
        return Ok(());
    }
    let name = reference.name().as_bstr().to_owned();
    let commit_id = reference
        .peel_to_id()
        .with_context(|| format!("Failed to peel {name} to a commit"))?
        .detach();
    repo.edit_reference(RefEdit {
        change: Change::Update {
            log: LogChange {
                mode: RefLog::AndReference,
                force_create_reflog: false,
                message: format!("tinty: detached HEAD onto {name}").into(),
            },
            expected: PreviousValue::Any,
            new: Target::Object(commit_id),
        },
        name: "HEAD".try_into().map_err(anyhow::Error::new)?,
        deref: false,
    })
    .with_context(|| format!("Failed to detach HEAD from {name}"))?;
    Ok(())
}

/// Returns true iff `rev` is shaped like a (possibly abbreviated) SHA-1.
/// Mirrors the regex used by the CLI backend: `^[0-9a-f]{1,40}$`.
fn looks_like_sha(rev: &str) -> bool {
    !rev.is_empty()
        && rev.len() <= 40
        && rev
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Materialize the worktree to match the tree of `commit_id`. Rebuilds the
/// index from that tree and runs gix's worktree-state checkout to write
/// every file.
fn reset_worktree_to(repo: &gix::Repository, commit_id: gix::ObjectId) -> Result<()> {
    let commit = repo
        .find_object(commit_id)
        .with_context(|| format!("Failed to find commit {commit_id}"))?
        .into_commit();
    let tree_id = commit
        .tree_id()
        .with_context(|| format!("Failed to read tree of commit {commit_id}"))?
        .detach();

    let mut index = repo
        .index_from_tree(&tree_id)
        .with_context(|| format!("Failed to build index from tree {tree_id}"))?;

    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow!("Repository has no working directory"))?
        .to_owned();

    let opts = gix::worktree::state::checkout::Options::default();
    gix::worktree::state::checkout(
        &mut index,
        &workdir,
        repo.objects.clone().into_arc()?,
        &gix::progress::Discard,
        &gix::progress::Discard,
        &AtomicBool::new(false),
        opts,
    )
    .with_context(|| "Failed to materialize worktree")?;

    index
        .write(gix::index::write::Options::default())
        .with_context(|| "Failed to write index")?;

    Ok(())
}
