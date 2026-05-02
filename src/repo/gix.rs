#![allow(clippy::module_name_repetitions)]

use crate::repo::RepositoryBackend;
use anyhow::{anyhow, bail, Context, Result};
use std::path::Path;
use std::sync::atomic::AtomicBool;

/// Repository backend implemented with the pure-Rust `gix` (gitoxide) library.
pub struct GixBackend;

impl RepositoryBackend for GixBackend {
    fn install(&self, url: &str, target: &Path, revision: Option<&str>) -> Result<()> {
        if target.exists() {
            return Err(anyhow!(
                "Error cloning {}. Target directory '{}' already exists",
                url,
                target.display()
            ));
        }

        if revision.is_some() {
            bail!(
                "gix backend: install with a pinned revision is not yet implemented \
                 (set TINTY_USE_GIX=0 to use the git CLI)"
            );
        }

        let interrupt = AtomicBool::new(false);
        let mut prepare = gix::prepare_clone(url, target)
            .with_context(|| format!("Failed to set up clone of {url}"))?;
        let (mut checkout, _outcome) = prepare
            .fetch_then_checkout(gix::progress::Discard, &interrupt)
            .with_context(|| format!("Failed to clone repository from {url}"))?;
        checkout
            .main_worktree(gix::progress::Discard, &interrupt)
            .with_context(|| format!("Failed to check out worktree at {}", target.display()))?;
        Ok(())
    }

    fn update(&self, _target: &Path, _url: &str, _revision: Option<&str>) -> Result<()> {
        bail!("gix backend: update is not yet implemented (set TINTY_USE_GIX=0 to use the git CLI)")
    }

    /// Reports whether the working directory is clean.
    ///
    /// Intentional divergence from the CLI backend: untracked files do **not**
    /// count as dirty here. The CLI backend uses `git status --porcelain` which
    /// reports untracked entries; this implementation excludes them so that
    /// artifacts written by `tinty generate-scheme` (and similar) into a cloned
    /// template repo don't block `tinty update` / `tinty sync`. See
    /// tinted-theming/tinty#130.
    ///
    /// Modified, deleted, renamed, or conflicted entries on tracked files all
    /// still count as dirty, matching the CLI backend.
    fn is_clean(&self, target: &Path) -> Result<bool> {
        let repo = gix::open(target)
            .with_context(|| format!("Failed to open git repository at {}", target.display()))?;
        let mut iter = repo
            .status(gix::progress::Discard)
            .with_context(|| format!("Failed to read status in {}", target.display()))?
            .untracked_files(gix::status::UntrackedFiles::None)
            .into_iter(None)
            .with_context(|| format!("Failed to iterate status in {}", target.display()))?;
        let any_change = iter.next().is_some();
        Ok(!any_change)
    }
}
