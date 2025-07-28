use crate::config::Config;
use crate::constants::{
    ARTIFACTS_DIR, CURRENT_SCHEME_FILE_NAME, CUSTOM_SCHEMES_DIR_NAME, DEFAULT_SCHEME_SYSTEM,
    REPO_DIR, REPO_NAME, REPO_URL, SCHEMES_REPO_NAME,
};
use crate::utils::{
    create_theme_filename_without_extension, get_all_scheme_file_paths, get_shell_command_from_string, write_to_file,
};
use anyhow::{anyhow, Context, Error, Result};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::str::FromStr;
use std::{fs, io};
use tinted_builder::SchemeSystem;
use tinted_builder_rust::operation_build::build;
use tinted_builder_rust::operation_build::utils::SchemeFile;

use super::list::SchemeEntry;

fn str_matches_scheme_system(value: &str) -> bool {
    match value {
        _ if value == SchemeSystem::Base16.as_str() => true,
        _ if value == SchemeSystem::Base24.as_str() => true,
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
    active_operation: Option<&str>,
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
            SchemeSystem::Base16.as_str(),
            SchemeSystem::Base24.as_str(),
            DEFAULT_SCHEME_SYSTEM,
            full_scheme_name
        ));
    }

    // Create a temporary data directory
    let staging_data_dir = tempfile::Builder::new()
        .prefix(format!("{}-", ARTIFACTS_DIR).as_str())
        .tempdir_in(data_path)?;
    let staging_data_path = staging_data_dir.path();

    // Go through custom schemes
    let scheme_system =
        SchemeSystem::from_str(&scheme_system_option.unwrap_or("base16".to_string()))?;
    let schemes_path = &data_path.join(format!("{}/{}", REPO_DIR, SCHEMES_REPO_NAME));
    let custom_schemes_path = &data_path.join(CUSTOM_SCHEMES_DIR_NAME);

    let builtin_scheme_files = get_all_scheme_file_paths(&schemes_path, None)?;
    let custom_scheme_files = get_all_scheme_file_paths(&custom_schemes_path, None).ok();

    let config = Config::read(config_path)?;

    let builtin_scheme = builtin_scheme_files.get(full_scheme_name);
    let custom_scheme = custom_scheme_files
        .as_ref()
        .and_then(|m| {
            m.get(full_scheme_name)
        });

    let scheme_file = builtin_scheme.xor(custom_scheme);
    // We expect the scheme to be a built-in scheme or a custom schemes, not both.
    if let None = scheme_file {
        if builtin_scheme.is_none() {
            return Err(anyhow!("Scheme does not exist: {}", full_scheme_name));
        } else {
            let scheme_partial_name = &scheme_name_arr[1..].join("-");
            return Err(anyhow!(
                "You have a Tinty generated scheme named the same as an official tinted-theming/schemes name, please rename or remove it: {}",
                format!("{}/{}.yaml", custom_schemes_path.display(), scheme_partial_name),
            ));
        }
    }

    if let Some(_) = custom_scheme {
        build_and_get_custom_scheme_file(custom_schemes_path, data_path, &config)?;
    }

    write_to_file(
        &staging_data_path.join(CURRENT_SCHEME_FILE_NAME),
        full_scheme_name,
    )?;

    let system_items = config
        .items
        .map(|f| {
            f.into_iter()
                .filter(|f| {
                    f.supported_systems
                        .clone()
                        .map(|s| s.contains(&scheme_system))
                        .is_some()
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut hook_commands: Vec<Hook> = Vec::new();

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
            .map_err(Error::new)
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
                let extension = match theme_file_path.extension() {
                    Some(ext) => format!(".{}", ext.to_str().unwrap_or_default()),
                    None => String::new(),
                };
                let filename = format!(
                    "{}{}",
                    create_theme_filename_without_extension(&item)?,
                    extension,
                );
                let data_theme_path = staging_data_path.join(&filename);
                let theme_content = fs::read_to_string(theme_file.path())?;

                write_to_file(&data_theme_path, theme_content.as_str())?;

                // Gather the hook commands, we will run them after we've committed all items onto
                // the final artifacts directory.
                if let Some(hook_text) = &item.hook {
                    let hook_parts = Hook {
                        name: item.name.to_string(),
                        hook_command_template: hook_text.to_string(),
                        operation: active_operation.unwrap_or("apply").to_string(),
                        relative_file_path: PathBuf::from(filename),
                    };
                    hook_commands.push(hook_parts);
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

    let target_path = data_path.join(ARTIFACTS_DIR);
    if target_path.exists() {
        // Replace the existing artifacts directory with the staging one.
        fs::remove_dir_all(&target_path)?;
    }
    fs::rename(staging_data_path, &target_path)?;
    std::mem::forget(staging_data_dir);

    for hook in hook_commands {
        hook.run_command(
            &target_path,
            config_path,
            full_scheme_name,
            scheme_file.unwrap(),
        )?;
    }

    create_symlinks_for_backwards_compat(&target_path, data_path)?;

    // Run global tinty/config.toml hooks
    if let Some(hooks_vec) = config.hooks.clone() {
        for hook in hooks_vec.iter() {
            let hook_command_vec = get_shell_command_from_string(config_path, hook.as_str())?;
            Command::new(&hook_command_vec[0])
                .args(&hook_command_vec[1..])
                .envs(SchemeEntry::from_scheme(&scheme_file.unwrap().get_scheme()?).to_envs())
                .spawn()
                .with_context(|| format!("Failed to execute global hook: {}", hook))?;
        }
    }

    Ok(())
}

fn build_and_get_custom_scheme_file(
    custom_schemes_path: &Path,
    data_path: &Path,
    config: &Config,
) -> Result<()> {
    if let Some(items) = &config.items {
        let item_name_vec: Vec<String> = items.iter().map(|p| p.name.clone()).collect();
        for item_name in item_name_vec {
            let item_template_path: PathBuf =
                data_path.join(format!("{}/{}", REPO_DIR, &item_name));
            build(&item_template_path, custom_schemes_path, true)?;
        }
    };
    Ok(())
}

fn create_symlinks_for_backwards_compat(source_path: &PathBuf, target_path: &Path) -> Result<()> {
    for entry in fs::read_dir(source_path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            let file_name = entry.file_name();
            let src_file = entry.path();
            let dst_file = target_path.join(file_name);
            // Delete existing destination file or symlink if it exists
            if dst_file.exists() {
                fs::remove_file(&dst_file)?;
            }
            symlink_any(&src_file, &dst_file)?;
        }
    }
    delete_non_dirs_and_broken_symlinks(target_path)?;

    Ok(())
}

struct Hook {
    name: String,
    hook_command_template: String,
    operation: String,
    relative_file_path: PathBuf,
}

impl Hook {
    fn run_command(
        &self,
        artifacts_path: &Path,
        config_path: &Path,
        full_scheme_name: &str,
        scheme_file: &SchemeFile,
    ) -> Result<Child, Error> {
        let theme_file_path = artifacts_path
            .join(self.relative_file_path.clone())
            .display()
            .to_string();
        let hook_script = self
            .hook_command_template
            .replace("%o", self.operation.as_str())
            .replace("%f", format!("\"{}\"", theme_file_path).as_str())
            .replace("%n", full_scheme_name);
        let command_vec = get_shell_command_from_string(config_path, hook_script.as_str())?;
        Command::new(&command_vec[0])
            .args(&command_vec[1..])
            .env("TINTY_THEME_FILE_PATH", theme_file_path)
            .env("TINTY_THEME_OPERATION", self.operation.as_str())
            .envs(SchemeEntry::from_scheme(&scheme_file.get_scheme()?).to_envs())
            .spawn()
            .with_context(|| {
                format!(
                    "Failed to execute {} hook: {}",
                    self.name, self.hook_command_template
                )
            })
    }
}

fn symlink_any(src: &Path, dst: &Path) -> Result<(), Error> {
    std::os::unix::fs::symlink(src, dst)?;
    Ok(())
}

fn delete_non_dirs_and_broken_symlinks(dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?; // Don't follow symlinks

        let file_type = metadata.file_type();

        if file_type.is_dir() {
            continue;
        }

        if file_type.is_symlink() {
            // Try to follow the symlink
            match fs::metadata(&path) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    // Broken symlink
                    fs::remove_file(&path)?;
                }
                Err(_) | Ok(_) => continue, // Valid symlink or any other error, skip
            }
        } else {
            // Regular file
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}
