mod common;

use crate::common::{cleanup, read_file_to_string, write_to_file, COMMAND_NAME, REPO_NAME};
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

#[test]
fn test_cli_apply_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_apply_subcommand_with_setup.toml");
    let scheme_name = "base16-oceanicnext";
    let command = format!(
        "{} --config=\"{}\" apply {}",
        COMMAND_NAME,
        config_path.display(),
        &scheme_name,
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    let system_data_path: PathBuf =
        dirs::data_dir().ok_or_else(|| anyhow!("Error getting data directory"))?;
    let data_dir = system_data_path.join(format!("tinted-theming/{}", REPO_NAME));
    let shell_theme_filename = "base16-shell-scripts-file.sh";
    let current_scheme_path = data_dir.join("current_scheme");
    cleanup(config_path)?;
    write_to_file(config_path, "")?;

    // // ---
    // // Act
    // // ---
    common::run_install_command(config_path)?;
    let (stdout, _) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    assert!(
        stdout.is_empty(),
        "stdout does not contain the expected output"
    );
    assert!(
        data_dir.join(shell_theme_filename).exists(),
        "Path does not exist"
    );
    assert_eq!(read_file_to_string(&current_scheme_path)?, scheme_name);

    cleanup(config_path)?;
    Ok(())
}

#[test]
fn test_cli_apply_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_apply_subcommand_without_setup.toml");
    let scheme_name = "base16-oceanicnext";
    let command = format!(
        "{} --config=\"{}\" apply {}",
        COMMAND_NAME,
        config_path.display(),
        &scheme_name,
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    let expected_output = format!(
        "Schemes do not exist, run install and try again: `{} install`",
        REPO_NAME
    );
    write_to_file(config_path, "")?;

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

#[test]
fn test_cli_apply_subcommand_invalid_scheme_name() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_apply_subcommand_invalid_scheme_name.toml");
    let scheme_name = "base16-invalid-scheme";
    let command = format!(
        "{} --config=\"{}\" apply {}",
        COMMAND_NAME,
        config_path.display(),
        &scheme_name,
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    let expected_output = format!("Scheme does not exist: {}", scheme_name);
    write_to_file(config_path, "")?;

    // // ---
    // // Act
    // // ---
    common::run_install_command(config_path)?;
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

#[test]
fn test_cli_apply_subcommand_invalid_scheme_system() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_apply_subcommand_invalid_scheme_system.toml");
    let scheme_name = "some-invalid-scheme";
    let command = format!(
        "{} --config=\"{}\" apply {}",
        COMMAND_NAME,
        config_path.display(),
        &scheme_name,
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    let expected_output = format!("Invalid scheme name. Make sure your scheme is prefixed with a supprted system (\"base16\" or \"base24\"), eg: base16-{}", scheme_name);
    write_to_file(config_path, "")?;

    // // ---
    // // Act
    // // ---
    let (_, stderr) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    cleanup(config_path)?;
    assert!(
        stderr.contains(&expected_output),
        "stderr does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_no_scheme_system() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_apply_subcommand_no_scheme_system");
    let scheme_name = "ocean";
    let command = format!(
        "{} --config=\"{}\" apply {}",
        COMMAND_NAME,
        config_path.display(),
        &scheme_name,
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    let expected_output = "Invalid scheme name. Make sure the scheme system is prefixed <SCHEME_SYSTEM>-<SCHEME_NAME>, eg: `base16-ayu-dark`";
    write_to_file(config_path, "")?;

    // // ---
    // // Act
    // // ---
    let (_, stderr) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    cleanup(config_path)?;
    assert!(
        stderr.contains(&expected_output),
        "stderr does not contain the expected output"
    );

    Ok(())
}
