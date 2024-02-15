mod common;

use crate::common::{cleanup, read_file_to_string, write_to_file, COMMAND_NAME, REPO_NAME};
use anyhow::{anyhow, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn test_cli_init_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_init_subcommand_without_setup");
    let expected_output = format!(
        "Failed to initialize, config files seem to be missing. Try setting a theme first with `{} set <SCHEME_NAME>`.",
        REPO_NAME
    );
    let command = format!(
        "{} init --config=\"{}\"",
        COMMAND_NAME,
        config_path.display()
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    cleanup(config_path)?;

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

    cleanup(config_path)?;
    Ok(())
}

#[test]
fn test_cli_init_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_init_subcommand_with_setup");
    let command = format!(
        "{} init --config=\"{}\"",
        COMMAND_NAME,
        config_path.display()
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;

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

    cleanup(config_path)?;
    Ok(())
}

#[test]
fn test_cli_init_subcommand_with_config_default_scheme() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_init_subcommand_with_config_default_scheme");
    let system_data_path: PathBuf =
        dirs::data_dir().ok_or_else(|| anyhow!("Error getting data directory"))?;
    let data_dir = system_data_path.join(format!("tinted-theming/{}", REPO_NAME));
    let config_file_path = config_path.join("config.toml");
    let scheme_name = "base16-mocha";
    let command = format!(
        "{} init --config=\"{}\"",
        COMMAND_NAME,
        config_path.display()
    );
    let config_content = format!("default-scheme = \"{}\"", scheme_name);
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    cleanup(config_path)?;
    fs::create_dir(config_path)?;
    write_to_file(&config_file_path, config_content.as_str())?;

    // // ---
    // // Act
    // // ---
    common::run_setup_command(config_path)?;
    let (stdout, stderr) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    let expected_scheme_name = read_file_to_string(&data_dir.join("current_scheme"))?;
    assert!(
        stdout.is_empty(),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stdout does not contain the expected output"
    );
    assert_eq!(scheme_name, expected_scheme_name);

    cleanup(config_path)?;
    Ok(())
}
