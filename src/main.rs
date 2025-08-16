mod cli;
mod config;
mod constants;
mod operations {
    pub mod apply;
    pub mod build;
    pub mod config;
    pub mod current;
    pub mod cycle;
    pub mod generate_scheme;
    pub mod info;
    pub mod init;
    pub mod install;
    pub mod list;
    pub mod sync;
    pub mod update;
}
mod utils;

use crate::cli::{build_cli, get_matches};
use anyhow::{anyhow, Context, Result};
use clap::Command;
use clap_complete::{generate, Generator, Shell};
use config::{CONFIG_FILE_NAME, ORG_NAME};
use constants::{CUSTOM_SCHEMES_DIR_NAME, REPO_DIR, REPO_NAME, SCHEMES_REPO_NAME};
use operations::generate_scheme;
use std::path::PathBuf;
use tinted_builder::{SchemeSystem, SchemeVariant};
use utils::{ensure_directory_exists, replace_tilde_slash_with_home};
use xdg::BaseDirectories;

/// Entry point of the application.
fn main() -> Result<()> {
    // Parse the command line arguments
    let matches = get_matches();
    let xdg_dirs = BaseDirectories::with_prefix(format!("{ORG_NAME}/{REPO_NAME}")).unwrap();

    // Other configuration paths
    let config_path_result: Result<PathBuf> =
        if let Some(config_file_path) = matches.get_one::<String>("config") {
            replace_tilde_slash_with_home(config_file_path)
        } else {
            xdg_dirs
                .place_config_file(CONFIG_FILE_NAME)
                .context(format!(
                    "Unable to create XDG_HOME/{ORG_NAME}/{REPO_NAME}/{CONFIG_FILE_NAME}",
                ))
        };
    let config_path = config_path_result?;
    let data_path: PathBuf = if let Some(data_file_path) = matches.get_one::<String>("data-dir") {
        replace_tilde_slash_with_home(data_file_path)?
    } else {
        xdg_dirs.get_data_home()
    };
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
                let schemes_repo_path = data_path.join(format!("{REPO_DIR}/{SCHEMES_REPO_NAME}"));
                let template_path = PathBuf::from(template_dir);

                operations::build::build(&template_path, &schemes_repo_path)?;
            }
        }
        Some(("current", sub_matches)) => {
            let property_name = sub_matches
                .get_one::<String>("property_name")
                .map(|s| s.as_str())
                .unwrap_or_default();

            operations::current::current(&data_path, property_name)?;
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
            let is_custom = sub_matches
                .get_one::<bool>("custom-schemes")
                .map(|b| b.to_owned())
                .unwrap_or(false);
            let scheme_name_option = sub_matches.get_one::<String>("scheme_name");

            operations::info::info(&data_path, scheme_name_option, is_custom)?;
        }
        Some(("init", sub_matches)) => {
            let is_verbose = sub_matches
                .get_one::<bool>("verbose")
                .map(|b| b.to_owned())
                .unwrap_or(false);

            operations::init::init(&config_path, &data_path, is_verbose)?;
        }
        Some(("list", sub_matches)) => {
            let is_custom = sub_matches
                .get_one::<bool>("custom-schemes")
                .map(|b| b.to_owned())
                .unwrap_or(false);
            let is_json = sub_matches
                .get_one::<bool>("json")
                .map(|b| b.to_owned())
                .unwrap_or(false);

            operations::list::list(&data_path, is_custom, is_json)?;
        }
        Some(("apply", sub_matches)) => {
            if let Some(theme) = sub_matches.get_one::<String>("scheme_name") {
                let is_quiet = sub_matches
                    .get_one::<bool>("quiet")
                    .map(|b| b.to_owned())
                    .unwrap_or(false);

                let scheme_name = theme.as_str();
                operations::apply::apply(&config_path, &data_path, scheme_name, is_quiet, None)
                    .with_context(|| format!("Failed to apply theme \"{scheme_name}\""))?;
            }
        }
        Some(("cycle", sub_matches)) => {
            let is_quiet = sub_matches
                .get_one::<bool>("quiet")
                .map(|b| b.to_owned())
                .unwrap_or(false);

            operations::cycle::cycle(&config_path, &data_path, is_quiet, None)
                .context("Failed to cycle to your next preferred theme")?;
        }
        Some(("install", sub_matches)) => {
            let is_quiet = sub_matches
                .get_one::<bool>("quiet")
                .map(|b| b.to_owned())
                .unwrap_or(false);

            operations::install::install(&config_path, &data_path, is_quiet)?;
        }
        Some(("update", sub_matches)) => {
            let is_quiet = sub_matches
                .get_one::<bool>("quiet")
                .map(|b| b.to_owned())
                .unwrap_or(false);

            operations::update::update(&config_path, &data_path, is_quiet)?;
        }
        Some(("sync", sub_matches)) => {
            let is_quiet = sub_matches
                .get_one::<bool>("quiet")
                .map(|b| b.to_owned())
                .unwrap_or(false);

            operations::sync::sync(&config_path, &data_path, is_quiet)?;
        }
        Some(("generate-scheme", sub_matches)) => {
            let slug_default = "tinty-generated".to_string();
            let slug = sub_matches
                .get_one::<String>("slug")
                .unwrap_or(&slug_default);
            let name_default = "Tinty Generated".to_string();
            let description = sub_matches
                .get_one::<String>("description")
                .map(String::from);
            let name = sub_matches
                .get_one::<String>("name")
                .unwrap_or(&name_default);
            let author_default = "Tinty".to_string();
            let author = sub_matches
                .get_one::<String>("author")
                .unwrap_or(&author_default);
            let image_path = match sub_matches.get_one::<String>("image_path") {
                Some(content) => PathBuf::from(content)
                    .canonicalize()
                    .with_context(|| "Invalid image file supplied"),
                None => Err(anyhow!("No image file specified")),
            }?;
            let system = match sub_matches.get_one::<String>("system").map(|s| s.as_str()) {
                Some("base24") => SchemeSystem::Base24,
                _ => SchemeSystem::Base16,
            };
            let variant = match sub_matches.get_one::<String>("variant").map(|s| s.as_str()) {
                Some("light") => SchemeVariant::Light,
                _ => SchemeVariant::Dark,
            };
            let outfile_path_option = {
                let custom_scheme_path = data_path.join(CUSTOM_SCHEMES_DIR_NAME);
                let save = sub_matches.get_one::<bool>("save").unwrap_or(&false);

                // Ensure schemes/base16 and schemes/base24 paths exist
                ensure_directory_exists(custom_scheme_path.join("base16")).with_context(|| {
                    format!(
                        "Failed to create custom scheme directory at {}",
                        data_path.display()
                    )
                })?;
                ensure_directory_exists(custom_scheme_path.join("base24")).with_context(|| {
                    format!(
                        "Failed to create custom scheme directory at {}",
                        data_path.display()
                    )
                })?;

                if *save {
                    let filename = format!("{slug}.yaml");

                    Some(custom_scheme_path.join(format!("{system}/{filename}")))
                } else {
                    None
                }
            };

            generate_scheme::generate_scheme(
                image_path,
                outfile_path_option,
                author.to_string(),
                description,
                name.to_string(),
                slug.to_string(),
                system,
                variant,
            )?;
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
