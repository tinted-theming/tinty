use crate::config::{ensure_schemes_path_not_circular, Config};
use crate::constants::{REPO_DIR, REPO_NAME, SCHEMES_REPO_NAME};
use crate::repo::{self, UpdateStatus};
use anyhow::{Context, Result};
use std::path::Path;
use url::Url;

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
/// left untouched, so we echo git's own message verbatim — it already names
/// the offending files and how to proceed.
fn print_conflict_message(item_name: &str, git_stderr: &str) {
    println!("{item_name}: could not update — your local changes are preserved:");
    for line in git_stderr.lines() {
        println!("{line}");
    }
}

/// Updates the built-in schemes repository from its configured source.
///
/// A Git URL source is pulled to its revision exactly like an item. A local-path
/// source is a live symlink to the user's directory, so there is nothing to
/// fetch and `revision` is irrelevant — we only confirm the symlink is in place.
fn update_schemes_repo(
    schemes_repo_path: &Path,
    source: &str,
    revision: Option<&str>,
    allow_dirty: bool,
    is_quiet: bool,
) -> Result<()> {
    if Url::parse(source).is_ok() {
        update_item(
            SCHEMES_REPO_NAME,
            source,
            schemes_repo_path,
            revision,
            allow_dirty,
            is_quiet,
        )
    } else {
        if !is_quiet {
            if schemes_repo_path.exists() {
                println!("{SCHEMES_REPO_NAME} up to date (local directory)");
            } else {
                println!("{SCHEMES_REPO_NAME} not installed (run `{REPO_NAME} install`)");
            }
        }
        Ok(())
    }
}

/// Updates local files
///
/// Updates the provided repositories in config file by doing a git pull
pub fn update(config_path: &Path, data_path: &Path, is_quiet: bool) -> Result<()> {
    let config = Config::read(config_path)?;
    // The built-in schemes repo has no `[[items]]` entry, so its leniency is
    // configured separately under `[schemes]`.
    let schemes_allow_dirty = config.schemes.allow_dirty_update;
    let (schemes_source, schemes_revision) = config.schemes_source();
    let items = config.items.unwrap_or_default();
    let hooks_path = data_path.join(REPO_DIR);

    for item in items {
        let item_path = hooks_path.join(&item.name);

        update_item(
            item.name.as_str(),
            item.path.as_str(),
            &item_path,
            item.revision.as_deref(),
            item.allow_dirty_update,
            is_quiet,
        )?;
    }

    let schemes_repo_path = hooks_path.join(SCHEMES_REPO_NAME);

    ensure_schemes_path_not_circular(&schemes_source, &schemes_repo_path)?;
    update_schemes_repo(
        &schemes_repo_path,
        &schemes_source,
        schemes_revision.as_deref(),
        schemes_allow_dirty,
        is_quiet,
    )?;

    Ok(())
}
