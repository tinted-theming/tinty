use crate::config::{
    BASE16_SHELL_CONFIG_PATH_ENV, BASE16_SHELL_THEME_NAME_PATH_ENV, BASE16_THEME_ENV, HOOKS_DIR,
    REPO_NAME,
};
use crate::hooks::{base16_shell, base16_shell_manager, base16_tmux};
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

/// Sets the selected colorscheme and runs associated hook scripts.
///
/// This function sets the desired colorscheme based on the provided theme name.
/// It determines whether to use the provided repository path or embedded resources
/// to locate the colorscheme script. After setting the colorscheme, it runs the hook
/// scripts to apply the colorscheme to the current environment.
pub fn set(
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

    let hooks = [
        base16_shell_manager::has_theme(theme_name, app_data_path)?,
        base16_shell::has_theme(theme_name, app_data_path)?,
        base16_tmux::has_theme(theme_name, app_data_path)?,
    ];
    let is_theme_available = hooks.iter().all(|has_theme| *has_theme);

    if !is_theme_available {
        println!("The theme isn't available for all hooks. Please run `{} update` to make sure they're all up to date and then `{} set {}` again.", REPO_NAME, REPO_NAME, theme_name);

        return Ok(());
    }

    // Write theme name to file
    write_to_file(theme_name_path, theme_name)?;

    base16_shell::set_theme(theme_name, app_config_path, app_data_path)
        .with_context(|| format!("Failed to set colorscheme \"{:?}\"", theme_name))?;
    base16_tmux::set_theme(theme_name, app_config_path)
        .with_context(|| format!("Failed to set colorscheme \"{:?}\"", theme_name))?;

    run_shell_hooks(theme_name, app_config_path, repo_path, theme_name_path)
        .context("Failed to run hooks")?;

    println!("Theme set to: {}", theme_name);

    Ok(())
}
