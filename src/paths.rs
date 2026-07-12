//! Centralized derivation of Tinty's on-disk data-directory layout.
//!
//! Every operation locates installed template `[[items]]` repositories under
//! `<data_dir>/repos/`. These helpers are the single source of truth for that
//! layout, so call sites don't each re-spell the `repos/<name>` join
//! (previously duplicated across nearly every operation). Scheme repositories
//! (the built-in `schemes` repo and any `[[schemes.extras]]`) live under
//! `<data_dir>/scheme-repos/` and are derived by the `scheme_repos` module.
//!
//! These take an already-resolved `data_path` (tilde expansion for the data
//! directory happens once at startup in `main`); they do no `~` expansion of
//! their own.

use crate::constants::REPO_DIR;
use std::path::{Path, PathBuf};

/// The directory holding every installed template repository: `<data_dir>/repos`.
pub fn repos_dir(data_path: &Path) -> PathBuf {
    data_path.join(REPO_DIR)
}

/// The local path of an installed `[[items]]` template repository, by item
/// name: `<data_dir>/repos/<name>`.
pub fn item_repo_path(data_path: &Path, item_name: &str) -> PathBuf {
    repos_dir(data_path).join(item_name)
}
