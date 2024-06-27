use anyhow::Result;
use std::path::Path;
use tinted_builder_rust::operation_build;

/// Builds the provided template using tinted_builder_rust
pub fn build(template_path: &Path, schemes_repo_path: &Path) -> Result<()> {
    operation_build::build(template_path, schemes_repo_path, false)?;

    Ok(())
}
