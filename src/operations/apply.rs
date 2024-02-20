use crate::config::{Config, SupportedSchemeSystems};
use crate::constants::{
    CURRENT_SCHEME_FILE_NAME, DEFAULT_SCHEME_SYSTEM, REPO_DIR, REPO_NAME, REPO_URL,
};
use crate::utils::{
    create_theme_filename_without_extension, get_all_scheme_names, get_shell_command_from_string,
    read_file_to_string, write_to_file,
};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

fn str_matches_scheme_system(value: &str) -> bool {
    match value {
        _ if value == SupportedSchemeSystems::Base16.to_str() => true,
        _ if value == SupportedSchemeSystems::Base24.to_str() => true,
        _ => false,
    }
}

/// Apply theme
///
/// For each of the provided config items, copy the theme to the data_dir based on the provided
/// scheme_name
pub fn apply(config_path: &Path, data_path: &Path, full_scheme_name: &str) -> Result<()> {
    let scheme_name_arr = full_scheme_name.split('-');
    let scheme_system_option = scheme_name_arr.clone().next();

    // Check provided scheme exists
    if scheme_name_arr.count() < 2 {
        return Err(anyhow!(
            "Invalid scheme name. Make sure the scheme system is prefixed <SCHEME_SYSTEM>-<SCHEME_NAME>, eg: `{}-ayu-dark`",
            DEFAULT_SCHEME_SYSTEM,
        ));
    }

    // Check provided scheme is valid
    if !str_matches_scheme_system(scheme_system_option.unwrap_or_default()) {
        return Err(anyhow!(
            "Invalid scheme name. Make sure your scheme is prefixed with a supprted system (\"{}\" or \"{}\"), eg: {}-{}",
            SupportedSchemeSystems::Base16.to_str(),
            SupportedSchemeSystems::Base24.to_str(),
            DEFAULT_SCHEME_SYSTEM,
            full_scheme_name
        ));
    }

    // Check theme
    let scheme_system = scheme_system_option.unwrap_or("");
    let config = Config::read(config_path)?;
    let items = config.items.unwrap_or_default();
    let schemes_vec = get_all_scheme_names(data_path)?;

    if !schemes_vec.contains(&full_scheme_name.to_string()) {
        return Err(anyhow!("Scheme does not exist: {}", full_scheme_name));
    }

    write_to_file(&data_path.join(CURRENT_SCHEME_FILE_NAME), full_scheme_name)?;

    // Collect config items that match the provided system
    let system_items = items.iter().filter(|item| match &item.supported_systems {
        Some(supported_systems) => {
            supported_systems.contains(&SupportedSchemeSystems::from_str(scheme_system))
        }
        None => false,
    });

    // Run through provided items in config.toml
    for item in system_items {
        let repo_path = data_path.join(REPO_DIR).join(&item.name);
        let themes_path = repo_path.join(&item.themes_dir);

        if !themes_path.exists() {
            return Err(anyhow!(format!(
                "Provided theme path for {} does not exist: {}\nTry running `{} install` or `{} update` or check your config.toml file and try again.",
                item.name,
                themes_path.display(),
                REPO_NAME, REPO_NAME,
            )));
        }

        // Find the corresponding theme file for the provided item
        let theme_dir = fs::read_dir(&themes_path)
            .map_err(anyhow::Error::new)
            .with_context(|| format!("Themes are missing from {}, try running `{} install` or `{} update` and try again.", item.name, REPO_NAME, REPO_NAME))?;
        let theme_option = &theme_dir.filter_map(Result::ok).find(|entry| {
            let path = entry.path();
            let filename = path.file_stem().and_then(|name| name.to_str());

            full_scheme_name == filename.unwrap_or_default()
        });

        // Copy that theme to the data_path or log a message that it isn't found
        match theme_option {
            Some(theme_file) => {
                let theme_file_path = &theme_file.path();
                let extension = theme_file_path
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();
                let filename = format!(
                    "{}.{}",
                    create_theme_filename_without_extension(item)?,
                    extension,
                );
                let data_theme_path = data_path.join(filename);
                let theme_content = read_file_to_string(theme_file.path().as_ref())?;

                write_to_file(&data_theme_path, theme_content.as_str())?;

                // Run hook for item if provided
                if let Some(hook_text) = &item.hook {
                    let hook_script =
                        hook_text.replace("%f", format!("\"{}\"", theme_file_path.display()).as_str());
                    let command_vec =
                        get_shell_command_from_string(config_path, hook_script.as_str())?;
                    Command::new(&command_vec[0])
                        .args(&command_vec[1..])
                        .spawn()
                        .with_context(|| {
                            format!("Failed to execute {} hook: {}", item.name, hook_text)
                        })?;
                }
            }
            None => println!(
                "Theme does not exists for {} in {}. Try running `{} update` or submit an issue on {}",
                item.name, themes_path.display(), REPO_NAME, REPO_URL
            ),
        }
    }

    Ok(())
}
