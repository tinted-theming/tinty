mod utils;

use std::fs;

use crate::utils::{setup, write_to_file, CURRENT_SCHEME_FILE_NAME, REPO_NAME};
use anyhow::Result;
use utils::ARTIFACTS_DIR;

#[test]
fn test_cli_init_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_init_subcommand_without_setup", "init")?;
    let expected_output = format!(
        "Failed to initialize, config files seem to be missing. Try applying a theme first with `{} apply <SCHEME_NAME>`.",
        REPO_NAME
    );

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stderr.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_init_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_init_subcommand_with_setup", "init")?;

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.is_empty(),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_init_subcommand_with_config_default_scheme() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_init_subcommand_with_config_default_scheme",
        "init",
    )?;
    let scheme_name = "base16-mocha";
    let config_content = format!("default-scheme = \"{}\"", scheme_name);
    write_to_file(&config_path, config_content.as_str())?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------
    let expected_scheme_name =
        fs::read_to_string(data_path.join(ARTIFACTS_DIR).join(CURRENT_SCHEME_FILE_NAME))?;
    assert!(
        stdout.is_empty(),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stdout does not contain the expected output"
    );
    assert_eq!(scheme_name, expected_scheme_name);

    cleanup()?;
    Ok(())
}
