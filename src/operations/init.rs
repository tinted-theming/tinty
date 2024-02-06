use crate::config::Config;
use crate::constants::{CURRENT_SCHEME_FILE_NAME, REPO_NAME};
use crate::operations;
use crate::utils::read_file_to_string;
use anyhow::{Context, Result};
use std::path::Path;

/// Initializes the base16 colorscheme and runs the associated colorscheme script.
///
/// This function sets up the base16 colorscheme by executing a shell script specified by
/// `theme_path`. It also checks if the necessary configuration files exist
/// and if not, it attempts to read the theme name from `theme_name_path`.
pub fn init(config_path: &Path, data_path: &Path) -> Result<()> {
    let config = Config::read(config_path)?;
    let active_scheme_name = read_file_to_string(&data_path.join(CURRENT_SCHEME_FILE_NAME))
        .unwrap_or(config.default_theme.unwrap_or_default());

    operations::set::set(config_path, data_path, active_scheme_name.as_str())
            .with_context(|| {
                format!(
                    "Failed to initialize {}, config files are missing. Try setting a theme first with `{} set <SCHEME_NAME>`.",
                    REPO_NAME,
                    REPO_NAME,
                )
            })
}
