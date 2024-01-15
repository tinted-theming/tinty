use anyhow::Result;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

pub fn write_to_file(path: &Path, contents: &str) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }

    let mut file = File::create(path)?;
    file.write_all(contents.as_bytes())?;

    Ok(())
}

#[test]
fn test_cli_no_arguments() {
    let output = Command::new("./target/debug/base16_shell")
        .output()
        .expect("Failed to execute command");
    let stdout = str::from_utf8(&output.stdout).expect("Not valid UTF-8");

    assert!(output.status.success());
    assert!(stdout.contains("Basic usage: base16-shell set <SCHEME_NAME>"));
    assert!(stdout.contains("For more information try --help"));
}

#[test]
fn test_cli_init_command_existing_config() {
    // -------
    // Arrange
    // -------

    let expected_output = "some random text";
    let config_path: PathBuf = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| env::var("HOME").and_then(|home| Ok(PathBuf::from(home).join(".config"))))
        .unwrap();
    let base16_config_path = config_path.join("tinted-theming");
    let base16_shell_colorscheme_path = base16_config_path.join("base16_shell_theme");
    let base16_shell_theme_name_path = base16_config_path.join("theme_name");
    if base16_shell_colorscheme_path.exists() {
        if let Err(e) = fs::remove_file(&base16_shell_colorscheme_path) {
            println!("Error removing file: {}", e);
        }
    }
    if base16_shell_theme_name_path.exists() {
        if let Err(e) = fs::remove_file(&base16_shell_theme_name_path) {
            println!("Error removing file: {}", e);
        }
    }

    // Make sure the files don't exist so we can ensure the cli tool has created them
    assert!(
        !base16_shell_colorscheme_path.exists(),
        "Colorscheme file should not exist before test"
    );
    assert!(
        !base16_shell_theme_name_path.exists(),
        "Theme name file should not exist before test"
    );

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
        .arg("init")
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
}

#[test]
fn test_cli_init_command_empty_config() {
    // -------
    // Arrange
    // -------

    let config_path: PathBuf = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| env::var("HOME").and_then(|home| Ok(PathBuf::from(home).join(".config"))))
        .unwrap();
    let base16_config_path = config_path.join("tinted-theming");
    let base16_shell_colorscheme_path = base16_config_path.join("base16_shell_theme");
    let base16_shell_theme_name_path = base16_config_path.join("theme_name");
    let expected_output =
        "Config files don't exist, run `base16_shell set <THEME_NAME>` to create them";
    if base16_shell_colorscheme_path.exists() {
        if let Err(e) = fs::remove_file(&base16_shell_colorscheme_path) {
            println!("Error removing file: {}", e);
        }
    }
    if base16_shell_theme_name_path.exists() {
        if let Err(e) = fs::remove_file(&base16_shell_theme_name_path) {
            println!("Error removing file: {}", e);
        }
    }

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
        .arg("init")
        .output()
        .expect("Failed to execute command");
    let stdout = str::from_utf8(&output.stdout).expect("Not valid UTF-8");

    println!("from test {}, claiming its not: {}", stdout, &expected_output);

    // ------
    // Assert
    // ------

    assert!(
        stdout.contains(&expected_output),
        "stdout does not contain the expected output"
    );
}

#[test]
fn test_cli_list_subcommand() {
    // -------
    // Arrange
    // -------

    let colorschemes_dir = Path::new("./scripts");
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
        .arg("list")
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
}

#[test]
fn test_cli_set_command() {
    // -------
    // Arrange
    // -------

    let scheme_name = "ocean";
    let config_path: PathBuf = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| env::var("HOME").and_then(|home| Ok(PathBuf::from(home).join(".config"))))
        .unwrap();
    let base16_config_path = config_path.join("tinted-theming");
    let base16_shell_colorscheme_path = base16_config_path.join("base16_shell_theme");
    let base16_shell_theme_name_path = base16_config_path.join("theme_name");
    let expected_output = format!("Theme set to: {}", scheme_name);

    if base16_shell_colorscheme_path.exists() {
        match fs::remove_file(&base16_shell_colorscheme_path) {
            Ok(_) => {}
            Err(e) => println!("Error removing file: {}", e),
        }
    }
    if base16_shell_theme_name_path.exists() {
        match fs::remove_file(&base16_shell_theme_name_path) {
            Ok(_) => {}
            Err(e) => println!("Error removing file: {}", e),
        }
    }

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
        .args(&["set", scheme_name])
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
}
