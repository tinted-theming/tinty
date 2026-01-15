use crate::operations::{install, update};
use anyhow::Result;
use std::path::Path;

/// Syncs all dependencies
///
/// Syncs dependencies by doing an `operation::install` and then `operation::update`
pub fn sync(config_path: &Path, data_path: &Path, is_quiet: bool) -> Result<()> {
    install::install(config_path, data_path, is_quiet)?;
    update::update(config_path, data_path, is_quiet)?;

    Ok(())
}
