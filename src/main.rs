mod cli;
mod commands;
mod config;
mod hooks;
mod utils;

use crate::cli::build_cli;
use crate::commands::{init_command, list_command, set_command};
use crate::config::BASE16_SHELL_THEME_DEFAULT_ENV;
use anyhow::{Context, Result};
use commands::{setup_command, update_command};
use config::{HOME_ENV, REPO_NAME, SCHEMES_LIST_FILENAME, XDG_CONFIG_HOME_ENV, XDG_DATA_HOME_ENV};
use std::env;
use std::path::PathBuf;
use utils::ensure_config_files_exist;

/// Entry point of the application.
fn main() -> Result<()> {
    // Parse the command line arguments
    let matches = build_cli().get_matches();

    // Determine the configuration path, falling back to the home directory if necessary
    let config_path: PathBuf = env::var(XDG_CONFIG_HOME_ENV)
        .map(PathBuf::from)
        .or_else(|_| {
            env::var(HOME_ENV)
                .map_err(anyhow::Error::new)
                .map(|home| PathBuf::from(home).join(".config"))
                .context("HOME environment variable not set")
        })?;
    let data_path: PathBuf = env::var(XDG_DATA_HOME_ENV)
        .map(PathBuf::from)
        .or_else(|_| {
            env::var(HOME_ENV)
                .map_err(anyhow::Error::new)
                .map(|home| PathBuf::from(home).join(".local/share"))
                .context("HOME environment variable not set")
        })?;
    // Other configuration paths
    let default_theme_name = env::var(BASE16_SHELL_THEME_DEFAULT_ENV).unwrap_or_default();
    let app_config_path: PathBuf = if let Some(config) = matches.get_one::<String>("config") {
        PathBuf::from(config)
    } else {
        config_path.join("tinted-theming")
    };
    let app_data_path = data_path.join("tinted-theming");
    let theme_name_path = app_config_path.join("theme_name");
    let repo_path = app_data_path.join(REPO_NAME);
    let schemes_list_path = repo_path.join(SCHEMES_LIST_FILENAME);

    ensure_config_files_exist(app_config_path.as_path(), theme_name_path.as_path())
        .context("Error creating config files")?;

    // Handle the subcommands passed to the CLI
    match matches.subcommand() {
        Some(("init", _)) => {
            init_command(
                &app_data_path,
                &theme_name_path,
                &default_theme_name,
            )
            .with_context(|| {
                format!(
                    "Failed to initialize {}, config files are missing. Try setting a theme first.\"{:?}\"",
                    REPO_NAME,
                    default_theme_name,
                )
            })?;
        }
        Some(("list", _)) => {
            list_command(&schemes_list_path)?;
        }
        Some(("set", sub_matches)) => {
            if let Some(theme) = sub_matches.get_one::<String>("theme_name") {
                let theme_name = theme.as_str();
                set_command(
                    theme_name,
                    &app_config_path,
                    &repo_path,
                    &app_data_path,
                    &theme_name_path,
                )
                .with_context(|| format!("Failed to set theme \"{:?}\"", theme_name,))?;
            } else {
                anyhow::bail!("theme_name is required for set command");
            }
        }
        Some(("setup", _)) => {
            setup_command(&app_data_path)?;
        }
        Some(("update", _)) => {
            update_command(&repo_path)?;
        }
        _ => {
            println!("Basic usage: {} set <SCHEME_NAME>", REPO_NAME);
            println!("For more information try --help");
        }
    }

    Ok(())
}
