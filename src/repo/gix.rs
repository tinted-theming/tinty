#![allow(clippy::module_name_repetitions)]

use crate::repo::RepositoryBackend;
use anyhow::{bail, Result};
use std::path::Path;

/// Repository backend implemented with the pure-Rust `gix` (gitoxide) library.
///
/// Phase 2 stub: every method returns an error so the dispatcher path is
/// reachable but the operations are explicitly not yet wired up. Real
/// implementations land in Phase 3.
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

    fn is_clean(&self, _target: &Path) -> Result<bool> {
        bail!(
            "gix backend: is_clean is not yet implemented (set TINTY_USE_GIX=0 to use the git CLI)"
        )
    }
}
