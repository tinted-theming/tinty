use crate::config::{
    get_embedded_colorscheme_dir, get_embedded_hooks_dir, BASE16_SHELL_CONFIG_PATH_ENV,
    BASE16_SHELL_THEME_NAME_PATH_ENV, BASE16_THEME_ENV,
};
use anyhow::{Context, Result};
use include_dir::Dir;
use std::fs::{self, File};
use std::io::Read;
use std::io::Write;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;

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

fn run_hooks_embedded(
    embedded_dir: &'static Dir<'static>,
    env_vars_to_set: Vec<(&str, &str)>,
) -> Result<()> {
    for dir_entry in embedded_dir.find("*.sh").unwrap() {
        let file = dir_entry.as_file().unwrap();
        let contents = file.contents_utf8().unwrap();
        let mut temp_file = NamedTempFile::new()?;
        let entry = dir_entry;
        let path = entry.path();

        write!(temp_file, "{}", contents)
            .map_err(anyhow::Error::new)
            .context("Unable to write to temporary colorscheme file")?;

        let mut command = Command::new("/bin/bash");

        command.arg(temp_file.path());

        // Set each environment variable for the script
        for (key, value) in &env_vars_to_set {
            command.env(key, value);
        }

        command
            .status()
            .with_context(|| format!("Failed to execute shell hook script: {:?}", path))?;
    }

    Ok(())
}

fn set_colorscheme(
    theme_name: &String,
    base16_shell_repo_path_option: &Option<PathBuf>,
    base16_shell_colorscheme_path: &Path,
    base16_shell_theme_name_path: &Path,
) -> Result<()> {
    // Read value from file
    let current_theme_name =
        read_file_to_string(base16_shell_theme_name_path).context("Failed to read from file")?;

    if theme_name.as_str() == current_theme_name {
        anyhow::bail!("Theme \"{}\" is already set", theme_name)
    }

    if let Some(base16_shell_repo_path) = base16_shell_repo_path_option {
        let theme_script_path =
            base16_shell_repo_path.join(format!("scripts/base16-{}.sh", theme_name));
        if !theme_script_path.exists() {
            anyhow::bail!(
                "Theme \"{}\" does not exist at \"{}\", try a different theme",
                theme_name,
                theme_script_path.display()
            )
        }

        // Remove symlink file and create colorscheme symlink
        if base16_shell_colorscheme_path.exists() {
            fs::remove_file(base16_shell_colorscheme_path)?;
        }

        let theme_script_path =
            base16_shell_repo_path.join(format!("scripts/base16-{}.sh", theme_name));
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
    } else {
        let embedded_colorscheme_dir = get_embedded_colorscheme_dir();

        let theme_script_file = embedded_colorscheme_dir
            .get_file(format!("base16-{}.sh", theme_name))
            .unwrap();
        let theme_script_contents = theme_script_file.contents();

        // Remove old colorscheme script file
        if base16_shell_colorscheme_path.exists() {
            fs::remove_file(base16_shell_colorscheme_path)?;
        }

        // Create new colorscheme script file
        fs::write(base16_shell_colorscheme_path, theme_script_contents)?;

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
    }

    Ok(())
}

fn run_hooks_provided(
    base16_shell_repo_path: &PathBuf,
    env_vars_to_set: Vec<(&str, &str)>,
) -> Result<()> {
    let base16_shell_hooks_path = base16_shell_repo_path.join("hooks");

    if !base16_shell_hooks_path.exists() {
        anyhow::bail!(
            "Provided hooks path does not exist: \"{}\"",
            base16_shell_hooks_path.display()
        )
    }

    for entry in fs::read_dir(base16_shell_hooks_path)? {
        let entry = entry?;

        let path = entry.path();

        // Check if the file name ends with .sh
        if path.extension().and_then(|ext| ext.to_str()) == Some("sh") {
            let mut command = Command::new("/bin/bash");

            command.arg(&path);

            // Set each environment variable for the script
            for (key, value) in &env_vars_to_set {
                command.env(key, value);
            }

            command
                .status()
                .with_context(|| format!("Failed to execute shell hook script: {:?}", path))?;
        }
    }

    Ok(())
}

// Set env variables for hooks and then execute .sh hooks
fn run_hooks(
    theme_name: &String,
    base16_config_path: &Path,
    base16_shell_repo_path_option: Option<PathBuf>,
    base16_shell_theme_name_path: &Path,
) -> Result<()> {
    let env_vars_to_set: Vec<(&str, &str)> = vec![
        (
            BASE16_SHELL_THEME_NAME_PATH_ENV,
            base16_shell_theme_name_path.to_str().unwrap(),
        ),
        (
            BASE16_SHELL_CONFIG_PATH_ENV,
            base16_config_path.to_str().unwrap(),
        ),
        (BASE16_THEME_ENV, theme_name),
    ];

    if let Some(base16_shell_repo_path) = base16_shell_repo_path_option {
        run_hooks_provided(&base16_shell_repo_path, env_vars_to_set).context(format!(
            "Error executing colorscheme scripts in dir: {}",
            base16_shell_repo_path.join("scripts").display()
        ))?;
    } else {
        let embedded_hooks_dir = get_embedded_hooks_dir();

        run_hooks_embedded(embedded_hooks_dir, env_vars_to_set).context(format!(
            "Error executing scripts in embedded dir: {}",
            embedded_hooks_dir.path().display()
        ))?;
    }

    Ok(())
}

pub fn set_command(
    theme_name: &String,
    base16_config_path: &PathBuf,
    base16_shell_repo_path_option: Option<PathBuf>,
    base16_shell_colorscheme_path: &PathBuf,
    base16_shell_theme_name_path: &PathBuf,
) -> Result<()> {
    set_colorscheme(
        theme_name,
        &base16_shell_repo_path_option,
        base16_shell_colorscheme_path,
        base16_shell_theme_name_path,
    )
    .with_context(|| format!("Failed to set colorscheme \"{:?}\"", theme_name))?;

    run_hooks(
        theme_name,
        base16_config_path,
        base16_shell_repo_path_option,
        base16_shell_theme_name_path,
    )
    .context("Failed to run hooks")?;

    Ok(())
}
