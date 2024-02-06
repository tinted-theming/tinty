use crate::config::Config;
use crate::constants::REPO_DIR;
use crate::utils::git_clone;
use anyhow::Result;
use std::path::Path;

/// Setup cli tool
///
/// Clones the provided config repositories and ensures everything is ready for when the user runs
/// any other command
pub fn setup(config_path: &Path, data_path: &Path) -> Result<()> {
    let config = Config::read(config_path)?;
    let items = config.items.unwrap_or_default();
    let hooks_path = data_path.join(REPO_DIR);

    for item in items {
        let item_path = hooks_path.join(&item.name);

        if !item_path.is_dir() {
            git_clone(item.git_url.as_str(), &item_path)?;

            println!("{} installed", item.name);
        } else {
            println!("{} already installed", item.name);
        }
    }

    Ok(())
}
