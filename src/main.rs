mod cli;
mod commands;
mod config;

use crate::cli::build_cli;
use crate::commands::{ensure_config_files_exist, set_command};
use anyhow::{Context, Result};
use config::{
    BASE16_SHELL_PATH_ENV, DEFAULT_BASE16_SHELL_PATH, HOME_ENV, REPO_URL, XDG_CONFIG_HOME_ENV,
};
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<()> {
    let matches = build_cli().get_matches();
    let config_path: PathBuf = env::var(XDG_CONFIG_HOME_ENV)
        .map(PathBuf::from)
        .or_else(|_| {
            env::var(HOME_ENV)
                .map_err(anyhow::Error::new)
                .and_then(|home| Ok(PathBuf::from(home).join(".config")))
                .context("HOME environment variable not set")
        })?;
    let base16_config_path = config_path.join("tinted-theming");
    let base16_shell_path: PathBuf = env::var(BASE16_SHELL_PATH_ENV)
        .or_else(|_| Ok(String::from(DEFAULT_BASE16_SHELL_PATH)))
        .map(PathBuf::from)
        .and_then(|path| {
            if path.exists() {
                Ok(path)
            } else {
                anyhow::bail!(format!("The config path does not exist: {}", path.display()))
            }
        }).context("Path to tinted-theming/base16-shell not found. Either set $BASE16_SHELL_PATH or clone tinted-theming/base16-shell to ~/.config/tinted-theming/shell")?;
    let base16_shell_colorscheme_path = base16_config_path.join("base16_shell_theme");
    let base16_shell_theme_name_path = base16_config_path.join("theme_name");

    let base16_shell_repo_path_option: Option<PathBuf> = matches
        .get_one::<String>("repo-dir")
        .map(|p| PathBuf::from(p));

    if let Some(base16_shell_repo_path) = &base16_shell_repo_path_option {
        if !base16_shell_repo_path.exists()
            || !base16_shell_repo_path.join("hooks").exists()
            || !base16_shell_repo_path.join("scripts").exists()
        {
            anyhow::bail!(
                "The config path does not exist or does not contain the correct directory structure. Make sure \"{}\" is cloned from: {}",
                base16_shell_repo_path.display(),
                REPO_URL
            );
        }
    }

    ensure_config_files_exist(
        base16_config_path.as_path(),
        base16_shell_theme_name_path.as_path(),
    )
    .context("Error creating config files")?;

    match matches.subcommand() {
        Some(("list", _)) => {
            let scripts_path = base16_shell_path.join("scripts");

            if !scripts_path.is_dir() {
                anyhow::bail!(
                    "Scripts directory does not exist or is not a directory: {:?}",
                    scripts_path
                );
            }

            let themes: Vec<String> = fs::read_dir(&scripts_path)
                .with_context(|| format!("Failed to read directory: {:?}", &scripts_path))?
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();

                    if path.is_file() {
                        return path
                            .file_stem()
                            .and_then(|name| name.to_str())
                            .and_then(|name| name.strip_prefix("base16-"))
                            .map(|name| name.to_string());
                    }

                    None
                })
                .collect();

            if themes.is_empty() {
                println!("No themes found in the scripts directory.");
            } else {
                for theme in themes {
                    println!("{}", theme);
                }
            }
        }
        Some(("set", sub_matches)) => {
            if let Some(theme_name) = sub_matches.get_one::<String>("theme_name") {
                set_command(
                    &theme_name,
                    &base16_config_path,
                    base16_shell_repo_path_option,
                    &base16_shell_colorscheme_path,
                    &base16_shell_theme_name_path,
                )
                .with_context(|| format!("Failed to set theme \"{:?}\"", theme_name,))?;

                println!("Theme set to: {}", theme_name);
            } else {
                anyhow::bail!("theme_name is required for set command",);
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
