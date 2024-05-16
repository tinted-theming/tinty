mod utils;

use crate::utils::{setup, write_to_file, REPO_NAME};
use anyhow::Result;

#[test]
fn test_cli_info_subcommand_all_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) =
        setup("test_cli_info_subcommand_all_with_setup", "info")?;
    let scheme_count = 250;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (stdout, _) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains("OceanicNext (base16-oceanicnext)"),
        "stdout does not contain the expected output"
    );
    assert!(
        stdout.lines().count() > (scheme_count * 16),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_info_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_info_subcommand_with_setup",
        format!("info {}", scheme_name).as_str(),
    )?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (stdout, _) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains("OceanicNext (base16-oceanicnext)"),
        "stdout does not contain the expected output"
    );
    assert!(
        stdout.contains(" #1B2B34   #1B2B34"),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_info_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, _, command_vec, cleanup) =
        setup("test_cli_info_subcommand_without_setup", "info")?;
    write_to_file(&config_path, "")?;

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stderr.contains("Scheme repo path does not exist:"),
        "stderr does not contain the expected output"
    );
    assert!(
        stderr.contains("Run `tinty install` and try again"),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_info_subcommand_with_setup_invalid_scheme_name() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "mocha";
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_info_subcommand_with_setup_invalid_scheme_name",
        format!("info {}", scheme_name).as_str(),
    )?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (_, stderr) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stderr.contains(
            format!(
                r##"Invalid scheme system: "{}" from scheme name "{}"
Make sure to add the system prefix to the theme name. Eg: base16-oceanicnext
Run `{} list` to get a list of scheme names"##,
                scheme_name, scheme_name, REPO_NAME
            )
            .as_str()
        ),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}
