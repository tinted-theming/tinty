use anyhow::{anyhow, Result};
use std::path::Path;

use crate::config::Config;

pub fn config(
    config_path: &Path,
    data_path: &Path,
    config_path_flag: bool,
    data_dir_path_flag: bool,
) -> Result<()> {
    let config = Config::read(config_path)?;
    let path_tuple: (bool, bool) = (config_path_flag, data_dir_path_flag);

    match path_tuple {
        (true, false) => {
            println!("{}", config_path.display());
        }
        (false, true) => {
            println!("{}", data_path.display());
        }
        (false, false) => {
            println!("{config}");
        }
        (true, true) => {
            // This case should already be handled by clap
            return Err(anyhow!(
                "the argument '--data-dir-path' cannot be used with '--config-path'",
            ));
        }
    }

    Ok(())
}
