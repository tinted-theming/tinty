use anyhow::Result;
use std::path::Path;

pub mod git_shell;
pub mod gix;

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

/// Environment variable for opting in to the gix backend at runtime.
///
/// Truthy values: `1`, `true`, `TRUE`, `yes`. Anything else (including unset)
/// keeps the default git-shell-out backend.
const USE_GIX_ENV: &str = "TINTY_USE_GIX";

fn use_gix() -> bool {
    std::env::var(USE_GIX_ENV)
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes"))
        .unwrap_or(false)
}

/// Returns the active repository backend for this invocation.
///
/// Defaults to the git-shell-out backend. Set `TINTY_USE_GIX=1` to opt in to
/// the gix backend (currently a stub — Phase 2 of the migration).
#[must_use]
pub fn backend() -> Box<dyn RepositoryBackend> {
    if use_gix() {
        Box::new(gix::GixBackend)
    } else {
        Box::new(git_shell::GitShellBackend)
    }
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
