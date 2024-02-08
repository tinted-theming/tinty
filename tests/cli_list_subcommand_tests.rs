mod common;

extern crate dirs;

use crate::common::{cleanup, COMMAND_NAME, REPO_NAME};
use anyhow::Result;
use std::fs;
use std::path::Path;

#[test]
fn test_cli_list_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_list_subcommand");
    let expected_output = fs::read_to_string(Path::new("fixtures/schemes.txt"))?;
    let command = format!(
        "{} list --config=\"{}\"",
        COMMAND_NAME,
        config_path.display()
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;

    // // ---
    // // Act
    // // ---
    common::run_setup_command(config_path)?;
    let (stdout, _) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    // The sort order of the schemes differ slightly so do an assert on each line instead of the
    // whole file
    let lines: Vec<&str> = expected_output.lines().collect();
    for line in lines {
        assert!(
            stdout.contains(line),
            "stdout does not contain the expected output"
        );
    }

    cleanup(config_path)?;
    Ok(())
}

#[test]
fn test_cli_list_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_list_subcommand");
    let expected_output = format!(
        "Error: Scheme files are missing. Run `{} setup` and try again.",
        REPO_NAME
    );
    let command = format!(
        "{} list --config=\"{}\"",
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
