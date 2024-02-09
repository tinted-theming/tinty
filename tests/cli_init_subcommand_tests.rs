mod common;

use crate::common::{cleanup, COMMAND_NAME, REPO_NAME};
use anyhow::Result;
use std::path::Path;

#[test]
fn test_cli_init_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_init_subcommand_without_setup");
    let expected_output = format!(
        "Failed to initialize, config files seem to be missing. Try setting a theme first with `{} set <SCHEME_NAME>`.",
        REPO_NAME
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
fn test_cli_init_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_init_subcommand_with_setup");
    let command = format!(
        "{} init --config=\"{}\"",
        COMMAND_NAME,
        config_path.display()
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;

    // // ---
    // // Act
    // // ---
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
