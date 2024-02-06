mod common;

extern crate dirs;

use crate::common::{cleanup, COMMAND_NAME, REPO_NAME};
use anyhow::Result;
use std::fs;
use std::path::Path;

#[test]
fn test_cli_set_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_set_subcommand");
    let scheme_name = "oceanicnext";
    let command = format!(
        "{} --config=\"{}\" set {}",
        COMMAND_NAME,
        config_path.display(),
        &scheme_name,
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    cleanup(config_path)?;
    fs::create_dir(config_path)?;

    // // ---
    // // Act
    // // ---
    common::run_setup_command(config_path)?;
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

#[test]
fn test_cli_set_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_set_subcommand");
    let scheme_name = "oceanicnext";
    let command = format!(
        "{} --config=\"{}\" set {}",
        COMMAND_NAME,
        config_path.display(),
        &scheme_name,
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    let expected_output = format!(
        "Themes files are missing, try running `{} setup` or `{} update` and try again.",
        REPO_NAME, REPO_NAME
    );
    fs::create_dir(config_path)?;

    // // ---
    // // Act
    // // ---
    let (_, stderr) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    assert!(
        stderr.contains(&expected_output),
        "stderr does not contain the expected output"
    );

    cleanup(config_path)?;
    Ok(())
}
