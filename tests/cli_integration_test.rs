use anyhow::{Context, Result};
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
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
        fs::remove_dir_all(path)?;
    }

    Ok(())
}

fn run_setup_command(config_path: &Path) -> Result<()> {
    // Execute the first command
    let output_setup = Command::new("./target/debug/base16_shell")
        .args([
            "setup",
            format!("--config={}", config_path.display()).as_str(),
        ])
        .output()
        .expect("Failed to execute setup command");

    assert!(
        output_setup.status.success(),
        "Setup command failed with status: {}",
        output_setup.status
    );
    if !output_setup.stderr.is_empty() {
        anyhow::bail!(
            "Setup command stderr: {}",
            String::from_utf8_lossy(&output_setup.stderr)
        );
    }

    Ok(())
}

fn run_target_command(args: &[&str]) -> Result<String, Box<dyn Error>> {
    let output = Command::new("./target/debug/base16_shell")
        .args(args)
        .output()
        .expect("Failed to execute command");

    if !output.stderr.is_empty() {
        println!(
            "Init command stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout).expect("Not valid UTF-8");

    Ok(stdout)
}

#[test]
fn test_cli_no_arguments() -> Result<()> {
    let config_path = Path::new("base16_shell_test_cli_no_arguments");
    remove_dir(&config_path)?;

    let output = Command::new("./target/debug/base16_shell")
        .arg(format!("--config={}", config_path.display()))
        .output()
        .expect("Failed to execute command");
    let stdout = str::from_utf8(&output.stdout).expect("Not valid UTF-8");

    assert!(output.status.success());
    assert!(stdout.contains("Basic usage: base16-shell-manager set <SCHEME_NAME>"));
    assert!(stdout.contains("For more information try --help"));

    // Cleanup
    remove_dir(&config_path)?;

    Ok(())
}

#[test]
fn test_cli_setup_command_existing_repo() -> Result<()> {
    // -------
    // Arrange
    // -------

    let config_path = Path::new("base16_shell_test_cli_setup_command_existing_config");
    remove_dir(&config_path)?;
    let data_path: PathBuf = env::var("XDG_DATA_HOME").map(PathBuf::from).or_else(|_| {
        env::var("HOME")
            .map_err(anyhow::Error::new)
            .map(|home| PathBuf::from(home).join(".local/share"))
            .context("HOME environment variable not set")
    })?;
    let repo_path = data_path.join("tinted-theming/base16-shell-manager");
    let expected_output = format!("{} already exists. If you want to update try running the `update` subcommand or `--help` for more information.", repo_path.display());
    let mut is_test_repo_dir = false;

    if !repo_path.exists() {
        is_test_repo_dir = true;
        fs::create_dir_all(&repo_path)?;
    }

    assert!(repo_path.exists(), "Repo path should exist");

    // ---
    // Act
    // ---

    let subcommand = "setup";
    let config_flag = format!("--config={}", config_path.display());
    let args: &[&str] = &[subcommand, &config_flag];
    let stdout = run_target_command(args).unwrap();

    println!("1: {}", stdout);
    println!("2: {}", &expected_output);

    // ------
    // Assert
    // ------

    assert!(
        stdout.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    // Cleanup
    if is_test_repo_dir {
        remove_dir(&repo_path)?;
    }
    remove_dir(&config_path)?;

    Ok(())
}

#[test]
fn test_cli_setup_command_with_repo_dir_flag() -> Result<()> {
    // -------
    // Arrange
    // -------

    let name = "base16_shell_test_cli_setup_command_with_repo_dir_flag";
    let config_path = Path::new(&name);
    let repo_path = Path::new(&name);
    remove_dir(&config_path)?;
    let expected_output = format!("The setup command will not work since you have provided your own `--repo-dir` at {}. Visit https://github.com/tinted-theming/base16-shell-manager to see instructions on manually updating.", name);

    // ---
    // Act
    // ---

    let subcommand = "setup";
    let repo_dir_flag = format!("--repo-dir={}", repo_path.display());
    let config_flag = format!("--config={}", config_path.display());
    let args: &[&str] = &[subcommand, &config_flag, &repo_dir_flag];
    let stdout = run_target_command(args).unwrap();

    // ------
    // Assert
    // ------

    assert!(
        stdout.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    remove_dir(&config_path)?;
    Ok(())
}

#[test]
fn test_cli_init_command_existing_config() -> Result<()> {
    // -------
    // Arrange
    // -------

    let config_path = Path::new("base16_shell_test_cli_init_command_existing_config");
    remove_dir(&config_path)?;
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

    fs::create_dir_all(config_path)?;
    write_to_file(
        &base16_shell_colorscheme_path,
        format!("echo '{}'", expected_output).as_str(),
    )?;
    write_to_file(&base16_shell_theme_name_path, "mocha")?;

    // ---
    // Act
    // ---

    let subcommand = "init";
    let config_flag = format!("--config={}", config_path.display());
    let args: &[&str] = &[subcommand, &config_flag];
    run_setup_command(config_path).unwrap();
    let stdout = run_target_command(args).unwrap();

    // ------
    // Assert
    // ------

    assert!(
        stdout.contains(expected_output),
        "stdout does not contain the expected output"
    );

    // Cleanup
    remove_dir(&config_path)?;

    Ok(())
}

#[test]
fn test_cli_init_command_empty_config() -> Result<()> {
    // -------
    // Arrange
    // -------

    let config_path = Path::new("base16_shell_test_cli_init_command_empty_config");
    remove_dir(&config_path)?;
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
    let subcommand = "init";
    let config_flag = format!("--config={}", config_path.display());
    let args: &[&str] = &[subcommand, &config_flag];
    let stdout = run_target_command(args).unwrap();

    // ------
    // Assert
    // ------

    assert!(
        stdout.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    // Cleanup
    remove_dir(config_path)?;

    Ok(())
}

#[test]
fn test_cli_list_subcommand() -> Result<()> {
    // -------
    // Arrange
    // -------

    let config_path = Path::new("base16_shell_test_cli_list_subcommand");
    remove_dir(&config_path)?;
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
    let subcommand = "list";
    let config_flag = format!("--config={}", config_path.display());
    let args: &[&str] = &[subcommand, &config_flag];
    run_setup_command(config_path)?;
    let stdout = run_target_command(args).unwrap();
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
    remove_dir(&config_path)?;

    Ok(())
}

#[test]
fn test_cli_set_command() -> Result<()> {
    // -------
    // Arrange
    // -------

    let config_path = Path::new("base16_shell_test_cli_set_command");
    remove_dir(&config_path)?;
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
    let subcommand = "set";
    let config_flag = format!("--config={}", config_path.display());
    let args: &[&str] = &[subcommand, scheme_name, &config_flag];
    run_setup_command(config_path)?;
    let stdout = run_target_command(args).unwrap();
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
    remove_dir(&config_path)?;

    Ok(())
}
