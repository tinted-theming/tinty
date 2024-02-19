mod common;

use crate::common::{
    read_file_to_string, setup, write_to_file, CURRENT_SCHEME_FILE_NAME, REPO_NAME,
};
use anyhow::Result;

#[test]
fn test_cli_init_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, _, command_vec, cleanup) = setup("test_cli_init_subcommand_without_setup", "init")?;
    let expected_output = format!(
        "Failed to initialize, config files seem to be missing. Try applying a theme first with `{} apply <SCHEME_NAME>`.",
        REPO_NAME
    );

    // // ---
    // // Act
    // // ---
    let (_, stderr) = common::run_command(command_vec).unwrap();
    println!("stderr: {}", stderr);
    println!("exptectedc: {}", expected_output);

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

#[test]
fn test_cli_init_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, _, command_vec, cleanup) = setup("test_cli_init_subcommand_with_setup", "init")?;

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

    // // ---
    // // Act
    // // ---
    common::run_install_command(&config_path, &data_path)?;
    let (stdout, stderr) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    let expected_scheme_name = read_file_to_string(&data_path.join(CURRENT_SCHEME_FILE_NAME))?;
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
