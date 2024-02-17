mod cli;
mod config;
mod constants;
mod operations;
mod utils;

use crate::cli::build_cli;
use anyhow::{anyhow, Context, Result};
use constants::{REPO_DIR, REPO_NAME};
use std::path::PathBuf;
use utils::ensure_directory_exists;

/// Entry point of the application.
fn main() -> Result<()> {
    // Parse the command line arguments
    let matches = build_cli().get_matches();

    // Determine the configuration path, falling back to the home directory if necessary
    let system_data_path: PathBuf =
        dirs::data_dir().ok_or_else(|| anyhow!("Error getting data directory"))?;

    // Other configuration paths
    let config_path: PathBuf = if let Some(config) = matches.get_one::<String>("config") {
        PathBuf::from(config)
    } else {
        dirs::config_dir()
            .ok_or_else(|| anyhow!("Error getting config directory"))?
            .join(format!("tinted-theming/{}", REPO_NAME))
    };
    let data_path = system_data_path.join(format!("tinted-theming/{}", REPO_NAME));
    let data_repo_path = data_path.join(REPO_DIR);

    // Ensure config dirs exist
    ensure_directory_exists(&data_path)
        .with_context(|| format!("Failed to create data directory at {}", data_path.display()))?;
    ensure_directory_exists(&data_repo_path).with_context(|| {
        format!(
            "Failed to create config directory at {}",
            data_repo_path.display()
        )
    })?;
    ensure_directory_exists(&config_path).with_context(|| {
        format!(
            "Failed to create config directory at {}",
            config_path.display()
        )
    })?;

    // Handle the subcommands passed to the CLI
    match matches.subcommand() {
        Some(("current", _)) => {
            operations::current::current(&data_path)?;
        }
        Some(("info", sub_matches)) => {
            let scheme_name_option = sub_matches.get_one::<String>("scheme_name");

            operations::info::info(&data_path, scheme_name_option)?;
        }
        Some(("init", _)) => {
            operations::init::init(&config_path, &data_path)?;
        }
        Some(("list", _)) => {
            operations::list::list(&data_path)?;
        }
        Some(("set", sub_matches)) => {
            if let Some(theme) = sub_matches.get_one::<String>("scheme_name") {
                let scheme_name = theme.as_str();
                operations::set::set(&config_path, &data_path, scheme_name)
                    .with_context(|| format!("Failed to set theme \"{:?}\"", scheme_name,))?;
            } else {
                anyhow::bail!("scheme_name is required for set command");
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
