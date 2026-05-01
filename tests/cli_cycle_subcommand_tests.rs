//! Integration tests for the `cycle` subcommand.
//!
//! Covers cycling through configured rings, default-cycle-ring behavior,
//! wrap-around at end of list, and legacy preferred-schemes migration errors.
//!
//! Requires network access on first run (repos are cached in `tmp/repos/`).

mod utils;

use crate::utils::{setup, write_to_file};
use anyhow::{ensure, Result};
use std::fs;
use utils::build_command_vec;

#[test]
fn test_cli_cycle_subcommand_with_explicit_ring() -> Result<()> {
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, apply_command_vec, _temp_dir) = setup(
        "test_cli_cycle_subcommand_with_explicit_ring",
        format!("apply {scheme_name}").as_str(),
        true,
    )?;
    let config_content = r#"
[[rings]]
name = "dark"
schemes = ["base24-dracula", "base24-zenburn", "base24-ubuntu"]

[[rings]]
name = "light"
schemes = ["base16-github", "base16-gruvbox-material-light-soft"]
"#;
    write_to_file(&config_path, config_content)?;

    let (_, apply_stderr) = utils::run_command(&apply_command_vec)?;

    let (cycle_stdout, cycle_stderr) = utils::run_command(&build_command_vec(
        "cycle --ring dark",
        config_path.as_path(),
        data_path.as_path(),
    )?)?;

    ensure!(
        apply_stderr.is_empty(),
        "Expected empty stderr, got: {apply_stderr}"
    );
    ensure!(
        cycle_stdout == "Applying next theme in cycle: base24-dracula\n",
        "cycle_stdout not as expected"
    );
    ensure!(
        cycle_stderr.is_empty(),
        "Expected empty stderr, got: {cycle_stderr}"
    );

    Ok(())
}

#[test]
fn test_cli_cycle_subcommand_uses_default_cycle_ring() -> Result<()> {
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, apply_command_vec, _temp_dir) = setup(
        "test_cli_cycle_subcommand_uses_default_cycle_ring",
        format!("apply {scheme_name}").as_str(),
        true,
    )?;
    let config_content = r#"
default-cycle-ring = "dark"

[[rings]]
name = "dark"
schemes = ["base24-dracula", "base24-zenburn", "base24-ubuntu"]
"#;
    write_to_file(&config_path, config_content)?;

    let (_, apply_stderr) = utils::run_command(&apply_command_vec)?;

    let (cycle_stdout, cycle_stderr) = utils::run_command(&build_command_vec(
        "cycle",
        config_path.as_path(),
        data_path.as_path(),
    )?)?;

    ensure!(
        apply_stderr.is_empty(),
        "Expected empty stderr, got: {apply_stderr}"
    );
    ensure!(
        cycle_stdout == "Applying next theme in cycle: base24-dracula\n",
        "cycle_stdout not as expected"
    );
    ensure!(
        cycle_stderr.is_empty(),
        "Expected empty stderr, got: {cycle_stderr}"
    );

    Ok(())
}

#[test]
fn test_cli_cycle_subcommand_correct_next_scheme_in_ring() -> Result<()> {
    let scheme_name = "base24-zenburn";
    let (config_path, data_path, apply_command_vec, _temp_dir) = setup(
        "test_cli_cycle_subcommand_correct_next_scheme_in_ring",
        format!("apply {scheme_name}").as_str(),
        true,
    )?;
    let config_content = r#"
default-cycle-ring = "default"

[[rings]]
name = "default"
schemes = ["base24-dracula", "base24-zenburn", "base24-ubuntu"]
"#;
    write_to_file(&config_path, config_content)?;

    let (_, apply_stderr) = utils::run_command(&apply_command_vec)?;

    let (cycle_stdout, cycle_stderr) = utils::run_command(&build_command_vec(
        "cycle",
        config_path.as_path(),
        data_path.as_path(),
    )?)?;

    ensure!(
        apply_stderr.is_empty(),
        "Expected empty stderr, got: {apply_stderr}"
    );
    ensure!(
        cycle_stdout == "Applying next theme in cycle: base24-ubuntu\n",
        "cycle_stdout not as expected"
    );
    ensure!(
        cycle_stderr.is_empty(),
        "Expected empty stderr, got: {cycle_stderr}"
    );

    Ok(())
}

#[test]
fn test_cli_cycle_subcommand_wraps_around_ring() -> Result<()> {
    let scheme_name = "base24-ubuntu";
    let (config_path, data_path, apply_command_vec, _temp_dir) = setup(
        "test_cli_cycle_subcommand_wraps_around_ring",
        format!("apply {scheme_name}").as_str(),
        true,
    )?;
    let config_content = r#"
default-cycle-ring = "default"

[[rings]]
name = "default"
schemes = ["base24-dracula", "base24-zenburn", "base24-ubuntu"]
"#;
    write_to_file(&config_path, config_content)?;

    let (_, apply_stderr) = utils::run_command(&apply_command_vec)?;

    let (cycle_stdout, cycle_stderr) = utils::run_command(&build_command_vec(
        "cycle",
        config_path.as_path(),
        data_path.as_path(),
    )?)?;

    ensure!(
        apply_stderr.is_empty(),
        "Expected empty stderr, got: {apply_stderr}"
    );
    ensure!(
        cycle_stdout == "Applying next theme in cycle: base24-dracula\n",
        "cycle_stdout not as expected"
    );
    ensure!(
        cycle_stderr.is_empty(),
        "Expected empty stderr, got: {cycle_stderr}"
    );

    Ok(())
}

#[test]
fn test_cli_cycle_subcommand_errors_for_empty_ring() -> Result<()> {
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, apply_command_vec, _temp_dir) = setup(
        "test_cli_cycle_subcommand_errors_for_empty_ring",
        format!("apply {scheme_name}").as_str(),
        true,
    )?;
    let config_content = r#"
default-cycle-ring = "default"

[[rings]]
name = "default"
schemes = []
"#;
    write_to_file(&config_path, config_content)?;

    let (_, apply_stderr) = utils::run_command(&apply_command_vec)?;

    let (cycle_stdout, cycle_stderr) = utils::run_command(&build_command_vec(
        "cycle",
        config_path.as_path(),
        data_path.as_path(),
    )?)?;

    let current_scheme = fs::read_to_string(data_path.join("current_scheme"))?;

    ensure!(
        apply_stderr.is_empty(),
        "Expected empty stderr, got: {apply_stderr}"
    );
    ensure!(cycle_stdout.is_empty(), "Expected empty stdout");
    ensure!(
        cycle_stderr.contains("Ring \"default\" does not contain any schemes and cannot be cycled"),
        "cycle_stderr not as expected: {cycle_stderr}"
    );
    ensure!(
        current_scheme == scheme_name,
        "Expected current scheme to remain unchanged"
    );

    Ok(())
}

#[test]
fn test_cli_cycle_subcommand_errors_for_missing_ring() -> Result<()> {
    let (config_path, data_path, _apply_command_vec, _temp_dir) = setup(
        "test_cli_cycle_subcommand_errors_for_missing_ring",
        "cycle --ring dark",
        false,
    )?;
    let config_content = r#"
[[rings]]
name = "light"
schemes = ["base16-github"]
"#;
    write_to_file(&config_path, config_content)?;

    let (cycle_stdout, cycle_stderr) = utils::run_command(&build_command_vec(
        "cycle --ring dark",
        config_path.as_path(),
        data_path.as_path(),
    )?)?;

    ensure!(cycle_stdout.is_empty(), "Expected empty stdout");
    ensure!(
        cycle_stderr.contains("No ring named \"dark\" exists. Available rings: light"),
        "cycle_stderr not as expected: {cycle_stderr}"
    );

    Ok(())
}

#[test]
fn test_cli_cycle_subcommand_errors_for_duplicate_ring_names() -> Result<()> {
    let (config_path, data_path, _apply_command_vec, _temp_dir) = setup(
        "test_cli_cycle_subcommand_errors_for_duplicate_ring_names",
        "cycle",
        false,
    )?;
    let config_content = r#"
default-cycle-ring = "default"

[[rings]]
name = "default"
schemes = ["base16-github"]

[[rings]]
name = "default"
schemes = ["base16-dracula"]
"#;
    write_to_file(&config_path, config_content)?;

    let (cycle_stdout, cycle_stderr) = utils::run_command(&build_command_vec(
        "cycle",
        config_path.as_path(),
        data_path.as_path(),
    )?)?;

    ensure!(cycle_stdout.is_empty(), "Expected empty stdout");
    ensure!(
        cycle_stderr.contains("config.toml rings.name should be unique values"),
        "cycle_stderr not as expected: {cycle_stderr}"
    );

    Ok(())
}

#[test]
fn test_cli_cycle_subcommand_errors_for_preferred_schemes_with_migration() -> Result<()> {
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, apply_command_vec, _temp_dir) = setup(
        "test_cli_cycle_subcommand_errors_for_preferred_schemes_with_migration",
        format!("apply {scheme_name}").as_str(),
        true,
    )?;
    let config_content = r#"
default-scheme = "base24-ubuntu"
preferred-schemes = ["base24-dracula", "base24-zenburn"]
"#;
    write_to_file(&config_path, config_content)?;

    let (_, apply_stderr) = utils::run_command(&apply_command_vec)?;

    let (cycle_stdout, cycle_stderr) = utils::run_command(&build_command_vec(
        "cycle",
        config_path.as_path(),
        data_path.as_path(),
    )?)?;

    let current_scheme = fs::read_to_string(data_path.join("current_scheme"))?;

    ensure!(
        apply_stderr.is_empty(),
        "Expected empty stderr, got: {apply_stderr}"
    );
    ensure!(cycle_stdout.is_empty(), "Expected empty stdout");
    ensure!(
        cycle_stderr.contains("`preferred-schemes` is no longer supported by `tinty cycle`"),
        "cycle_stderr did not include deprecation message: {cycle_stderr}"
    );
    ensure!(
        cycle_stderr
            .contains("schemes = [\"base24-ubuntu\", \"base24-dracula\", \"base24-zenburn\"]"),
        "cycle_stderr did not include migration snippet: {cycle_stderr}"
    );
    ensure!(
        cycle_stderr.contains("default-cycle-ring = \"default\""),
        "cycle_stderr did not include default-cycle-ring: {cycle_stderr}"
    );
    ensure!(
        cycle_stderr.find("default-cycle-ring = \"default\"") < cycle_stderr.find("[[rings]]"),
        "default-cycle-ring should appear before [[rings]] in migration snippet: {cycle_stderr}"
    );
    ensure!(
        current_scheme == scheme_name,
        "Expected current scheme to remain unchanged"
    );

    Ok(())
}
