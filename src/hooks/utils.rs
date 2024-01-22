use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;

pub fn git_clone(repo_url: &str, target_dir: &Path) -> Result<()> {
    if target_dir.exists() {
        anyhow::bail!("Target directory '{}' already exists", target_dir.display());
    }

    Command::new("git")
        .arg("clone")
        .arg(repo_url)
        .arg(target_dir)
        .stdout(Stdio::null())
        .status()
        .with_context(|| format!("Failed to clone repository from {}", repo_url))?;

    Ok(())
}

pub fn git_pull(repo_path: &Path) -> Result<()> {
    if !repo_path.is_dir() {
        anyhow::bail!("{} is not a directory", repo_path.display());
    }

    let status = Command::new("git")
        .arg("pull")
        .current_dir(repo_path)
        .stdout(Stdio::null())
        .status()
        .with_context(|| format!("Failed to execute process in {}", repo_path.display()))?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("git pull failed in {}", repo_path.display());
    }
}

fn git_diff(target_dir: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(target_dir)
        .output()
        .with_context(|| format!("Failed to execute process in {}", target_dir.display()))?;
    let stdout = str::from_utf8(&output.stdout).expect("Not valid UTF-8");

    if stdout.is_empty() {
        Ok(false)
    } else {
        Ok(true)
    }
}

pub fn remove_repo(repo_path: &Path) -> Result<()> {
    if !repo_path.exists() {
        return Ok(());
    }

    match fs::metadata(repo_path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                fs::remove_dir_all(repo_path)?;
            } else {
                fs::remove_file(repo_path)?;
            }
        }
        Err(e) => anyhow::bail!("Error getting metadata for {}: {}", repo_path.display(), e),
    }

    Ok(())
}

pub fn update_repo(repo_url: &str, target_dir: &Path) -> Result<bool> {
    if let Ok(is_diff) = git_diff(target_dir) {
        if !is_diff {
            if git_pull(target_dir).is_err() {
                remove_repo(target_dir)?;
                git_clone(repo_url, target_dir)?;
            }

            return Ok(true);
        }
    }

    Ok(false)
}

pub fn hook_has_theme(
    theme_name: &str,
    template_name: &str,
    themes_dir_name: &str,
    app_data_path: &Path,
    theme_file_prefix: &str,
    theme_file_suffix: &str,
) -> Result<bool> {
    let local_repo_path = app_data_path.join(template_name);
    let themes_path = local_repo_path.join(themes_dir_name);
    let mut theme_exists = false;

    for entry in fs::read_dir(themes_path)? {
        let path = entry?.path();

        if path.is_file() {
            if let Some(full_name) = path.file_name().and_then(|n| n.to_str()) {
                let name = full_name
                    .strip_prefix(theme_file_prefix)
                    .and_then(|s| s.strip_suffix(theme_file_suffix))
                    .unwrap();
                if name == theme_name {
                    theme_exists = true;
                }
            }
        }
    }

    Ok(theme_exists)
}

pub fn setup_hook<'name>(
    name: &'name str,
    repo_url: &str,
    local_repo_path: &Path,
) -> Result<(&'name str, bool)> {
    let mut is_setup_success = false;

    if !local_repo_path.exists() {
        git_clone(repo_url, local_repo_path)?;
        is_setup_success = true;
    }

    Ok((name, is_setup_success))
}

pub fn update_hook<'name>(
    name: &'name str,
    repo_url: &str,
    local_repo_path: &Path,
) -> Result<(&'name str, bool)> {
    let mut is_update_success = false;

    if local_repo_path.exists() {
        is_update_success = update_repo(repo_url, local_repo_path)?;
    } else if git_clone(repo_url, local_repo_path).is_ok() {
        is_update_success = true;
    }

    Ok((name, is_update_success))
}
