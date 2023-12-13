mod cli;
mod commands;
mod config;
mod utils;

use crate::cli::build_cli;
use crate::commands::{list_command, set_command};
use anyhow::{Context, Result};
use config::{HOME_ENV, REPO_URL, XDG_CONFIG_HOME_ENV};
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
                .and_then(|home| Ok(PathBuf::from(home).join(".config")))
                .context("HOME environment variable not set")
        })?;
    // Other configuration paths
    let base16_config_path = config_path.join("tinted-theming");
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

    // Handle the subcommands passed to the CLI
    match matches.subcommand() {
        // Handle the 'list' subcommand
        Some(("list", _)) => {
            list_command(base16_shell_repo_path_option)?;
        }
        // Handle the 'set' subcommand
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
