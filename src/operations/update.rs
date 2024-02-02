use crate::config::REPO_NAME;
use crate::hooks::{base16_shell, base16_shell_manager, base16_tmux};
use anyhow::Result;
use std::path::Path;

/// Sets up the base16-shell-manager repository at the specified path.
///
/// This function checks if the repository path already exists. If it does, it prints a message indicating
/// that the repository is already set up and suggests using the `update` subcommand for updates. If the
/// repository path does not exist, it proceeds to clone and set up the base16-shell-manager repository at the given path.
pub fn update(app_data_path: &Path) -> Result<()> {
    let mut is_update_successful = true;

    let apps: [(&str, bool); 3] = [
        base16_shell_manager::update_hook(app_data_path)?,
        base16_shell::update_hook(app_data_path)?,
        base16_tmux::update_hook(app_data_path)?,
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
