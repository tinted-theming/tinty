mod common;

use crate::common::{cleanup, write_to_file, COMMAND_NAME, REPO_NAME};
use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn test_cli_current_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_current_subcommand_with_setup");
    let scheme_name = "base16-oceanicnext";
    let command = format!(
        "{} --config=\"{}\" current",
        COMMAND_NAME,
        config_path.display(),
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    let system_data_path: PathBuf =
        dirs::data_dir().ok_or_else(|| anyhow!("Error getting data directory"))?;
    let data_dir = system_data_path.join(format!("tinted-theming/{}", REPO_NAME));
    let current_scheme_path = data_dir.join("current_scheme");
    cleanup(config_path)?;
    if !config_path.exists() {
        fs::create_dir(config_path)?;
    }
    write_to_file(&current_scheme_path, scheme_name)?;

    // // ---
    // // Act
    // // ---
    let (stdout, stderr) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    assert!(
        stdout.contains(scheme_name),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup(config_path)?;
    Ok(())
}

#[test]
fn test_cli_current_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_current_subcommand_without_setup");
    let command = format!(
        "{} --config=\"{}\" current",
        COMMAND_NAME,
        config_path.display(),
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    cleanup(config_path)?;
    fs::create_dir(config_path)?;

    // // ---
    // // Act
    // // ---
    let (_, stderr) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    cleanup(config_path)?;
    assert!(
        stderr
            .contains("Failed to read last scheme from file. Try applying a scheme and try again."),
        "stderr does not contain the expected output"
    );

    Ok(())
}
