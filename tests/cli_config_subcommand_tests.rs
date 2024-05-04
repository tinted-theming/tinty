mod common;

use crate::common::{setup, write_to_file};
use anyhow::Result;

#[test]
fn test_cli_config_without_config() -> Result<()> {
    // -------
    // Arrange
    // -------
    let expected = r#"shell = "sh -c '{}'"

[[items]]
name = "base16-shell"
path = "https://github.com/tinted-theming/base16-shell"
hook = ". %f"
supported-systems = ["base16"]
themes-dir = "scripts"

"#;
    let (_, _, command_vec, cleanup) = setup("test_cli_config_without_config", "config")?;

    // ---
    // Act
    // ---
    let (stdout, _) = common::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert_eq!(stdout, expected);

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
    let (config_path, _, command_vec, cleanup) = setup("test_cli_config_with_config", "config")?;

    write_to_file(&config_path, config_text)?;

    // ---
    // Act
    // ---
    let (stdout, _) = common::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert_eq!(stdout, config_text);

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_config_with_config_flag() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, _, command_vec, cleanup) =
        setup("test_cli_config_with_config_flag", "config --config-path")?;

    // ---
    // Act
    // ---
    let (stdout, _) = common::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert!(
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
        setup("test_cli_config_with_data_flag", "config --data-path")?;

    // ---
    // Act
    // ---
    let (stdout, _) = common::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains(format!("{}", data_path.display()).as_str()),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}
