use crate::config::Config;
use crate::constants::{REPO_DIR, SCHEMES_REPO_NAME, SCHEMES_REPO_URL};
use crate::utils::git_clone;
use anyhow::Result;
use std::path::Path;

fn setup_item(item_path: &Path, item_name: &str, item_git_url: &str) -> Result<()> {
    if !item_path.is_dir() {
        git_clone(item_git_url, item_path)?;

        println!("{} installed", item_name);
    } else {
        println!("{} already installed", item_name);
    }

    Ok(())
}

/// Setup cli tool
///
/// Clones the provided config repositories and ensures everything is ready for when the user runs
/// any other command
pub fn setup(config_path: &Path, data_path: &Path) -> Result<()> {
    // Hooks
    let config = Config::read(config_path)?;
    let items = config.items.unwrap_or_default();
    let hooks_path = data_path.join(REPO_DIR);

    for item in items {
        let item_path = hooks_path.join(&item.name);

        setup_item(&item_path, item.name.as_str(), item.git_url.as_str())?;
    }

    // Schemes
    let schemes_repo_path = hooks_path.join(SCHEMES_REPO_NAME);
    setup_item(&schemes_repo_path, SCHEMES_REPO_NAME, SCHEMES_REPO_URL)?;

    Ok(())
}
