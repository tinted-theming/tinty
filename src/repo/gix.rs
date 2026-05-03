#![allow(clippy::module_name_repetitions)]

use crate::repo::RepositoryBackend;
use anyhow::{anyhow, bail, Context, Result};
use gix::refs::transaction::{Change, LogChange, PreviousValue, RefEdit, RefLog};
use gix::refs::Target;
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
        // up front so the clone fetches and checks it out in one step. This
        // also gets HEAD attached when the ref is a branch, matching what
        // `git clone --branch` does. SHA revisions don't go through this
        // path because they aren't a ref name; they fall through to the
        // post-fetch resolve-and-checkout below.
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
                // For non-SHA revisions, gix surfaces an error like
                // "The remote didn't have any ref that matched '<rev>'"
                // when neither a tag nor a branch matches. Normalize to the
                // CLI backend's wording so callers (and tests) see the same
                // message regardless of which backend ran.
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
                checkout_sha(&repo, rev)
            } else {
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
    /// Implementation note: rather than mutating the existing repository in
    /// place (which the CLI backend does via a temp-remote dance to keep
    /// `.git/config` consistent), we move the existing repo aside, run a
    /// fresh `install` into the target path, and only delete the moved-aside
    /// copy on success. On failure we restore the moved-aside copy. This
    /// gives atomic semantics via filesystem rename without ever touching
    /// the .git/config file mid-operation, and lets us share all the
    /// resolve / checkout logic with `install`.
    fn update(&self, target: &Path, url: &str, revision: Option<&str>) -> Result<()> {
        if !target.is_dir() {
            return Err(anyhow!(
                "Error with updating. {} is not a directory",
                target.display()
            ));
        }

        let backup = backup_path_for(target);
        if backup.exists() {
            std::fs::remove_dir_all(&backup).with_context(|| {
                format!("Failed to remove stale backup at {}", backup.display())
            })?;
        }
        std::fs::rename(target, &backup).with_context(|| {
            format!(
                "Failed to move {} aside to {}",
                target.display(),
                backup.display()
            )
        })?;

        match self.install(url, target, revision) {
            Ok(()) => {
                std::fs::remove_dir_all(&backup)
                    .with_context(|| format!("Failed to remove backup at {}", backup.display()))?;
                Ok(())
            }
            Err(e) => {
                // Clean up any partial install at the target path, then
                // restore the original repo from backup.
                if target.exists() {
                    let _ = std::fs::remove_dir_all(target);
                }
                std::fs::rename(&backup, target).with_context(|| {
                    format!(
                        "Failed to restore backup from {} to {} (original error: {e:#})",
                        backup.display(),
                        target.display()
                    )
                })?;
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

/// Picks a sibling backup directory next to `target` for use during `update`.
fn backup_path_for(target: &Path) -> std::path::PathBuf {
    let mut name = target.file_name().map_or_else(
        || std::ffi::OsString::from("repo"),
        std::ffi::OsString::from,
    );
    name.push(".tinty-update-bak");
    target.with_file_name(name)
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

/// After a default-branch clone, switch the worktree onto a specific commit
/// referenced by SHA. Verifies the SHA is reachable from a remote-tracking
/// branch (matches the CLI backend's containment check) and detaches HEAD.
fn checkout_sha(repo: &gix::Repository, rev: &str) -> Result<()> {
    // Resolve the (possibly abbreviated) SHA to a full ObjectId.
    let candidate = repo
        .rev_parse_single(rev)
        .map_err(|_| anyhow!("cannot find revision {rev} in remote origin"))?
        .detach();

    // Reachability check: the commit must be an ancestor of some
    // refs/remotes/origin/* tip. Matches the semantic of
    // `git branch -a --contains <sha>` filtered to that prefix in the CLI
    // backend.
    let prefix = "refs/remotes/origin/";
    let mut reachable = false;
    let refs = repo
        .references()
        .with_context(|| "Failed to read references")?;
    let prefixed = refs
        .prefixed(prefix)
        .with_context(|| "Failed to filter references")?;
    for reference in prefixed {
        let Ok(mut reference) = reference else {
            continue;
        };
        // Skip symbolic refs (e.g. `refs/remotes/origin/HEAD` → `…/main`)
        // by peeling; if peeling fails the ref is unusable for reachability.
        let Ok(tip_id) = reference.peel_to_id() else {
            continue;
        };
        let tip = tip_id.detach();
        if let Ok(merge_base) = repo.merge_base(tip, candidate) {
            if merge_base.detach() == candidate {
                reachable = true;
                break;
            }
        }
    }
    if !reachable {
        bail!("cannot find revision {rev} in remote origin");
    }

    // Detach HEAD onto the resolved commit.
    repo.edit_reference(RefEdit {
        change: Change::Update {
            log: LogChange {
                mode: RefLog::AndReference,
                force_create_reflog: false,
                message: format!("tinty: detached HEAD onto {rev}").into(),
            },
            expected: PreviousValue::Any,
            new: Target::Object(candidate),
        },
        name: "HEAD".try_into().map_err(anyhow::Error::new)?,
        deref: false,
    })
    .with_context(|| format!("Failed to detach HEAD onto {rev}"))?;

    // Re-materialize the worktree at the new HEAD. Use the index built from
    // the target tree, then write it through gix's worktree-state checkout.
    reset_worktree_to(repo, candidate)
        .with_context(|| format!("Failed to check out worktree at {rev}"))?;
    Ok(())
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

    // Rebuild the index from the target tree.
    let index = repo
        .index_from_tree(&tree_id)
        .with_context(|| format!("Failed to build index from tree {tree_id}"))?;
    let mut index = index;

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

    // Persist the new index so subsequent `git status` calls have a sane view.
    index
        .write(gix::index::write::Options::default())
        .with_context(|| "Failed to write index")?;

    Ok(())
}
