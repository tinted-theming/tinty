use crate::config::{
    BASE16_SHELL_CONFIG_PATH_ENV, BASE16_SHELL_THEME_NAME_PATH_ENV, BASE16_THEME_ENV, HOOKS_DIR,
    REPO_NAME, REPO_URL, SETUP_SCRIPT_PATH, THEMES_DIR,
};
use crate::utils::{read_file_to_string, write_to_file};
use anyhow::{Context, Result};
use git2::Repository;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;

fn set_colorscheme(
    theme_name: &str,
    repo_path: &Path,
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

    let theme_script_path = repo_path.join(format!("{}/base16-{}.sh", THEMES_DIR, theme_name));
    if !theme_script_path.exists() {
        anyhow::bail!(
            "Theme \"{}\" does not exist at \"{}\", try a different theme",
            theme_name,
            theme_script_path.display()
        )
    }
    let theme_script_contents =
        read_file_to_string(&repo_path.join(format!("{}/base16-{}.sh", THEMES_DIR, theme_name)))?;

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
    fs::write(base16_shell_theme_name_path, theme_name)?;

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

// Set env variables for hooks and then executes .sh hook scripts
fn run_hooks(
    theme_name: &str,
    base16_config_path: &Path,
    repo_path: &Path,
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

    let base16_shell_hooks_path = repo_path.join(HOOKS_DIR);

    if !base16_shell_hooks_path.exists() {
        anyhow::bail!(
            "Provided hooks path does not exist: \"{}\"",
            base16_shell_hooks_path.display()
        )
    }

    for entry in fs::read_dir(base16_shell_hooks_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) == Some("sh") {
            let mut command = Command::new("/bin/sh");
            // Set each environment variable for the script
            for (key, value) in &env_vars_to_set {
                command.env(key, value);
            }
            command.arg("-c");
            command.arg(&path);

            let mut child = command.spawn()?;
            child.wait()?;
        }
    }

    Ok(())
}

fn remove_repo(repo_path: &Path) -> Result<()> {
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

fn clone_and_setup_repo(repo_path: &Path) -> Result<()> {
    match Repository::clone(REPO_URL, repo_path) {
        Ok(_) => {
            let mut child = Command::new("/bin/sh")
                .arg(repo_path.join(SETUP_SCRIPT_PATH))
                .spawn()
                .context("Failed to execute setup script: {:?}")?;
            let status = child.wait().context("Failed to wait on bash status")?;

            if !status.success() {
                anyhow::bail!("Command finished with a non-zero status: {}", status)
            }
        }

        Err(e) => {
            anyhow::bail!("Error cloning repo: {}", e);
        }
    }

    let mut child = Command::new("/bin/sh")
        .arg(repo_path.join(SETUP_SCRIPT_PATH))
        .spawn()
        .context("Failed to execute setup script: {:?}")?;
    let status = child.wait().context("Failed to wait on bash status")?;
    if !status.success() {
        anyhow::bail!("Command finished with a non-zero status: {}", status)
    }
    if !repo_path.exists()
        || !repo_path.join(HOOKS_DIR).exists()
        || !repo_path.join(THEMES_DIR).exists()
    {
        anyhow::bail!(
            "Error with {} repository at path: {}",
            REPO_NAME,
            repo_path.display()
        );
    }

    Ok(())
}

fn check_requirements(repo_path: &Path) -> bool {
    if !repo_path.exists()
        || !repo_path.join(HOOKS_DIR).exists()
        || !repo_path.join(THEMES_DIR).exists()
    {
        println!(
            "{} is not set up properly. Try running `base16_shell setup`.",
            REPO_NAME
        );

        return false;
    }

    true
}

/// Initializes the base16 colorscheme and runs the associated colorscheme script.
///
/// This function sets up the base16 colorscheme by executing a shell script specified by
/// `base16_shell_colorscheme_path`. It also checks if the necessary configuration files exist
/// and if not, it attempts to read the theme name from `base16_shell_theme_name_path`.
///
/// # Arguments
///
/// * `base16_shell_theme_default_name` - The default base16 theme name to use.
/// * `base16_shell_colorscheme_path` - The path to the colorscheme script.
/// * `base16_shell_theme_name_path` - The path to the file containing the theme name.
///
/// # Errors
///
/// This function returns an error if any of the following conditions are met:
/// - The configuration files do not exist.
/// - The colorscheme script fails to execute.
/// - The colorscheme script exits with a non-zero status.
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # fn run_example() -> anyhow::Result<()> {
/// let theme_name = "default";
/// let colorscheme_path = Path::new("/path/to/colorscheme/script");
/// let theme_name_path = Path::new("/path/to/theme/name");
///
/// init_command(
///     theme_name,
///     colorscheme_path,
///     theme_name_path,
/// )?;
///
/// # Ok(())
/// # }
/// ```
///
/// # Note
///
/// This function assumes that the necessary configuration files and scripts exist in the specified paths.
///
pub fn init_command(
    repo_path: &Path,
    base16_shell_theme_default_name: &str,
    base16_shell_colorscheme_path: &Path,
    base16_shell_theme_name_path: &Path,
) -> Result<()> {
    let is_setup_correct = check_requirements(repo_path);
    if !is_setup_correct {
        return Ok(());
    }

    let mut init_theme_name: String = base16_shell_theme_default_name.to_string();

    if base16_shell_theme_default_name.is_empty() {
        init_theme_name = read_file_to_string(base16_shell_theme_name_path)?;
    }

    if !base16_shell_colorscheme_path.exists() || init_theme_name.is_empty() {
        println!("Config files don't exist, run `base16_shell set <THEME_NAME>` to create them");
        return Ok(());
    }

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

    Ok(())
}

/// Sets the selected colorscheme and runs associated hook scripts.
///
/// This function sets the desired colorscheme based on the provided theme name.
/// It determines whether to use the provided repository path or embedded resources
/// to locate the colorscheme script. After setting the colorscheme, it runs the hook
/// scripts to apply the colorscheme to the current environment.
///
/// # Arguments
///
/// * `theme_name` - The name of the theme to set.
/// * `base16_config_path` - The path to the base16 configuration directory.
/// * `repo_path` - The path to the base16-shell-manager repository.
/// * `base16_shell_colorscheme_path` - The path to the colorscheme script file.
/// * `base16_shell_theme_name_path` - The path to the file where the theme name should be stored.
///
/// # Errors
///
/// This function returns an error if any of the following conditions are met:
/// - The colorscheme script fails to execute.
/// - The colorscheme script exits with a non-zero status.
/// - The hook scripts fail to run.
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # fn run_example() -> anyhow::Result<()> {
/// let theme_name = "default";
/// let base16_config_path = Path::new("/path/to/base16/config");
/// let repo_path = Path::new("/path/to/base16-shell-manager");
/// let base16_shell_colorscheme_path = Path::new("/path/to/base16/colorscheme/script");
/// let base16_shell_theme_name_path = Path::new("/path/to/base16/theme/name");
///
/// set_command(
///     theme_name,
///     base16_config_path,
///     repo_path,
///     base16_shell_colorscheme_path,
///     base16_shell_theme_name_path,
/// )?;
///
/// # Ok(())
/// # }
/// ```
///
pub fn set_command(
    theme_name: &str,
    base16_config_path: &Path,
    repo_path: &Path,
    base16_shell_colorscheme_path: &Path,
    base16_shell_theme_name_path: &Path,
) -> Result<()> {
    let is_setup_correct = check_requirements(repo_path);
    if !is_setup_correct {
        return Ok(());
    }

    set_colorscheme(
        theme_name,
        repo_path,
        base16_shell_colorscheme_path,
        base16_shell_theme_name_path,
    )
    .with_context(|| format!("Failed to set colorscheme \"{:?}\"", theme_name))?;

    run_hooks(
        theme_name,
        base16_config_path,
        repo_path,
        base16_shell_theme_name_path,
    )
    .context("Failed to run hooks")?;

    Ok(())
}

/// Lists available color schemes in the base16-shell-manager repository.
///
/// This function checks the provided base16-shell-manager repository path to determine if it contains
/// color scheme scripts. It validates that the provided path is a directory, collects the names
/// of available color schemes by inspecting the scripts in the directory, and prints them.
///
/// # Arguments
///
/// * `repo_path` - A reference to a `Path` pointing to the base16-shell-manager repository directory.
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
/// let repo_path = Path::new("/path/to/base16-shell-manage");
///
/// commands::list_command(&repo_path)?;
/// # Ok(())
/// # }
/// ```
///
/// The example demonstrates how to use this function to list available color schemes in the base16-shell-manager repository.
///
/// # Note
///
/// - The function prints the names of available color schemes to the console.
pub fn list_command(repo_path: &Path) -> Result<()> {
    let is_setup_correct = check_requirements(repo_path);
    if !is_setup_correct {
        return Ok(());
    }

    // Check if a custom path to base16-shell-manager repository is provided
    let themes_path = repo_path.join(THEMES_DIR);

    // Validate that the provided scripts path is a directory
    if !themes_path.is_dir() {
        anyhow::bail!(
            "Scripts directory does not exist or is not a directory: {:?}",
            themes_path
        );
    }

    // Collect color scheme names from the scripts directory
    let colorschemes: Vec<String> = fs::read_dir(&themes_path)
        .with_context(|| format!("Failed to read directory: {:?}", &themes_path))?
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
