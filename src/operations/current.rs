use crate::constants::{
    ARTIFACTS_DIR, CURRENT_SCHEME_FILE_NAME, CUSTOM_SCHEMES_DIR_NAME, REPO_DIR, REPO_NAME,
    SCHEMES_REPO_NAME,
};
use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;
use tinted_builder_rust::utils::get_scheme_files;

pub fn get_current_scheme_slug(data_path: &Path) -> String {
    fs::read_to_string(data_path.join(ARTIFACTS_DIR).join(CURRENT_SCHEME_FILE_NAME))
        .unwrap_or_default()
}

/// Prints out the name of the last scheme applied
pub fn current(data_path: &Path, property_name: &str) -> Result<()> {
    let current_scheme_slug = get_current_scheme_slug(data_path);
    let schemes_path = data_path.join(format!("{}/{}", REPO_DIR, SCHEMES_REPO_NAME));

    if current_scheme_slug.is_empty() {
        return Err(anyhow!(
            "Failed to read last scheme from file. Try applying a scheme and try again."
        ));
    }

    if property_name.is_empty() {
        println!("{}", current_scheme_slug);

        return Ok(());
    }

    if !schemes_path.is_dir() {
        return Err(anyhow!(
            "No schemes exist. Run `{} sync` and try again.",
            REPO_NAME
        ));
    }

    let custom_schemes_path = data_path.join(CUSTOM_SCHEMES_DIR_NAME);
    let scheme_files = {
        let mut scheme_files = get_scheme_files(&schemes_path, true)?;

        if custom_schemes_path.is_dir() {
            let custom_scheme_files = get_scheme_files(&custom_schemes_path, true)?;
            scheme_files.extend(custom_scheme_files);
        }

        scheme_files
    };

    let current_scheme_container = scheme_files.iter().find_map(|scheme_file| {
        let path = scheme_file.get_path().unwrap_or_default();
        let file_stem = path
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();

        if current_scheme_slug.contains(file_stem) {
            if let Ok(scheme) = scheme_file.get_scheme() {
                let scheme_slug = scheme.get_scheme_slug();
                let system = scheme.get_scheme_system();
                let tinty_slug = format!("{}-{}", system, scheme_slug);

                if tinty_slug == current_scheme_slug {
                    return Some(scheme);
                }
            }
        }

        None
    });

    if let Some(current_scheme_container) = current_scheme_container {
        match property_name {
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
                eprintln!("Unable to find property: {}", property_name);
            }
        }
    } else {
        eprintln!("Unable to find property: {}", property_name);
    }

    Ok(())
}
