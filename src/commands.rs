use crate::config::{
    BASE16_SHELL_CONFIG_PATH_ENV, BASE16_SHELL_THEME_NAME_PATH_ENV, BASE16_THEME_ENV,
};
use anyhow::{Context, Result};
use include_dir::{include_dir, Dir};
use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::io::Write;
use std::os::unix::fs as unix_fs;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

static HOOKS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/hooks");

// Create a directory if it does not already exist
fn ensure_directory_exists<P: AsRef<Path>>(dir_path: P) -> Result<()> {
    let path = dir_path.as_ref();

    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory at {:?}", path))?;
    }

    Ok(())
}

// Create a file if it does not already exist
fn ensure_file_exists<P: AsRef<Path>>(file_path: P) -> Result<()> {
    let path = file_path.as_ref();

    if !path.exists() {
        File::create(path).with_context(|| format!("Failed to create file at {:?}", path))?;
    };

    Ok(())
}

// Create config files if they don't already exist
pub fn ensure_config_files_exist(
    base16_config_path: &Path,
    base16_shell_theme_name_path: &Path,
) -> Result<()> {
    ensure_directory_exists(base16_config_path).with_context(|| {
        format!(
            "Failed to create config directory at {:?}",
            base16_config_path
        )
    })?;
    ensure_file_exists(base16_shell_theme_name_path).with_context(|| {
        format!(
            "Failed to create config file at {:?}",
            base16_shell_theme_name_path
        )
    })?;

    Ok(())
}

// Convert file contents to string
fn read_file_to_string(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    Ok(contents)
}

fn set_colorscheme(
    theme_name: &String,
    base16_shell_path: &Path,
    base16_shell_colorscheme_path: &Path,
    base16_shell_theme_name_path: &Path,
) -> Result<()> {
    let theme_script_path = base16_shell_path.join(format!("scripts/base16-{}.sh", theme_name));
    if !theme_script_path.exists() {
        anyhow::bail!(
            "Theme \"{}\" does not exist, try a different theme",
            theme_name
        )
    }

    // Read value from file
    let current_theme_name =
        read_file_to_string(base16_shell_theme_name_path).context("Failed to read from file")?;

    if theme_name.as_str() == current_theme_name {
        anyhow::bail!("Theme \"{}\" is already set", theme_name)
    }

    // Remove symlink file and create colorscheme symlink
    if base16_shell_colorscheme_path.exists() {
        fs::remove_file(base16_shell_colorscheme_path)?;
    }

    unix_fs::symlink(theme_script_path, base16_shell_colorscheme_path)?;

    // Write theme name to file
    fs::write(base16_shell_theme_name_path, theme_name)?;

    // Source colorscheme script
    // Wait for script to fully execute before continuing
    let mut child = Command::new("/bin/bash")
        .arg(base16_shell_colorscheme_path)
        .spawn()
        .with_context(|| {
            format!(
                "Failed to execute script: {:?}",
                base16_shell_colorscheme_path
            )
        })?;
    let status = child.wait().context("Failed to wait on bash status")?;
    if !status.success() {
        anyhow::bail!("Command finished with a non-zero status: {}", status)
    }

    Ok(())
}

// Set env variables for hooks and then execute .sh hooks
fn run_hooks(
    theme_name: &String,
    base16_config_path: &Path,
    base16_shell_theme_name_path: &Path,
) -> Result<()> {
    env::set_var(
        BASE16_SHELL_THEME_NAME_PATH_ENV,
        base16_shell_theme_name_path,
    );
    env::set_var(BASE16_SHELL_CONFIG_PATH_ENV, base16_config_path);
    env::set_var(BASE16_THEME_ENV, theme_name);

    for dir_entry in HOOKS_DIR.find("*.sh").unwrap() {
        let file = dir_entry.as_file().unwrap();
        let contents = file.contents_utf8().unwrap();
        let mut temp_file = NamedTempFile::new()?;
        let entry = dir_entry;
        let path = entry.path();

        write!(temp_file, "{}", contents)
            .map_err(anyhow::Error::new)
            .context("Unable to write to temporary hook file")?;
        Command::new("/bin/bash")
            .arg(temp_file.path())
            .status()
            .with_context(|| format!("Failed to execute shell hook script: {:?}", path))
            .unwrap();
    }

    Ok(())
}

pub fn set_command(
    theme_name: &String,
    base16_config_path: &Path,
    base16_shell_path: &Path,
    base16_shell_colorscheme_path: &Path,
    base16_shell_theme_name_path: &Path,
) -> Result<()> {
    set_colorscheme(
        theme_name,
        base16_shell_path,
        base16_shell_colorscheme_path,
        base16_shell_theme_name_path,
    )
    .with_context(|| format!("Failed to set colorscheme \"{:?}\"", theme_name))?;

    env::set_var(BASE16_SHELL_THEME_NAME_PATH_ENV, &theme_name);

    run_hooks(theme_name, base16_config_path, base16_shell_theme_name_path)
        .context("Failed to run hooks")?;

    Ok(())
}
