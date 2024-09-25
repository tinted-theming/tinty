mod utils;

use std::fs;
use std::path::Path;

use crate::utils::{
    read_file_to_string, setup, write_to_file, CURRENT_SCHEME_FILE_NAME, REPO_NAME,
};
use anyhow::Result;

#[test]
fn test_cli_apply_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_apply_subcommand_with_setup",
        format!("apply {}", &scheme_name).as_str(),
    )?;
    let shell_theme_filename = "tinted-shell-scripts-file.sh";
    let current_scheme_path = data_path.join(CURRENT_SCHEME_FILE_NAME);

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (stdout, _) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.is_empty(),
        "stdout does not contain the expected output"
    );
    assert!(
        data_path.join(shell_theme_filename).exists(),
        "Path does not exist"
    );
    assert_eq!(read_file_to_string(&current_scheme_path)?, scheme_name);

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_apply_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (_, _, command_vec, cleanup) = setup(
        "test_cli_apply_subcommand_without_setup",
        format!("apply {}", &scheme_name).as_str(),
    )?;
    let expected_output = format!(
        "Schemes do not exist, run install and try again: `{} install`",
        REPO_NAME
    );

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stderr.contains(&expected_output),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_apply_subcommand_invalid_scheme_name() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-invalid-scheme";
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_apply_subcommand_invalid_scheme_name",
        format!("apply {}", &scheme_name).as_str(),
    )?;
    let expected_output = format!("Scheme does not exist: {}", scheme_name);

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (_, stderr) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stderr.contains(&expected_output),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_apply_subcommand_invalid_scheme_system() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "some-invalid-scheme";
    let (_, _, command_vec, cleanup) = setup(
        "test_cli_apply_subcommand_invalid_scheme_system",
        format!("apply {}", &scheme_name).as_str(),
    )?;
    let expected_output = format!("Invalid scheme name. Make sure your scheme is prefixed with a supprted system (\"base16\" or \"base24\"), eg: base16-{}", scheme_name);

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    cleanup()?;
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
    let scheme_name = "ocean";
    let (_, _, command_vec, cleanup) = setup(
        "test_cli_apply_subcommand_no_scheme_system",
        format!("apply {}", &scheme_name).as_str(),
    )?;
    let expected_output = "Invalid scheme name. Make sure the scheme system is prefixed <SCHEME_SYSTEM>-<SCHEME_NAME>, eg: `base16-ayu-dark`";

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    cleanup()?;
    assert!(
        stderr.contains(expected_output),
        "stderr does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_with_custom_schemes() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_system = "base16";
    let scheme_name = "tinty-generated";
    let scheme_name_with_system = format!("{}-{}", scheme_system, scheme_name);
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_apply_subcommand_with_custom_schemes",
        format!("apply {}", &scheme_name_with_system).as_str(),
    )?;
    let custom_scheme_file_path =
        data_path.join(format!("custom-schemes/base16/{}.yaml", scheme_name));
    let expected_output = format!(
        r#"Successfully generated "base16" themes for "base16" at "{}/repos/tinted-shell/scripts/*.sh"#,
        data_path.display()
    );
    let current_scheme_path = data_path.join(CURRENT_SCHEME_FILE_NAME);
    let scheme_content =
        fs::read_to_string(Path::new("./tests/fixtures/schemes/tinty-generated.yaml"))?;
    write_to_file(&custom_scheme_file_path, &scheme_content)?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (stdout, stderr) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert_eq!(
        fs::read_to_string(current_scheme_path)?,
        scheme_name_with_system,
    );
    assert!(
        stdout.contains(&expected_output),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_apply_subcommand_with_custom_schemes_quiet_flag() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_system = "base16";
    let scheme_name = "tinty-generated";
    let scheme_name_with_system = format!("{}-{}", scheme_system, scheme_name);
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_apply_subcommand_with_custom_schemes_quiet_flag",
        format!("apply {} --quiet", &scheme_name_with_system).as_str(),
    )?;
    let custom_scheme_file_path =
        data_path.join(format!("custom-schemes/base16/{}.yaml", scheme_name));
    let current_scheme_path = data_path.join(CURRENT_SCHEME_FILE_NAME);
    let scheme_content =
        fs::read_to_string(Path::new("./tests/fixtures/schemes/tinty-generated.yaml"))?;
    write_to_file(&custom_scheme_file_path, &scheme_content)?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (stdout, stderr) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert_eq!(
        fs::read_to_string(current_scheme_path)?,
        scheme_name_with_system,
    );
    assert!(
        stdout.is_empty(),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_apply_subcommand_root_hooks_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_apply_subcommand_root_hooks_with_setup",
        format!("apply {}", &scheme_name).as_str(),
    )?;
    let expected_output = "This\nis\nexpected\noutput.";
    let config_content =
        r#"hooks = ["echo 'This'", "echo 'is'", "echo 'expected'", "echo 'output.'"]"#;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (stdout, stderr) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains(expected_output),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_apply_subcommand_hook_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_apply_subcommand_hook_with_setup",
        format!("apply {}", &scheme_name).as_str(),
    )?;
    let config_content = r##"
[[items]]
path = "https://github.com/tinted-theming/base16-vim"
name = "tinted-vim"
themes-dir = "colors"
hook = "echo \"path: %f\""
"##;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (stdout, stderr) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout
            .contains(format!("path: {}/tinted-vim-colors-file.vim", data_path.display()).as_str()),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_apply_subcommand_with_config_theme_file_extension() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-uwunicorn";
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_apply_subcommand_with_custom_schemes",
        format!("apply {}", &scheme_name).as_str(),
    )?;
    let config_content = r#"
[[items]]
path = "https://github.com/tinted-theming/tinted-alacritty"
name = "tinted-alacritty"
themes-dir = "colors"
hook = "echo \"expected alacritty output: %n\""

[[items]]
name = "base16-emacs"
path = "https://github.com/tinted-theming/base16-emacs"
theme-file-extension="-theme.el"
themes-dir="build"
hook = "echo \"expected emacs output: %n\""
"#;
    let expected_output =
        "expected alacritty output: base16-uwunicorn\nexpected emacs output: base16-uwunicorn\n";
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (stdout, stderr) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    assert_eq!(stdout, expected_output,);
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}
