use crate::constants::{CUSTOM_SCHEMES_DIR_NAME, REPO_DIR, REPO_NAME, SCHEMES_REPO_NAME};
use anyhow::{anyhow, Result};
use std::path::Path;
use tinted_builder_rust::utils::get_scheme_files;

/// Lists available color schemes
///
/// Lists colorschemes file which is updated via scripts/install by getting a list of schemes
/// available in https://github.com/tinted-theming/schemes
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
                "Schemes are missing, run install and then try again: `{} install`",
                REPO_NAME
            ))
        }
        _ => {}
    }

    let scheme_files = get_scheme_files(&schemes_dir_path, true)?;
    for scheme_file in scheme_files {
        let scheme_container = scheme_file.get_scheme()?;
        let system = scheme_container.get_scheme_system();
        let slug = scheme_container.get_scheme_slug();

        println!("{}-{}", system, slug);
    }

    Ok(())
}
