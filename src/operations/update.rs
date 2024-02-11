use crate::constants::{REPO_DIR, SCHEMES_REPO_NAME, SCHEMES_REPO_URL};
use crate::utils::{git_diff, git_pull};
use crate::{config::Config, constants::REPO_NAME};
use anyhow::{Context, Result};
use std::path::Path;

fn update_item(item_name: &str, item_url: &str, item_path: &Path) -> Result<()> {
    if item_path.is_dir() {
        let is_diff = git_diff(item_path)?;

        if !is_diff {
            git_pull(item_path)
                .with_context(|| format!("Error pulling {} from {}", item_name, item_url))?;

            println!("{} up to date", item_name);
        } else {
            println!("{} contains uncommitted changes, please commit or remove and then run `{} update` again.", item_name, REPO_NAME);
        }
    } else {
        println!("{} not installed (run `{} setup`)", item_name, REPO_NAME);
    }

    Ok(())
}

/// Updates local files
///
/// Updates the provided repositories in config file by doing a git pull
pub fn update(config_path: &Path, data_path: &Path) -> Result<()> {
    let config = Config::read(config_path)?;
    let items = config.items.unwrap_or_default();
    let hooks_path = data_path.join(REPO_DIR);

    for item in items {
        let item_path = hooks_path.join(&item.name);

        update_item(item.name.as_str(), item.path.as_str(), &item_path)?;
    }

    // Schemes
    let schemes_repo_path = hooks_path.join(SCHEMES_REPO_NAME);

    update_item(SCHEMES_REPO_NAME, SCHEMES_REPO_URL, &schemes_repo_path)?;

    Ok(())
}
