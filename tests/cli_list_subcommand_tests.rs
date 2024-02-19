mod common;

use crate::common::REPO_NAME;
use anyhow::Result;
use common::setup;
use std::fs;
use std::path::Path;

#[test]
fn test_cli_list_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) =
        setup("test_cli_list_subcommand_with_setup", "list")?;
    let expected_output = fs::read_to_string(Path::new("fixtures/schemes.txt"))?;

    // // ---
    // // Act
    // // ---
    common::run_install_command(&config_path, &data_path)?;
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

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_list_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, _, command_vec, cleanup) = setup("test_cli_list_subcommand_without_setup", "list")?;
    let expected_output = format!(
        "Schemes are missing, run install and then try again: `{} install`",
        REPO_NAME
    );

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

    cleanup()?;
    Ok(())
}
