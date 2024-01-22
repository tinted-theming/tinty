use crate::utils::{read_file_to_string, write_to_file};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;

use super::utils::hook_has_theme;

const TEMPLATE_NAME: &str = "base16-shell";
const TEMPLATE_REPO_URL: &str = "https://github.com/tinted-theming/base16-shell";
const LOCAL_THEME: &str = "base16_shell_theme";
const LOCAL_THEMES_DIR: &str = "scripts";

pub fn has_theme(theme_name: &str, app_data_path: &Path) -> Result<bool> {
    hook_has_theme(
        theme_name,
        TEMPLATE_NAME,
        LOCAL_THEMES_DIR,
        app_data_path,
        "base16-",
        ".sh",
    )
}

pub fn init_theme(
    app_config_path: &Path,
    theme_name_path: &Path,
    default_theme_name: &str,
) -> Result<(String, bool)> {
    let theme_path = app_config_path.join(LOCAL_THEME);

    let mut init_theme_name: String = default_theme_name.to_string();
    let mut is_init_theme_success = false;

    if default_theme_name.is_empty() {
        init_theme_name = read_file_to_string(theme_name_path)?;
    }

    if !theme_path.exists() || init_theme_name.is_empty() {
        return Ok((
            "Config files don't exist, run `base16_shell set <THEME_NAME>` to create them"
                .to_string(),
            is_init_theme_success,
        ));
    }

    let mut child = Command::new("sh")
        .arg(&theme_path)
        .spawn()
        .with_context(|| format!("Failed to execute script: {:?}", theme_path))?;
    let status = child.wait().context("Failed to wait on bash status")?;

    if !status.success() {
        is_init_theme_success = false;

        return Ok((
            format!(
                "There was an issue executing theme with `sh` at: {}",
                &theme_path.display()
            ),
            is_init_theme_success,
        ));
    }

    is_init_theme_success = true;
    Ok(("".to_string(), is_init_theme_success))
}

pub fn setup_hook(app_data_path: &Path) -> Result<(&str, bool)> {
    let local_repo_path = app_data_path.join(TEMPLATE_NAME);

    crate::hooks::utils::setup_hook(TEMPLATE_NAME, TEMPLATE_REPO_URL, &local_repo_path)
}

pub fn update_hook(app_data_path: &Path) -> Result<(&str, bool)> {
    let local_repo_path = app_data_path.join(TEMPLATE_NAME);

    crate::hooks::utils::update_hook(TEMPLATE_NAME, TEMPLATE_REPO_URL, &local_repo_path)
}

pub fn set_theme(theme_name: &str, app_config_path: &Path, app_data_path: &Path) -> Result<()> {
    let local_repo_path = app_data_path.join(TEMPLATE_NAME);
    let theme_path = app_config_path.join(LOCAL_THEME);

    let theme_script_path =
        local_repo_path.join(format!("{}/base16-{}.sh", LOCAL_THEMES_DIR, theme_name));
    if !theme_script_path.exists() {
        anyhow::bail!(
            "Theme \"{}\" does not exist at \"{}\", try a different theme",
            theme_name,
            theme_script_path.display()
        )
    }
    let theme_script_contents = read_file_to_string(
        &local_repo_path.join(format!("{}/base16-{}.sh", LOCAL_THEMES_DIR, theme_name)),
    )?;

    // Write shell theme script to file
    write_to_file(&theme_path, from_utf8(theme_script_contents.as_bytes())?)
        .with_context(|| format!("Unable to write to file: {}", theme_path.display()))?;

    // Source colorscheme script
    // Wait for script to fully execute before continuing
    let mut child = Command::new("sh")
        .arg(&theme_path)
        .spawn()
        .with_context(|| format!("Failed to execute script: {:?}", theme_path))?;
    let status = child.wait().context("Failed to wait on bash status")?;
    if !status.success() {
        anyhow::bail!("Command finished with a non-zero status: {}", status)
    }

    Ok(())
}
