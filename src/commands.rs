use crate::config::{
    BASE16_SHELL_CONFIG_PATH_ENV, BASE16_SHELL_THEME_NAME_PATH_ENV, BASE16_THEME_ENV, HOOKS_DIR,
    REPO_NAME, REPO_URL,
};
use crate::hooks::{self, base16_shell, base16_shell_manager};
use crate::utils::{read_file_to_string, write_to_file};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

// Set env variables for hooks and then executes .sh hook scripts
fn run_shell_hooks(
    theme_name: &str,
    app_config_path: &Path,
    repo_path: &Path,
    theme_name_path: &Path,
) -> Result<()> {
    let env_vars_to_set: Vec<(&str, &str)> = vec![
        (
            BASE16_SHELL_THEME_NAME_PATH_ENV,
            theme_name_path.to_str().unwrap(),
        ),
        (
            BASE16_SHELL_CONFIG_PATH_ENV,
            app_config_path.to_str().unwrap(),
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
            let mut command = Command::new("sh");
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

/// Initializes the base16 colorscheme and runs the associated colorscheme script.
///
/// This function sets up the base16 colorscheme by executing a shell script specified by
/// `theme_path`. It also checks if the necessary configuration files exist
/// and if not, it attempts to read the theme name from `theme_name_path`.
pub fn init_command(
    app_data_path: &Path,
    theme_name_path: &Path,
    default_theme_name: &str,
) -> Result<()> {
    hooks::base16_shell::init_theme(app_data_path, theme_name_path, default_theme_name)?;

    Ok(())
}

/// Sets the selected colorscheme and runs associated hook scripts.
///
/// This function sets the desired colorscheme based on the provided theme name.
/// It determines whether to use the provided repository path or embedded resources
/// to locate the colorscheme script. After setting the colorscheme, it runs the hook
/// scripts to apply the colorscheme to the current environment.
pub fn set_command(
    theme_name: &str,
    app_config_path: &Path,
    repo_path: &Path,
    app_data_path: &Path,
    theme_name_path: &Path,
) -> Result<()> {
    let current_theme_name =
        read_file_to_string(theme_name_path).context("Failed to read from file")?;

    if theme_name == current_theme_name {
        println!("Theme \"{}\" is already set", theme_name);
        return Ok(());
    }

    let base16_shell_manager_has_theme =
        base16_shell_manager::has_theme(theme_name, app_data_path)?;
    let base16_shell_has_theme = base16_shell::has_theme(theme_name, app_data_path)?;
    let apps = [base16_shell_manager_has_theme, base16_shell_has_theme];
    let is_theme_available = apps.iter().all(|has_theme| *has_theme);

    if !is_theme_available {
        println!("The theme isn't available for all hooks. Please run `{} update` to make sure they're all up to date and then `{} set {}` again.", REPO_NAME, REPO_NAME, theme_name);

        return Ok(());
    }

    // Write theme name to file
    write_to_file(theme_name_path, theme_name)?;

    base16_shell::set_theme(theme_name, app_data_path)
        .with_context(|| format!("Failed to set colorscheme \"{:?}\"", theme_name))?;

    run_shell_hooks(theme_name, app_config_path, repo_path, theme_name_path)
        .context("Failed to run hooks")?;

    println!("Theme set to: {}", theme_name);

    Ok(())
}

/// Lists available color schemes in the base16-shell-manager repository.
///
/// This function checks the provided base16-shell-manager repository path to determine if it contains
/// color scheme scripts. It validates that the provided path is a directory, collects the names
/// of available color schemes by inspecting the scripts in the directory, and prints them.
pub fn list_command(schemes_list_path: &Path) -> Result<()> {
    if !schemes_list_path.exists() {
        println!("Unable to retrieve the schemes list. Please run `{} setup` again. If this error persists, please file an issue at {}/issues", REPO_NAME, REPO_URL);
        return Ok(());
    }

    match base16_shell_manager::get_themes(schemes_list_path) {
        Some(schemes_list) => {
            for scheme in &schemes_list {
                println!("{}", scheme);
            }
        }
        None => {
            println!("Unable to retrieve the schemes list. Please run `{} setup` again. If this error persists, please file an issue at {}/issues", REPO_NAME, REPO_URL);
        }
    }

    Ok(())
}

/// Sets up the base16-shell-manager repository at the specified path.
///
/// This function checks if the repository path already exists. If it does, it prints a message indicating
/// that the repository is already set up and suggests using the `update` subcommand for updates. If the
/// repository path does not exist, it proceeds to clone and set up the base16-shell-manager repository at the given path.
pub fn setup_command(app_data_path: &Path) -> Result<()> {
    let (base16_shell_manager_name, is_base16_shell_manager_setup_success) =
        base16_shell_manager::setup_hook(app_data_path)?;
    let (base16_shell_template_name, is_base16_shell_setup_success) =
        base16_shell::setup_hook(app_data_path)?;

    let hooks = [
        (
            base16_shell_manager_name,
            is_base16_shell_manager_setup_success,
        ),
        (
            base16_shell_template_name,
            is_base16_shell_setup_success,
        ),
    ];

    for (name, is_success) in hooks {
        if is_success {
            println!("{} created", name);
        } else {
            println!("{} already exists", name);
        }
    }

    let is_all_success = hooks.iter().all(|(_, is_success)| *is_success);

    if !is_all_success {
        println!(
            "\nRun `{} update` to update the existing repositories",
            REPO_NAME
        );
    }

    Ok(())
}

/// Sets up the base16-shell-manager repository at the specified path.
///
/// This function checks if the repository path already exists. If it does, it prints a message indicating
/// that the repository is already set up and suggests using the `update` subcommand for updates. If the
/// repository path does not exist, it proceeds to clone and set up the base16-shell-manager repository at the given path.
pub fn update_command(app_data_path: &Path) -> Result<()> {
    let mut is_update_successful = true;

    let apps: [(&str, bool); 2] = [
        base16_shell_manager::update_hook(app_data_path)?,
        base16_shell::update_hook(app_data_path)?,
    ];

    for (name, is_update_success) in apps {
        if is_update_success {
            println!("{} updated", name);
        } else {
            println!("{} not updated", name);
            is_update_successful = false;
        }
    }

    if !is_update_successful {
        println!("\nNot all hooks updated successfully. Try removing local changes in those repos or delete them and run `{} setup`", REPO_NAME);
    }

    Ok(())
}
