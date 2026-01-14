mod utils;

use std::{process::Command, str};

use crate::utils::{setup, write_to_file};
use anyhow::{ensure, Result};

#[test]
fn test_cli_install_subcommand_non_unique_config_item_name() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_install_subcommand_non_unique_config_item_name",
        "install",
    )?;
    let config_content = r#"[[items]]
path = "https://github.com/tinted-theming/tinted-shell"
name = "non-unique-name"
themes-dir = "some-dir"

[[items]]
path = "https://github.com/tinted-theming/tinted-shell"
name = "non-unique-name"
themes-dir = "some-dir"
"#;
    let expected_output = "config.toml item.name should be unique values, but \"non-unique-name\" is used for more than 1 item.name. Please change this to a unique value.";
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    cleanup()?;
    ensure!(
        stderr.contains(expected_output),
        "stdout does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_install_subcommand_invalid_config_item_path() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_install_subcommand_invalid_config_item_path",
        "install",
    )?;
    let config_content = r#"[[items]]
path = "/path/to/non-existant/directory"
name = "some-name"
themes-dir = "some-dir""#;
    let expected_output = "One of your config.toml items has an invalid `path` value. \"/path/to/non-existant/directory\" is not a valid url and is not a path to an existing local directory";
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    cleanup()?;
    ensure!(
        stderr.contains(expected_output),
        "stdout does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_install_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_install_subcommand_without_setup", "install")?;
    let expected_output = "tinted-shell installed";

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(&command_vec, &data_path, false)?;

    // ------
    // Assert
    // ------
    ensure!(
        stdout.contains(expected_output),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_install_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_install_subcommand_with_setup", "install")?;
    let expected_output = "tinted-shell already installed";

    // ---
    // Act
    // ---
    utils::run_command(&command_vec, &data_path, true)?;
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------

    ensure!(
        stdout.contains(expected_output),
        "stdout does not contain the expected output"
    );
    ensure!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_install_subcommand_with_setup_quiet_flag() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) = setup(
        "test_cli_install_subcommand_with_setup_quiet_flag",
        "install --quiet",
    )?;

    // ---
    // Act
    // ---
    utils::run_command(&command_vec, &data_path, true)?;
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------

    ensure!(
        stdout.is_empty(),
        "stdout does not contain the expected output"
    );
    ensure!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_install_subcommand_with_tag_revision() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) =
        setup("test_cli_install_subcommand_with_tag_revision", "install")?;
    let commit_sha = "0e4f0d222b9013cc7e537ac6cd29bf83ba19094a";
    let config_content = format!(
        r#"[[items]]
path = "https://github.com/tinted-theming/tinted-vim"
name = "tinted-vim"
themes-dir = "colors"
revision = "{commit_sha}"
"#
    );
    write_to_file(&config_path, &config_content)?;

    let repo_path = data_path.join("repos/tinted-vim");

    // ---
    // Act
    // ---
    let (_, _) = utils::run_command(&command_vec, &data_path, false)?;
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(vec!["rev-parse", "--verify", "HEAD"])
        .output()
        .expect("Failed to execute git rev-parse --verify HEAD");
    let stdout = str::from_utf8(&output.stdout).expect("Not valid UTF-8");

    // ------
    // Assert
    // ------
    let has_match = stdout.lines().any(|line| line == commit_sha);
    cleanup()?;
    ensure!(
        has_match,
        format!("Expected revision {} not found", commit_sha)
    );

    Ok(())
}

#[test]
fn test_cli_install_subcommand_with_branch_revision() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_install_subcommand_with_branch_revision",
        "install",
    )?;
    let rev = "tinty-test-01";
    let config_content = format!(
        r#"[[items]]
path = "https://github.com/tinted-theming/tinted-jqp"
name = "tinted-jqp"
themes-dir = "themes"
revision = "{rev}"
"#
    );
    write_to_file(&config_path, &config_content)?;

    // ---
    // Act
    // ---
    let (_, _) = utils::run_command(&command_vec, &data_path, false)?;

    let repo_path = data_path.join("repos/tinted-jqp");
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(vec!["rev-parse", "--verify", "HEAD"])
        .output()
        .expect("Failed to execute git rev-parse --verify HEAD");

    let stdout = str::from_utf8(&output.stdout).expect("Not valid UTF-8");

    // ------
    // Assert
    // ------
    let expected_revision = "43b36ed5eadad59a5027e442330d2485b8607b34";
    let has_match = stdout.lines().any(|line| line == expected_revision);
    cleanup()?;
    ensure!(
        has_match,
        format!("Expected revision {} not found", expected_revision)
    );

    Ok(())
}

#[test]
fn test_cli_install_subcommand_with_commit_sha1_revision() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_install_subcommand_with_commit_sha1_revision",
        "install",
    )?;
    let commit_sha = "f998d17414a7218904bb5b4fdada5daa2b2d9d5e";
    let config_content = format!(
        r#"[[items]]
path = "https://github.com/tinted-theming/tinted-jqp"
name = "tinted-jqp"
themes-dir = "themes"
revision = "{commit_sha}"
"#
    );
    write_to_file(&config_path, &config_content)?;

    // ---
    // Act
    // ---
    let (_, _) = utils::run_command(&command_vec, &data_path, false)?;

    let repo_path = data_path.join("repos/tinted-jqp");
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(vec!["rev-parse", "--verify", "HEAD"])
        .output()
        .expect("Failed to execute git rev-parse --verify HEAD");

    let stdout = str::from_utf8(&output.stdout).expect("Not valid UTF-8");

    // ------
    // Assert
    // ------
    // This SHA1 is only reachable through the tinted-test-01 branch, but is not the tip of that
    // branch.
    let has_match = stdout.lines().any(|line| line == commit_sha);
    cleanup()?;
    ensure!(
        has_match,
        format!("Expected revision {} not found", commit_sha)
    );

    Ok(())
}

#[test]
fn test_cli_install_subcommand_with_non_existent_revision() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_install_subcommand_with_non_existent_revision",
        "install",
    )?;
    let commit_sha = "invalid-revision";
    let config_content = format!(
        r#"[[items]]
path = "https://github.com/tinted-theming/tinted-jqp"
name = "tinted-jqp"
themes-dir = "themes"
revision = "{commit_sha}"
"#
    );
    write_to_file(&config_path, &config_content)?;

    let repo_path = data_path.join("repos/tinted-jqp");

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(&command_vec, &data_path, false)?;

    // ------
    // Assert
    // ------
    let path_exists = repo_path.exists();
    cleanup()?;
    ensure!(
        stderr.contains(format!("cannot resolve {commit_sha}").as_str()),
        "Expected revision not found",
    );

    ensure!(
        !path_exists,
        format!("Expected repo path {} to not exist", repo_path.display())
    );

    Ok(())
}
