mod common;

extern crate dirs;

use crate::common::{cleanup, COMMAND_NAME, REPO_NAME};
use anyhow::Result;
use std::path::Path;

#[test]
fn test_cli_no_arguments() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_no_arguments");
    let command = format!("{} --config=\"{}\"", COMMAND_NAME, config_path.display());
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;

    // // ---
    // // Act
    // // ---
    let (stdout, _) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    assert!(stdout.contains(format!("Basic usage: {} set <SCHEME_NAME>", REPO_NAME).as_str()));
    assert!(stdout.contains("For more information try --help"));

    cleanup(config_path)?;
    Ok(())
}
