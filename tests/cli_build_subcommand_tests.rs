//! Integration tests for the `build` subcommand.
//!
//! Covers building a template against the synced schemes repo, including the
//! regression where non-scheme files at the schemes-repo root (e.g. `LICENSE`,
//! `README.md`) caused `build` to fail with `E111`.

mod utils;

use crate::utils::{build_command_vec, builtin_schemes_repo_path, run_command, write_to_file};
use anyhow::Result;
use std::fs;

#[test]
fn test_cli_build_ignores_non_scheme_files_in_schemes_repo() -> Result<()> {
    // -------
    // Arrange
    // -------
    let temp_dir = tempfile::Builder::new()
        .prefix("tinty-test-build-")
        .tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let data_path = temp_dir.path().join("data");
    let schemes_repo = builtin_schemes_repo_path(&data_path);

    // A real base16 scheme under `base16/`, alongside non-scheme files at the
    // schemes-repo root that previously made `build` fail with E111.
    let scheme = fs::read_to_string("./tests/fixtures/schemes/tinty-generated.yaml")?;
    write_to_file(
        schemes_repo.join("base16").join("tinty-generated.yaml"),
        &scheme,
    )?;
    write_to_file(schemes_repo.join("LICENSE"), "license text")?;
    write_to_file(schemes_repo.join("README.md"), "# readme")?;

    // A minimal base16 template.
    let template_dir = temp_dir.path().join("template");
    write_to_file(
        template_dir.join("templates").join("config.yaml"),
        "base16-test:\n  filename: output/base16-{{ scheme-slug }}.txt\n  supported-systems: [base16]\n",
    )?;
    write_to_file(
        template_dir.join("templates").join("base16-test.mustache"),
        "{{scheme-name}}\n",
    )?;

    write_to_file(&config_path, "")?;

    // ---
    // Act
    // ---
    let command = format!("build \"{}\"", template_dir.display());
    let command_vec = build_command_vec(&command, &config_path, &data_path)?;
    let (_stdout, stderr) = run_command(&command_vec)?;

    // ------
    // Assert
    // ------
    let output_path = template_dir
        .join("output")
        .join("base16-tinty-generated.txt");
    assert!(
        output_path.exists(),
        "expected build output at {}; stderr: {stderr}",
        output_path.display()
    );
    assert!(!stderr.contains("E111"), "unexpected E111 error: {stderr}");

    Ok(())
}
