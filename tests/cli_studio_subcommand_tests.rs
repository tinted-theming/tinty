mod utils;

use anyhow::{ensure, Result};
use std::fs;
use utils::setup;

#[test]
fn test_cli_studio_subcommand_dump_creates_static_site() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, mut command_vec, _temp_dir) = setup(
        "test_cli_studio_subcommand_dump_creates_static_site",
        "studio --no-open",
        false,
    )?;
    let dump_path = data_path.join("studio-dump");

    command_vec.push("--dump".to_string());
    command_vec.push(dump_path.display().to_string());

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(&command_vec)?;

    // ------
    // Assert
    // ------
    ensure!(
        stderr.is_empty(),
        "Expected stderr to be empty, got: {stderr}"
    );
    ensure!(
        dump_path.join("index.html").is_file(),
        "Expected studio dump to produce index.html"
    );
    ensure!(
        dump_path.join("assets/studio.js").is_file(),
        "Expected studio dump to produce assets/studio.js"
    );
    ensure!(
        dump_path.join("assets/studio.css").is_file(),
        "Expected studio dump to produce assets/studio.css"
    );

    Ok(())
}

#[test]
fn test_cli_studio_subcommand_embeds_scheme_library() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, mut command_vec, _temp_dir) = setup(
        "test_cli_studio_subcommand_embeds_scheme_library",
        "studio --no-open",
        false,
    )?;
    let dump_path = data_path.join("studio-dump");

    command_vec.push("--dump".to_string());
    command_vec.push(dump_path.display().to_string());

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(&command_vec)?;

    // ------
    // Assert
    // ------
    ensure!(
        stderr.is_empty(),
        "Expected stderr to be empty, got: {stderr}"
    );

    let studio_js = fs::read_to_string(dump_path.join("assets/studio.js"))?;
    ensure!(
        !studio_js.contains("__TINTY_SCHEMES__"),
        "Expected the scheme-library placeholder to be substituted"
    );
    ensure!(
        studio_js.contains("LIBRARY = ["),
        "Expected studio.js to embed a JSON scheme library array"
    );

    Ok(())
}
