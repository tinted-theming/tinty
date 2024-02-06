use crate::config::Config;
use crate::constants::{CURRENT_SCHEME_FILE_NAME, REPO_NAME};
use crate::operations;
use crate::utils::read_file_to_string;
use anyhow::{Context, Result};
use std::path::Path;

/// Initialize based on existing data_path files
///
/// This is used to set the theme when your shell is opened. It is based on your previously set
/// theme or your default theme set in config.
pub fn init(config_path: &Path, data_path: &Path) -> Result<()> {
    let config = Config::read(config_path)?;
    let active_scheme_name = read_file_to_string(&data_path.join(CURRENT_SCHEME_FILE_NAME))
        .unwrap_or(config.default_scheme.unwrap_or_default());

    operations::set::set(config_path, data_path, active_scheme_name.as_str())
            .with_context(|| {
                format!(
                    "Failed to initialize {}, config files are missing. Try setting a theme first with `{} set <SCHEME_NAME>`.",
                    REPO_NAME,
                    REPO_NAME,
                )
            })
}
