use crate::constants::{REPO_DIR, SCHEMES_REPO_NAME, SCHEMES_REPO_REVISION, SCHEMES_REPO_URL};
use crate::repo::{self, UpdateStatus};
use crate::{config::Config, constants::REPO_NAME};
use anyhow::{Context, Result};
use std::path::Path;

fn update_item(
    item_name: &str,
    item_url: &str,
    item_path: &Path,
    revision: Option<&str>,
    allow_dirty: bool,
    is_quiet: bool,
) -> Result<()> {
    if item_path.is_dir() {
        let rev = revision.unwrap_or("main");
        let is_clean = repo::is_clean(item_path)?;

        if is_clean {
            repo::update(item_path, item_url, revision, false)
                .with_context(|| format!("Error updating {item_name} to {item_url}@{rev}"))?;

            if !is_quiet {
                println!("{item_name} up to date");
            }
        } else if allow_dirty {
            let status = repo::update(item_path, item_url, revision, true)
                .with_context(|| format!("Error updating {item_name} to {item_url}@{rev}"))?;

            if !is_quiet {
                match status {
                    UpdateStatus::Updated => {
                        println!("{item_name} up to date (local changes preserved)");
                    }
                    UpdateStatus::ConflictPreserved { stderr } => {
                        print_conflict_message(item_name, &stderr);
                    }
                }
            }
        } else if !is_quiet {
            println!("{item_name} contains uncommitted changes, please commit or remove and then run `{REPO_NAME} update` again.");
        }
    } else if !is_quiet {
        println!("{item_name} not installed (run `{REPO_NAME} install`)");
    }

    Ok(())
}

/// Prints a human-facing explanation when an update was refused because it
/// would have overwritten the user's uncommitted work. The working tree is
/// left untouched, so we echo git's own message verbatim (in yellow) — it
/// already names the offending files and how to proceed.
fn print_conflict_message(item_name: &str, git_stderr: &str) {
    const YELLOW: &str = "\x1b[33m";
    const RESET: &str = "\x1b[0m";

    println!("{item_name}: could not update — your local changes are preserved:");
    for line in git_stderr.lines() {
        println!("{YELLOW}{line}{RESET}");
    }
}

/// Updates local files
///
/// Updates the provided repositories in config file by doing a git pull
pub fn update(config_path: &Path, data_path: &Path, is_quiet: bool) -> Result<()> {
    let config = Config::read(config_path)?;
    // The top-level value is the default for every item and also governs the
    // built-in schemes repo, which has no `[[items]]` entry of its own.
    let global_allow_dirty = config.allow_dirty_update;
    let items = config.items.unwrap_or_default();
    let hooks_path = data_path.join(REPO_DIR);

    for item in items {
        let item_path = hooks_path.join(&item.name);
        let allow_dirty = item.allow_dirty_update.unwrap_or(global_allow_dirty);

        update_item(
            item.name.as_str(),
            item.path.as_str(),
            &item_path,
            item.revision.as_deref(),
            allow_dirty,
            is_quiet,
        )?;
    }

    let schemes_repo_path = hooks_path.join(SCHEMES_REPO_NAME);

    update_item(
        SCHEMES_REPO_NAME,
        SCHEMES_REPO_URL,
        &schemes_repo_path,
        Some(SCHEMES_REPO_REVISION),
        global_allow_dirty,
        is_quiet,
    )?;

    Ok(())
}
