mod utils;

use anyhow::Result;
use utils::{
    copy_dir_all, setup, write_to_file, ARTIFACTS_DIR, CURRENT_SCHEME_FILE_NAME, REPO_DIR,
    SCHEMES_REPO_NAME,
};

#[test]
fn test_cli_current_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_current_subcommand_with_setup", "current")?;
    let scheme_name = "base16-oceanicnext";
    let current_scheme_path = data_path.join(ARTIFACTS_DIR).join(CURRENT_SCHEME_FILE_NAME);
    let schemes_dir = data_path.join(format!("{}/{}", REPO_DIR, SCHEMES_REPO_NAME));

    write_to_file(&current_scheme_path, scheme_name)?;
    write_to_file(&current_scheme_path, scheme_name)?;
    copy_dir_all("./tests/fixtures/schemes", schemes_dir)?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------
    assert_eq!(stdout, format!("{}\n", scheme_name));
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_current_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_current_subcommand_without_setup", "current")?;

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------
    cleanup()?;
    assert!(
        stderr
            .contains("Failed to read last scheme from file. Try applying a scheme and try again."),
        "stderr does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_current_subcommand_with_variant_existing() -> Result<()> {
    run_test_for_subcommand("variant", |output| output == "dark\n")
}

#[test]
fn test_cli_current_subcommand_with_system_existing() -> Result<()> {
    run_test_for_subcommand("system", |output| output == "base16\n")
}

#[test]
fn test_cli_current_subcommand_with_name_existing() -> Result<()> {
    run_test_for_subcommand("name", |output| output == "Tinty Generated\n")
}

#[test]
fn test_cli_current_subcommand_with_slug_existing() -> Result<()> {
    run_test_for_subcommand("slug", |output| output == "tinty-generated\n")
}

#[test]
fn test_cli_current_subcommand_with_description_not_existing() -> Result<()> {
    run_test_for_subcommand("description", |output| output == "\n")
}

#[test]
fn test_cli_current_subcommand_with_author_existing() -> Result<()> {
    run_test_for_subcommand("author", |output| output == "Tinty\n")
}

fn run_test_for_subcommand<F>(subcommand: &str, validate_output: F) -> Result<()>
where
    F: Fn(&str) -> bool,
{
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) = setup(
        &format!("test_cli_current_subcommand_with_{}_existing", subcommand),
        &format!("current {}", subcommand),
    )?;

    let scheme_name = "base16-tinty-generated";
    let current_scheme_path = data_path.join(format!("{ARTIFACTS_DIR}/{CURRENT_SCHEME_FILE_NAME}"));
    let schemes_dir = data_path.join(format!("{}/{}", REPO_DIR, SCHEMES_REPO_NAME));

    write_to_file(&current_scheme_path, scheme_name)?;
    copy_dir_all("./tests/fixtures/schemes", schemes_dir)?;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(command_vec, &data_path, false).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        validate_output(&stdout), // Pass the actual output to the validation function
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}
