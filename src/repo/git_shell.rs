#![allow(clippy::module_name_repetitions)]

use crate::repo::{RepositoryBackend, UpdateStatus};
use anyhow::{anyhow, Context, Error, Result};
use rand::Rng;
use regex::bytes::Regex;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

/// Repository backend implemented by shelling out to the user's `git` binary.
pub struct GitShellBackend;

impl RepositoryBackend for GitShellBackend {
    fn install(&self, url: &str, target: &Path, revision: Option<&str>) -> Result<()> {
        git_clone(url, target, revision)
    }

    fn update(
        &self,
        target: &Path,
        url: &str,
        revision: Option<&str>,
        allow_dirty: bool,
    ) -> Result<UpdateStatus> {
        git_update(target, url, revision, allow_dirty)
    }

    fn is_clean(&self, target: &Path) -> Result<bool> {
        git_is_working_dir_clean(target)
    }
}

fn git_clone(repo_url: &str, target_dir: &Path, revision: Option<&str>) -> Result<()> {
    if target_dir.exists() {
        return Err(anyhow!(
            "Error cloning {}. Target directory '{}' already exists",
            repo_url,
            target_dir.display()
        ));
    }

    let git_command = format!("git clone \"{repo_url}\" \"{}\"", target_dir.display());
    let command_vec = shell_words::split(git_command.as_str()).map_err(anyhow::Error::new)?;

    let Some(command) = command_vec.first() else {
        return Err(anyhow!("Unable to extract cli command"));
    };
    let Some(args) = command_vec.get(1..) else {
        return Err(anyhow!("Unable to extract cli args"));
    };

    Command::new(command)
        .args(args)
        .stdout(Stdio::null())
        .status()
        .with_context(|| format!("Failed to clone repository from {repo_url}"))?;

    if let Some(revision_str) = revision {
        // A freshly cloned tree is clean, so a conflict can never arise here;
        // `allow_dirty` is irrelevant and the returned status is ignored.
        let result = git_to_revision(target_dir, "origin", revision_str, false);
        if let Err(e) = result {
            // Cleanup! If we cannot checkout the revision, remove the directory.
            fs::remove_dir_all(target_dir)
                .with_context(|| format!("Failed to remove directory {}", target_dir.display()))?;
            return Err(e);
        }
    }

    Ok(())
}

fn git_update(
    repo_path: &Path,
    repo_url: &str,
    revision: Option<&str>,
    allow_dirty: bool,
) -> Result<UpdateStatus> {
    if !repo_path.is_dir() {
        return Err(anyhow!(
            "Error with updating. {} is not a directory",
            repo_path.display()
        ));
    }

    // To make this operation atomic, we'll satisfy the remote & revision in this sequence:
    // 1.) add the remote URL as a new temporary remote.
    // 2.) check if the revision exists in the temporary remote.
    // 3.) checkout the revision from temporary remote
    // 4.) On success:
    //      4.1) replace the origin remote URL
    //      4.2) remove the temporary remote
    // 5.) On error, remove temporary remote
    //
    // Note that this sequence works even if the directory is already on that remote & revision.
    //
    let tmp_remote_name = random_remote_name();

    // Create a temporary remote
    safe_command(
        format!("git remote add \"{tmp_remote_name}\" \"{repo_url}\"").as_str(),
        repo_path,
    )?
    .current_dir(repo_path)
    .stdout(Stdio::null())
    .status()
    .with_context(|| {
        format!(
            "Error with adding {} as a remote named {} in {}",
            repo_url,
            tmp_remote_name,
            repo_path.display()
        )
    })?;

    let revision_str = revision.unwrap_or("main");
    let res = git_to_revision(repo_path, &tmp_remote_name, revision_str, allow_dirty);

    let status = match res {
        Ok(status) => status,
        Err(e) => {
            // Failed to switch to the desired revision. Cleanup!
            safe_command(
                format!("git remote rm \"{tmp_remote_name}\"").as_str(),
                repo_path,
            )?
            .stdout(Stdio::null())
            .status()
            .with_context(|| {
                format!(
                    "Failed to remove temporary remote {} in {}",
                    tmp_remote_name,
                    repo_path.display()
                )
            })?;
            return Err(e);
        }
    };

    safe_command(
        format!("git remote set-url origin \"{repo_url}\"").as_str(),
        repo_path,
    )?
    .stdout(Stdio::null())
    .status()
    .with_context(|| {
        format!(
            "Failed to set origin remote to {repo_url} in {}",
            repo_path.display()
        )
    })?;
    safe_command(
        format!("git remote rm \"{tmp_remote_name}\"").as_str(),
        repo_path,
    )?
    .stdout(Stdio::null())
    .status()
    .with_context(|| {
        format!(
            "Failed to remove temporary remote {tmp_remote_name} in {}",
            repo_path.display()
        )
    })?;

    Ok(status)
}

fn random_remote_name() -> String {
    let mut rng = rand::thread_rng();
    let random_number: u32 = rng.gen();

    format!("tinty-remote-{random_number}")
}

enum RevisionType {
    Tag,
    Branch,
    Sha,
}

struct ResolvedRevision {
    sha: String,
    kind: RevisionType,
}

// Resolves the SHA1 of revision at remote_name.
// revision can be a tag, a branch, or a commit SHA1.
#[allow(clippy::too_many_lines)]
fn git_resolve_revision(
    repo_path: &Path,
    remote_name: &str,
    revision: &str,
) -> Result<ResolvedRevision> {
    // 1.) Check if its a tag.
    let expected_tag_ref = format!("refs/tags/{revision}");
    let mut command = safe_command(
        format!("git ls-remote --quiet --tags \"{remote_name}\" \"{expected_tag_ref}\"").as_str(),
        repo_path,
    )?;
    let mut child = command
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| "Failed to spawn".to_string())?;

    let Some(stdout) = child.stdout.take() else {
        return Err(anyhow!("failed to capture stdout"));
    };
    let reader = BufReader::new(stdout);

    if let Some(parts) = reader
        .lines()
        .map_while(Result::ok)
        .map(|line| line.split('\t').map(String::from).collect::<Vec<String>>())
        .filter(|parts| parts.len() == 2)
        .find(|parts| {
            parts
                .get(1)
                .map_or_else(|| false, |second_part| *second_part == expected_tag_ref)
        })
    {
        if let Some(first_part) = parts.first() {
            // we found a tag that matches
            child.kill()?; // Abort the child process.
            child.wait()?; // Cleanup

            return Ok(ResolvedRevision {
                sha: first_part.clone(),
                kind: RevisionType::Tag,
            });
        }
    }

    child
        .wait()
        .with_context(|| format!("Failed to list remote tags from {remote_name}"))?;

    // 2.) Check if its a branch
    let expected_branch_ref = format!("refs/heads/{revision}");
    let mut command = safe_command(
        format!("git ls-remote --quiet \"{remote_name}\" \"{expected_branch_ref}\"").as_str(),
        repo_path,
    )?;
    let mut child = command
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| "Failed to spawn".to_string())?;

    let Some(stdout) = child.stdout.take() else {
        return Err(anyhow!("failed to capture stdout"));
    };
    let reader = BufReader::new(stdout);

    if let Some(parts) = reader
        .lines()
        .map_while(Result::ok)
        .map(|line| line.split('\t').map(String::from).collect::<Vec<String>>())
        .filter(|parts| parts.len() == 2)
        .find(|parts| {
            parts
                .get(1)
                .map_or_else(|| false, |second_part| *second_part == expected_branch_ref)
        })
    {
        if let Some(first_part) = parts.first() {
            // we found a branch that matches.
            child.kill()?; // Abort the child process.
            child.wait()?; // Cleanup

            return Ok(ResolvedRevision {
                sha: first_part.clone(),
                kind: RevisionType::Branch,
            });
        }
    }

    child
        .wait()
        .with_context(|| format!("Failed to list branches tags from {remote_name}"))?;

    // We are here because revision isn't a tag or a branch.
    // First, we'll check if revision itself *could* be a SHA1.
    // If it doesn't look like one, we'll return early.
    let pattern = r"^[0-9a-f]{1,40}$";
    let Ok(re) = Regex::new(pattern) else {
        return Err(anyhow!("Invalid regex"));
    };
    if !re.is_match(revision.as_bytes()) {
        return Err(anyhow!("cannot resolve {revision} into a Git SHA1"));
    }

    safe_command(
        format!("git fetch --quiet \"{remote_name}\"").as_str(),
        repo_path,
    )?
    .stdout(Stdio::null())
    .status()
    .with_context(|| format!("unable to fetch objects from remote {remote_name}"))?;

    // 3.) Check if any branch in remote contains the SHA1:
    // It seems that the only way to do this is to list the branches that contain the SHA1
    // and check if it belongs in the remote.
    let remote_branch_prefix = format!("refs/remotes/{remote_name}/");
    let mut command = safe_command(
        format!("git branch --format=\"%(refname)\" -a --contains \"{revision}\"").as_str(),
        repo_path,
    )?;
    let mut child = command.stdout(Stdio::piped()).spawn().with_context(|| {
        format!("Failed to find branches containing commit {revision} from {remote_name}")
    })?;
    let Some(stdout) = child.stdout.take() else {
        return Err(anyhow!("failed to capture stdout"));
    };
    let reader = BufReader::new(stdout);

    if reader
        .lines()
        .map_while(Result::ok)
        .any(|line| line.starts_with(&remote_branch_prefix))
    {
        // we found a remote ref that contains the commit sha
        child.kill()?; // Abort the child process.
        child.wait()?; // Cleanup

        return Ok(ResolvedRevision {
            sha: revision.to_string(),
            kind: RevisionType::Sha,
        });
    }

    child.wait().with_context(|| {
        format!("Failed to list branches from {remote_name} containing SHA1 {revision}")
    })?;

    Err(anyhow!(
        "cannot find revision {revision} in remote {remote_name}",
    ))
}

fn safe_command(command_str: &str, cwd: &Path) -> Result<Command, Error> {
    let command_vec = shell_words::split(command_str).map_err(anyhow::Error::new)?;
    let Some(command) = command_vec.first() else {
        return Err(anyhow!("Unable to extract cli command"));
    };
    let Some(args) = command_vec.get(1..) else {
        return Err(anyhow!("Unable to extract cli args"));
    };
    let mut command = Command::new(command);

    command.args(args).current_dir(cwd);
    Ok(command)
}

fn git_to_revision(
    repo_path: &Path,
    remote_name: &str,
    revision: &str,
    allow_dirty: bool,
) -> Result<UpdateStatus> {
    // Download the object from the remote
    safe_command(
        format!("git fetch --quiet \"{remote_name}\" \"{revision}\"").as_str(),
        repo_path,
    )?
    .status()
    .with_context(|| {
        format!(
            "Error with fetching revision {revision} in {}",
            repo_path.display()
        )
    })?;

    // Normalize the revision into the SHA.
    let resolved = git_resolve_revision(repo_path, remote_name, revision)?;

    // Build the checkout. Branches are checked out by name (with `-B`) to keep
    // HEAD attached; tags and specific SHAs are expected to detach HEAD.
    let checkout_command = match resolved.kind {
        RevisionType::Branch => format!(
            "git checkout --quiet -B \"{revision}\" \"{}\"",
            resolved.sha
        ),
        RevisionType::Tag | RevisionType::Sha => format!(
            "git -c advice.detachedHead=false checkout --quiet \"{}\"",
            resolved.sha
        ),
    };

    // `git checkout` is atomic with respect to local changes: if the checkout
    // would overwrite uncommitted work it aborts without modifying anything,
    // and if it touches only untouched files it carries local edits forward.
    // We capture its output so a would-be-overwrite can be surfaced cleanly
    // instead of being treated as a hard error.
    let output = safe_command(checkout_command.as_str(), repo_path)?
        .stdout(Stdio::null())
        .output()
        .with_context(|| format!("Failed to checkout {revision} in {}", repo_path.display()))?;

    if output.status.success() {
        return Ok(UpdateStatus::Updated);
    }

    // Preserve git's own (possibly localized) message so we can show it to the
    // user verbatim.
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    // When the caller permits a dirty working tree, classify the failure with a
    // locale-independent plumbing probe instead of parsing the message text: if
    // switching to this revision would overwrite uncommitted or untracked work,
    // that is an expected, recoverable outcome and the working tree is left
    // untouched. Any other non-zero exit is a genuine error.
    //
    // Note the ordering: we run `checkout` first and probe with `read-tree`
    // only afterwards, rather than using `read-tree` as a pre-flight gate. The
    // probe could equally well run first, but `checkout` is what produces the
    // human-facing message we want to show the user — and it must run in their
    // locale to do so. `read-tree` is used purely as a yes/no oracle, never for
    // its output. Checkout's atomicity makes this safe: on a conflict it changes
    // nothing, so there is no half-applied state to undo before we probe.
    if allow_dirty && checkout_blocked_by_local_changes(repo_path, &resolved.sha)? {
        return Ok(UpdateStatus::ConflictPreserved { stderr });
    }

    Err(anyhow!(
        "Failed to checkout {revision} in {}:\n{stderr}",
        repo_path.display(),
    ))
}

/// Locale-independent classification of a failed checkout: returns `true` when
/// switching to `sha` would be refused because it would overwrite uncommitted
/// changes or untracked files. Uses `git read-tree`'s dry-run (`-n`), which
/// reports this through its exit code alone — no translatable text — and
/// modifies neither the index nor the working tree.
fn checkout_blocked_by_local_changes(repo_path: &Path, sha: &str) -> Result<bool> {
    let status = safe_command(
        format!("git read-tree -n -m -u \"{sha}\"").as_str(),
        repo_path,
    )?
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .with_context(|| {
        format!(
            "Failed to check working tree safety in {}",
            repo_path.display()
        )
    })?;

    Ok(!status.success())
}

fn git_is_working_dir_clean(target_dir: &Path) -> Result<bool> {
    // We use the Git plumbing `status --porcelain` command to tell us of files that has changed,
    // both staged and unstaged.
    let output = safe_command("git status --porcelain", target_dir)?
        .output()
        .with_context(|| format!("Failed to execute process in {}", target_dir.display()))?;

    // With the --quiet flag, it will return a 0 exit-code if no files has changed.
    Ok(output.stdout.is_empty())
}
