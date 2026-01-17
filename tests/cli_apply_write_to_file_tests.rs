mod utils;

use anyhow::{ensure, Result};
use std::env::current_dir;
use std::fs;
use std::path::Path;

use crate::utils::{setup, write_to_file, ARTIFACTS_DIR};

fn prepare_minimal_repos(data_path: &Path, scheme_name: &str, theme_contents: &str) -> Result<()> {
    // Create a minimal builtin schemes directory with a valid scheme file
    let builtin_scheme_file = data_path
        .join("repos/schemes/base16")
        .join(format!("{}.yaml", scheme_name.replace("base16-", "")));

    if let Some(builtin_scheme_dir) = builtin_scheme_file.parent() {
        if !builtin_scheme_dir.is_dir() {
            fs::create_dir_all(builtin_scheme_dir)?;
        }
    }

    // Use the existing fixture to satisfy SchemeFile parsing
    {
        let fixture = Path::new("./tests/fixtures/schemes/tinty-generated.yaml");
        let fixture_contents = fs::read_to_string(fixture)?;
        write_to_file(&builtin_scheme_file, &fixture_contents)?;
    };

    // Create a minimal repo for tinted-shell with a controllable theme file
    {
        let themes_dir = data_path.join("repos/tinted-shell/scripts");
        if !themes_dir.is_dir() {
            fs::create_dir_all(&themes_dir)?;
        }
        let theme_file = themes_dir.join(format!("{scheme_name}.sh"));
        write_to_file(&theme_file, theme_contents)?;
    };

    // Ensure artifacts dir exists for downstream steps that expect it
    fs::create_dir_all(data_path.join(ARTIFACTS_DIR))?;

    Ok(())
}

#[test]
fn test_cli_apply_write_to_file_with_start_and_end_markers() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-tinty-generated";
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_apply_write_to_file_with_start_and_end_markers",
        format!("apply {scheme_name}").as_str(),
    )?;
    let theme_contents = "THEME-CONTENT-123";
    let target_path = data_path.join("data/markers.txt");
    let start_marker = "<!-- TINTY START -->";
    let end_marker = "<!-- TINTY END -->";
    let abs_target_path = current_dir()?.join(&target_path);

    prepare_minimal_repos(&data_path, scheme_name, theme_contents)?;
    {
        let original_target_content =
            format!("prologue\n{start_marker}\n\n\nold-content\n{end_marker}\nepilogue\n");
        write_to_file(&target_path, &original_target_content)?;
    }
    {
        let config_content = format!(
            r#"[[items]]
path = "https://github.com/tinted-theming/tinted-shell"
name = "tinted-shell"
themes-dir = "scripts"
write-to-file = ["{}", "{start_marker}", "{end_marker}"]
"#,
            abs_target_path.display()
        );
        write_to_file(&config_path, &config_content)?;
    }

    // ---
    // Act
    // ---
    let (_stdout, _stderr) = utils::run_command(&command_vec, &data_path, false)?;

    // ------
    // Assert
    // ------
    let updated_target_content = fs::read_to_string(&target_path)?;

    println!("{updated_target_content}");
    ensure!(
        updated_target_content.contains(start_marker),
        "target missing start marker"
    );
    ensure!(
        updated_target_content.contains(end_marker),
        "target missing end marker"
    );
    ensure!(
        updated_target_content.contains(theme_contents),
        "target missing theme content"
    );
    ensure!(
        !updated_target_content.contains("old-content"),
        "target still contains old contents"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_apply_write_to_file_with_start_marker_only() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-tinty-generated";
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_apply_write_to_file_with_start_marker_only",
        format!("apply {scheme_name}").as_str(),
    )?;
    let theme_contents = "THEME-CONTENT-ABC";
    let target_path = data_path.join("data/markers.txt");
    let start_marker = "# TINTY START";

    prepare_minimal_repos(&data_path, scheme_name, theme_contents)?;
    {
        let original_target = format!("prefix\n{start_marker}\nremove-this-trailing-content\n");
        write_to_file(&target_path, &original_target)?;
    };
    {
        let config_content = format!(
            r#"[[items]]
path = "https://github.com/tinted-theming/tinted-shell"
name = "tinted-shell"
themes-dir = "scripts"
write-to-file = ["{}", "{}"]
"#,
            target_path.display(),
            start_marker
        );
        write_to_file(&config_path, &config_content)?;
    };

    // ---
    // Act
    // ---
    let (_stdout, _stderr) = utils::run_command(&command_vec, &data_path, false)?;

    // ------
    // Assert
    // ------
    let new_target = fs::read_to_string(&target_path)?;
    ensure!(
        new_target.contains(start_marker),
        "target missing start marker"
    );
    ensure!(
        new_target.contains(theme_contents),
        "target missing theme content"
    );
    ensure!(
        !new_target.contains("remove-this-trailing-content"),
        "target still contains old trailing content"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_apply_write_to_file_overwrite_full_file() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_name = "base16-tinty-generated";
    let (config_path, data_path, command_vec, cleanup) = setup(
        "test_cli_apply_write_to_file_overwrite_full_file",
        format!("apply {scheme_name}").as_str(),
    )?;
    let theme_contents = "THEME-CONTENT-XYZ";
    let target_path = data_path.join("data/markers.txt");

    prepare_minimal_repos(&data_path, scheme_name, theme_contents)?;
    write_to_file(&target_path, "Some random file content")?;
    {
        let config_content = format!(
            r#"[[items]]
path = "https://github.com/tinted-theming/tinted-shell"
name = "tinted-shell"
themes-dir = "scripts"
write-to-file = ["{}"]
"#,
            target_path.display()
        );
        write_to_file(&config_path, &config_content)?;
    };

    // ---
    // Act
    // ---
    let (_stdout, _stderr) = utils::run_command(&command_vec, &data_path, false)?;

    // ------
    // Assert
    // ------
    let new_target = fs::read_to_string(&target_path)?;
    ensure!(
        new_target == theme_contents,
        "file contents not overwritten"
    );

    cleanup()?;
    Ok(())
}
