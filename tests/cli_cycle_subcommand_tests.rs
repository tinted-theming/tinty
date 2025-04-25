mod utils;

use std::fs;
use std::path::Path;

use crate::utils::{setup, write_to_file, CURRENT_SCHEME_FILE_NAME, REPO_NAME};
use anyhow::Result;
use utils::build_comamnd_vec;

#[test]
fn test_cli_cycle_subcommand_with_default_scheme_only() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, apply_command_vec, cleanup) = setup(
        "test_cli_cycle_subcommand_with_default_scheme_only",
        format!("apply {}", &scheme_name).as_str(),
    )?;
    let config_content = r##"
default-scheme = "base16-github"
"##;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (_, apply_stderr) = utils::run_command(apply_command_vec).unwrap();

    let (cycle_stdout, cycle_stderr) = utils::run_command(build_comamnd_vec(
        "cycle",
        config_path.as_path(),
        data_path.as_path(),
    )?)
    .unwrap();

    // ------
    // Assert
    // ------
    assert!(
        apply_stderr.is_empty(),
        "stderr does not contain the expected output"
    );
    assert_eq!(
        cycle_stdout,
        "Applying next theme in cycle: base16-github\n"
    );
    assert!(
        cycle_stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_cycle_subcommand_with_preferred_schemes() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, apply_command_vec, cleanup) = setup(
        "test_cli_cycle_subcommand_with_default_scheme_only",
        format!("apply {}", &scheme_name).as_str(),
    )?;
    let config_content = r##"
preferred-schemes = ["base24-github", "base24-zenburn", "base24-ubuntu"]
"##;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (_, apply_stderr) = utils::run_command(apply_command_vec).unwrap();

    let (cycle1_stdout, cycle1_stderr) = utils::run_command(build_comamnd_vec(
        "cycle",
        config_path.as_path(),
        data_path.as_path(),
    )?)
    .unwrap();



    // ------
    // Assert
    // ------
    assert!(
        apply_stderr.is_empty(),
        "stderr does not contain the expected output"
    );
    assert_eq!(
        cycle1_stdout,
        "Applying next theme in cycle: base24-github\n"
    );
    assert!(
        cycle1_stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    let (cycle2_stdout, cycle2_stderr) = utils::run_command(build_comamnd_vec(
        "cycle",
        config_path.as_path(),
        data_path.as_path(),
    )?)
    .unwrap();

    assert_eq!(
        cycle2_stdout,
        "Applying next theme in cycle: base24-zenburn\n"
    );
    assert!(
        cycle2_stderr.is_empty(),
        "stderr does not contain the expected output"
    );
    let (cycle3_stdout, cycle3_stderr) = utils::run_command(build_comamnd_vec(
        "cycle",
        config_path.as_path(),
        data_path.as_path(),
    )?)
    .unwrap();

    assert_eq!(
        cycle3_stdout,
        "Applying next theme in cycle: base24-ubuntu\n"
    );
    assert!(
        cycle3_stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_cycle_subcommand_correct_next_scheme() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base24-zenburn";
    let (config_path, data_path, apply_command_vec, cleanup) = setup(
        "test_cli_cycle_subcommand_with_default_scheme_only",
        format!("apply {}", &scheme_name).as_str(),
    )?;
    let config_content = r##"
preferred-schemes = ["base24-github", "base24-zenburn", "base24-ubuntu"]
"##;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (_, apply_stderr) = utils::run_command(apply_command_vec).unwrap();

    let (cycle_stdout, cycle_stderr) = utils::run_command(build_comamnd_vec(
        "cycle",
        config_path.as_path(),
        data_path.as_path(),
    )?)
    .unwrap();



    // ------
    // Assert
    // ------
    assert!(
        apply_stderr.is_empty(),
        "stderr does not contain the expected output"
    );
    assert_eq!(
        cycle_stdout,
        "Applying next theme in cycle: base24-ubuntu\n"
    );
    assert!(
        cycle_stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_cycle_subcommand_wraps_around() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base24-ubuntu";
    let (config_path, data_path, apply_command_vec, cleanup) = setup(
        "test_cli_cycle_subcommand_with_default_scheme_only",
        format!("apply {}", &scheme_name).as_str(),
    )?;
    let config_content = r##"
preferred-schemes = ["base24-github", "base24-zenburn", "base24-ubuntu"]
"##;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (_, apply_stderr) = utils::run_command(apply_command_vec).unwrap();

    let (cycle_stdout, cycle_stderr) = utils::run_command(build_comamnd_vec(
        "cycle",
        config_path.as_path(),
        data_path.as_path(),
    )?)
    .unwrap();



    // ------
    // Assert
    // ------
    assert!(
        apply_stderr.is_empty(),
        "stderr does not contain the expected output"
    );
    assert_eq!(
        cycle_stdout,
        "Applying next theme in cycle: base24-github\n"
    );
    assert!(
        cycle_stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_cycle_subcommand_default_scheme_prepended_to_cycle() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, apply_command_vec, cleanup) = setup(
        "test_cli_cycle_subcommand_with_default_scheme_only",
        format!("apply {}", &scheme_name).as_str(),
    )?;
    let config_content = r##"
default-scheme = "base24-github"
preferred-schemes = ["base24-zenburn", "base24-ubuntu"]
"##;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (_, apply_stderr) = utils::run_command(apply_command_vec).unwrap();

    let (cycle_stdout, cycle_stderr) = utils::run_command(build_comamnd_vec(
        "cycle",
        config_path.as_path(),
        data_path.as_path(),
    )?)
    .unwrap();



    // ------
    // Assert
    // ------
    assert!(
        apply_stderr.is_empty(),
        "stderr does not contain the expected output"
    );
    assert_eq!(
        cycle_stdout,
        "Applying next theme in cycle: base24-github\n"
    );
    assert!(
        cycle_stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_cycle_subcommand_default_scheme_not_duplicated_in_cycle() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, apply_command_vec, cleanup) = setup(
        "test_cli_cycle_subcommand_with_default_scheme_only",
        format!("apply {}", &scheme_name).as_str(),
    )?;
    let config_content = r##"
default-scheme = "base24-github"
preferred-schemes = ["base24-zenburn", "base24-github", "base24-ubuntu"]
"##;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (_, apply_stderr) = utils::run_command(apply_command_vec).unwrap();

    let (cycle_stdout, cycle_stderr) = utils::run_command(build_comamnd_vec(
        "cycle",
        config_path.as_path(),
        data_path.as_path(),
    )?)
    .unwrap();



    // ------
    // Assert
    // ------
    assert!(
        apply_stderr.is_empty(),
        "stderr does not contain the expected output"
    );
    assert_eq!(
        cycle_stdout,
        "Applying next theme in cycle: base24-zenburn\n"
    );
    assert!(
        cycle_stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}
