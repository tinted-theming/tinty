use crate::config::{Config, ConfigItem, DEFAULT_CONFIG_SHELL};
use crate::constants::{REPO_NAME, SCHEME_EXTENSION};
use anyhow::{anyhow, Context, Result};
use home::home_dir;
use rand::Rng;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str;
use tinted_builder::SchemeSystem;

/// Ensures that a directory exists, creating it if it does not.
pub fn ensure_directory_exists<P: AsRef<Path>>(dir_path: P) -> Result<()> {
    let path = dir_path.as_ref();

    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory at {}", path.display()))?;
    }

    Ok(())
}

pub fn write_to_file(path: &Path, contents: &str) -> Result<()> {
    let mut file = File::create(path)
        .map_err(anyhow::Error::new)
        .with_context(|| format!("Unable to create file: {}", path.display()))?;

    file.write_all(contents.as_bytes())?;

    Ok(())
}

pub fn get_shell_command_from_string(config_path: &Path, command: &str) -> Result<Vec<String>> {
    let config = Config::read(config_path)?;
    let shell = config
        .shell
        .unwrap_or_else(|| DEFAULT_CONFIG_SHELL.to_string());
    let full_command = shell.replace("{}", command);

    shell_words::split(&full_command).map_err(anyhow::Error::new)
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

    let revision_str = revision.unwrap_or("main");
    return git_to_revision(target_dir, "origin", revision_str);
}

pub fn git_update(repo_path: &Path, repo_url: &str, revision: Option<&str>) -> Result<()> {
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

    let command = format!("git remote add \"{}\" \"{}\"", tmp_remote_name, repo_url);
    let command_vec = shell_words::split(&command).map_err(anyhow::Error::new)?;

    // Create a temporary remote
    Command::new(&command_vec[0])
        .args(&command_vec[1..])
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

    // Attempt to switch to the revision on temporary remote
    let revision_str = revision.unwrap_or("main");
    let res = git_to_revision(repo_path, &tmp_remote_name, revision_str);

    if let Err(e) = res {
        // Failed to switch to the desired revision. Cleanup!
        Command::new("git")
            .args(vec!["remote", "rm", &tmp_remote_name])
            .current_dir(repo_path)
            .stdout(Stdio::null())
            .status()
            .with_context(|| format!("Failed to execute process in {}", repo_path.display()))?;
        return Err(e);
    }

    let command = format!("git remote set-url origin \"{}\"", repo_url);
    let command_vec = shell_words::split(&command).map_err(anyhow::Error::new)?;

    // Success! Cleanup: update the origin remote to remote URL & delete temporary remote.
    Command::new(&command_vec[0])
        .args(&command_vec[1..])
        .current_dir(repo_path)
        .stdout(Stdio::null())
        .status()
        .with_context(|| {
            format!(
                "Failed to set origin remote to {} in {}",
                repo_url,
                repo_path.display()
            )
        })?;
    Command::new("git")
        .args(vec!["remote", "rm", &tmp_remote_name])
        .current_dir(repo_path)
        .stdout(Stdio::null())
        .status()
        .with_context(|| {
            format!(
                "Failed to remove temporary remote {} in {}",
                tmp_remote_name,
                repo_path.display()
            )
        })?;
    return Ok(());
}

fn random_remote_name() -> String {
    let mut rng = rand::thread_rng();
    let random_number: u32 = rng.gen();
    format!("tinty-remote-{}", random_number)
}

fn git_to_revision(repo_path: &Path, remote_name: &str, revision: &str) -> Result<()> {
    let command = format!("git fetch --quiet \"{}\" \"{}\"", remote_name, revision);
    let command_vec = shell_words::split(&command).map_err(anyhow::Error::new)?;

    Command::new(&command_vec[0])
        .args(&command_vec[1..])
        .current_dir(repo_path)
        .stdout(Stdio::null())
        .status()
        .with_context(|| {
            format!(
                "Error with fetching revision {} in {}",
                revision,
                repo_path.display()
            )
        })?;

    // Normalize the revision into the SHA. This way we can support all sorts of revisions, from
    // branches, tags, SHAs, etc.
    let command = format!("git rev-parse \"{}/{}\"", remote_name, revision);
    let command_vec = shell_words::split(&command).map_err(anyhow::Error::new)?;

    let parse_out = Command::new(&command_vec[0])
        .args(&command_vec[1..])
        .current_dir(repo_path)
        .output()
        .with_context(|| {
            format!(
                "Unable to parse revision {} in {}",
                revision,
                repo_path.display()
            )
        })?;

    let stdout = String::from_utf8_lossy(&parse_out.stdout);

    let commit_sha = match stdout.lines().next() {
        Some(sha) => sha,
        None => {
            return Err(anyhow!(
                "Unable to parse revision {} in {}",
                revision,
                repo_path.display()
            ))
        }
    };

    let command = format!(
        "git -c advice.detachedHead=false checkout --quiet \"{}\"",
        commit_sha
    );
    let command_vec = shell_words::split(&command).map_err(anyhow::Error::new)?;

    Command::new(&command_vec[0])
        .args(&command_vec[1..])
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

pub fn git_diff(target_dir: &Path) -> Result<bool> {
    let command = "git status --porcelain";
    let command_vec = shell_words::split(command).map_err(anyhow::Error::new)?;
    let output = Command::new(&command_vec[0])
        .args(&command_vec[1..])
        .current_dir(target_dir)
        .output()
        .with_context(|| format!("Failed to execute process in {}", target_dir.display()))?;
    let stdout = str::from_utf8(&output.stdout).expect("Not valid UTF-8");

    // If there is no output, then there is no diff
    if stdout.is_empty() {
        return Ok(false);
    }

    // Iterate over the lines and check for changes that should be considered a diff
    // Don't consider untracked files a diff
    let has_diff = stdout.lines().any(|line| {
        let status_code = &line[..2];
        // Status codes: M = modified, A = added, ?? = untracked
        status_code != "??"
    });

    Ok(has_diff)
}

pub fn create_theme_filename_without_extension(item: &ConfigItem) -> Result<String> {
    Ok(format!(
        "{}-{}-file",
        item.name.clone(),
        item.themes_dir.clone().replace('/', "-"), // Flatten path/to/dir to path-to-dir
    ))
}

pub fn get_all_scheme_names(
    schemes_path: &Path,
    scheme_systems_option: Option<SchemeSystem>,
) -> Result<Vec<String>> {
    if !schemes_path.exists() {
        return Err(anyhow!(
            "Schemes do not exist, run install and try again: `{} install`",
            REPO_NAME
        ));
    }

    // For each supported scheme system, add schemes to vec
    let mut scheme_vec: Vec<String> = Vec::new();
    let scheme_systems = scheme_systems_option
        .map(|s| vec![s])
        .unwrap_or(SchemeSystem::variants().to_vec());
    for scheme_system in scheme_systems {
        let scheme_system_dir = schemes_path.join(scheme_system.as_str());
        if !scheme_system_dir.exists() {
            continue;
        }

        for file in fs::read_dir(&scheme_system_dir)? {
            let file_path = file.as_ref().unwrap().path();
            let extension = file_path
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();

            if extension == SCHEME_EXTENSION {
                scheme_vec.push(format!(
                    "{}-{}",
                    scheme_system.as_str(),
                    file.unwrap()
                        .path()
                        .file_stem()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                ));
            }
        }
    }

    scheme_vec.sort();

    Ok(scheme_vec)
}
pub fn replace_tilde_slash_with_home(path_str: &str) -> Result<PathBuf> {
    let trimmed_path_str = path_str.trim();
    if trimmed_path_str.starts_with("~/") {
        match home_dir() {
            Some(home_dir) => Ok(PathBuf::from(trimmed_path_str.replacen(
                "~/",
                format!("{}/", home_dir.display()).as_str(),
                1,
            ))),
            None => Err(anyhow!("Unable to determine a home directory for \"{}\", please use an absolute path instead", trimmed_path_str))
        }
    } else {
        Ok(PathBuf::from(trimmed_path_str))
    }
}
