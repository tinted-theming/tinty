mod common;

use crate::common::{setup, write_to_file, CURRENT_SCHEME_FILE_NAME};
use anyhow::Result;

#[test]
fn test_cli_current_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_current_subcommand_with_setup", "current")?;
    let scheme_name = "base16-oceanicnext";
    let current_scheme_path = data_path.join(CURRENT_SCHEME_FILE_NAME);
    write_to_file(&current_scheme_path, scheme_name)?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = common::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains(scheme_name),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_current_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, _, command_vec, cleanup) =
        setup("test_cli_current_subcommand_without_setup", "current")?;

    // ---
    // Act
    // ---
    let (_, stderr) = common::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    cleanup()?;
    assert!(
        stderr
            .contains("Failed to read last scheme from file. Try applying a scheme and try again."),
        "stderr does not contain the expected output"
    );

    Ok(())
}
