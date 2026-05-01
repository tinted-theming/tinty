use anyhow::Result;
use std::path::Path;

pub mod cli;

/// Operations tinty performs against a git repository (clone, update, status).
///
/// Implementations may shell out to the `git` binary or use a Rust-native
/// library; callers should not depend on which one runs.
pub trait GitBackend {
    fn clone_repo(&self, url: &str, target: &Path, revision: Option<&str>) -> Result<()>;
    fn update_repo(&self, repo: &Path, url: &str, revision: Option<&str>) -> Result<()>;
    fn is_working_dir_clean(&self, repo: &Path) -> Result<bool>;
}

/// Returns the active git backend for this invocation.
///
/// Phase 1 always returns the CLI shell-out backend. Phase 2 introduces the
/// gix backend and runtime dispatch via `TINTY_USE_GIX`.
#[must_use]
pub fn backend() -> Box<dyn GitBackend> {
    Box::new(cli::CliBackend)
}

pub fn clone_repo(url: &str, target: &Path, revision: Option<&str>) -> Result<()> {
    backend().clone_repo(url, target, revision)
}

pub fn update_repo(repo: &Path, url: &str, revision: Option<&str>) -> Result<()> {
    backend().update_repo(repo, url, revision)
}

pub fn is_working_dir_clean(repo: &Path) -> Result<bool> {
    backend().is_working_dir_clean(repo)
}
