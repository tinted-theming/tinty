//! Integration tests for the `config` subcommand.
//!
//! Covers: displaying default config, custom config, `--config-path` and
//! `--data-dir-path` flags.

mod utils;

use crate::utils::{setup, write_to_file};
use anyhow::{ensure, Result};

#[test]
fn test_cli_config_without_config() -> Result<()> {
    // -------
    // Arrange
    // -------
    let expected = r#"shell = "sh -c '{}'"

[[items]]
name = "tinted-shell"
path = "https://github.com/tinted-theming/tinted-shell"
hook = ". %f"
supported-systems = ["base16"]
themes-dir = "scripts"

"#;
    let (_, data_path, command_vec, _temp_dir) = setup("test_cli_config_without_config", "config")?;

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(stdout == expected, "std not as expected");

    Ok(())
}

#[test]
fn test_cli_config_with_config() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_text = r#"shell = "zsh -c '{}'"
default-scheme = "base16-oceanicnext"

[[items]]
name = "tinted-vim"
path = "https://github.com/tinted-theming/tinted-vim"
supported-systems = ["base16", "base24"]
themes-dir = "colors"

"#;
    let (config_path, data_path, command_vec, _temp_dir) =
        setup("test_cli_config_with_config", "config")?;

    write_to_file(&config_path, config_text)?;

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(stdout == config_text, "std not as expected");

    Ok(())
}

#[test]
fn test_cli_config_with_rings() -> Result<()> {
    let config_text = r#"shell = "zsh -c '{}'"
default-scheme = "base16-oceanicnext"
default-cycle-ring = "default"
hooks = [
  "echo hook"
]

[[rings]]
name = "default"
schemes = ["base16-oceanicnext", "base16-gruvbox-dark"]

[[rings]]
name = "light"
schemes = ["base16-github", "base16-gruvbox-light"]

[[items]]
name = "tinted-vim"
path = "https://github.com/tinted-theming/tinted-vim"
supported-systems = ["base16", "base24"]
themes-dir = "colors"

"#;
    let (config_path, data_path, command_vec, _temp_dir) =
        setup("test_cli_config_with_rings", "config")?;

    write_to_file(&config_path, config_text)?;

    let (stdout, _) = utils::run_command(&command_vec, &data_path, true)?;

    ensure!(stdout == config_text, "std not as expected");
    ensure!(
        stdout.find("default-cycle-ring").unwrap_or_default()
            < stdout.find("[[rings]]").unwrap_or_default(),
        "default-cycle-ring should be printed before [[rings]]"
    );
    ensure!(
        stdout.find("hooks = [").unwrap_or_default() < stdout.find("[[rings]]").unwrap_or_default(),
        "hooks should be printed before [[rings]]"
    );

    Ok(())
}

#[test]
fn test_cli_config_with_config_flag() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, _temp_dir) =
        setup("test_cli_config_with_config_flag", "config --config-path")?;

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        stdout.contains(format!("{}", config_path.display()).as_str()),
        "Expected stdout to contain config path: {}\nGot: {stdout}",
        config_path.display()
    );

    Ok(())
}

#[test]
fn test_cli_config_with_data_flag() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, _temp_dir) =
        setup("test_cli_config_with_data_flag", "config --data-dir-path")?;

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        stdout.contains(format!("{}", data_path.display()).as_str()),
        "Expected stdout to contain data path: {}\nGot: {stdout}",
        data_path.display()
    );

    Ok(())
}
