mod common;

use crate::common::{cleanup, COMMAND_NAME};
use anyhow::Result;
use std::path::Path;

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
