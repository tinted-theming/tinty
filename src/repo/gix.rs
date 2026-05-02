#![allow(clippy::module_name_repetitions)]

use crate::repo::RepositoryBackend;
use anyhow::{bail, Context, Result};
use std::path::Path;

/// Repository backend implemented with the pure-Rust `gix` (gitoxide) library.
pub struct GixBackend;

impl RepositoryBackend for GixBackend {
    fn install(&self, _url: &str, _target: &Path, _revision: Option<&str>) -> Result<()> {
        bail!(
            "gix backend: install is not yet implemented (set TINTY_USE_GIX=0 to use the git CLI)"
        )
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
