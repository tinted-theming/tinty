mod common;

use crate::common::{
    cleanup, read_file_to_string, setup, write_to_file, COMMAND_NAME, CURRENT_SCHEME_FILE_NAME,
    REPO_NAME,
};
use anyhow::{anyhow, Result};
use std::{fs, path::PathBuf};

#[test]
fn test_cli_no_arguments() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, _, command_vec, cleanup_setup) = setup("test_cli_no_arguments", "")?;

    // // ---
    // // Act
    // // ---
    let (stdout, _) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    assert!(stdout.contains(format!("Basic usage: {} apply <SCHEME_NAME>", REPO_NAME).as_str()));
    assert!(stdout.contains("For more information try --help"));

    cleanup_setup()?;
    Ok(())
}

#[test]
fn test_cli_config_path_tilde_as_home() -> Result<()> {
    // -------
    // Arrange
    // -------
    let name = "test_cli_config_path_tilde_as_home";
    let config_path_name = format!("config_path_{}", name);
    let home_path = dirs::home_dir().unwrap();
    let config_path = PathBuf::from(home_path.join(&config_path_name));
    let provided_config_path = format!("~/{}", config_path_name);
    let data_path = PathBuf::from(format!("data_path_{}", name));
    let command = format!(
        "{} --config=\"{}\" --data-dir=\"{}\" init",
        COMMAND_NAME,
        provided_config_path,
        data_path.display()
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    let expected_stdout = "test_cli_config_path_tilde_as_home_config_output";
    let config_content = r##"default-scheme = "base16-mocha"
[[items]]
path = "https://github.com/tinted-theming/base16-vim"
name = "tinted-vim"
themes-dir = "colors"
hook = "echo 'test_cli_config_path_tilde_as_home_config_output'"
"##;
    write_to_file(&config_path, config_content)?;

    // // ---
    // // Act
    // // ---
    common::run_install_command(&config_path, &data_path)?;
    let (stdout, stderr) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    assert!(
        stdout.contains(expected_stdout),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stdout does not contain the expected output"
    );

    cleanup(&config_path, &data_path)?;
    Ok(())
}

#[test]
fn test_cli_default_data_path() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = PathBuf::from(format!("{}.toml", "test_cli_default_data_path"));
    let scheme_name = "base16-uwunicorn";
    let init_scheme_name = "base16-mocha";
    let data_path = dirs::data_dir()
        .ok_or_else(|| anyhow!("Error getting data directory"))?
        .join(format!("tinted-theming/{}", REPO_NAME));
    let init_command = format!(
        "{} --config=\"{}\" init",
        COMMAND_NAME,
        config_path.display(),
    );
    let init_command_vec = shell_words::split(init_command.as_str()).map_err(anyhow::Error::new)?;
    let apply_command = format!(
        "{} --config=\"{}\" apply {}",
        COMMAND_NAME,
        config_path.display(),
        &scheme_name,
    );
    let apply_command_vec =
        shell_words::split(apply_command.as_str()).map_err(anyhow::Error::new)?;
    write_to_file(
        &config_path,
        format!("default-scheme = \"{}\"", init_scheme_name).as_str(),
    )?;

    // // ---
    // // Act
    // // ---
    common::run_install_command(&config_path, &data_path)?;
    common::run_command(init_command_vec).unwrap();
    assert_eq!(
        read_file_to_string(&data_path.join(CURRENT_SCHEME_FILE_NAME))?,
        scheme_name
    );
    let (stdout, stderr) = common::run_command(apply_command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    fs::remove_file(&config_path)?; // cleanup
    assert!(
        data_path.join("repos/base16-shell").exists(),
        "stdout does not contain the expected output"
    );
    assert_eq!(
        read_file_to_string(&data_path.join(CURRENT_SCHEME_FILE_NAME))?,
        scheme_name
    );
    assert!(
        stdout.is_empty(),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stdout does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_data_path_tilde_as_home() -> Result<()> {
    // -------
    // Arrange
    // -------
    let data_path_name = "test_cli_data_path_tilde_as_home";
    let home_path = dirs::home_dir().unwrap();
    let config_path = PathBuf::from(format!("{}.toml", data_path_name));
    let data_path = PathBuf::from(home_path.join(data_path_name));
    let provided_data_path = format!("~/{}", data_path_name);
    let command = format!(
        "{} --config=\"{}\" --data-dir=\"{}\" apply base16-mocha",
        COMMAND_NAME,
        config_path.display(),
        provided_data_path
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;
    write_to_file(&config_path, "")?;

    // // ---
    // // Act
    // // ---
    common::run_install_command(&config_path, &data_path)?;
    let (stdout, stderr) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    assert!(
        data_path.join("repos/base16-shell").exists(),
        "stdout does not contain the expected output"
    );
    assert!(
        stdout.is_empty(),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stdout does not contain the expected output"
    );

    cleanup(&config_path, &data_path)?;
    Ok(())
}
