use crate::config::Config;
use crate::constants::{CURRENT_SCHEME_FILE_NAME, REPO_DIR, REPO_NAME, REPO_URL};
use crate::utils::{get_shell_command_from_string, read_file_to_string, write_to_file};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn set(config_path: &Path, data_path: &Path, scheme_name: &str) -> Result<()> {
    let config = Config::read(config_path)?;
    let items = config.items.unwrap_or_default();

    write_to_file(&data_path.join(CURRENT_SCHEME_FILE_NAME), scheme_name)?;

    // Run through provided items in config.toml
    for item in items {
        let repo_path = data_path.join(REPO_DIR).join(&item.name);
        let themes_path = repo_path.join(&item.themes_dir);
        let target_theme = format!("base16-{}", scheme_name);

        if !themes_path.exists() {
            return Err(anyhow!(format!(
                "Themes files are missing, try running `{} setup` or `{} update` and try again.",
                REPO_NAME, REPO_NAME
            )));
        }

        // Find the corresponding theme file for the provided item
        let theme_option = fs::read_dir(&themes_path).map_err(anyhow::Error::new)
            .expect(format!("Themes are missing from {}, try running `{} setup` or `{} update` and try again.", item.name, REPO_NAME, REPO_NAME).as_str())
            .filter_map(Result::ok)
            .find(|entry| {
                let path = entry.path();
                let filename = path.file_stem().and_then(|name| name.to_str());

                target_theme == filename.unwrap_or_default()
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
                    "{}-{}-file.{}",
                    item.name.clone(),
                    item.themes_dir.clone(),
                    extension,
                );
                let data_theme_path = data_path.join(filename);
                let theme_content = read_file_to_string(theme_file.path().as_ref())?;

                write_to_file(&data_theme_path, theme_content.as_str())?;

                // Run hook for item if provided
                if let Some(hook_text) = item.hook {
                    let hook_script =
                        hook_text.replace("%f", format!("{}", theme_file_path.display()).as_str());
                    let command_vec = get_shell_command_from_string(config_path, hook_script.as_str())?;
                    Command::new(&command_vec[0])
                        .args(&command_vec[1..])
                        .spawn()
                        .with_context(|| {
                            format!("Failed to execute {} hook: {:?}", item.name, hook_text)
                        })?;
                }
            }
            None => println!(
                "Theme does not exists for {}. Try running `{} update` or submit an issue on {}",
                item.name, REPO_NAME, REPO_URL
            ),
        }
    }

    Ok(())
}
