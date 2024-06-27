mod utils;

use crate::utils::{setup, write_to_file};
use anyhow::Result;

#[test]
fn test_cli_install_subcommand_non_unique_config_item_name() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, _, command_vec, cleanup) = setup(
        "test_cli_install_subcommand_non_unique_config_item_name",
        "install",
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
    let (_, stderr) = utils::run_command(command_vec).unwrap();

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
fn test_cli_install_subcommand_invalid_config_item_path() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, _, command_vec, cleanup) = setup(
        "test_cli_install_subcommand_invalid_config_item_path",
        "install",
    )?;
    let config_content = r##"[[items]]
path = "/path/to/non-existant/directory"
name = "some-name"
themes-dir = "some-dir""##;
    let expected_output = "One of your config.toml items has an invalid `path` value. \"/path/to/non-existant/directory\" is not a valid url and is not a path to an existing local directory";
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(command_vec).unwrap();

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
fn test_cli_install_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, _, command_vec, cleanup) =
        setup("test_cli_install_subcommand_without_setup", "install")?;
    let expected_output = "base16-shell installed";

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(command_vec).unwrap();

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
fn test_cli_install_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, _, command_vec, cleanup) = setup("test_cli_install_subcommand_with_setup", "install")?;
    let expected_output = "base16-shell already installed";

    // ---
    // Act
    // ---
    utils::run_command(command_vec.clone()).unwrap();
    let (stdout, _) = utils::run_command(command_vec).unwrap();

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
