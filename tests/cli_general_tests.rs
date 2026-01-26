mod utils;

use crate::utils::{
    cleanup, setup, write_to_file, COMMAND_NAME, CURRENT_SCHEME_FILE_NAME, ORG_NAME, REPO_NAME,
};
use anyhow::{ensure, Result};
use std::{fs, path::PathBuf};

#[test]
fn test_cli_no_arguments() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup_setup) = setup("test_cli_no_arguments", "")?;

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(stdout.contains(format!("Basic usage: {REPO_NAME} apply <SCHEME_NAME>").as_str()));
    ensure!(stdout.contains("For more information try --help"));

    cleanup_setup()?;
    Ok(())
}

#[test]
fn test_cli_config_path_tilde_as_home() -> Result<()> {
    // -------
    // Arrange
    // -------
    let name = "test_cli_config_path_tilde_as_home";
    let config_path_name = format!("config_path_{name}");
    let home_path = home::home_dir().unwrap();
    let config_path = home_path.join(&config_path_name);
    let provided_config_path = format!("~/{config_path_name}");
    let data_path = PathBuf::from(format!("data_path_{name}"));
    let command = format!(
        "{COMMAND_NAME} --config=\"{provided_config_path}\" --data-dir=\"{}\" init",
        data_path.display()
    );
    let command_vec = shell_words::split(command.as_str())?;
    let expected_stdout = "test_cli_config_path_tilde_as_home_config_output";
    let config_content = r#"default-scheme = "base16-mocha"
[[items]]
path = "https://github.com/tinted-theming/tinted-vim"
name = "tinted-vim"
themes-dir = "colors"
hook = "echo 'test_cli_config_path_tilde_as_home_config_output'"
"#;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        stdout.contains(expected_stdout),
        "stdout does not contain the expected output"
    );
    ensure!(
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
    let config_path = PathBuf::from("test_cli_default_data_path.toml");
    let scheme_name = "base16-uwunicorn";
    let init_scheme_name = "base16-mocha";
    let xdg_dirs = xdg::BaseDirectories::with_prefix(format!("{ORG_NAME}/{REPO_NAME}"))?;
    let data_path = xdg_dirs.get_data_home();
    let init_command = format!("{COMMAND_NAME} --config=\"{}\" init", config_path.display(),);
    let init_command_vec = shell_words::split(init_command.as_str())?;
    let apply_command = format!(
        "{COMMAND_NAME} --config=\"{}\" apply {scheme_name}",
        config_path.display(),
    );
    let current_scheme_file_path = data_path.join(CURRENT_SCHEME_FILE_NAME);
    let apply_command_vec = shell_words::split(apply_command.as_str())?;
    write_to_file(
        &config_path,
        format!("default-scheme = \"{init_scheme_name}\"").as_str(),
    )?;
    if current_scheme_file_path.exists() {
        fs::remove_file(&current_scheme_file_path)?;
    }

    // ---
    // Act
    // ---
    utils::run_command(&init_command_vec, &data_path, true)?;

    // This test is important to determine the config.toml is being read correctly
    ensure!(
        fs::read_to_string(&current_scheme_file_path)? == init_scheme_name,
        "current_scheme_file_path not as expected"
    );

    utils::run_command(&apply_command_vec, &data_path, true)?;
    utils::run_command(&init_command_vec, &data_path, true)?;
    let (stdout, stderr) = utils::run_command(&apply_command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    fs::remove_file(&config_path)?; // cleanup
    ensure!(
        data_path.join("repos/tinted-shell").exists(),
        "stdout does not contain the expected output"
    );
    ensure!(
        fs::read_to_string(data_path.join(CURRENT_SCHEME_FILE_NAME))? == scheme_name,
        "current_scheme_file_path not as expected"
    );
    ensure!(
        stdout.is_empty(),
        "stdout does not contain the expected output"
    );
    ensure!(
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
    let home_path = home::home_dir().unwrap();
    let config_path = PathBuf::from(format!("{data_path_name}.toml"));
    let data_path = home_path.join(data_path_name);
    let provided_data_path = format!("~/{data_path_name}");
    let command = format!(
        "{COMMAND_NAME} --config=\"{}\" --data-dir=\"{provided_data_path}\" apply base16-mocha",
        config_path.display(),
    );
    let command_vec = shell_words::split(command.as_str())?;
    write_to_file(&config_path, "")?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        data_path.join("repos/tinted-shell").exists(),
        "stdout does not contain the expected output"
    );
    ensure!(
        stdout.is_empty(),
        "stdout does not contain the expected output"
    );
    ensure!(
        stderr.is_empty(),
        "stdout does not contain the expected output"
    );

    cleanup(&config_path, &data_path)?;
    Ok(())
}
