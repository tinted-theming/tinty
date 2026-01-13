mod utils;

use crate::utils::{setup, write_to_file, ARTIFACTS_DIR, CURRENT_SCHEME_FILE_NAME, REPO_NAME};
use anyhow::Result;

#[test]
fn test_cli_info_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_info_subcommand_all_with_setup", "info")?;
    let stdout_line_count = 24;
    let scheme_name = "base16-oceanicnext";
    let current_scheme_path = data_path.join(ARTIFACTS_DIR).join(CURRENT_SCHEME_FILE_NAME);
    write_to_file(&current_scheme_path, scheme_name)?;

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains("System: base16\nSlug: oceanicnext\nName: OceanicNext"),
        "stdout does not contain the expected output"
    );

    assert!(
        stdout.lines().count() == stdout_line_count,
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_info_subcommand_all_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_info_subcommand_all_with_setup", "info --all")?;
    let scheme_count = 250;

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains("System: base16\nSlug: oceanicnext\nName: OceanicNext"),
        "stdout does not contain the expected output"
    );
    assert!(
        stdout.lines().count() > (scheme_count * 16),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_info_subcommand_with_setup_for_base16() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (_, data_path, command_vec, cleanup) = setup(
        "test_cli_info_subcommand_with_setup_for_base16",
        format!("info {scheme_name}").as_str(),
    )?;

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains("| Color       | Name   | Hex     | ANSI     |"),
        "stdout does not contain the expected output"
    );
    assert!(
        stdout.contains("Name: OceanicNext"),
        "stdout does not contain the expected output"
    );
    assert!(
        stdout.contains("System: base16"),
        "stdout does not contain the expected output"
    );
    assert!(
        stdout.contains("Slug: oceanicnext"),
        "stdout does not contain the expected output"
    );
    assert!(
        stdout.contains(" | base00 | #1B2B34 | 0 "),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_info_subcommand_with_setup_for_base24() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base24-ayu-dark";
    let (_, data_path, command_vec, cleanup) = setup(
        "test_cli_info_subcommand_with_setup_for_base24",
        format!("info {scheme_name}").as_str(),
    )?;

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains("| Color       | Name   | Hex     | ANSI |"),
        "stdout does not contain the expected output"
    );
    assert!(
        stdout.contains("Name: Ayu Dark"),
        "stdout does not contain the expected output"
    );
    assert!(
        stdout.contains("System: base24"),
        "stdout does not contain the expected output"
    );
    assert!(
        stdout.contains("Slug: ayu-dark"),
        "stdout does not contain the expected output"
    );
    assert!(
        stdout.contains(" | base12 | #f26d78 | 9 "),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_info_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) =
        setup("test_cli_info_subcommand_without_setup", "info")?;
    write_to_file(&config_path, "")?;

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(command_vec, &data_path, false).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stderr.contains("Scheme repo path does not exist:"),
        "stderr does not contain the expected output"
    );
    assert!(
        stderr.contains("Run `tinty install` and try again"),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_info_subcommand_without_setup_with_custom_schemes_flag() -> Result<()> {
    // -------
    // Arrange
    // -------
    let test_name = "test_info_list_subcommand_without_setup_with_custom_schemes_flag";
    let (_, data_path, command_vec, cleanup) = setup(test_name, "list --custom-schemes")?;
    let expected_output = format!(
        "You don't have any local custom schemes at: data_path_{test_name}/custom-schemes",
    );

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stderr.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_info_subcommand_with_setup_invalid_scheme_name() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "mocha";
    let (_, data_path, command_vec, cleanup) = setup(
        "test_cli_info_subcommand_with_setup_invalid_scheme_name",
        format!("info {scheme_name}").as_str(),
    )?;

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stderr.contains(
            format!(
                r##"Invalid scheme system: "{scheme_name}" from scheme name "{scheme_name}"
Make sure to add the system prefix to the theme name. Eg: base16-oceanicnext
Run `{REPO_NAME} list` to get a list of scheme names"##,
            )
            .as_str()
        ),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}
