use crate::config::REPO_NAME;
use crate::hooks::{base16_shell, base16_shell_manager, base16_tmux};
use anyhow::Result;
use std::path::Path;

/// Sets up the base16-shell-manager repository at the specified path.
///
/// This function checks if the repository path already exists. If it does, it prints a message indicating
/// that the repository is already set up and suggests using the `update` subcommand for updates. If the
/// repository path does not exist, it proceeds to clone and set up the base16-shell-manager repository at the given path.
pub fn setup(app_data_path: &Path) -> Result<()> {
    let hooks = [
        base16_shell_manager::setup_hook(app_data_path)?,
        base16_shell::setup_hook(app_data_path)?,
        base16_tmux::setup_hook(app_data_path)?,
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
