//! Integration tests for the `build` subcommand.
//!
//! Covers building a template against the synced schemes repo, including the
//! regression where non-scheme files at the schemes-repo root (e.g. `LICENSE`,
//! `README.md`) caused `build` to fail with `E111`.

mod utils;

use crate::utils::{build_command_vec, run_command, write_to_file, REPO_DIR, SCHEMES_REPO_NAME};
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Writes a minimal base16 schemes repo (one scheme) at `schemes_repo`.
fn write_schemes_repo(schemes_repo: &Path) -> Result<()> {
    let scheme = fs::read_to_string("./tests/fixtures/schemes/tinty-generated.yaml")?;
    write_to_file(
        schemes_repo.join("base16").join("tinty-generated.yaml"),
        &scheme,
    )?;

    Ok(())
}

/// Writes a minimal, valid base16 template repo at `item_path`. Its single
/// template emits `output/<item_name>-{{ scheme-slug }}.txt`.
fn write_template_item(item_path: &Path, item_name: &str) -> Result<()> {
    write_to_file(
        item_path.join("templates").join("config.yaml"),
        &format!(
            "{item_name}:\n  filename: output/{item_name}-{{{{ scheme-slug }}}}.txt\n  supported-systems: [base16]\n"
        ),
    )?;
    write_to_file(
        item_path
            .join("templates")
            .join(format!("{item_name}.mustache")),
        "{{scheme-name}}\n",
    )?;

    Ok(())
}

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
    let schemes_repo = data_path.join(REPO_DIR).join(SCHEMES_REPO_NAME);

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

#[test]
fn test_cli_build_no_dir_builds_every_installed_item() -> Result<()> {
    // -------
    // Arrange
    // -------
    let temp_dir = tempfile::Builder::new()
        .prefix("tinty-test-build-all-")
        .tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let data_path = temp_dir.path().join("data");
    let repos_path = data_path.join(REPO_DIR);
    let schemes_repo = repos_path.join(SCHEMES_REPO_NAME);

    write_schemes_repo(&schemes_repo)?;

    // Two installed template repos live under `<data>/repos/<name>`, exactly
    // where `install` would place them.
    write_template_item(&repos_path.join("item-one"), "item-one")?;
    write_template_item(&repos_path.join("item-two"), "item-two")?;

    write_to_file(
        &config_path,
        "[[items]]\n\
         name = \"item-one\"\n\
         path = \"https://example.com/item-one\"\n\
         themes-dir = \"templates\"\n\n\
         [[items]]\n\
         name = \"item-two\"\n\
         path = \"https://example.com/item-two\"\n\
         themes-dir = \"templates\"\n",
    )?;

    // ---
    // Act
    // ---
    let command_vec = build_command_vec("build", &config_path, &data_path)?;
    let (_stdout, stderr) = run_command(&command_vec)?;

    // ------
    // Assert
    // ------
    for item_name in ["item-one", "item-two"] {
        let output_path = repos_path
            .join(item_name)
            .join("output")
            .join(format!("{item_name}-tinty-generated.txt"));
        assert!(
            output_path.exists(),
            "expected build output at {}; stderr: {stderr}",
            output_path.display()
        );
    }

    Ok(())
}

#[test]
fn test_cli_build_no_dir_continues_after_failure() -> Result<()> {
    // -------
    // Arrange
    // -------
    let temp_dir = tempfile::Builder::new()
        .prefix("tinty-test-build-all-err-")
        .tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let data_path = temp_dir.path().join("data");
    let repos_path = data_path.join(REPO_DIR);
    let schemes_repo = repos_path.join(SCHEMES_REPO_NAME);

    write_schemes_repo(&schemes_repo)?;

    // The first item has a malformed template config, so its build fails. The
    // second is valid and must still be built despite the earlier failure.
    write_to_file(
        repos_path
            .join("broken-item")
            .join("templates")
            .join("config.yaml"),
        "this: is: not: valid: yaml:\n",
    )?;
    write_template_item(&repos_path.join("good-item"), "good-item")?;

    write_to_file(
        &config_path,
        "[[items]]\n\
         name = \"broken-item\"\n\
         path = \"https://example.com/broken-item\"\n\
         themes-dir = \"templates\"\n\n\
         [[items]]\n\
         name = \"good-item\"\n\
         path = \"https://example.com/good-item\"\n\
         themes-dir = \"templates\"\n",
    )?;

    // ---
    // Act
    // ---
    let command_vec = build_command_vec("build", &config_path, &data_path)?;
    let (_stdout, stderr) = run_command(&command_vec)?;

    // ------
    // Assert
    // ------
    // The valid item was built even though an earlier item failed.
    let good_output = repos_path
        .join("good-item")
        .join("output")
        .join("good-item-tinty-generated.txt");
    assert!(
        good_output.exists(),
        "expected good-item to build despite earlier failure; stderr: {stderr}"
    );

    // The failure is surfaced and names the offending item.
    assert!(
        stderr.contains("broken-item"),
        "expected failure to be reported for broken-item; stderr: {stderr}"
    );

    Ok(())
}
