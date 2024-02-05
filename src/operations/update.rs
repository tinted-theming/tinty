use crate::constants::REPO_DIR;
use crate::utils::{git_diff, git_pull};
use crate::{config::Config, constants::REPO_NAME};
use anyhow::{Context, Result};
use std::path::Path;

/// Sets up the base16-shell-manager repository at the specified path.
///
/// This function checks if the repository path already exists. If it does, it prints a message indicating
/// that the repository is already set up and suggests using the `update` subcommand for updates. If the
/// repository path does not exist, it proceeds to clone and set up the base16-shell-manager repository at the given path.
pub fn update(config_path: &Path, data_path: &Path) -> Result<()> {
    let config = Config::read(config_path)?;
    let items = config.items.unwrap_or_default();
    let hooks_path = data_path.join(REPO_DIR);

    for item in items {
        let item_path = hooks_path.join(&item.name);

        if item_path.is_dir() {
            let is_diff = git_diff(&item_path)?;

            if !is_diff {
                git_pull(&item_path).with_context(|| {
                    format!("Error pulling {} from {}", item.name, item.git_url)
                })?;
                println!("{} up to date", item.name);
            } else {
                println!("{} contains uncommitted changes, please commit or remove and then run `{} update` again.", item.name, REPO_NAME);
            }
        } else {
            println!("{} not installed (run `{} setup`)", item.name, REPO_NAME);
        }
    }

    Ok(())
}
