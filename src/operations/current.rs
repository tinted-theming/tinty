use crate::config::Config;
use crate::constants::{ARTIFACTS_DIR, CURRENT_SCHEME_FILE_NAME, CUSTOM_SCHEMES_DIR_NAME};
use crate::scheme_repos::merged_schemes;
use crate::utils::get_all_scheme_file_paths;
use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

pub fn get_current_scheme_slug(data_path: &Path) -> String {
    fs::read_to_string(data_path.join(ARTIFACTS_DIR).join(CURRENT_SCHEME_FILE_NAME))
        .unwrap_or_default()
}

/// Prints out the name of the last scheme applied
pub fn current(config_path: &Path, data_path: &Path, property_name: &str) -> Result<()> {
    let current_scheme_slug = get_current_scheme_slug(data_path);

    if current_scheme_slug.is_empty() {
        return Err(anyhow!(
            "Failed to read last scheme from file. Try applying a scheme and try again."
        ));
    }

    if property_name.is_empty() {
        println!("{current_scheme_slug}");

        return Ok(());
    }

    // The applied scheme may come from the built-in repo, an extra, or the
    // locally generated custom schemes, so look across all of them.
    let config = Config::read(config_path)?;
    let mut scheme_files = merged_schemes(data_path, &config)?.files;
    let custom_schemes_path = data_path.join(CUSTOM_SCHEMES_DIR_NAME);
    if custom_schemes_path.is_dir() {
        let custom_scheme_files = get_all_scheme_file_paths(&custom_schemes_path, None)?;
        scheme_files.extend(custom_scheme_files);
    }

    let current_scheme_container = scheme_files
        .get(&current_scheme_slug)
        .and_then(|scheme_file| scheme_file.get_scheme().ok());

    if let Some(current_scheme_container) = current_scheme_container {
        match property_name.trim() {
            "author" => {
                println!("{}", current_scheme_container.get_scheme_author());
            }
            "description" => {
                println!("{}", current_scheme_container.get_scheme_description());
            }
            "name" => {
                println!("{}", current_scheme_container.get_scheme_name());
            }
            "slug" => {
                println!("{}", current_scheme_container.get_scheme_slug());
            }
            "system" => {
                println!("{}", current_scheme_container.get_scheme_system());
            }
            "variant" => {
                println!("{}", current_scheme_container.get_scheme_variant());
            }
            _ => {
                eprintln!("Unable to find property: {property_name}");
            }
        }
    } else {
        eprintln!("Unable to find property: {property_name}");
    }

    Ok(())
}
