//! Integration tests for the `sync` subcommand.
//!
//! Covers: config validation (non-unique item names, invalid paths), fresh
//! and repeated syncs, and `--quiet` flag. Sync combines install + update.
//!
//! Requires network access on first run (repos are cached in `tmp/repos/`).

mod utils;

use crate::utils::{setup, write_to_file};
use anyhow::{ensure, Result};

#[test]
fn test_cli_sync_subcommand_non_unique_config_item_name() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, _data_path, command_vec, _temp_dir) = setup(
        "test_cli_sync_subcommand_non_unique_config_item_name",
        "sync",
        true,
    )?;
    let config_content = r#"[[items]]
path = "https://github.com/tinted-theming/tinted-shell"
name = "non-unique-name"
themes-dir = "some-dir"

[[items]]
path = "https://github.com/tinted-theming/tinted-shell"
name = "non-unique-name"
themes-dir = "some-dir"
"#;
    let expected_output = "config.toml item.name should be unique values, but \"non-unique-name\" is used for more than 1 item.name. Please change this to a unique value.";
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(&command_vec)?;

    // ------
    // Assert
    // ------
    ensure!(
        stderr.contains(expected_output),
        "stdout does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_sync_subcommand_invalid_config_item_path() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, _data_path, command_vec, _temp_dir) = setup(
        "test_cli_sync_subcommand_invalid_config_item_path",
        "sync",
        true,
    )?;
    let config_content = r#"[[items]]
path = "/path/to/non-existant/directory"
name = "some-name"
themes-dir = "some-dir""#;
    let expected_output = "One of your config.toml items has an invalid `path` value. \"/path/to/non-existant/directory\" is not a valid url and is not a path to an existing local directory";
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(&command_vec)?;

    // ------
    // Assert
    // ------
    ensure!(
        stderr.contains(expected_output),
        "stdout does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_sync_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, _data_path, command_vec, _temp_dir) =
        setup("test_cli_sync_subcommand_without_setup", "sync", false)?;
    let expected_output = "tinted-shell installed";

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(&command_vec)?;

    // ------
    // Assert
    // ------
    ensure!(
        stdout.contains(expected_output),
        "Expected stdout to contain: {expected_output}\nGot: {stdout}"
    );

    Ok(())
}

#[test]
fn test_cli_sync_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, _data_path, command_vec, _temp_dir) =
        setup("test_cli_sync_subcommand_with_setup", "sync", true)?;
    let expected_output = "tinted-shell already installed";

    // ---
    // Act
    // ---
    utils::run_command(&command_vec)?;
    let (stdout, _) = utils::run_command(&command_vec)?;

    // ------
    // Assert
    // ------

    ensure!(
        stdout.contains(expected_output),
        "Expected stdout to contain: {expected_output}\nGot: {stdout}"
    );

    Ok(())
}

#[test]
fn test_cli_sync_subcommand_with_setup_quiet_flag() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, _data_path, command_vec, _temp_dir) = setup(
        "test_cli_sync_subcommand_with_setup_quiet_flag",
        "sync --quiet",
        true,
    )?;

    // ---
    // Act
    // ---
    utils::run_command(&command_vec)?;
    let (stdout, stderr) = utils::run_command(&command_vec)?;

    // ------
    // Assert
    // ------
    ensure!(stdout.is_empty(), "Expected empty stdout, got: {stdout}");
    ensure!(stderr.is_empty(), "Expected empty stderr, got: {stderr}");

    Ok(())
}
