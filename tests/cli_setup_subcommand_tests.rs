mod common;

use crate::common::{cleanup, write_to_file, COMMAND_NAME};
use anyhow::Result;
use std::{fs, path::Path};

#[test]
fn test_cli_setup_subcommand_invalid_config_item_path() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_setup_subcommand_invalid_config_item_path");
    let config_file_path = config_path.join("config.toml");
    let command = format!(
        "{} setup --config=\"{}\"",
        COMMAND_NAME,
        config_path.display()
    );
    let config_content = r##"[[items]]
path = "/path/to/non-existant/directory"
name = "some-name"
themes_dir = "some-dir""##;
    let expected_output = "One of your config.toml items has an invalid `path` value. \"/path/to/non-existant/directory\" is not a valid url and is not a path to an existing local directory";
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    cleanup(config_path)?;
    fs::create_dir(config_path)?;
    write_to_file(&config_file_path, config_content)?;

    // // ---
    // // Act
    // // ---
    let (_, stderr) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    cleanup(config_path)?;
    assert!(
        stderr.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_setup_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_setup_subcommand_without_setup");
    let expected_output = "base16-shell installed";
    let command = format!(
        "{} setup --config=\"{}\"",
        COMMAND_NAME,
        config_path.display()
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    cleanup(config_path)?;

    // // ---
    // // Act
    // // ---
    let (stdout, _) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    assert!(
        stdout.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    cleanup(config_path)?;
    Ok(())
}

#[test]
fn test_cli_setup_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_setup_subcommand_with_setup");
    let expected_output = "base16-shell already installed";
    let command = format!(
        "{} setup --config=\"{}\"",
        COMMAND_NAME,
        config_path.display()
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;

    // // ---
    // // Act
    // // ---
    common::run_command(command_vec.clone()).unwrap();
    let (stdout, _) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------

    assert!(
        stdout.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    cleanup(config_path)?;
    Ok(())
}
