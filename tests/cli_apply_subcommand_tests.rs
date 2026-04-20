//! Integration tests for the `apply` subcommand.
//!
//! Covers: applying schemes (builtin and custom), hook execution (root and
//! per-item), environment variable injection, theme file extensions,
//! vestigial file/symlink cleanup, `--quiet` flag, and error handling for
//! invalid scheme names, missing system prefixes, and invalid shell config.
//!
//! Requires network access on first run (repos are cached in `tmp/repos/`).

mod utils;

use std::fs;
use std::path::Path;

use crate::utils::{setup, write_to_file, CURRENT_SCHEME_FILE_NAME, REPO_NAME};
use anyhow::{ensure, Result};
use utils::ARTIFACTS_DIR;

#[test]
fn test_cli_apply_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (_, data_path, command_vec, _temp_dir) = setup(
        "test_cli_apply_subcommand_with_setup",
        format!("apply {scheme_name}").as_str(),
    )?;
    let shell_theme_filename = "tinted-shell-scripts-file.sh";
    let current_scheme_path = data_path.join(ARTIFACTS_DIR).join(CURRENT_SCHEME_FILE_NAME);

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(stdout.is_empty(), "Expected empty stdout, got: {stdout}");
    ensure!(
        data_path
            .join(ARTIFACTS_DIR)
            .join(shell_theme_filename)
            .exists(),
        "Path does not exist"
    );
    ensure!(
        fs::read_to_string(&current_scheme_path)? == scheme_name,
        "scheme_name not the same"
    );

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (_, data_path, command_vec, _temp_dir) = setup(
        "test_cli_apply_subcommand_without_setup",
        format!("apply {scheme_name}").as_str(),
    )?;
    let expected_output =
        format!("Schemes do not exist, run install and try again: `{REPO_NAME} install`");

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(&command_vec, &data_path, false)?;

    // ------
    // Assert
    // ------
    ensure!(
        stderr.contains(&expected_output),
        "Expected stderr to contain: {expected_output}\nGot: {stderr}"
    );

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_invalid_scheme_name() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-invalid-scheme";
    let (_, data_path, command_vec, _temp_dir) = setup(
        "test_cli_apply_subcommand_invalid_scheme_name",
        format!("apply {scheme_name}").as_str(),
    )?;
    let expected_output = format!("Scheme does not exist: {scheme_name}");

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        stderr.contains(&expected_output),
        "Expected stderr to contain: {expected_output}\nGot: {stderr}"
    );

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_invalid_scheme_system() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "some-invalid-scheme";
    let (_, data_path, command_vec, _temp_dir) = setup(
        "test_cli_apply_subcommand_invalid_scheme_system",
        format!("apply {scheme_name}").as_str(),
    )?;
    let expected_output = format!("Invalid scheme name. Make sure your scheme is prefixed with a supprted system (\"base16\", \"base24\" or \"tinted8\"), eg: base16-{scheme_name}");

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        stderr.contains(&expected_output),
        "Expected stderr to contain: {expected_output}\nGot: {stderr}"
    );

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_no_scheme_system() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "ocean";
    let (_, data_path, command_vec, _temp_dir) = setup(
        "test_cli_apply_subcommand_no_scheme_system",
        format!("apply {scheme_name}").as_str(),
    )?;
    let expected_output = "Invalid scheme name. Make sure the scheme system is prefixed <SCHEME_SYSTEM>-<SCHEME_NAME>, eg: `base16-ayu-dark`";

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        stderr.contains(expected_output),
        "Expected stderr to contain: {expected_output}\nGot: {stderr}"
    );

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_with_custom_schemes() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "tinty-generated";
    let scheme_content =
        fs::read_to_string(Path::new("./tests/fixtures/schemes/tinty-generated.yaml"))?;
    let scheme_system = scheme_content
        .lines()
        .find_map(|line| line.strip_prefix("system: "))
        .expect("Fixture scheme should have a 'system' field");
    let scheme_name_with_system = format!("{scheme_system}-{scheme_name}");
    let (_, data_path, command_vec, _temp_dir) = setup(
        "test_cli_apply_subcommand_with_custom_schemes",
        format!("apply {scheme_name_with_system}").as_str(),
    )?;
    let custom_scheme_file_path =
        data_path.join(format!("custom-schemes/{scheme_system}/{scheme_name}.yaml"));
    let current_scheme_path = data_path.join(ARTIFACTS_DIR).join(CURRENT_SCHEME_FILE_NAME);
    write_to_file(&custom_scheme_file_path, &scheme_content)?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        fs::read_to_string(current_scheme_path)? == scheme_name_with_system,
        "current_scheme_path different to scheme_name_with_system"
    );
    ensure!(stdout.is_empty(), "Expected empty stdout, got: {stdout}");
    ensure!(
        stderr.contains("W001"),
        "Expected stderr containing \"W001\", got: {stderr}"
    );

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_with_custom_schemes_quiet_flag() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "tinty-generated";
    let scheme_content =
        fs::read_to_string(Path::new("./tests/fixtures/schemes/tinty-generated.yaml"))?;
    let scheme_system = scheme_content
        .lines()
        .find_map(|line| line.strip_prefix("system: "))
        .expect("Fixture scheme should have a 'system' field");
    let scheme_name_with_system = format!("{scheme_system}-{scheme_name}");
    let (_, data_path, command_vec, _temp_dir) = setup(
        "test_cli_apply_subcommand_with_custom_schemes_quiet_flag",
        format!("apply {} --quiet", &scheme_name_with_system).as_str(),
    )?;
    let custom_scheme_file_path =
        data_path.join(format!("custom-schemes/{scheme_system}/{scheme_name}.yaml"));
    let current_scheme_path = data_path.join(ARTIFACTS_DIR).join(CURRENT_SCHEME_FILE_NAME);
    write_to_file(&custom_scheme_file_path, &scheme_content)?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        fs::read_to_string(current_scheme_path)? == scheme_name_with_system,
        "current_scheme_path different to scheme_name_with_system"
    );
    ensure!(stdout.is_empty(), "Expected empty stdout, got: {stdout}");
    ensure!(
        stderr.contains("W001"),
        "Expected stderr containing \"W001\", got: {stderr}"
    );

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_root_hooks_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, command_vec, _temp_dir) = setup(
        "test_cli_apply_subcommand_root_hooks_with_setup",
        format!("apply {scheme_name}").as_str(),
    )?;
    let expected_output = "This\nis\nexpected\noutput.\n";
    let config_content =
        r#"hooks = ["echo 'This'", "echo 'is'", "echo 'expected'", "echo 'output.'"]"#;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(stdout == expected_output, "stdout not as expected");
    ensure!(stderr.is_empty(), "Expected empty stderr, got: {stderr}");

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_root_hooks_has_envs_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-gruvbox-dark-hard";
    let (config_path, data_path, command_vec, _temp_dir) = setup(
        "test_cli_apply_subcommand_root_hooks_has_envs_with_setup",
        format!("apply {scheme_name}").as_str(),
    )?;
    let expected_output = "gruvbox-dark-hard 1d 20 21 79.727066 11.984515\n";
    let config_content = r#"hooks = ["echo $TINTY_SCHEME_SLUG $TINTY_SCHEME_PALETTE_BASE00_HEX_R $TINTY_SCHEME_PALETTE_BASE00_HEX_G $TINTY_SCHEME_PALETTE_BASE00_HEX_B $TINTY_SCHEME_LIGHTNESS_FOREGROUND $TINTY_SCHEME_LIGHTNESS_BACKGROUND"]"#;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(stdout == expected_output, "stdout not as expected");
    ensure!(stderr.is_empty(), "Expected empty stderr, got: {stderr}");

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_hook_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, command_vec, _temp_dir) = setup(
        "test_cli_apply_subcommand_hook_with_setup",
        format!("apply {scheme_name}").as_str(),
    )?;
    let config_content = r#"
[[items]]
path = "https://github.com/tinted-theming/tinted-vim"
name = "tinted-vim"
themes-dir = "colors"
hook = "echo \"path: %f, operation: %o\""
"#;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        stdout
            == format!(
                "path: {}/tinted-vim-colors-file.vim, operation: apply\n",
                data_path.join(ARTIFACTS_DIR).display()
            ),
        "stdout not as expected"
    );
    ensure!(stderr.is_empty(), "Expected empty stderr, got: {stderr}");

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_hook_with_envs_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, command_vec, _temp_dir) = setup(
        "test_cli_apply_subcommand_hook_with_envs_with_setup",
        format!("apply {scheme_name}").as_str(),
    )?;
    let config_content = r#"
[[items]]
path = "https://github.com/tinted-theming/tinted-vim"
name = "tinted-vim"
themes-dir = "colors"
hook = "echo \"path: $TINTY_THEME_FILE_PATH, operation: $TINTY_THEME_OPERATION, scheme: $TINTY_SCHEME_SLUG, color00 hex: ${TINTY_SCHEME_PALETTE_BASE00_HEX_R}${TINTY_SCHEME_PALETTE_BASE00_HEX_G}${TINTY_SCHEME_PALETTE_BASE00_HEX_B}\""
"#;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        stdout
        == format!(
            "path: {}/tinted-vim-colors-file.vim, operation: apply, scheme: oceanicnext, color00 hex: 1b2b34\n",
            data_path.join(ARTIFACTS_DIR).display()
        ),
        "stdout not as expected"
    );
    ensure!(stderr.is_empty(), "Expected empty stderr, got: {stderr}");

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_with_config_theme_file_extension() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-uwunicorn";
    let (config_path, data_path, command_vec, _temp_dir) = setup(
        "test_cli_apply_subcommand_with_config_theme_file_extension",
        format!("apply {scheme_name}").as_str(),
    )?;
    let config_content = r#"
[[items]]
path = "https://github.com/tinted-theming/tinted-vim"
name = "tinted-vim"
themes-dir = "colors"
hook = "echo \"expected vim output: %n\""

[[items]]
name = "tinted-shell"
path = "https://github.com/tinted-theming/tinted-shell"
theme-file-extension=".sh"
themes-dir="scripts"
hook = "echo \"expected shell output: %n\""
"#;
    let expected_output =
        "expected vim output: base16-uwunicorn\nexpected shell output: base16-uwunicorn\n";
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(stdout == expected_output, "stdout not as expected");
    ensure!(stderr.is_empty(), "Expected empty stderr, got: {stderr}");

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_removes_vestigial_files() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, command_vec, _temp_dir) = setup(
        "test_cli_apply_subcommand_removes_vestigial_files",
        format!("apply {scheme_name}").as_str(),
    )?;
    let config_content = r#"
[[items]]
path = "https://github.com/tinted-theming/tinted-vim"
name = "tinted-vim"
themes-dir = "colors"
hook = "echo \"path: %f, operation: %o\""
"#;
    write_to_file(&config_path, config_content)?;

    let vestigial_file = data_path.join("vestigial-file");
    write_to_file(&vestigial_file, "hello world")?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        stdout
            == format!(
                "path: {}/tinted-vim-colors-file.vim, operation: apply\n",
                data_path.join(ARTIFACTS_DIR).display()
            ),
        "stdout not as expected"
    );
    ensure!(stderr.is_empty(), "Expected empty stderr, got: {stderr}");

    ensure!(!vestigial_file.exists(), "vestigial file not removed");

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_removes_broken_symlinks() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, command_vec, _temp_dir) = setup(
        "test_cli_apply_subcommand_removes_broken_symlinks",
        format!("apply {scheme_name}").as_str(),
    )?;
    let config_content = r#"
[[items]]
path = "https://github.com/tinted-theming/tinted-vim"
name = "tinted-vim"
themes-dir = "colors"
hook = "echo \"path: %f, operation: %o\""
"#;
    write_to_file(&config_path, config_content)?;

    let symlink_name = "im-no-longer-here";
    let missing_file = data_path.join(ARTIFACTS_DIR).join(symlink_name);
    write_to_file(&missing_file, "hello")?;
    let symlink = data_path.join(symlink_name);
    std::os::unix::fs::symlink(&missing_file, &symlink)?;
    // Regular file
    fs::remove_file(&missing_file)?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        stdout
            == format!(
                "path: {}/tinted-vim-colors-file.vim, operation: apply\n",
                data_path.join(ARTIFACTS_DIR).display()
            ),
        "stdout not as expected"
    );
    ensure!(stderr.is_empty(), "Expected empty stderr, got: {stderr}");

    ensure!(!missing_file.exists(), "file is supposed to be missing");

    ensure!(!symlink.exists(), "broken symlink wasn't deleted");

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_without_config_shell_required_string() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, command_vec, _temp_dir) = setup(
        "test_cli_apply_subcommand_without_config_shell_required_string",
        format!("apply {scheme_name}").as_str(),
    )?;
    let config_content = "shell = \"string does not contain curcle braces\"";
    write_to_file(&config_path, config_content)?;

    let expected_stderr =
        "The configured shell does not contain the required command placeholder '{}'. Check the default file or github for config examples.";

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(stdout.is_empty(), "stdout not as expected");
    ensure!(
        stderr.contains(expected_stderr),
        "Expected stderr to contain: {expected_stderr}\nGot: {stderr}"
    );

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_with_failing_hook() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-oceanicnext";
    let (config_path, data_path, command_vec, _temp_dir) = setup(
        "test_cli_apply_subcommand_with_failing_hook",
        format!("apply {scheme_name}").as_str(),
    )?;
    let config_content = r#"hooks = ["nonexistent-command-that-should-fail"]"#;
    write_to_file(&config_path, config_content)?;

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        !stderr.is_empty(),
        "Expected stderr to report hook failure, but stderr was empty"
    );

    Ok(())
}

#[test]
fn test_cli_apply_subcommand_with_unicode_scheme_name() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-\u{00e9}\u{00e8}\u{00ea}";
    let (_, data_path, command_vec, _temp_dir) = setup(
        "test_cli_apply_subcommand_with_unicode_scheme_name",
        format!("apply {scheme_name}").as_str(),
    )?;

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        !stderr.is_empty(),
        "Expected error for non-existent unicode scheme name, but stderr was empty"
    );

    Ok(())
}
