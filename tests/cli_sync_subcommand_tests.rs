mod utils;

use crate::utils::{setup, write_to_file};
use anyhow::Result;

#[test]
fn test_cli_sync_subcommand_non_unique_config_item_name() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_sync_subcommand_non_unique_config_item_name",
        "sync",
    )?;
    let config_content = r##"[[items]]
path = "https://github.com/tinted-theming/tinted-shell"
name = "non-unique-name"
themes-dir = "some-dir"

[[items]]
path = "https://github.com/tinted-theming/tinted-shell"
name = "non-unique-name"
themes-dir = "some-dir"
"##;
    let expected_output = "config.toml item.name should be unique values, but \"non-unique-name\" is used for more than 1 item.name. Please change this to a unique value.";
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------
    cleanup()?;
    assert!(
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
    let (config_path, data_path, command_vec, cleanup) =
        setup("test_cli_sync_subcommand_invalid_config_item_path", "sync")?;
    let config_content = r##"[[items]]
path = "/path/to/non-existant/directory"
name = "some-name"
themes-dir = "some-dir""##;
    let expected_output = "One of your config.toml items has an invalid `path` value. \"/path/to/non-existant/directory\" is not a valid url and is not a path to an existing local directory";
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------
    cleanup()?;
    assert!(
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
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_sync_subcommand_without_setup", "sync")?;
    let expected_output = "tinted-shell installed";

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(command_vec, &data_path, false).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains(expected_output),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_sync_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_sync_subcommand_with_setup", "sync")?;
    let expected_output = "tinted-shell already installed";

    // ---
    // Act
    // ---
    utils::run_command(command_vec.clone(), &data_path, true).unwrap();
    let (stdout, _) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------

    assert!(
        stdout.contains(expected_output),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_sync_subcommand_with_setup_quiet_flag() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) = setup(
        "test_cli_sync_subcommand_with_setup_quiet_flag",
        "sync --quiet",
    )?;

    // ---
    // Act
    // ---
    utils::run_command(command_vec.clone(), &data_path, true).unwrap();
    let (stdout, stderr) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.is_empty(),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}
