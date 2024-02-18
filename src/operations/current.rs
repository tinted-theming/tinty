use crate::constants::CURRENT_SCHEME_FILE_NAME;
use crate::utils::read_file_to_string;
use anyhow::{anyhow, Result};
use std::path::Path;

/// Prints out the name of the last scheme applied
pub fn current(data_path: &Path) -> Result<()> {
    let current_scheme_name = read_file_to_string(&data_path.join(CURRENT_SCHEME_FILE_NAME)).ok();

    match current_scheme_name {
        Some(scheme_name) => {
            println!("{}", scheme_name);
            Ok(())
        }
        None => Err(anyhow!(
            "Failed to read last scheme from file. Try applying a scheme and try again."
        )),
    }
}
