mod utils;

use crate::utils::{setup, write_to_file};
use anyhow::{ensure, Result};
use utils::build_command_vec;

#[test]
fn test_cli_cycle_subcommand_with_default_scheme_only() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, apply_command_vec, cleanup) = setup(
        "test_cli_cycle_subcommand_with_default_scheme_only",
        format!("apply {scheme_name}").as_str(),
    )?;
    let config_content = r#"
default-scheme = "base16-dracula"
"#;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (_, apply_stderr) = utils::run_command(&apply_command_vec, &data_path, true)?;

    let (cycle_stdout, cycle_stderr) = utils::run_command(
        &build_command_vec("cycle", config_path.as_path(), data_path.as_path())?,
        &data_path,
        true,
    )?;

    // ------
    // Assert
    // ------
    ensure!(
        apply_stderr.is_empty(),
        "stderr does not contain the expected output"
    );
    ensure!(
        cycle_stdout == "Applying next theme in cycle: base16-dracula\n",
        "cycle_stdout not as expected"
    );
    ensure!(
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
        "test_cli_cycle_subcommand_with_preferred_schemes",
        format!("apply {scheme_name}").as_str(),
    )?;
    let config_content = r#"
preferred-schemes = ["base24-dracula", "base24-zenburn", "base24-ubuntu"]
"#;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (_, apply_stderr) = utils::run_command(&apply_command_vec, &data_path, true)?;

    let (cycle1_stdout, cycle1_stderr) = utils::run_command(
        &build_command_vec("cycle", config_path.as_path(), data_path.as_path())?,
        &data_path,
        true,
    )?;

    // ------
    // Assert
    // ------
    ensure!(
        apply_stderr.is_empty(),
        "stderr does not contain the expected output"
    );
    ensure!(
        cycle1_stdout == "Applying next theme in cycle: base24-dracula\n",
        "cycle1_stdout not as expected"
    );
    ensure!(
        cycle1_stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    let (cycle2_stdout, cycle2_stderr) = utils::run_command(
        &build_command_vec("cycle", config_path.as_path(), data_path.as_path())?,
        &data_path,
        true,
    )?;

    ensure!(
        cycle2_stdout == "Applying next theme in cycle: base24-zenburn\n",
        "cycle2_stdout not as expected"
    );
    ensure!(
        cycle2_stderr.is_empty(),
        "stderr does not contain the expected output"
    );
    let (cycle3_stdout, cycle3_stderr) = utils::run_command(
        &build_command_vec("cycle", config_path.as_path(), data_path.as_path())?,
        &data_path,
        true,
    )?;

    ensure!(
        cycle3_stdout == "Applying next theme in cycle: base24-ubuntu\n",
        "cycle3 stdout not as expected"
    );
    ensure!(
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
        "test_cli_cycle_subcommand_correct_next_scheme",
        format!("apply {scheme_name}").as_str(),
    )?;
    let config_content = r#"
preferred-schemes = ["base24-dracula", "base24-zenburn", "base24-ubuntu"]
"#;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (_, apply_stderr) = utils::run_command(&apply_command_vec, &data_path, true)?;

    let (cycle_stdout, cycle_stderr) = utils::run_command(
        &build_command_vec("cycle", config_path.as_path(), data_path.as_path())?,
        &data_path,
        true,
    )?;

    // ------
    // Assert
    // ------
    ensure!(
        apply_stderr.is_empty(),
        "stderr does not contain the expected output"
    );
    ensure!(
        cycle_stdout == "Applying next theme in cycle: base24-ubuntu\n",
        "cycle_stdout not as expected"
    );
    ensure!(
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
        "test_cli_cycle_subcommand_wraps_around",
        format!("apply {scheme_name}").as_str(),
    )?;
    let config_content = r#"
preferred-schemes = ["base24-dracula", "base24-zenburn", "base24-ubuntu"]
"#;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (_, apply_stderr) = utils::run_command(&apply_command_vec, &data_path, true)?;

    let (cycle_stdout, cycle_stderr) = utils::run_command(
        &build_command_vec("cycle", config_path.as_path(), data_path.as_path())?,
        &data_path,
        true,
    )?;

    // ------
    // Assert
    // ------
    ensure!(
        apply_stderr.is_empty(),
        "stderr does not contain the expected output"
    );
    ensure!(
        cycle_stdout == "Applying next theme in cycle: base24-dracula\n",
        "cycle_stdout not as expected"
    );

    ensure!(
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
        "test_cli_cycle_subcommand_default_scheme_prepended_to_cycle",
        format!("apply {scheme_name}").as_str(),
    )?;
    let config_content = r#"
default-scheme = "base24-dracula"
preferred-schemes = ["base24-zenburn", "base24-ubuntu"]
"#;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (_, apply_stderr) = utils::run_command(&apply_command_vec, &data_path, true)?;

    let (cycle_stdout, cycle_stderr) = utils::run_command(
        &build_command_vec("cycle", config_path.as_path(), data_path.as_path())?,
        &data_path,
        true,
    )?;

    // ------
    // Assert
    // ------
    ensure!(
        apply_stderr.is_empty(),
        "stderr does not contain the expected output"
    );
    ensure!(
        cycle_stdout == "Applying next theme in cycle: base24-dracula\n",
        "cycle_stdout not as expected"
    );
    ensure!(
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
        "test_cli_cycle_subcommand_default_scheme_not_duplicated_in_cycle",
        format!("apply {scheme_name}").as_str(),
    )?;
    let config_content = r#"
default-scheme = "base24-dracula"
preferred-schemes = ["base24-zenburn", "base24-dracula", "base24-ubuntu"]
"#;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (_, apply_stderr) = utils::run_command(&apply_command_vec, &data_path, true)?;

    let (cycle_stdout, cycle_stderr) = utils::run_command(
        &build_command_vec("cycle", config_path.as_path(), data_path.as_path())?,
        &data_path,
        true,
    )?;

    // ------
    // Assert
    // ------
    ensure!(
        apply_stderr.is_empty(),
        "stderr does not contain the expected output"
    );
    ensure!(
        cycle_stdout == "Applying next theme in cycle: base24-zenburn\n",
        "cycle_stdout not as expected"
    );
    ensure!(
        cycle_stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}
