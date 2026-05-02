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

    fn is_clean(&self, target: &Path) -> Result<bool> {
        let repo = gix::open(target)
            .with_context(|| format!("Failed to open git repository at {}", target.display()))?;
        // `is_dirty` reports any change to tracked files plus any untracked
        // (non-ignored) files — equivalent to `git status --porcelain` being
        // non-empty, which is the contract the CLI backend implements.
        let dirty = repo
            .is_dirty()
            .with_context(|| format!("Failed to read status in {}", target.display()))?;
        Ok(!dirty)
    }
}
