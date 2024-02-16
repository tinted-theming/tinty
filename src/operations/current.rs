use crate::constants::CURRENT_SCHEME_FILE_NAME;
use crate::utils::read_file_to_string;
use anyhow::{anyhow, Result};
use std::path::Path;

/// Initialize based on existing data_path files
///
/// This is used to set the theme when your shell is opened. It is based on your previously set
/// theme or your default theme set in config.
pub fn current(data_path: &Path) -> Result<()> {
    let current_scheme_name = read_file_to_string(&data_path.join(CURRENT_SCHEME_FILE_NAME)).ok();

    match current_scheme_name {
        Some(scheme_name) => {
            println!("{}", scheme_name);
            Ok(())
        }
        None => Err(anyhow!(
            "Failed to read last scheme from file. Try setting a scheme and try again."
        )),
    }
}
