use anyhow::{Context, Result};
use git2::Repository;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Ensures that a directory exists, creating it if it does not.
fn ensure_directory_exists<P: AsRef<Path>>(dir_path: P) -> Result<()> {
    let path = dir_path.as_ref();

    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory at {:?}", path))?;
    }

    Ok(())
}

/// Ensures that a file exists, creating it if it does not.
fn ensure_file_exists<P: AsRef<Path>>(file_path: P) -> Result<()> {
    let path = file_path.as_ref();

    if !path.exists() {
        File::create(path).with_context(|| format!("Failed to create file at {:?}", path))?;
    };

    Ok(())
}

/// Ensures the existence of configuration files required by the application.
pub fn ensure_config_files_exist(
    base16_config_path: &Path,
    data_path: &Path,
    base16_shell_theme_name_path: &Path,
) -> Result<()> {
    ensure_directory_exists(base16_config_path).with_context(|| {
        format!(
            "Failed to create config directory at {:?}",
            base16_config_path
        )
    })?;
    ensure_directory_exists(data_path)
        .with_context(|| format!("Failed to create config directory at {:?}", data_path))?;
    ensure_file_exists(base16_shell_theme_name_path).with_context(|| {
        format!(
            "Failed to create config file at {:?}",
            base16_shell_theme_name_path
        )
    })?;

    Ok(())
}

/// Reads the contents of a file and returns it as a string.
pub fn read_file_to_string(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn write_to_file(path: &Path, contents: &str) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }

    let mut file = File::create(path)?;
    file.write_all(contents.as_bytes())?;

    Ok(())
}

pub fn get_and_setup_repo_path(
    tintedtheming_data_path: &Path,
    base16_shell_repo_path_option: Option<PathBuf>,
    repo_url: &str,
) -> Result<PathBuf> {
    fn check_repo_path_exists(repo_path: &PathBuf) -> bool {
        if !repo_path.exists()
            || !repo_path.join("hooks").exists()
            || !repo_path.join("scripts").exists()
        {
            return false;
        }

        true
    }

    let repo_path: PathBuf = base16_shell_repo_path_option
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| tintedtheming_data_path.join("base16-shell"));

    if base16_shell_repo_path_option.is_none() && !check_repo_path_exists(&repo_path) {
        if let Err(e) = Repository::clone(repo_url, repo_path.clone()) {
            anyhow::bail!("Error cloning repo: {}", e);
        }
    }

    if !check_repo_path_exists(&repo_path) {
        anyhow::bail!(
            "Error with base16-shell repository at path: {}",
            repo_path.display()
        );
    }

    Ok(repo_path)
}
