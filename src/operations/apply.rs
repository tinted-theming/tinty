use crate::config::{Config, SupportedSchemeSystems};
use crate::constants::{
    CURRENT_SCHEME_FILE_NAME, CUSTOM_SCHEMES_DIR_NAME, DEFAULT_SCHEME_SYSTEM, REPO_DIR, REPO_NAME,
    REPO_URL, SCHEMES_REPO_NAME,
};
use crate::utils::{
    create_theme_filename_without_extension, get_all_scheme_names, get_shell_command_from_string,
    write_to_file,
};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tinted_builder_rust::operation_build::build;

fn str_matches_scheme_system(value: &str) -> bool {
    match value {
        _ if value == SupportedSchemeSystems::Base16.as_str() => true,
        _ if value == SupportedSchemeSystems::Base24.as_str() => true,
        _ => false,
    }
}

/// Apply theme
///
/// For each of the provided config items, copy the theme to the data_dir based on the provided
/// scheme_name
pub fn apply(
    config_path: &Path,
    data_path: &Path,
    full_scheme_name: &str,
    is_quiet: bool,
) -> Result<()> {
    let scheme_name_arr: Vec<String> = full_scheme_name.split('-').map(|s| s.to_string()).collect();
    let scheme_system_option = scheme_name_arr.clone().first().map(|s| s.to_string());

    // Check provided scheme exists
    if scheme_name_arr.len() < 2 {
        return Err(anyhow!(
            "Invalid scheme name. Make sure the scheme system is prefixed <SCHEME_SYSTEM>-<SCHEME_NAME>, eg: `{}-ayu-dark`",
            DEFAULT_SCHEME_SYSTEM,
        ));
    }

    // Check provided scheme is valid
    if !str_matches_scheme_system(scheme_system_option.clone().unwrap_or_default().as_str()) {
        return Err(anyhow!(
            "Invalid scheme name. Make sure your scheme is prefixed with a supprted system (\"{}\" or \"{}\"), eg: {}-{}",
            SupportedSchemeSystems::Base16.as_str(),
            SupportedSchemeSystems::Base24.as_str(),
            DEFAULT_SCHEME_SYSTEM,
            full_scheme_name
        ));
    }

    // Go through custom schemes
    let scheme_system =
        SupportedSchemeSystems::from_str(&scheme_system_option.unwrap_or("base16".to_string()));
    let schemes_path = &data_path.join(format!("{}/{}", REPO_DIR, SCHEMES_REPO_NAME));
    let schemes_vec = get_all_scheme_names(schemes_path, Some(scheme_system))?;
    let custom_schemes_path = &data_path.join(CUSTOM_SCHEMES_DIR_NAME);
    let custom_schemes_vec = if custom_schemes_path.is_dir() {
        get_all_scheme_names(custom_schemes_path, Some(scheme_system))?
    } else {
        Vec::new()
    };

    // Check theme
    let config = Config::read(config_path)?;
    let items = config.items.unwrap_or_default();
    let generate_custom_schemes: Result<()> = {
        match (
            schemes_vec.contains(&full_scheme_name.to_string()),
            custom_schemes_vec.contains(&full_scheme_name.to_string()),
        ) {
            (true, false) => Ok(()),
            (false, true) => {
                let config = Config::read(config_path)?;

                if let Some(items) = config.items {
                    let item_name_vec: Vec<String> = items.iter().map(|p| p.name.clone()).collect();

                    for item_name in item_name_vec {
                        let item_template_path: PathBuf =
                            data_path.join(format!("{}/{}", REPO_DIR, &item_name));

                        build(&item_template_path, custom_schemes_path, is_quiet)?;
                    }

                    Ok(())
                } else {
                    Ok(())
                }
            }
            (true, true) => {
                let scheme_partial_name = &scheme_name_arr[1..].join("-");

                Err(anyhow!("You have a Tinty generated scheme named the same as an official tinted-theming/schemes name, please rename or remove it: {}", format!("{}/{}.yaml", custom_schemes_path.display(), scheme_partial_name)))
            }
            _ => Err(anyhow!("Scheme does not exist: {}", full_scheme_name)),
        }
    };

    generate_custom_schemes?;
    write_to_file(&data_path.join(CURRENT_SCHEME_FILE_NAME), full_scheme_name)?;

    // Collect config items that match the provided system
    let system_items = items.iter().filter(|item| match &item.supported_systems {
        Some(supported_systems) => supported_systems.contains(&scheme_system),
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
            match &item.theme_file_extension {
                Some(extension) => {
                    let filename = path.file_name().and_then(|name| name.to_str());
                    format!("{}{}", full_scheme_name, extension) == filename.unwrap_or_default()
                }
                None => {
                    let filename = path.file_stem().and_then(|name| name.to_str());
                    full_scheme_name == filename.unwrap_or_default()
                }
            }
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
                let theme_content = fs::read_to_string(theme_file.path())?;

                write_to_file(&data_theme_path, theme_content.as_str())?;

                // Run hook for item if provided
                if let Some(hook_text) = &item.hook {
                    let hook_script = hook_text
                        .replace("%f", format!("\"{}\"", data_theme_path.display()).as_str())
                        .replace("%n", full_scheme_name);
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
            None => {
                if !is_quiet {
                    println!(
                        "Theme does not exists for {} in {}. Try running `{} update` or submit an issue on {}",
                        item.name, themes_path.display(), REPO_NAME, REPO_URL
                    )
                }
            }
        }
    }

    // Run global tinty/config.toml hooks
    if let Some(hooks_vec) = config.hooks.clone() {
        for hook in hooks_vec.iter() {
            let hook_command_vec = get_shell_command_from_string(config_path, hook.as_str())?;
            Command::new(&hook_command_vec[0])
                .args(&hook_command_vec[1..])
                .spawn()
                .with_context(|| format!("Failed to execute global hook: {}", hook))?;
        }
    }

    Ok(())
}
