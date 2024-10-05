mod utils;

use crate::utils::REPO_NAME;
use anyhow::Result;
use std::fs;
use std::path::Path;
use utils::{setup, write_to_file};

#[test]
fn test_cli_list_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, _, command_vec, cleanup) = setup("test_cli_list_subcommand_without_setup", "list")?;
    let expected_output = format!(
        "Schemes are missing, run install and then try again: `{} install`",
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
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_list_subcommand_without_setup_with_custom_schemes_flag() -> Result<()> {
    // -------
    // Arrange
    // -------
    let test_name = "test_cli_list_subcommand_without_setup_with_custom_schemes_flag";
    let (_, _, command_vec, cleanup) = setup(test_name, "list --custom-schemes")?;
    let expected_output = format!(
        "You don't have any local custom schemes at: data_path_{}/custom-schemes",
        test_name
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
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_list_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) =
        setup("test_cli_list_subcommand_with_setup", "list")?;
    let expected_output = fs::read_to_string(Path::new("fixtures/schemes.txt"))?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (stdout, _) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    // The sort order of the schemes differ slightly so do an assert on each line instead of the
    // whole file
    let lines: Vec<&str> = expected_output.lines().collect();
    for line in lines {
        assert!(
            stdout.contains(line),
            "stdout does not contain the expected output"
        );
    }

    cleanup()?;
    Ok(())
}

// Also tests recursive dir listing since custom scheme isn't nested within scheme_system directory
#[test]
fn test_cli_list_subcommand_with_custom() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_list_subcommand_with_custom", "list")?;
    let scheme_system = "base16";
    let scheme_name_one = "tinty-generated";
    let scheme_name_two = "tinty";
    let expected_output = format!(
        "{}-{}\n{}-{}",
        scheme_system, scheme_name_one, scheme_system, scheme_name_two
    );
    let custom_scheme_path = data_path.join("custom-schemes");

    fs::create_dir_all(custom_scheme_path.join(scheme_system))?;
    fs::copy(
        "./tests/fixtures/schemes/tinty-generated.yaml",
        custom_scheme_path.join("base16-tinty-generated.yaml"),
    )?;
    write_to_file(
        custom_scheme_path.join(format!("{}.yaml", scheme_name_two)),
        r#"
system: base16
name: Tinty
slug: tinty
author: Tinty
variant: dark
palette:
  base00: '#282628'
  base01: '#403e3f'
  base02: '#595757'
  base03: '#71706e'
  base04: '#8a8986'
  base05: '#a2a29d'
  base06: '#bbbbb5'
  base07: '#d4d4cd'
  base08: '#bf2546'
  base09: '#f69622'
  base0A: '#f99923'
  base0B: '#19953f'
  base0C: '#40dab9'
  base0D: '#0666dc'
  base0E: '#8554ac'
  base0F: '#ac7424'"#,
    )?;

    let mut command_vec = command_vec.clone();
    command_vec.push("--custom-schemes".to_string());

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(command_vec).unwrap();

    // ------
    // Assert
    // ------
    // The sort order of the schemes differ slightly so do an assert on each line instead of the
    // whole file
    let lines: Vec<&str> = expected_output.lines().collect();
    for line in lines {
        assert!(
            stdout.contains(line),
            "stdout does not contain the expected output"
        );
    }

    cleanup()?;
    Ok(())
}
