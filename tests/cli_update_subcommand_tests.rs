mod utils;

use std::process::Command;

use crate::utils::{setup, REPO_NAME};
use anyhow::{ensure, Result};
use utils::write_to_file;

#[test]
fn test_cli_update_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) = setup(
        "test_cli_install_subcommand_non_unique_config_item_name",
        "update",
    )?;
    let expected_output = format!("tinted-shell not installed (run `{REPO_NAME} install`)");

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(&command_vec, &data_path, false)?;

    // ------
    // Assert
    // ------
    cleanup()?;
    ensure!(
        stdout.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_update_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_update_subcommand_with_setup", "update")?;
    let expected_output = "tinted-shell up to date";

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    cleanup()?;
    ensure!(
        stdout.contains(expected_output),
        "stdout does not contain the expected output"
    );
    ensure!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_update_subcommand_with_setup_quiet_flag() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) = setup(
        "test_cli_update_subcommand_with_setup_quiet_flag",
        "update --quiet",
    )?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    cleanup()?;
    ensure!(
        stdout.is_empty(),
        "stdout does not contain the expected output"
    );
    ensure!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_update_subcommand_with_new_remote() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) =
        setup("test_cli_update_subcommand_with_new_remote", "update")?;
    let expected_output = "tinted-vim up to date";

    let config_content = r#"[[items]]
path = "https://github.com/tinted-theming/base16-vim"
name = "tinted-vim"
themes-dir = "colors"
"#;

    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path, false)?;

    // Replace the remote with a new one
    let config_content = r#"[[items]]
path = "https://github.com/tinted-theming/tinted-vim"
name = "tinted-vim"
themes-dir = "colors"
"#;
    write_to_file(&config_path, config_content)?;
    let (stdout, _) = utils::run_command(&command_vec, &data_path, false)?;

    let repo_path = data_path.join("repos/tinted-vim");
    let output = Command::new("git")
        .args(vec!["remote", "get-url", "origin"])
        .current_dir(&repo_path)
        .output()?;

    let remote_stdout = String::from_utf8(output.stdout)?;

    // ------
    // Assert
    // ------
    cleanup()?;
    ensure!(
        stdout.contains(expected_output),
        "stdout does not contain the expected output"
    );
    let remote_url = "https://github.com/tinted-theming/tinted-vim";
    let has_match = remote_stdout.lines().any(|line| line == remote_url);

    ensure!(
        has_match,
        format!("Expected origin remote to point to URL {}", remote_url)
    );

    Ok(())
}

#[test]
fn test_cli_update_subcommand_with_new_revision() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) =
        setup("test_cli_update_subcommand_with_new_revision", "update")?;
    let expected_output = "tinted-vim up to date";
    let expected_revision = "c7ab4daadd143a78d4fc561d216d83ef0188f343";
    let config_content = r#"[[items]]
path = "https://github.com/tinted-theming/tinted-vim"
name = "tinted-vim"
themes-dir = "colors"
"#;

    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    utils::run_install_command(&config_path, &data_path, true)?;

    // Replace the remote with a new one
    let config_content = r#"[[items]]
path = "https://github.com/tinted-theming/tinted-vim"
name = "tinted-vim"
themes-dir = "colors"
revision = "tinty-test-tag-01"
"#;
    write_to_file(&config_path, config_content)?;
    let (stdout, _) = utils::run_command(&command_vec, &data_path, false)?;

    let repo_path = data_path.join("repos/tinted-vim");
    let output = Command::new("git")
        .args(vec!["rev-parse", "--verify", "HEAD"])
        .current_dir(&repo_path)
        .output()?;
    let rev_parse_out = String::from_utf8(output.stdout)?;

    // ------
    // Assert
    // ------
    cleanup()?;
    ensure!(
        stdout.contains(expected_output),
        "stdout does not contain expected output"
    );
    let has_match = rev_parse_out.lines().any(|line| line == expected_revision);
    ensure!(has_match, "Expected revision {}", expected_revision);

    Ok(())
}

#[test]
fn test_cli_update_subcommand_with_new_remote_but_invalid_tag_revision() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_update_subcommand_with_new_remote_but_invalid_tag_revision",
        "update",
    )?;
    let expected_output = "tinted-vim up to date";
    let config_content = r#"[[items]]
path = "https://github.com/tinted-theming/tinted-vim"
name = "tinted-vim"
themes-dir = "colors"
"#;
    let git_tag_name = "invalid-git-tag";

    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path, true)?;

    // Replace the remote with a new one
    // tinty-test-tag-01 exist in tinted-theming but not on this one.
    let config_content = format!(
        r#"[[items]]
path = "https://github.com/tinted-theming/base16-vim"
name = "tinted-vim"
themes-dir = "colors"
revision = "{git_tag_name}"
"#
    );
    write_to_file(&config_path, &config_content)?;
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, false)?;

    let repo_path = data_path.join("repos/tinted-vim");
    let output = Command::new("git")
        .args(vec!["remote", "get-url", "origin"])
        .current_dir(&repo_path)
        .output()?;

    let remote_out = String::from_utf8(output.stdout)?;

    // ------
    // Assert
    // ------
    cleanup()?;
    ensure!(
        !stdout.contains(expected_output),
        "stdout contains unexpected output"
    );
    ensure!(
        stderr.contains(format!("cannot resolve {git_tag_name}").as_str()),
        "stderr does not contain the expected output"
    );
    let expected_remote_url = "https://github.com/tinted-theming/tinted-vim";
    let has_match = remote_out.starts_with(expected_remote_url);

    ensure!(
        has_match,
        format!("Expected remote URL {}", expected_remote_url)
    );

    Ok(())
}

#[test]
fn test_cli_update_subcommand_with_new_remote_but_invalid_branch_revision() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_update_subcommand_with_new_remote_but_invalid_branch_revision",
        "update",
    )?;
    let unexpected_output = "tinted-vim up to date";
    let config_content = r#"[[items]]
path = "https://github.com/tinted-theming/tinted-vim"
name = "tinted-vim"
themes-dir = "colors"
"#;

    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    // Replace the remote with a new one
    // tinty-test-01 exist in tinted-theming but not on this one.
    let config_content = r#"[[items]]
path = "https://github.com/tinted-theming/base16-vim"
name = "tinted-vim"
themes-dir = "colors"
revision = "tinty-test-01"
"#;
    write_to_file(&config_path, config_content)?;
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;
    let repo_path = data_path.join("repos/tinted-vim");
    let output = Command::new("git")
        .args(vec!["remote", "get-url", "origin"])
        .current_dir(&repo_path)
        .output()?;
    let remote_out = String::from_utf8(output.stdout)?;

    // ------
    // Assert
    // ------
    cleanup()?;
    ensure!(
        !stdout.contains(unexpected_output),
        "stdout contains unexpected output"
    );
    ensure!(
        stderr.contains("cannot resolve tinty-test-01"),
        "stderr does not contain the expected output"
    );
    let expected_remote_url = "https://github.com/tinted-theming/tinted-vim";
    let has_match = remote_out.trim().starts_with(expected_remote_url);

    ensure!(
        has_match,
        format!("Expected remote URL {}", expected_remote_url)
    );

    Ok(())
}

#[test]
fn test_cli_update_subcommand_with_new_remote_but_invalid_commit_sha1_revision() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_update_subcommand_with_new_remote_but_commit_sha1_revision",
        "update",
    )?;
    let expected_output = "tinted-vim up to date";
    let config_content = r#"[[items]]
path = "https://github.com/tinted-theming/tinted-vim"
name = "tinted-vim"
themes-dir = "colors"
"#;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path, false)?;

    // Replace the remote with a new one
    // This commit SHA only exist in tinted-theming but not on this one.
    let config_content = r#"[[items]]
path = "https://github.com/tinted-theming/base16-vim"
name = "tinted-vim"
themes-dir = "colors"
revision = "43b36ed5eadad59a5027e442330d2485b8607b34"
"#;
    write_to_file(&config_path, config_content)?;

    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, false)?;

    let repo_path = data_path.join("repos/tinted-vim");
    let output = Command::new("git")
        .args(vec!["remote", "get-url", "origin"])
        .current_dir(&repo_path)
        .output()?;

    let remote_out = String::from_utf8(output.stdout)?;

    // ------
    // Assert
    // ------
    cleanup()?;
    ensure!(
        !stdout.contains(expected_output),
        "stdout contains unexpected output"
    );
    ensure!(
        stderr.contains("cannot find revision 43b36ed5eadad59a5027e442330d2485b8607b34"),
        "stderr does not contain the expected output"
    );
    let expected_remote_url = "https://github.com/tinted-theming/tinted-vim";
    let has_match = remote_out.lines().any(|line| line == expected_remote_url);

    ensure!(
        has_match,
        format!("Expected remote URL {expected_remote_url}")
    );

    Ok(())
}
