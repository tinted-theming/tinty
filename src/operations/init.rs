use crate::config::Config;
use crate::constants::{CURRENT_SCHEME_FILE_NAME, REPO_NAME};
use crate::operations;
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::Path;

/// Initialize based on existing `data_path` files
///
/// This is used to apply the theme when your shell is opened. It is based on your previously applied
/// theme or your default theme set in config.
pub fn init(config_path: &Path, data_path: &Path, is_verbose: bool) -> Result<()> {
    let config = Config::read(config_path)?;
    let active_scheme_name = fs::read_to_string(data_path.join(CURRENT_SCHEME_FILE_NAME))
        .unwrap_or_else(|_| config.default_scheme.clone().unwrap_or_default());

    if active_scheme_name.is_empty() {
        return Err(anyhow!("Failed to initialize, config files seem to be missing. Try applying a theme first with `{} apply <SCHEME_NAME>`.", REPO_NAME));
    }

    operations::apply::apply(config_path, data_path, active_scheme_name.as_str(), !is_verbose, Some("init"))
            .with_context(|| {
                format!(
                    "Failed to initialize {REPO_NAME}, config files are missing. Try applying a theme first with `{REPO_NAME} apply <SCHEME_NAME>`.",
                )
            })
}
