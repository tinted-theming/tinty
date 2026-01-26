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
  base00: "#f7f7f7"
  base01: "#d5d9d6"
  base02: "#b3bbb6"
  base03: "#919d95"
  base04: "#707f75"
  base05: "#4e6154"
  base06: "#2c4334"
  base07: "#0b2614"
  base08: "#dd5218"
  base09: "#e29c0c"
  base0A: "#e29c0c"
  base0B: "#069f31"
  base0C: "#00fffe"
  base0D: "#045de1"
  base0E: "#8e6682"
  base0F: "#98421d"
  base10: "#aa664a"
  base11: "#ab8942"
  base12: "#ab8942"
  base13: "#2d7742"
  base14: "#41bdbd"
  base15: "#3c67a8"
  base16: "#83707d"
  base17: "#784e3c"
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
  base00: "#0e2d19"
  base01: "#2f4938"
  base02: "#506658"
  base03: "#718378"
  base04: "#93a097"
  base05: "#b4bdb7"
  base06: "#d5dad7"
  base07: "#f7f7f7"
  base08: "#dd5218"
  base09: "#e29c0c"
  base0A: "#e29c0c"
  base0B: "#069f31"
  base0C: "#00fffe"
  base0D: "#045de1"
  base0E: "#8e6682"
  base0F: "#98421d"
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
  base00: '#0e2d19'
  base01: '#2f4938'
  base02: '#506658'
  base03: '#718378'
  base04: '#93a097'
  base05: '#b4bdb7'
  base06: '#d5dad7'
  base07: '#f7f7f7'
  base08: '#dd5218'
  base09: '#e29c0c'
  base0A: '#e29c0c'
  base0B: '#069f31'
  base0C: '#00fffe'
  base0D: '#045de1'
  base0E: '#8e6682'
  base0F: '#98421d'
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
