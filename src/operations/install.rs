use crate::config::{ensure_schemes_path_not_circular, Config};
use crate::constants::SCHEMES_REPO_NAME;
use crate::paths;
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

/// Normalizes a Git URL for equality checks by trimming a trailing slash and a
/// `.git` suffix, so `.../schemes`, `.../schemes/`, and `.../schemes.git` all
/// compare equal. Used only to decide whether an existing clone already points
/// at the configured source; the stored URL is never rewritten.
fn git_url_eq(a: &str, b: &str) -> bool {
    fn normalize(url: &str) -> &str {
        let url = url.trim().trim_end_matches('/');
        url.strip_suffix(".git").unwrap_or(url)
    }
    normalize(a) == normalize(b)
}

/// Prepares the `repos/schemes` slot to hold a fresh clone of `source`. Removes
/// a symlink left by a previous local-path source, or a stale clone whose
/// `origin` points at a different URL, so `install_git_url` re-clones from the
/// now-configured source. A clone already pointing at `source` (or an empty
/// slot) is left untouched, so the common case does no extra work.
fn prepare_clone_slot(schemes_repo_path: &Path, source: &str) -> Result<()> {
    let Ok(metadata) = symlink_metadata(schemes_repo_path) else {
        return Ok(()); // Nothing occupies the slot yet.
    };

    if metadata.file_type().is_symlink() {
        return remove_symlink(schemes_repo_path).with_context(|| {
            format!(
                "Failed to remove existing schemes symlink at {}",
                schemes_repo_path.display()
            )
        });
    }

    // A real directory: an existing clone. Re-clone only when its origin no
    // longer matches the configured source.
    let matches_source =
        repo::origin_url(schemes_repo_path)?.is_some_and(|origin| git_url_eq(&origin, source));
    if !matches_source {
        std::fs::remove_dir_all(schemes_repo_path).with_context(|| {
            format!(
                "Failed to remove existing schemes clone at {}",
                schemes_repo_path.display()
            )
        })?;
    }

    Ok(())
}

/// Prepares the `repos/schemes` slot to hold a symlink to a local directory.
/// Removes a clone left by a previous Git-URL source so `install_dir` can create
/// the symlink. An existing symlink (or empty slot) is left for `install_dir` to
/// refresh.
fn prepare_symlink_slot(schemes_repo_path: &Path) -> Result<()> {
    let Ok(metadata) = symlink_metadata(schemes_repo_path) else {
        return Ok(()); // Nothing occupies the slot yet.
    };

    if metadata.file_type().is_symlink() {
        return Ok(());
    }

    std::fs::remove_dir_all(schemes_repo_path).with_context(|| {
        format!(
            "Failed to remove existing schemes clone at {}",
            schemes_repo_path.display()
        )
    })
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
        prepare_clone_slot(schemes_repo_path, source)?;
        install_git_url(
            schemes_repo_path,
            SCHEMES_REPO_NAME,
            source,
            revision,
            is_quiet,
        )
    } else {
        prepare_symlink_slot(schemes_repo_path)?;
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

    for item in items {
        let data_item_path = paths::item_repo_path(data_path, &item.name);
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

    let schemes_repo_path = paths::schemes_repo_path(data_path);

    ensure_schemes_path_not_circular(&schemes_source, &schemes_repo_path)?;
    install_schemes_repo(
        &schemes_repo_path,
        &schemes_source,
        schemes_revision.as_deref(),
        is_quiet,
    )?;

    Ok(())
}
