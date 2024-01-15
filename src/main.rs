mod cli;
mod commands;
mod config;
mod utils;

use crate::cli::build_cli;
use crate::commands::{list_command, set_command};
use anyhow::{Context, Result};
use config::{HOME_ENV, REPO_URL, XDG_CONFIG_HOME_ENV, XDG_DATA_HOME_ENV};
use std::env;
use std::path::PathBuf;
use utils::{ensure_config_files_exist, get_and_setup_repo_path};

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
    let base16_config_path = config_path.join("tinted-theming");
    let tintedtheming_data_path = data_path.join("tinted-theming");
    let base16_shell_colorscheme_path = base16_config_path.join("base16_shell_theme");
    let base16_shell_theme_name_path = base16_config_path.join("theme_name");
    let base16_shell_repo_path_option: Option<PathBuf> =
        matches.get_one::<String>("repo-dir").map(PathBuf::from);

    ensure_config_files_exist(
        base16_config_path.as_path(),
        data_path.as_path(),
        base16_shell_theme_name_path.as_path(),
    )
    .context("Error creating config files")?;

    let base16_shell_repo_path = get_and_setup_repo_path(
        &tintedtheming_data_path,
        base16_shell_repo_path_option,
        REPO_URL,
    )?;

    // Handle the subcommands passed to the CLI
    match matches.subcommand() {
        Some(("list", _)) => {
            list_command(&base16_shell_repo_path)?;
        }
        Some(("set", sub_matches)) => {
            if let Some(theme) = sub_matches.get_one::<String>("theme_name") {
                let theme_name = theme.as_str();
                set_command(
                    theme_name,
                    &base16_config_path,
                    &base16_shell_repo_path,
                    &base16_shell_colorscheme_path,
                    &base16_shell_theme_name_path,
                )
                .with_context(|| format!("Failed to set theme \"{:?}\"", theme_name,))?;
            } else {
                anyhow::bail!("theme_name is required for set command");
            }
        }
        _ => {
            println!("Basic usage: base16-shell set <SCHEME_NAME>");
            println!("For more information try --help");
        }
    }

    Ok(())
}
