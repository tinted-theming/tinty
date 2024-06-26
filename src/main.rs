mod cli;
mod config;
mod constants;
mod operations {
    pub mod apply;
    pub mod build;
    pub mod config;
    pub mod current;
    pub mod info;
    pub mod init;
    pub mod install;
    pub mod list;
    pub mod update;
}
mod utils;

use crate::cli::{build_cli, get_matches};
use anyhow::{anyhow, Context, Result};
use clap::Command;
use clap_complete::{generate, Generator, Shell};
use config::CONFIG_FILE_NAME;
use constants::{REPO_DIR, REPO_NAME, SCHEMES_REPO_NAME};
use std::path::PathBuf;
use utils::{ensure_directory_exists, replace_tilde_slash_with_home};

/// Entry point of the application.
fn main() -> Result<()> {
    // Parse the command line arguments
    let matches = get_matches();

    // Other configuration paths
    let config_path_result: Result<PathBuf> =
        if let Some(config_file_path) = matches.get_one::<String>("config") {
            replace_tilde_slash_with_home(config_file_path)
        } else {
            Ok(dirs::config_dir()
                .ok_or_else(|| anyhow!("Error getting config directory"))?
                .join(format!("tinted-theming/{}/{}", REPO_NAME, CONFIG_FILE_NAME)))
        };
    let config_path = config_path_result?;
    // Determine the data-dir path
    let data_path_result: Result<PathBuf> =
        if let Some(data_file_path) = matches.get_one::<String>("data-dir") {
            replace_tilde_slash_with_home(data_file_path)
        } else {
            Ok(dirs::data_dir()
                .ok_or_else(|| anyhow!("Error getting data directory"))?
                .join(format!("tinted-theming/{}", REPO_NAME)))
        };
    let data_path = data_path_result?;
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

    // Handle the subcommands passed to the CLI
    match matches.subcommand() {
        Some(("build", sub_matches)) => {
            if let Some(template_dir) = sub_matches.get_one::<String>("template-dir") {
                let schemes_repo_path =
                    data_path.join(format!("{}/{}", REPO_DIR, SCHEMES_REPO_NAME));
                let template_path = PathBuf::from(template_dir);

                operations::build::build(&template_path, &schemes_repo_path)?;
            }
        }
        Some(("current", _)) => {
            operations::current::current(&data_path)?;
        }
        Some(("config", sub_matches)) => {
            let data_dir_path_flag = sub_matches.get_flag("data-dir-path");
            let config_path_flag = sub_matches.get_flag("config-path");

            operations::config::config(
                &config_path,
                &data_path,
                config_path_flag,
                data_dir_path_flag,
            )?;
        }
        Some(("generate-completion", sub_matches)) => {
            if let Some(generator) = sub_matches.get_one::<Shell>("shell_name") {
                let mut cmd = build_cli();

                print_completions(*generator, &mut cmd);
                return Ok(());
            };
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
        Some(("apply", sub_matches)) => {
            if let Some(theme) = sub_matches.get_one::<String>("scheme_name") {
                let scheme_name = theme.as_str();
                operations::apply::apply(&config_path, &data_path, scheme_name)
                    .with_context(|| format!("Failed to apply theme \"{:?}\"", scheme_name,))?;
            } else {
                return Err(anyhow!("scheme_name is required for apply command"));
            }
        }
        Some(("install", _)) => {
            operations::install::install(&config_path, &data_path)?;
        }
        Some(("update", _)) => {
            operations::update::update(&config_path, &data_path)?;
        }
        _ => {
            println!("Basic usage: {} apply <SCHEME_NAME>", REPO_NAME);
            println!("For more information try --help");
        }
    }

    Ok(())
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}
