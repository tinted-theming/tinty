use crate::config::Config;
use crate::constants::REPO_NAME;
use crate::paths;
use anyhow::{anyhow, Result};
use std::path::Path;
use tinted_builder_rust::operation_build;

/// Builds the provided template using `tinted_builder_rust`
pub fn build(template_path: &Path, schemes_repo_path: &Path) -> Result<()> {
    operation_build::build(template_path, schemes_repo_path, &[], false)?;

    Ok(())
}

/// Builds every installed `[[items]]` template repository listed in the config.
///
/// Items are built sequentially against the synced schemes repo. A failure
/// building one item does not abort the run: every item is attempted, per-item
/// errors are collected, and the operation reports failure at the end if any
/// item failed to build (or was not installed).
pub fn build_all_items(config_path: &Path, data_path: &Path, is_quiet: bool) -> Result<()> {
    let config = Config::read(config_path)?;
    let items = config.items.unwrap_or_default();
    let schemes_repo_path = paths::schemes_repo_path(data_path);

    if items.is_empty() {
        return Err(anyhow!(
            "No `[[items]]` found in config. Add template repositories to your config and run `{REPO_NAME} install` before building."
        ));
    }

    let mut failures: Vec<String> = Vec::new();

    for item in &items {
        let item_path = paths::item_repo_path(data_path, &item.name);

        if !item_path.is_dir() {
            let message = format!("{} not installed (run `{REPO_NAME} install`)", item.name);
            if !is_quiet {
                eprintln!("{message}");
            }
            failures.push(message);
            continue;
        }

        match build(&item_path, &schemes_repo_path) {
            Ok(()) => {
                if !is_quiet {
                    println!("{} built", item.name);
                }
            }
            Err(err) => {
                if !is_quiet {
                    eprintln!("{} failed to build: {err:#}", item.name);
                }
                failures.push(format!("{}: {err:#}", item.name));
            }
        }
    }

    if failures.is_empty() {
        if !is_quiet {
            println!("Built {} item(s)", items.len());
        }

        Ok(())
    } else {
        Err(anyhow!(
            "Failed to build {} of {} item(s):\n{}",
            failures.len(),
            items.len(),
            failures.join("\n")
        ))
    }
}
