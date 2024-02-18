mod common;

use crate::common::{cleanup, COMMAND_NAME, REPO_NAME};
use anyhow::Result;
use std::fs;
use std::path::Path;

#[test]
fn test_cli_info_subcommand_all_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_info_subcommand_with_setup");
    let command = format!(
        "{} --config=\"{}\" info",
        COMMAND_NAME,
        config_path.display(),
    );
    let scheme_count = 250;
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    cleanup(config_path)?;
    fs::create_dir(config_path)?;

    // // ---
    // // Act
    // // ---
    common::run_install_command(config_path)?;
    let (stdout, _) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    assert!(
        stdout.contains("OceanicNext (base16-oceanicnext)"),
        "stdout does not contain the expected output"
    );
    assert!(
        stdout.lines().count() > (scheme_count * 16),
        "stdout does not contain the expected output"
    );

    cleanup(config_path)?;
    Ok(())
}

#[test]
fn test_cli_info_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_info_subcommand_with_setup");
    let scheme_name = "base16-oceanicnext";
    let command = format!(
        "{} --config=\"{}\" info {}",
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
    common::run_install_command(config_path)?;
    let (stdout, _) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    assert!(
        stdout.contains("OceanicNext (base16-oceanicnext)"),
        "stdout does not contain the expected output"
    );
    assert!(
        stdout.contains(" #1B2B34   #1B2B34"),
        "stdout does not contain the expected output"
    );

    cleanup(config_path)?;
    Ok(())
}

#[test]
fn test_cli_info_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_info_subcommand_without_setup");
    let command = format!(
        "{} --config=\"{}\" info",
        COMMAND_NAME,
        config_path.display(),
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    fs::create_dir(config_path)?;

    // // ---
    // // Act
    // // ---
    let (_, stderr) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    assert!(
        stderr.contains("Scheme repo path does not exist:"),
        "stderr does not contain the expected output"
    );
    assert!(
        stderr.contains("Run `tinty install` and try again"),
        "stderr does not contain the expected output"
    );

    cleanup(config_path)?;
    Ok(())
}

#[test]
fn test_cli_info_subcommand_with_setup_invalid_scheme_name() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_info_subcommand_with_setup_invalid_scheme_name");
    let scheme_name = "mocha";
    let command = format!(
        "{} --config=\"{}\" info {}",
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
    common::run_install_command(config_path)?;
    let (_, stderr) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
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

    cleanup(config_path)?;
    Ok(())
}
