use anyhow::{anyhow, Context, Error as AnyhowError, Result};
use regex::bytes::Regex;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str;

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

pub fn run_command(
    command_vec: Vec<String>,
    data_path: &Path,
    cache: bool,
) -> Result<(String, String), Box<dyn Error>> {
    if cache {
        clone_test_repos(data_path)?;
    }

    let output = Command::new(&command_vec[0])
        .args(&command_vec[1..])
        .output()
        .expect("Failed to execute command");

    if !output.stderr.is_empty() {
        println!(
            "tests::utils::run_command stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = strip_ansi_escapes::strip(String::from_utf8(output.stdout)?);
    let stderr = strip_ansi_escapes::strip(String::from_utf8(output.stderr)?);

    Ok((String::from_utf8(stdout)?, String::from_utf8(stderr)?))
}

#[allow(dead_code)]
pub fn run_install_command(config_path: &Path, data_path: &Path, cache: bool) -> Result<()> {
    if cache {
        clone_test_repos(data_path).context("Unable to clone tmp repos")?;
    }

    let output_install = Command::new(COMMAND_NAME)
        .args([
            "install",
            format!("--config={}", config_path.display()).as_str(),
            format!("--data-dir={}", data_path.display()).as_str(),
        ])
        .status()
        .expect("Failed to execute install command");

    if output_install.success() {
        Ok(())
    } else {
        Err(anyhow!("Install command stderr: {}", output_install))
    }
}

fn clone_test_repos(data_path: &Path) -> Result<()> {
    let tmp_repos_dir = Path::new("tmp/repos");

    // schemes
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
        let repo_path = data_path.join(format!("repos/{}", repo.0));
        let tmp_repo_path = tmp_repos_dir.join(format!("repos/{}", repo.0));

        if !tmp_repo_path.exists() {
            git_clone(repo.1, &tmp_repo_path, repo.2)
                .context("Unable to clone tinted-theming/tinted-vim.git")?;
        }

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

    if path.as_ref().parent().is_some() && !path.as_ref().parent().unwrap().exists() {
        fs::create_dir_all(path.as_ref().parent().unwrap())?;
    }

    let mut file = File::create(path)?;

    file.write_all(contents.as_bytes())?;

    Ok(())
}

#[allow(clippy::type_complexity)]
pub fn setup(
    name: &str,
    command: &str,
) -> Result<(
    PathBuf,
    PathBuf,
    Vec<String>,
    Box<dyn FnOnce() -> Result<()>>,
)> {
    let config_path = PathBuf::from(format!("config_path_{name}.toml").as_str());
    let data_path = PathBuf::from(format!("data_path_{name}").as_str());

    let command_vec = build_comamnd_vec(command, &config_path, &data_path)?;

    cleanup(&config_path, &data_path)?;
    write_to_file(&config_path, "")?;

    let config_path_clone = config_path.clone();
    let data_path_clone = data_path.clone();

    Ok((
        config_path,
        data_path,
        command_vec,
        Box::new(move || cleanup(&config_path_clone, &data_path_clone)),
    ))
}

#[allow(clippy::type_complexity)]
pub fn build_comamnd_vec(
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

    let command = format!("git clone \"{}\" \"{}\"", repo_url, target_dir.display());
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;

    Command::new(&command_vec[0])
        .args(&command_vec[1..])
        .stdout(Stdio::null())
        .status()
        .with_context(|| format!("Failed to clone repository from {}", repo_url))?;

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
fn git_resolve_revision(repo_path: &Path, remote_name: &str, revision: &str) -> Result<String> {
    // 1.) Check if its a tag.
    let expected_tag_ref = format!("refs/tags/{}", revision);
    let mut command = safe_command(
        format!(
            "git ls-remote --quiet --tags \"{}\" \"{}\"",
            remote_name, expected_tag_ref
        ),
        repo_path,
    )?;
    let mut child = command
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| "Failed to spawn".to_string())?;

    let stdout = child.stdout.take().expect("failed to capture stdout");
    let reader = BufReader::new(stdout);

    if let Some(parts) = reader
        .lines()
        .map_while(Result::ok)
        .map(|line| line.split("\t").map(String::from).collect::<Vec<String>>())
        .filter(|parts| parts.len() == 2)
        .find(|parts| parts[1] == expected_tag_ref)
    {
        // we found a tag that matches
        child.kill()?; // Abort the child process.
        child.wait()?; // Cleanup
        return Ok(parts[0].to_string()); // Return early.
    }

    child
        .wait()
        .with_context(|| format!("Failed to list remote tags from {}", remote_name))?;

    // 2.) Check if its a branch
    let expected_branch_ref = format!("refs/heads/{}", revision);
    let mut command = safe_command(
        format!(
            "git ls-remote --quiet \"{}\" \"{}\"",
            remote_name, expected_branch_ref
        ),
        repo_path,
    )?;
    let mut child = command
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| "Failed to spawn".to_string())?;

    let stdout = child.stdout.take().expect("failed to capture stdout");
    let reader = BufReader::new(stdout);

    if let Some(parts) = reader
        .lines()
        .map_while(Result::ok)
        .map(|line| line.split("\t").map(String::from).collect::<Vec<String>>())
        .filter(|parts| parts.len() == 2)
        .find(|parts| parts[1] == expected_branch_ref)
    {
        // we found a branch that matches.
        child.kill()?; // Abort the child process.
        child.wait()?; // Cleanup
        return Ok(parts[0].to_string()); // Return early.
    }

    child
        .wait()
        .with_context(|| format!("Failed to list branches tags from {}", remote_name))?;

    // We are here because revision isn't a tag or a branch.
    // First, we'll check if revision itself *could* be a SHA1.
    // If it doesn't look like one, we'll return early.
    let pattern = r"^[0-9a-f]{1,40}$";
    let re = Regex::new(pattern).expect("Invalid regex");
    if !re.is_match(revision.as_bytes()) {
        return Err(anyhow!("cannot resolve {} into a Git SHA1", revision));
    }

    safe_command(format!("git fetch --quiet \"{}\"", remote_name), repo_path)?
        .stdout(Stdio::null())
        .status()
        .with_context(|| format!("unable to fetch objects from remote {}", remote_name))?;

    // 3.) Check if any branch in remote contains the SHA1:
    // It seems that the only way to do this is to list the branches that contain the SHA1
    // and check if it belongs in the remote.
    let remote_branch_prefix = format!("refs/remotes/{}/", remote_name);
    let mut command = safe_command(
        format!(
            "git branch --format=\"%(refname)\" -a --contains \"{}\"",
            revision
        ),
        repo_path,
    )?;
    let mut child = command.stdout(Stdio::piped()).spawn().with_context(|| {
        format!(
            "Failed to find branches containing commit {} from {}",
            revision, remote_name
        )
    })?;

    let stdout = child.stdout.take().expect("failed to capture stdout");
    let reader = BufReader::new(stdout);
    if reader
        .lines()
        .map_while(Result::ok)
        .any(|line| line.clone().starts_with(&remote_branch_prefix))
    {
        // we found a remote ref that contains the commit sha
        child.kill()?; // Abort the child process.
        child.wait()?; // Cleanup
        return Ok(revision.to_string()); // Return early.
    }

    child.wait().with_context(|| {
        format!(
            "Failed to list branches from {} containing SHA1 {}",
            remote_name, revision
        )
    })?;

    Err(anyhow!(
        "cannot find revision {} in remote {}",
        revision,
        remote_name
    ))
}

fn safe_command(command: String, cwd: &Path) -> Result<Command, AnyhowError> {
    let command_vec = shell_words::split(&command).map_err(anyhow::Error::new)?;
    let mut command = Command::new(&command_vec[0]);
    command.args(&command_vec[1..]).current_dir(cwd);
    Ok(command)
}

fn git_to_revision(repo_path: &Path, remote_name: &str, revision: &str) -> Result<()> {
    // Download the object from the remote
    safe_command(
        format!("git fetch --quiet \"{}\" \"{}\"", remote_name, revision),
        repo_path,
    )?
    .status()
    .with_context(|| {
        format!(
            "Error with fetching revision {} in {}",
            revision,
            repo_path.display()
        )
    })?;

    // Normalize the revision into the SHA.
    let commit_sha = git_resolve_revision(repo_path, remote_name, revision)?;

    safe_command(
        format!(
            "git -c advice.detachedHead=false checkout --quiet \"{}\"",
            commit_sha
        ),
        repo_path,
    )?
    .stdout(Stdio::null())
    .current_dir(repo_path)
    .status()
    .with_context(|| {
        format!(
            "Failed to checkout SHA {} in {}",
            commit_sha,
            repo_path.display()
        )
    })?;

    Ok(())
}
