mod cli;
mod config;
mod constants;
mod operations;
mod utils;

extern crate dirs;

use crate::cli::build_cli;
use anyhow::{anyhow, Context, Result};
use constants::{REPO_NAME, SCHEMES_LIST_FILENAME};
use std::path::PathBuf;
use utils::ensure_directory_exists;

/// Entry point of the application.
fn main() -> Result<()> {
    // Parse the command line arguments
    let matches = build_cli().get_matches();

    // Determine the configuration path, falling back to the home directory if necessary
    let data_path: PathBuf =
        dirs::data_dir().ok_or_else(|| anyhow!("Error getting data directory"))?;

    // Other configuration paths
    let config_path: PathBuf = if let Some(config) = matches.get_one::<String>("config") {
        PathBuf::from(config)
    } else {
        dirs::config_dir()
            .ok_or_else(|| anyhow!("Error getting config directory"))?
            .join("tinted-theming")
    };
    let data_path = data_path.join("tinted-theming");
    let repo_path = data_path.join(REPO_NAME);
    let schemes_list_path = repo_path.join(SCHEMES_LIST_FILENAME);

    // Ensure config dirs exist
    ensure_directory_exists(&config_path)
        .with_context(|| format!("Failed to create config directory at {:?}", config_path))?;

    // Handle the subcommands passed to the CLI
    match matches.subcommand() {
        Some(("init", _)) => {
            operations::init::init(&config_path, &data_path)?;
        }
        Some(("list", _)) => {
            operations::list::list(&schemes_list_path)?;
        }
        Some(("set", sub_matches)) => {
            if let Some(theme) = sub_matches.get_one::<String>("theme_name") {
                let theme_name = theme.as_str();
                operations::set::set(&config_path, &data_path, theme_name)
                    .with_context(|| format!("Failed to set theme \"{:?}\"", theme_name,))?;
            } else {
                anyhow::bail!("theme_name is required for set command");
            }
        }
        Some(("setup", _)) => {
            operations::setup::setup(&config_path, &data_path)?;
        }
        Some(("update", _)) => {
            operations::update::update(&config_path, &data_path)?;
        }
        _ => {
            println!("Basic usage: {} set <SCHEME_NAME>", REPO_NAME);
            println!("For more information try --help");
        }
    }

    Ok(())
}
