use crate::{
    constants::{CUSTOM_SCHEMES_DIR_NAME, REPO_DIR, REPO_NAME, SCHEMES_REPO_NAME},
    utils::get_all_scheme_names,
};
use anyhow::{anyhow, Result};
use std::path::Path;

/// Lists available color schemes
///
/// Lists colorschemes file which is updated via scripts/install by getting a list of schemes
/// available in https://github.com/tinted-theming/schemes/base16
pub fn list(data_path: &Path, is_custom: bool) -> Result<()> {
    let schemes_dir_path = if is_custom {
        data_path.join(CUSTOM_SCHEMES_DIR_NAME)
    } else {
        data_path.join(format!("{}/{}", REPO_DIR, SCHEMES_REPO_NAME))
    };

    match (schemes_dir_path.exists(), is_custom) {
        (false, true) => {
            return Err(anyhow!(
                "You don't have any local custom schemes at: {}",
                schemes_dir_path.display(),
            ))
        }
        (false, false) => {
            return Err(anyhow!(
                "Scheme repo path does not exist: {}\nRun `{} install` and try again",
                schemes_dir_path.display(),
                REPO_NAME
            ))
        }
        _ => {}
    }

    let scheme_vec = get_all_scheme_names(&schemes_dir_path, None)?;
    for scheme in scheme_vec {
        println!("{}", scheme);
    }

    Ok(())
}
