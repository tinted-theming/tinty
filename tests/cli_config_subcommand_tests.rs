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
    let (_, data_path, command_vec, cleanup) = setup("test_cli_config_without_config", "config")?;

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(stdout == expected, "std not as expected");

    cleanup()?;
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
    let (config_path, data_path, command_vec, cleanup) =
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

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_config_with_config_flag() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) =
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
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_config_with_data_flag() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
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
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}
