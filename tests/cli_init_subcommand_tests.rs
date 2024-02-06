mod common;

extern crate dirs;

use crate::common::{cleanup, COMMAND_NAME, REPO_NAME};
use anyhow::Result;
use std::path::Path;

#[test]
fn test_cli_init_subcommand_no_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_init_subcommand_existing_config");
    let expected_output = format!(
        "Themes files are missing, try running `{} setup` or `{} update` and try again.",
        REPO_NAME, REPO_NAME,
    );
    let command = format!(
        "{} init --config=\"{}\"",
        COMMAND_NAME,
        config_path.display()
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    cleanup(config_path)?;

    // // ---
    // // Act
    // // ---
    let (_, stderr) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    assert!(
        stderr.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    cleanup(config_path)?;
    Ok(())
}

#[test]
fn test_cli_init_subcommand_existing_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_init_subcommand_existing_config");
    let command = format!(
        "{} init --config=\"{}\"",
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
        stdout.is_empty(),
        "stdout does not contain the expected output"
    );

    cleanup(config_path)?;
    Ok(())
}
