//! General CLI integration tests not specific to a single subcommand.
//!
//! Covers: no-arguments help output, tilde (`~`) expansion for `--config`
//! and `--data-dir` paths, and default XDG data directory behaviour.
//!
//! Requires network access on first run (repos are cached in `tmp/repos/`).

mod utils;

use crate::utils::{
    clone_test_repos, setup, write_to_file, COMMAND_NAME, CURRENT_SCHEME_FILE_NAME, ORG_NAME,
    REPO_NAME,
};
use anyhow::{ensure, Result};
use std::fs;

#[test]
fn test_cli_no_arguments() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, _, command_vec, _temp_dir) = setup("test_cli_no_arguments", "", true)?;

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(&command_vec)?;

    // ------
    // Assert
    // ------
    ensure!(stdout.contains(format!("Basic usage: {REPO_NAME} apply <SCHEME_NAME>").as_str()));
    ensure!(stdout.contains("For more information try --help"));

    Ok(())
}

#[test]
fn test_cli_config_path_tilde_as_home() -> Result<()> {
    // -------
    // Arrange
    // -------
    let home_path = home::home_dir().unwrap();
    let home_temp_dir = tempfile::Builder::new()
        .prefix(".tinty-test-config-tilde-")
        .tempdir_in(&home_path)?;
    let config_path = home_temp_dir.path().join("config.toml");
    let dir_name = home_temp_dir.path().file_name().unwrap().to_str().unwrap();
    let provided_config_path = format!("~/{dir_name}/config.toml");
    let data_temp_dir = tempfile::Builder::new()
        .prefix("tinty-test-config-tilde-data-")
        .tempdir()?;
    let data_path = data_temp_dir.path().join("data");
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
    clone_test_repos(&data_path)?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec)?;

    // ------
    // Assert
    // ------
    ensure!(
        stdout.contains(expected_stdout),
        "Expected stdout to contain: {expected_stdout}\nGot: {stdout}"
    );
    ensure!(stderr.is_empty(), "Expected empty stderr, got: {stderr}");

    Ok(())
}

#[test]
fn test_cli_default_data_path() -> Result<()> {
    // -------
    // Arrange
    // -------
    let temp_dir = tempfile::Builder::new()
        .prefix("tinty-test-default-data-")
        .tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let xdg_data_home = temp_dir.path().join("share");
    let data_path = xdg_data_home.join(ORG_NAME).join(REPO_NAME);
    let scheme_name = "base16-uwunicorn";
    let init_scheme_name = "base16-mocha";
    let init_command = format!("{COMMAND_NAME} --config=\"{}\" init", config_path.display());
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
    let env_vars: &[(&str, &str)] = &[("XDG_DATA_HOME", xdg_data_home.to_str().unwrap())];
    clone_test_repos(&data_path)?;

    // ---
    // Act
    // ---
    utils::run_command_with_env(&init_command_vec, env_vars)?;

    // This test is important to determine the config.toml is being read correctly
    ensure!(
        fs::read_to_string(&current_scheme_file_path)? == init_scheme_name,
        "current_scheme_file_path not as expected"
    );

    utils::run_command_with_env(&apply_command_vec, env_vars)?;
    utils::run_command_with_env(&init_command_vec, env_vars)?;
    let (stdout, stderr) = utils::run_command_with_env(&apply_command_vec, env_vars)?;

    // ------
    // Assert
    // ------
    ensure!(
        data_path.join("repos/tinted-shell").exists(),
        "Expected tinted-shell repo to exist at: {}",
        data_path.join("repos/tinted-shell").display()
    );
    ensure!(
        fs::read_to_string(data_path.join(CURRENT_SCHEME_FILE_NAME))? == scheme_name,
        "current_scheme_file_path not as expected"
    );
    ensure!(stdout.is_empty(), "Expected empty stdout, got: {stdout}");
    ensure!(stderr.is_empty(), "Expected empty stderr, got: {stderr}");

    Ok(())
}

#[test]
fn test_cli_data_path_tilde_as_home() -> Result<()> {
    // -------
    // Arrange
    // -------
    let home_path = home::home_dir().unwrap();
    let home_temp_dir = tempfile::Builder::new()
        .prefix(".tinty-test-data-tilde-")
        .tempdir_in(&home_path)?;
    let data_path = home_temp_dir.path().to_path_buf();
    let dir_name = home_temp_dir.path().file_name().unwrap().to_str().unwrap();
    let provided_data_path = format!("~/{dir_name}");
    let config_temp_dir = tempfile::Builder::new()
        .prefix("tinty-test-data-tilde-config-")
        .tempdir()?;
    let config_path = config_temp_dir.path().join("config.toml");
    let command = format!(
        "{COMMAND_NAME} --config=\"{}\" --data-dir=\"{provided_data_path}\" apply base16-mocha",
        config_path.display(),
    );
    let command_vec = shell_words::split(command.as_str())?;
    write_to_file(&config_path, "")?;
    clone_test_repos(&data_path)?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec)?;

    // ------
    // Assert
    // ------
    ensure!(
        data_path.join("repos/tinted-shell").exists(),
        "Expected tinted-shell repo to exist at: {}",
        data_path.join("repos/tinted-shell").display()
    );
    ensure!(stdout.is_empty(), "Expected empty stdout, got: {stdout}");
    ensure!(stderr.is_empty(), "Expected empty stderr, got: {stderr}");

    Ok(())
}
