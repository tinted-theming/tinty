use crate::config::{ensure_schemes_path_not_circular, Config};
use crate::constants::{REPO_DIR, SCHEMES_REPO_NAME};
use crate::repo;
use anyhow::{anyhow, Context, Result};
use std::fs::{remove_file as remove_symlink, symlink_metadata};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use url::Url;

fn install_git_url(
    data_item_path: &Path,
    item_name: &str,
    item_git_url: &str,
    revision: Option<&str>,
    is_quiet: bool,
) -> Result<()> {
    if !data_item_path.is_dir() {
        repo::install(item_git_url, data_item_path, revision)?;

        if !is_quiet {
            println!("{item_name} installed");
        }
    } else if !is_quiet {
        println!("{item_name} already installed");
    }

    Ok(())
}

// TODO(repo-backend): non-URL `[[items]]` (a local path on disk) take this
// symlink-based path instead of going through `RepositoryBackend`. A future
// `LocalPathBackend` implementing `RepositoryBackend` would let us route both
// branches of the dispatch in `install()` below through a single trait, with
// `is_clean` returning a sensible answer for symlinked source dirs and
// `update` becoming a no-op (or re-validating the symlink target).
fn install_dir(
    data_item_path: &Path,
    item_name: &str,
    item_path: &Path,
    is_quiet: bool,
) -> Result<()> {
    if item_path.exists() && !item_path.is_dir() {
        return Err(anyhow!(
            "{} is not a symlink to a directory. Please remove it and try again",
            item_path.display()
        ));
    }

    if data_item_path.exists() {
        match symlink_metadata(data_item_path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    if remove_symlink(data_item_path).is_err() {
                        return Err(anyhow!("Error trying to remove symlink at \"{}\". Remove it manually and try again", data_item_path.display()));
                    }
                    symlink(item_path, data_item_path)?;

                    if !is_quiet {
                        println!("{item_name} already installed");
                    }
                }
            }
            Err(_) => {
                return Err(anyhow!("\"{}\" is a not a symlink, but according to your config it should be. Please remove this directory and try again", data_item_path.display()));
            }
        }
    } else {
        symlink(item_path, data_item_path)?;

        if !is_quiet {
            println!("{item_name} installed");
        }
    }

    Ok(())
}

/// The kind of entry the managed `repos/schemes` slot should hold, derived from
/// the configured schemes source.
enum SchemesSlotKind {
    /// A Git clone of a remote URL.
    Clone,
    /// A symlink to a local directory.
    Symlink,
}

/// Reconciles the `repos/schemes` slot with the desired kind before
/// (re)installing it. When the configured schemes source switches between a Git
/// URL and a local path, the previously-installed form is removed so the correct
/// one can take its place. A matching kind — or an empty slot — is left
/// untouched, so the normal `install_git_url`/`install_dir` fast paths still
/// run.
fn reconcile_schemes_slot(schemes_repo_path: &Path, want: &SchemesSlotKind) -> Result<()> {
    let Ok(metadata) = symlink_metadata(schemes_repo_path) else {
        // Nothing occupies the slot yet.
        return Ok(());
    };
    let is_symlink = metadata.file_type().is_symlink();

    match (want, is_symlink) {
        // Want a clone, but a symlink to a local dir is here: drop the symlink.
        (SchemesSlotKind::Clone, true) => remove_symlink(schemes_repo_path).with_context(|| {
            format!(
                "Failed to remove existing schemes symlink at {}",
                schemes_repo_path.display()
            )
        }),
        // Want a symlink, but a real directory (a clone) is here: remove it.
        (SchemesSlotKind::Symlink, false) => std::fs::remove_dir_all(schemes_repo_path)
            .with_context(|| {
                format!(
                    "Failed to remove existing schemes clone at {}",
                    schemes_repo_path.display()
                )
            }),
        // Already the right kind; leave it for the installer to refresh.
        _ => Ok(()),
    }
}

/// Installs the built-in schemes repository from its configured source. A Git
/// URL is cloned into `repos/schemes`; a local directory is symlinked as
/// `repos/schemes` (with `revision` ignored, exactly like a local-path
/// `[[items]]` entry).
fn install_schemes_repo(
    schemes_repo_path: &Path,
    source: &str,
    revision: Option<&str>,
    is_quiet: bool,
) -> Result<()> {
    if Url::parse(source).is_ok() {
        reconcile_schemes_slot(schemes_repo_path, &SchemesSlotKind::Clone)?;
        install_git_url(
            schemes_repo_path,
            SCHEMES_REPO_NAME,
            source,
            revision,
            is_quiet,
        )
    } else {
        reconcile_schemes_slot(schemes_repo_path, &SchemesSlotKind::Symlink)?;
        install_dir(
            schemes_repo_path,
            SCHEMES_REPO_NAME,
            Path::new(source),
            is_quiet,
        )
    }
}

/// Install cli tool
///
/// Clones the provided config repositories and ensures everything is ready for when the user runs
/// any other command
pub fn install(config_path: &Path, data_path: &Path, is_quiet: bool) -> Result<()> {
    let config = Config::read(config_path)?;
    let (schemes_source, schemes_revision) = config.schemes_source();
    let items = config.items.unwrap_or_default();
    let hooks_path = data_path.join(REPO_DIR);

    for item in items {
        let data_item_path = hooks_path.join(&item.name);
        let item_path = PathBuf::from(item.path.as_str());

        match Url::parse(item.path.as_str()) {
            Ok(_) => install_git_url(
                &data_item_path,
                item.name.as_str(),
                item.path.as_str(),
                item.revision.as_deref(),
                is_quiet,
            )?,
            Err(_) => install_dir(&data_item_path, item.name.as_str(), &item_path, is_quiet)?,
        }
    }

    let schemes_repo_path = hooks_path.join(SCHEMES_REPO_NAME);

    ensure_schemes_path_not_circular(&schemes_source, &schemes_repo_path)?;
    install_schemes_repo(
        &schemes_repo_path,
        &schemes_source,
        schemes_revision.as_deref(),
        is_quiet,
    )?;

    Ok(())
}
