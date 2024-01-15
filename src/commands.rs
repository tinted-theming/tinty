use crate::config::{
    BASE16_SHELL_CONFIG_PATH_ENV, BASE16_SHELL_THEME_NAME_PATH_ENV, BASE16_THEME_ENV,
};
use crate::utils::{read_file_to_string, write_to_file};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;

fn set_colorscheme(
    theme_name: &str,
    base16_shell_repo_path: &Path,
    base16_shell_colorscheme_path: &Path,
    base16_shell_theme_name_path: &Path,
) -> Result<()> {
    // Read value from file
    let current_theme_name =
        read_file_to_string(base16_shell_theme_name_path).context("Failed to read from file")?;

    if theme_name == current_theme_name {
        println!("Theme \"{}\" is already set", theme_name);
        return Ok(());
    }

    let theme_script_path =
        base16_shell_repo_path.join(format!("scripts/base16-{}.sh", theme_name));
    if !theme_script_path.exists() {
        anyhow::bail!(
            "Theme \"{}\" does not exist at \"{}\", try a different theme",
            theme_name,
            theme_script_path.display()
        )
    }
    let theme_script_contents = read_file_to_string(
        &base16_shell_repo_path.join(format!("scripts/base16-{}.sh", theme_name)),
    )?;

    // Remove symlink file and create colorscheme symlink
    if base16_shell_colorscheme_path.exists() {
        fs::remove_file(base16_shell_colorscheme_path)?;
    }

    // Write shell theme script to file
    write_to_file(
        base16_shell_colorscheme_path,
        from_utf8(theme_script_contents.as_bytes())?,
    )
    .with_context(|| {
        format!(
            "Unable to write to file: {}",
            base16_shell_colorscheme_path.display()
        )
    })?;

    // Write theme name to file
    fs::write(base16_shell_theme_name_path, &theme_name)?;

    // Source colorscheme script
    // Wait for script to fully execute before continuing
    let mut child = Command::new("/bin/sh")
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

    println!("Theme set to: {}", theme_name);

    Ok(())
}

// Set env variables for hooks and then execute .sh hooks
fn run_hooks(
    theme_name: &str,
    base16_config_path: &Path,
    base16_shell_repo_path: &Path,
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

/// Sets the selected colorscheme and runs associated hook scripts.
///
/// This function orchestrates the process of setting a colorscheme based on the provided theme name.
/// It first sets the colorscheme using the `set_colorscheme` function and then runs the associated
/// hook scripts using the `run_hooks` function. These hook scripts apply the colorscheme to the current
/// environment.
///
/// # Arguments
///
/// * `theme_name` - A reference to a string containing the name of the theme to set.
/// * `base16_config_path` - A reference to a `Path` pointing to the base16 configuration directory.
/// * `base16_shell_repo_path` - A reference to a `Path` pointing to the base16-shell repository directory.
/// * `base16_shell_colorscheme_path` - A reference to a `Path` pointing to the file where the colorscheme script
///   should be written or linked.
/// * `base16_shell_theme_name_path` - A reference to a `Path` pointing to the file where the theme name should
///   be stored.
///
/// # Errors
///
/// Returns an error if the colorscheme cannot be set, which could occur if the theme script does not exist
/// in the specified path, or if there's an issue executing the hook scripts.
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # fn run_example() -> anyhow::Result<()> {
/// let theme_name = "default".to_string();
/// let base16_config_path = Path::new("/path/to/base16/config");
/// let base16_shell_repo_path = Path::new("/path/to/base16-shell/repo");
/// let base16_shell_colorscheme_path = Path::new("/path/to/base16/colorscheme/script");
/// let base16_shell_theme_name_path = Path::new("/path/to/base16/theme/name");
///
/// commands::set_command(
///     &theme_name,
///     &base16_config_path,
///     &base16_shell_repo_path,
///     &base16_shell_colorscheme_path,
///     &base16_shell_theme_name_path,
/// )?;
/// # Ok(())
/// # }
/// ```
///
/// The example demonstrates how to use this function to set a colorscheme and run associated hooks.
pub fn set_command(
    theme_name: &str,
    base16_config_path: &Path,
    base16_shell_repo_path: &Path,
    base16_shell_colorscheme_path: &Path,
    base16_shell_theme_name_path: &Path,
) -> Result<()> {
    set_colorscheme(
        theme_name,
        &base16_shell_repo_path,
        base16_shell_colorscheme_path,
        base16_shell_theme_name_path,
    )
    .with_context(|| format!("Failed to set colorscheme \"{:?}\"", theme_name))?;

    run_hooks(
        theme_name,
        base16_config_path,
        base16_shell_repo_path,
        base16_shell_theme_name_path,
    )
    .context("Failed to run hooks")?;

    Ok(())
}

/// Lists available color schemes in the base16 shell repository.
///
/// This function checks the provided base16 shell repository path to determine if it contains
/// color scheme scripts. It validates that the provided path is a directory, collects the names
/// of available color schemes by inspecting the scripts in the directory, and prints them.
///
/// # Arguments
///
/// * `base16_shell_repo_path` - A reference to a `Path` pointing to the base16 shell repository directory.
///
/// # Errors
///
/// Returns an error if the provided path does not exist, is not a directory, or if there's an issue
/// reading the directory or extracting color scheme names from the scripts.
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # fn run_example() -> anyhow::Result<()> {
/// let base16_shell_repo_path = Path::new("/path/to/base16-shell/repo");
///
/// commands::list_command(&base16_shell_repo_path)?;
/// # Ok(())
/// # }
/// ```
///
/// The example demonstrates how to use this function to list available color schemes in the base16 shell repository.
///
/// # Note
///
/// - The function prints the names of available color schemes to the console.
pub fn list_command(base16_shell_repo_path: &Path) -> Result<()> {
    // Check if a custom path to base16 shell repository is provided
    let scripts_path = base16_shell_repo_path.join("scripts");

    // Validate that the provided scripts path is a directory
    if !scripts_path.is_dir() {
        anyhow::bail!(
            "Scripts directory does not exist or is not a directory: {:?}",
            scripts_path
        );
    }

    // Collect color scheme names from the scripts directory
    let colorschemes: Vec<String> = fs::read_dir(&scripts_path)
        .with_context(|| format!("Failed to read directory: {:?}", &scripts_path))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            // Filter for files and extract the color scheme name
            if path.is_file() {
                return path
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .and_then(|name| name.strip_prefix("base16-"))
                    .map(|name| name.to_string());
            }

            None
        })
        .collect();

    // Print the found color schemes or a message if none are found
    if colorschemes.is_empty() {
        println!("No themes found in the scripts directory.");
    } else {
        for colorscheme in colorschemes {
            println!("{}", colorscheme);
        }
    }

    Ok(())
}
