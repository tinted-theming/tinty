use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::constants::{REPO_NAME, SCHEME_EXTENSION, SUPPORTED_SCHEME};

/// Lists available color schemes
///
/// Lists colorschemes file which is updated via scripts/setup by getting a list of schemes
/// available in https://github.com/tinted-theming/schemes/base16
pub fn list(data_path: &Path) -> Result<()> {
    let schemes_repo_path = data_path.join("repos/schemes");

    // List schemes from tinted-theming/schemes if the repo exists
    if !schemes_repo_path.is_dir() {
        if !schemes_repo_path.exists() {
            return Err(anyhow!(format!(
                "Scheme files are missing. Run `{} setup` and try again.",
                REPO_NAME
            )));
        }
    }

    let schemes_dir = fs::read_dir(&schemes_repo_path)?;
    let mut scheme_vec: Vec<String> = Vec::new();

    for schemes_subdir in schemes_dir {
        let subdir_path = schemes_subdir.unwrap().path();
        let scheme_system_dir_name = subdir_path.file_name().unwrap_or_default();
        if SUPPORTED_SCHEME == scheme_system_dir_name {
            for file in fs::read_dir(&subdir_path)? {
                let file_path = file.as_ref().unwrap().path();
                let extension = file_path
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();

                if extension == SCHEME_EXTENSION {
                    scheme_vec.push(format!(
                        "{}-{}",
                        SUPPORTED_SCHEME,
                        file.unwrap()
                            .path()
                            .file_stem()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                    ));
                }
            }
        }
    }

    scheme_vec.sort();
    for scheme in scheme_vec {
        println!("{}", scheme);
    }

    Ok(())
}
