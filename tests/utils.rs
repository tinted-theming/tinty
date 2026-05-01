//! Shared test utilities for tinty integration tests.
//!
//! # Network dependency & caching
//! Many tests require cloning Git repositories from GitHub. To avoid redundant
//! clones, `clone_test_repos()` maintains a shared cache in a system temp
//! directory protected by a file lock. On the first run, repos are cloned once;
//! subsequent runs copy from the cache. Tests that pass `cache: true` to
//! `run_command()` trigger this caching automatically.
//!
//! # Test isolation
//! Each test gets its own `tempfile::TempDir` containing a config file and data
//! directory. The `setup()` function creates these and returns the `TempDir`
//! guard — cleanup happens automatically when the guard is dropped.
//!
//! # Command timeout
//! All commands executed via `run_command()` and `run_install_command()` are
//! subject to a 5-minute timeout to prevent the test suite from hanging.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use anyhow::{anyhow, ensure, Context, Error, Result};
use fs2::FileExt;
use regex::bytes::Regex;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str;
use std::time::{Duration, Instant};

#[allow(dead_code)]
pub const REPO_NAME: &str = env!("CARGO_PKG_NAME");
#[allow(dead_code)]
pub const ORG_NAME: &str = "tinted-theming";
pub const COMMAND_NAME: &str = env!("CARGO_BIN_EXE_tinty");
#[allow(dead_code)]
pub const CURRENT_SCHEME_FILE_NAME: &str = "current_scheme";
#[allow(dead_code)]
pub const REPO_DIR: &str = "repos";
#[allow(dead_code)]
pub const SCHEMES_REPO_NAME: &str = "schemes";
#[allow(dead_code)]
pub const CUSTOM_SCHEMES_DIR_NAME: &str = "custom-schemes";
#[allow(dead_code)]
pub const ARTIFACTS_DIR: &str = "artifacts";

const COMMAND_TIMEOUT: Duration = Duration::from_secs(300);

fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Result<std::process::ExitStatus> {
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if start.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(anyhow!("Command timed out after {timeout:?}"));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

pub fn run_command(
    command_vec: &[String],
    data_path: &Path,
    cache: bool,
) -> Result<(String, String)> {
    run_command_with_env(command_vec, data_path, cache, &[])
}

#[allow(dead_code)]
pub fn run_command_with_env(
    command_vec: &[String],
    data_path: &Path,
    cache: bool,
    env_vars: &[(&str, &str)],
) -> Result<(String, String)> {
    if cache {
        clone_test_repos(data_path)?;
    }

    let (command, args) = command_vec
        .split_first()
        .ok_or_else(|| anyhow!("command_vec is empty"))?;
    let mut cmd = Command::new(command);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    let mut child = cmd.spawn().map_err(|e| anyhow::anyhow!("{e}"))?;

    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();

    let stdout_handle = std::thread::spawn(move || -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        if let Some(mut pipe) = stdout_pipe {
            pipe.read_to_end(&mut buf)?;
        }
        Ok(buf)
    });

    let stderr_handle = std::thread::spawn(move || -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        if let Some(mut pipe) = stderr_pipe {
            pipe.read_to_end(&mut buf)?;
        }
        Ok(buf)
    });

    wait_with_timeout(&mut child, COMMAND_TIMEOUT)?;

    let stdout_raw = stdout_handle
        .join()
        .map_err(|_| anyhow!("stdout reader thread panicked"))??;
    let stderr_raw = stderr_handle
        .join()
        .map_err(|_| anyhow!("stderr reader thread panicked"))??;

    if !stderr_raw.is_empty() {
        println!(
            "tests::utils::run_command stderr: {}",
            String::from_utf8_lossy(&stderr_raw)
        );
    }

    let stdout = strip_ansi_escapes::strip(String::from_utf8(stdout_raw)?);
    let stderr = strip_ansi_escapes::strip(String::from_utf8(stderr_raw)?);

    Ok((String::from_utf8(stdout)?, String::from_utf8(stderr)?))
}

#[allow(dead_code)]
pub fn run_install_command(config_path: &Path, data_path: &Path, cache: bool) -> Result<()> {
    if cache {
        clone_test_repos(data_path).context("Unable to clone tmp repos")?;
    }

    let mut child = Command::new(COMMAND_NAME)
        .args([
            "install",
            format!("--config={}", config_path.display()).as_str(),
            format!("--data-dir={}", data_path.display()).as_str(),
        ])
        .spawn()
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let status = wait_with_timeout(&mut child, COMMAND_TIMEOUT)?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("Install command failed with status: {status}"))
    }
}

fn clone_test_repos(data_path: &Path) -> Result<()> {
    let tmp_repos_dir = std::env::temp_dir().join("tinty-test-repos");
    fs::create_dir_all(&tmp_repos_dir)?;

    // Use a file lock to prevent concurrent clones to the shared cache
    let lock_path = tmp_repos_dir.join(".lock");
    let lock_file = File::create(&lock_path)?;
    lock_file
        .lock_exclusive()
        .context("Failed to acquire lock on tmp/repos")?;

    for repo in [
        (
            "schemes",
            "https://github.com/tinted-theming/schemes.git",
            Some("spec-0.11"),
        ),
        (
            "tinted-shell",
            "https://github.com/tinted-theming/tinted-shell.git",
            None,
        ),
        (
            "tinted-vim",
            "https://github.com/tinted-theming/tinted-vim.git",
            None,
        ),
    ] {
        let tmp_repo_path = tmp_repos_dir.join(repo.0);

        if !tmp_repo_path.exists() {
            clone_with_retry(repo.1, &tmp_repo_path, repo.2, 2)
                .context(format!("Unable to clone {}", repo.1))?;
        }
    }

    // Release the lock before copying (copies go to test-specific paths)
    lock_file
        .unlock()
        .context("Failed to release lock on tmp/repos")?;

    for repo_name in ["schemes", "tinted-shell", "tinted-vim"] {
        let repo_path = data_path.join(format!("repos/{repo_name}"));
        let tmp_repo_path = tmp_repos_dir.join(repo_name);

        if repo_path.exists() {
            fs::remove_dir_all(&repo_path)
                .context(format!("Unable to remove {}", repo_path.display()))?;
        }

        fs::create_dir_all(&repo_path)
            .context(format!("Unable to create dir {}", &repo_path.display()))?;
        copy_dir_all(&tmp_repo_path, &repo_path).context(format!(
            "Unable to copy {} to {}",
            tmp_repo_path.display(),
            repo_path.display()
        ))?;
    }

    Ok(())
}

#[allow(dead_code)]
pub fn cleanup(config_path: impl AsRef<Path>, data_path: impl AsRef<Path>) -> Result<()> {
    if config_path.as_ref().is_file() {
        fs::remove_file(config_path)?;
    }

    if data_path.as_ref().is_dir() {
        fs::remove_dir_all(data_path)?;
    }

    Ok(())
}

pub fn write_to_file(path: impl AsRef<Path>, contents: &str) -> Result<()> {
    if path.as_ref().exists() {
        fs::remove_file(&path)?;
    }

    if let Some(parent) = path.as_ref().parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut file = File::create(path)?;

    file.write_all(contents.as_bytes())?;

    Ok(())
}

pub fn setup(
    name: &str,
    command: &str,
) -> Result<(PathBuf, PathBuf, Vec<String>, tempfile::TempDir)> {
    let temp_dir = tempfile::Builder::new()
        .prefix(&format!("tinty-test-{name}-"))
        .tempdir()?;

    let config_path = temp_dir.path().join("config.toml");
    let data_path = temp_dir.path().join("data");

    let command_vec = build_command_vec(command, &config_path, &data_path)?;
    write_to_file(&config_path, "")?;

    Ok((config_path, data_path, command_vec, temp_dir))
}

#[allow(clippy::type_complexity)]
pub fn build_command_vec(
    command: &str,
    config_path: &Path,
    data_path: &Path,
) -> Result<Vec<String>> {
    let command = format!(
        "{} --config=\"{}\" --data-dir=\"{}\" {}",
        COMMAND_NAME,
        config_path.display(),
        data_path.display(),
        command
    );
    shell_words::split(command.as_str()).map_err(anyhow::Error::new)
}

#[allow(dead_code)]
pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.as_ref().join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

pub fn git_clone(repo_url: &str, target_dir: &Path, revision: Option<&str>) -> Result<()> {
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
        let result = git_to_revision(target_dir, "origin", revision_str);
        if let Err(e) = result {
            // Cleanup! If we cannot checkout the revision, remove the directory.
            fs::remove_dir_all(target_dir)
                .with_context(|| format!("Failed to remove directory {}", target_dir.display()))?;
            return Err(e);
        }
    }

    Ok(())
}

// Resolvees the SHA1 of revision at remote_name.
// revision can be a tag, a branch, or a commit SHA1.
#[allow(clippy::too_many_lines)]
fn git_resolve_revision(repo_path: &Path, remote_name: &str, revision: &str) -> Result<String> {
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
            return Ok(first_part.clone()); // Return early.
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
            return Ok(first_part.clone()); // Return early.
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
        return Ok(revision.to_string()); // Return early.
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

fn clone_with_retry(
    repo_url: &str,
    target_dir: &Path,
    revision: Option<&str>,
    max_retries: u32,
) -> Result<()> {
    let mut last_error = None;
    for attempt in 0..=max_retries {
        if attempt > 0 {
            #[allow(clippy::arithmetic_side_effects)]
            {
                eprintln!(
                    "Retrying clone of {repo_url} (attempt {}/{})",
                    attempt + 1,
                    max_retries + 1
                );
            }
            if target_dir.exists() {
                let _ = fs::remove_dir_all(target_dir);
            }
            std::thread::sleep(Duration::from_secs(u64::from(attempt) * 2));
        }
        match git_clone(repo_url, target_dir, revision) {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_error = Some(e);
            }
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow!("Clone failed after {max_retries} retries")))
}

fn git_to_revision(repo_path: &Path, remote_name: &str, revision: &str) -> Result<()> {
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
    let commit_sha = git_resolve_revision(repo_path, remote_name, revision)?;

    safe_command(
        format!("git -c advice.detachedHead=false checkout --quiet \"{commit_sha}\"").as_str(),
        repo_path,
    )?
    .stdout(Stdio::null())
    .current_dir(repo_path)
    .status()
    .with_context(|| {
        format!(
            "Failed to checkout SHA {commit_sha} in {}",
            repo_path.display()
        )
    })?;

    Ok(())
}

#[allow(dead_code)]
pub fn test_install_with_revision(
    test_name: &str,
    repo_url: &str,
    repo_name: &str,
    themes_dir: &str,
    revision: &str,
    expected_sha: &str,
) -> Result<()> {
    let (config_path, data_path, command_vec, _temp_dir) = setup(test_name, "install")?;
    let config_content = format!(
        r#"[[items]]
path = "{repo_url}"
name = "{repo_name}"
themes-dir = "{themes_dir}"
revision = "{revision}"
"#
    );
    write_to_file(&config_path, &config_content)?;

    let (_, _) = run_command(&command_vec, &data_path, false)?;

    let repo_path = data_path.join(format!("repos/{repo_name}"));
    let output = Command::new("git")
        .current_dir(&repo_path)
        .args(["rev-parse", "--verify", "HEAD"])
        .output()
        .map_err(|e| anyhow!("Failed to execute git rev-parse: {e}"))?;
    let stdout = String::from_utf8(output.stdout)?;

    let has_match = stdout.lines().any(|line| line == expected_sha);
    ensure!(
        has_match,
        "Expected revision {expected_sha} not found in HEAD, got: {stdout}"
    );

    Ok(())
}
