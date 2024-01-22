use super::utils::hook_has_theme;
use crate::utils::write_to_file;
use anyhow::{Context, Result};
use std::env;
use std::path::Path;
use std::process::Command;

const TEMPLATE_NAME: &str = "base16-tmux";
const TEMPLATE_REPO_URL: &str = "https://github.com/tinted-theming/base16-tmux";
const FILE_NAME: &str = "tmux.base16.conf";
const LOCAL_THEMES_DIR: &str = "colors";

pub fn has_theme(theme_name: &str, app_data_path: &Path) -> Result<bool> {
    hook_has_theme(
        theme_name,
        TEMPLATE_NAME,
        LOCAL_THEMES_DIR,
        app_data_path,
        "base16-",
        ".conf",
    )
}

pub fn set_theme(theme_name: &str, app_config_path: &Path) -> Result<()> {
    let file_path = app_config_path.join(FILE_NAME);
    let file_contents = format!("set -g @colors-base16 '{}'", theme_name);

    write_to_file(&file_path, &file_contents)
        .with_context(|| format!("Unable to write to file: {}", file_path.display()))?;

    let tmux_exists_and_active = Command::new("sh")
        .arg("-c")
        .arg("command")
        .arg("-v")
        .arg("'tmux'")
        .output()
        .context("Failed to execute shell command to check tmux existence")?
        .status
        .success()
        && env::var("TMUX").is_ok();

    if tmux_exists_and_active {
        let tmux_config_path_from_tmux = Command::new("tmux")
            .arg("display-message")
            .arg("-p")
            .arg("\"#{config_files}\"")
            .output()
            .expect("Failed to execute command")
            .stdout;
        let tmux_config_path_text = String::from_utf8(tmux_config_path_from_tmux)?
            .trim()
            .trim_matches('\"')
            .to_string();
        let tmux_config_path = Path::new(&tmux_config_path_text);

        let output = Command::new("tmux")
            .arg("source-file")
            .arg(tmux_config_path)
            .output()
            .with_context(|| {
                format!("Failed to source tmux config file: {}", file_path.display())
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Tmux source-file command failed: {}", stderr);
        }

        Command::new("tmux")
            .arg("set-environment")
            .arg("-g")
            .arg("BASE16_THEME")
            .arg(theme_name)
            .output()
            .context("Failed to set tmux environment")?;
    }

    Ok(())
}

pub fn setup_hook(app_data_path: &Path) -> Result<(&str, bool)> {
    let local_repo_path = app_data_path.join(TEMPLATE_NAME);

    crate::hooks::utils::setup_hook(TEMPLATE_NAME, TEMPLATE_REPO_URL, &local_repo_path)
}

pub fn update_hook(app_data_path: &Path) -> Result<(&str, bool)> {
    let local_repo_path = app_data_path.join(TEMPLATE_NAME);

    crate::hooks::utils::update_hook(TEMPLATE_NAME, TEMPLATE_REPO_URL, &local_repo_path)
}
