mod utils;

use anyhow::{ensure, Context, Result};
use serde_json::Value;
use std::fs;
use utils::setup;

#[test]
fn test_cli_gallery_subcommand_dump_creates_html_file() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, mut command_vec, _temp_dir) = setup(
        "test_cli_gallery_subcommand_dump_creates_html_file",
        "gallery --custom-schemes --no-open",
    )?;
    let custom_base16_path = data_path.join("custom-schemes/base16");
    let dump_path = data_path.join("gallery-dump");

    fs::create_dir_all(&custom_base16_path)?;
    fs::copy(
        "fixtures/tinty-city-dark.yaml",
        custom_base16_path.join("tinty-city-dark.yaml"),
    )?;

    command_vec.push("--dump".to_string());
    command_vec.push(dump_path.display().to_string());

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(&command_vec, &data_path, false)?;

    // ------
    // Assert
    // ------
    ensure!(
        stderr.is_empty(),
        "Expected stderr to be empty, got: {stderr}"
    );
    ensure!(
        dump_path.join("index.html").is_file(),
        "Expected gallery dump to produce index.html"
    );

    Ok(())
}

#[test]
fn test_cli_gallery_subcommand_embeds_complete_scheme_json() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, mut command_vec, _temp_dir) = setup(
        "test_cli_gallery_subcommand_embeds_complete_scheme_json",
        "gallery --custom-schemes --no-open",
    )?;
    let custom_base16_path = data_path.join("custom-schemes/base16");
    let dump_path = data_path.join("gallery-dump");

    fs::create_dir_all(&custom_base16_path)?;
    fs::copy(
        "fixtures/tinty-city-dark.yaml",
        custom_base16_path.join("tinty-city-dark.yaml"),
    )?;
    fs::copy(
        "tests/fixtures/schemes/tinty-generated.yaml",
        custom_base16_path.join("tinty-generated.yaml"),
    )?;

    command_vec.push("--dump".to_string());
    command_vec.push(dump_path.display().to_string());

    // ---
    // Act
    // ---
    let (_, gallery_stderr) = utils::run_command(&command_vec, &data_path, false)?;
    let list_command_vec =
        utils::build_command_vec("list --custom-schemes --json", &config_path, &data_path)?;
    let (list_stdout, list_stderr) = utils::run_command(&list_command_vec, &data_path, false)?;

    // ------
    // Assert
    // ------
    ensure!(
        gallery_stderr.is_empty(),
        "Expected gallery stderr to be empty, got: {gallery_stderr}"
    );
    ensure!(
        list_stderr.is_empty(),
        "Expected list stderr to be empty, got: {list_stderr}"
    );

    let gallery_js = fs::read_to_string(dump_path.join("assets/gallery.js"))?;
    let gallery_json = embedded_schemes_json(&gallery_js)?;
    let gallery_schemes: Vec<Value> = serde_json::from_str(gallery_json)?;
    let list_schemes: Vec<Value> = serde_json::from_str(&list_stdout)?;

    ensure!(
        scheme_ids(&gallery_schemes)? == scheme_ids(&list_schemes)?,
        "Expected gallery scheme ids to match list --json scheme ids"
    );
    ensure!(
        gallery_schemes.iter().all(scheme_has_gallery_data),
        "Expected every embedded scheme to include palette and lightness data"
    );

    Ok(())
}

fn embedded_schemes_json(gallery_js: &str) -> Result<&str> {
    let value = gallery_js
        .strip_prefix("const SCHEMES = ")
        .context("gallery.js did not start with embedded scheme data")?;
    let (json, _) = value
        .split_once(";\n\nconst state =")
        .context("gallery.js did not contain the expected scheme data delimiter")?;

    Ok(json)
}

fn scheme_ids(schemes: &[Value]) -> Result<Vec<String>> {
    schemes
        .iter()
        .map(|scheme| {
            scheme
                .get("id")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .context("scheme entry did not include an id")
        })
        .collect()
}

fn scheme_has_gallery_data(scheme: &Value) -> bool {
    scheme
        .get("palette")
        .and_then(Value::as_object)
        .is_some_and(|palette| !palette.is_empty())
        && scheme.get("lightness").is_some_and(Value::is_object)
}
