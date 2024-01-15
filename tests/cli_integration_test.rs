use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::str;

fn write_to_file(path: &Path, contents: &str) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }

    let mut file = File::create(path)?;
    file.write_all(contents.as_bytes())?;

    Ok(())
}

fn remove_dir(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path).unwrap();
    }

    Ok(())
}

#[test]
fn test_cli_no_arguments() {
    let config_path = Path::new("base16_shell_test_cli_no_arguments");
    remove_dir(&config_path).unwrap();

    let output = Command::new("./target/debug/base16_shell")
        .arg(format!("--config={}", config_path.display()))
        .output()
        .expect("Failed to execute command");
    let stdout = str::from_utf8(&output.stdout).expect("Not valid UTF-8");

    assert!(output.status.success());
    assert!(stdout.contains("Basic usage: base16-shell set <SCHEME_NAME>"));
    assert!(stdout.contains("For more information try --help"));

    // Cleanup
    remove_dir(&config_path).unwrap();
}

#[test]
fn test_cli_init_command_existing_config() {
    // -------
    // Arrange
    // -------

    let config_path = Path::new("base16_shell_test_cli_init_command_existing_config");
    remove_dir(&config_path).unwrap();
    let expected_output = "some random text";
    let base16_shell_colorscheme_path = config_path.join("base16_shell_theme");
    let base16_shell_theme_name_path = config_path.join("theme_name");

    // Make sure the files don't exist so we can ensure the cli tool has created them
    assert!(
        !base16_shell_colorscheme_path.exists(),
        "Colorscheme file should not exist before test"
    );
    assert!(
        !base16_shell_theme_name_path.exists(),
        "Theme name file should not exist before test"
    );

    fs::create_dir_all(config_path).unwrap();
    write_to_file(
        &base16_shell_colorscheme_path,
        format!("echo '{}'", expected_output).as_str(),
    )
    .unwrap();
    write_to_file(&base16_shell_theme_name_path, "mocha").unwrap();

    // ---
    // Act
    // ---

    let output = Command::new("./target/debug/base16_shell")
        .args([
            "init",
            format!("--config={}", config_path.display()).as_str(),
        ])
        .output()
        .expect("Failed to execute command");
    let stdout = str::from_utf8(&output.stdout).expect("Not valid UTF-8");

    // ------
    // Assert
    // ------

    assert!(
        stdout.contains(expected_output),
        "stdout does not contain the expected output"
    );

    // Cleanup
    remove_dir(&config_path).unwrap();
}

#[test]
fn test_cli_init_command_empty_config() {
    // -------
    // Arrange
    // -------

    let config_path = Path::new("base16_shell_test_cli_init_command_empty_config");
    remove_dir(&config_path).unwrap();
    let base16_shell_colorscheme_path = config_path.join("base16_shell_theme");
    let base16_shell_theme_name_path = config_path.join("theme_name");
    let expected_output =
        "Config files don't exist, run `base16_shell set <THEME_NAME>` to create them";

    // Make sure the files don't exist so we can ensure the cli tool has created them
    assert!(
        !base16_shell_colorscheme_path.exists(),
        "Colorscheme file should not exist before test"
    );
    assert!(
        !base16_shell_theme_name_path.exists(),
        "Theme name file should not exist before test"
    );

    // ---
    // Act
    // ---

    let output = Command::new("./target/debug/base16_shell")
        .args([
            "init",
            format!("--config={}", config_path.display()).as_str(),
        ])
        .output()
        .expect("Failed to execute command");
    let stdout = str::from_utf8(&output.stdout).expect("Not valid UTF-8");

    // ------
    // Assert
    // ------

    assert!(
        stdout.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    // Cleanup
    remove_dir(config_path).unwrap();
}

#[test]
fn test_cli_list_subcommand() {
    // -------
    // Arrange
    // -------

    let config_path = Path::new("base16_shell_test_cli_list_subcommand");
    remove_dir(&config_path).unwrap();
    let colorschemes_dir = Path::new("./themes");
    let mut expected_colorschemes = fs::read_dir(colorschemes_dir)
        .expect("Failed to read colorschemes directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            path.file_stem()
                .and_then(|name| name.to_str())
                .and_then(|name| name.strip_prefix("base16-"))
                .map(|name| name.to_string())
        })
        .collect::<Vec<String>>();
    expected_colorschemes.sort();

    // ---
    // Act
    // ---

    let output = Command::new("./target/debug/base16_shell")
        .args([
            "list",
            format!("--config={}", config_path.display()).as_str(),
        ])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());
    let stdout = str::from_utf8(&output.stdout).expect("Output not valid UTF-8");
    let mut actual_colorschemes = stdout
        .lines()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    actual_colorschemes.sort();

    // ------
    // Assert
    // ------

    assert_eq!(expected_colorschemes, actual_colorschemes);

    // Cleanup
    remove_dir(&config_path).unwrap();
}

#[test]
fn test_cli_set_command() {
    // -------
    // Arrange
    // -------

    let config_path = Path::new("base16_shell_test_cli_set_command");
    remove_dir(&config_path).unwrap();
    let scheme_name = "ocean";
    let base16_shell_colorscheme_path = config_path.join("base16_shell_theme");
    let base16_shell_theme_name_path = config_path.join("theme_name");
    let expected_output = format!("Theme set to: {}", scheme_name);

    // Make sure the files don't exist so we can ensure the cli tool has created them
    assert!(
        !base16_shell_colorscheme_path.exists(),
        "Colorscheme file should not exist before test"
    );
    assert!(
        !base16_shell_theme_name_path.exists(),
        "Theme name file should not exist before test"
    );

    // ---
    // Act
    // ---

    let output = Command::new("./target/debug/base16_shell")
        .args(&[
            "set",
            scheme_name,
            format!("--config={}", config_path.display()).as_str(),
        ])
        .output()
        .expect("Failed to execute command");
    let stdout = str::from_utf8(&output.stdout).expect("Not valid UTF-8");
    let theme_name_content =
        fs::read_to_string(base16_shell_theme_name_path).expect("Failed to read theme name file");
    let colorscheme_content =
        fs::read_to_string(base16_shell_colorscheme_path).expect("Failed to read colorscheme file");

    // ------
    // Assert
    // ------

    assert!(
        stdout.contains(&expected_output),
        "stdout does not contain the expected output"
    );
    assert!(
        colorscheme_content.contains(scheme_name),
        "Colorscheme file content is incorrect"
    );
    assert!(
        theme_name_content.contains(scheme_name),
        "Theme name file content is incorrect"
    );

    // Cleanup
    remove_dir(&config_path).unwrap();
}
