use anyhow::Result;
use std::path::Path;

pub mod git_shell;

/// High-level repository operations tinty performs against a `[[items]]` entry:
/// fetching it onto disk for the first time (`install`), bringing it up to a
/// configured revision (`update`), and checking whether the local copy has
/// uncommitted changes (`is_clean`).
///
/// Implementations may shell out to the `git` binary, use a Rust-native git
/// library, or eventually back the operations with something other than git
/// (a local-path symlink backend, for example). Callers should not depend on
/// which implementation runs.
pub trait RepositoryBackend {
    fn install(&self, url: &str, target: &Path, revision: Option<&str>) -> Result<()>;
    fn update(&self, target: &Path, url: &str, revision: Option<&str>) -> Result<()>;
    fn is_clean(&self, target: &Path) -> Result<bool>;
}

/// Returns the active repository backend for this invocation.
///
/// Phase 1 always returns the git-shell-out backend. Phase 2 introduces the
/// gix backend and runtime dispatch via `TINTY_USE_GIX`.
#[must_use]
pub fn backend() -> Box<dyn RepositoryBackend> {
    Box::new(git_shell::GitShellBackend)
}

pub fn install(url: &str, target: &Path, revision: Option<&str>) -> Result<()> {
    backend().install(url, target, revision)
}

pub fn update(target: &Path, url: &str, revision: Option<&str>) -> Result<()> {
    backend().update(target, url, revision)
}

pub fn is_clean(target: &Path) -> Result<bool> {
    backend().is_clean(target)
}
