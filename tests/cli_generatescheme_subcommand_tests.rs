mod utils;

use std::fs;

use crate::utils::setup;
use anyhow::{ensure, Result};
use utils::CUSTOM_SCHEMES_DIR_NAME;

#[test]
fn test_cli_generatescheme_subcommand_custom_properties() -> Result<()> {
    // ---
    // Act
    // ---
    let system = "base24";
    let author = "Some Author (https://github.com/tinted-theming)";
    let name = "Some custom name";
    let slug = "some-custom-slug";
    let variant = "light";
    let (_, data_path, command_vec, cleanup) = setup(
        "test_cli_generatescheme_subcommand_custom_properties",
        format!(
          "generate-scheme --author \"{author}\" --name \"{name}\" --slug {slug} --system {system} --variant {variant} ./tests/fixtures/assets/article-featured-image.webp",
        )
        .as_str(),
    )?;
    let expected_output = format!(
        r##"author: "{author}"
name: "{name}"
slug: "{slug}"
system: "{system}"
variant: "{variant}"
palette:
  base00: "#f7f7f8"
  base01: "#d5d9d7"
  base02: "#b3bbb7"
  base03: "#929d96"
  base04: "#707f76"
  base05: "#4f6155"
  base06: "#2d4335"
  base07: "#0c2615"
  base08: "#d45520"
  base09: "#f0840a"
  base0A: "#e6a343"
  base0B: "#0d9c33"
  base0C: "#32ebea"
  base0D: "#075cdd"
  base0E: "#6a2b98"
  base0F: "#8e6880"
  base10: "#a6674d"
  base11: "#b58044"
  base12: "#bc9b6c"
  base13: "#317744"
  base14: "#61bbbb"
  base15: "#3d67a6"
  base16: "#65467c"
  base17: "#84717d"
"##
    );

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        stdout == expected_output,
        "stdout does not contain the expected output"
    );
    ensure!(
        stderr.is_empty(),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_generatescheme_subcommand_with_image() -> Result<()> {
    // ---
    // Act
    // ---
    let (_, data_path, command_vec, cleanup) = setup(
        "test_cli_generatescheme_subcommand_with_image",
        "generate-scheme --system base16 ./tests/fixtures/assets/article-featured-image.webp",
    )?;
    let expected_output = r##"author: "Tinty"
name: "Tinty Generated"
slug: "tinty-generated"
system: "base16"
variant: "dark"
palette:
  base00: "#0f2c19"
  base01: "#304938"
  base02: "#516658"
  base03: "#728378"
  base04: "#93a098"
  base05: "#b4bdb8"
  base06: "#d5dad8"
  base07: "#f7f7f8"
  base08: "#d45520"
  base09: "#f0840a"
  base0A: "#e6a343"
  base0B: "#0d9c33"
  base0C: "#32ebea"
  base0D: "#075cdd"
  base0E: "#6a2b98"
  base0F: "#8e6880"
"##;

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        stdout == expected_output,
        "stdout does not contain the expected output"
    );
    ensure!(
        stderr.is_empty(),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_generatescheme_subcommand_with_save() -> Result<()> {
    // ---
    // Act
    // ---
    let scheme_system = "base16";
    let scheme_slug = "test-scheme-slug";
    let scheme_description = "Some description";
    let (_, data_path, command_vec, cleanup) = setup(
        "test_cli_generatescheme_subcommand_with_save",
        format!("generate-scheme --slug {scheme_slug} --system {scheme_system} --description \"{scheme_description}\" --save ./tests/fixtures/assets/article-featured-image.webp").as_str(),
    )?;
    let out_scheme_path = data_path.join(format!(
        "{CUSTOM_SCHEMES_DIR_NAME}/{scheme_system}/{scheme_slug}.yaml"
    ));
    let expected_output = format!(
        r"system: {scheme_system}
name: Tinty Generated
slug: {scheme_slug}
author: Tinty
description: Some description
variant: dark
palette:
  base00: '#0f2c19'
  base01: '#304938'
  base02: '#516658'
  base03: '#728378'
  base04: '#93a098'
  base05: '#b4bdb8'
  base06: '#d5dad8'
  base07: '#f7f7f8'
  base08: '#d45520'
  base09: '#f0840a'
  base0A: '#e6a343'
  base0B: '#0d9c33'
  base0C: '#32ebea'
  base0D: '#075cdd'
  base0E: '#6a2b98'
  base0F: '#8e6880'
"
    );

    // ---
    // Act
    // ---
    let (stdout, stderr) = utils::run_command(&command_vec, &data_path, true)?;
    let actual_output = fs::read_to_string(&out_scheme_path)?;

    // ------
    // Assert
    // ------
    ensure!(
        actual_output == expected_output,
        "actual_output does not contain the expected output"
    );
    ensure!(
        stdout == format!("Scheme created: {}\n", out_scheme_path.display()),
        "stdout does not contain the expected output"
    );
    ensure!(
        stderr.is_empty(),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}
