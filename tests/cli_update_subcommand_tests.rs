mod utils;

use crate::utils::{setup, REPO_NAME};
use anyhow::Result;

#[test]
fn test_cli_update_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, _, command_vec, cleanup) = setup(
        "test_cli_install_subcommand_non_unique_config_item_name",
        "update",
    )?;
    let expected_output = format!("tinted-shell not installed (run `{} install`)", REPO_NAME);

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    cleanup()?;
    assert!(
        stdout.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_update_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) =
        setup("test_cli_update_subcommand_with_setup", "update")?;
    let expected_output = "tinted-shell up to date";

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (stdout, _) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    cleanup()?;
    assert!(
        stdout.contains(expected_output),
        "stdout does not contain the expected output"
    );

    Ok(())
}
