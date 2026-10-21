use anyhow::Result;
use std::path::Path;

pub mod git_shell;
pub mod gix;

/// Outcome of an `update` that was allowed to run against a dirty working tree.
#[derive(Debug, PartialEq, Eq)]
pub enum UpdateStatus {
    /// The repository was brought to the configured revision. Any
    /// non-overlapping local changes were carried forward untouched.
    Updated,
    /// The update was refused because it would have overwritten local work.
    /// The working tree was left exactly as it was; `stderr` is git's own
    /// explanation, which names the offending files and how to proceed.
    ConflictPreserved { stderr: String },
}

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
    /// Bring the local copy at `target` to `revision`. When `allow_dirty` is
    /// `false` the caller guarantees a clean working tree. When `true` the
    /// update may run against uncommitted changes: non-overlapping edits are
    /// carried forward and a would-be-overwrite is reported via
    /// [`UpdateStatus::ConflictPreserved`] rather than an error.
    fn update(
        &self,
        target: &Path,
        url: &str,
        revision: Option<&str>,
        allow_dirty: bool,
    ) -> Result<UpdateStatus>;
    fn is_clean(&self, target: &Path) -> Result<bool>;
    /// Returns the URL of the `origin` remote for the repository at `target`,
    /// or `None` when `target` is not a git repository or has no such remote.
    fn origin_url(&self, target: &Path) -> Result<Option<String>>;
}

/// Environment variable for opting in to the gix backend at runtime.
///
/// Truthy values: `1`, `true`, `TRUE`, `yes`. Anything else (including unset)
/// keeps the default git-shell-out backend.
const USE_GIX_ENV: &str = "TINTY_USE_GIX";

fn use_gix() -> bool {
    std::env::var(USE_GIX_ENV).is_ok_and(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes"))
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

pub fn update(
    target: &Path,
    url: &str,
    revision: Option<&str>,
    allow_dirty: bool,
) -> Result<UpdateStatus> {
    backend().update(target, url, revision, allow_dirty)
}

pub fn is_clean(target: &Path) -> Result<bool> {
    backend().is_clean(target)
}

pub fn origin_url(target: &Path) -> Result<Option<String>> {
    backend().origin_url(target)
}
