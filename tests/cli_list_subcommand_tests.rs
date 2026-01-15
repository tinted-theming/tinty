#![allow(clippy::float_cmp)]
mod utils;

use crate::utils::REPO_NAME;
use anyhow::{ensure, Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tinted_builder::{SchemeSystem, SchemeVariant};
use utils::setup;

const SCHEME_COUNT: usize = 459;

#[test]
fn test_cli_list_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_list_subcommand_without_setup", "list")?;
    let expected_output =
        format!("Schemes are missing, run install and then try again: `{REPO_NAME} install`",);

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(&command_vec, &data_path, false)?;

    // ------
    // Assert
    // ------
    ensure!(
        stderr.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_list_subcommand_without_setup_with_custom_schemes_flag() -> Result<()> {
    // -------
    // Arrange
    // -------
    let test_name = "test_cli_list_subcommand_without_setup_with_custom_schemes_flag";
    let (_, data_path, command_vec, cleanup) = setup(test_name, "list --custom-schemes")?;
    let expected_output = format!(
        "You don't have any local custom schemes at: data_path_{test_name}/custom-schemes",
    );

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    ensure!(
        stderr.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_list_subcommand_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_list_subcommand_with_setup", "list")?;
    let expected_output = fs::read_to_string(Path::new("fixtures/schemes.txt"))?;

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    // The sort order of the schemes differ slightly so do an assert on each line instead of the
    // whole file
    let lines: Vec<&str> = expected_output.lines().collect();
    for line in lines {
        ensure!(
            stdout.contains(line),
            format!("stdout does not contain the expected output: {line}")
        );
    }

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_list_subcommand_with_custom() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_list_subcommand_with_custom", "list")?;
    let scheme_system = "base16";
    let scheme_name_one = "tinted-theming";
    let scheme_name_two = "tinty";
    let expected_output =
        format!("{scheme_system}-{scheme_name_one}\n{scheme_system}-{scheme_name_two}",);
    let custom_scheme_path = data_path.join("custom-schemes");

    fs::create_dir_all(custom_scheme_path.join(scheme_system))?;
    fs::write(
        custom_scheme_path.join(format!("{scheme_system}/{scheme_name_one}.yaml")),
        "",
    )?;
    fs::write(
        custom_scheme_path.join(format!("{scheme_system}/{scheme_name_two}.yml")),
        "",
    )?;

    let mut command_vec = command_vec;
    command_vec.push("--custom-schemes".to_string());

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(&command_vec, &data_path, true)?;

    // ------
    // Assert
    // ------
    // The sort order of the schemes differ slightly so do an assert on each line instead of the
    // whole file
    let lines: Vec<&str> = expected_output.lines().collect();
    for line in lines {
        ensure!(
            stdout.contains(line),
            "stdout does not contain the expected output"
        );
    }

    cleanup()?;
    Ok(())
}

// These structs may appear as duplicate of the serializable structs in operations/list.rs, but
// this is intentional. These are asserting the expected structure of the JSON output, and
// sharing the exact same struct would hide a certain class of issues (i.e. comparing a broken
// implementation w/ an equally-broken assertion)
#[derive(Clone, Deserialize, PartialEq)]
struct TestSchemeEntry {
    pub id: String,
    pub name: String,
    pub author: String,
    pub system: SchemeSystem,
    pub variant: SchemeVariant,
    pub slug: String,
    pub palette: BTreeMap<String, TestColorOut>,
    pub lightness: Option<TestLightness>,
}

impl std::ops::Deref for TestSchemeEntry {
    type Target = SchemeSystem;

    fn deref(&self) -> &Self::Target {
        &self.system
    }
}

impl std::fmt::Debug for TestSchemeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestSchemeEntry")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("author", &self.author)
            .field("system", &self.system)
            .field("variant", &self.variant)
            .field("slug", &self.slug)
            .field("palette", &self.palette)
            .field("lightness", &self.lightness)
            .finish()
    }
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
struct TestColorOut {
    pub hex_str: String,
    pub hex: (String, String, String),
    pub rgb: (u8, u8, u8),
    pub dec: (f32, f32, f32),
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
struct TestLightness {
    pub foreground: f32,
    pub background: f32,
}

// This is the basis of the other as_json tests. We'll assert that TestSchemeEntry are
// deserializing fields as expected.
#[test]
#[allow(clippy::too_many_lines)]
fn test_cli_list_subcommand_deserialize_fixture_scheme_entry() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_json = fs::read_to_string(Path::new("fixtures/base16-dracula.json"))?;

    // ---
    // Act
    // ---
    let scheme_entry: TestSchemeEntry = serde_json::from_str(&scheme_json)?;

    ensure!(
        scheme_entry.name == "Dracula",
        format!("Expected name to be Dracula, got {}", scheme_entry.name)
    );
    ensure!(
        scheme_entry.system == SchemeSystem::Base16,
        format!("Expected system to be base16, got {}", scheme_entry.system)
    );
    ensure!(
        scheme_entry.variant == SchemeVariant::Dark,
        format!(
            "Expected variant to be base16, got {}",
            scheme_entry.variant
        )
    );
    ensure!(
        scheme_entry.author == "Jamy Golden (http://github.com/JamyGolden), based on Dracula Theme (http://github.com/dracula)",
        format!("Expected author to be 'Tinted Theming (https://github.com/tinted-theming)', got {}",
        scheme_entry.author)
    );
    ensure!(
        scheme_entry.slug == "dracula",
        format!("Expected slug to be dracula, got {}", scheme_entry.slug)
    );
    ensure!(
        scheme_entry.id == "base16-dracula",
        format!("Expected id to be base16-dracula, got {}", scheme_entry.id)
    );
    ensure!(
        scheme_entry.palette.len() == 16,
        format!(
            "Expected 16 colors in palette, got {}",
            scheme_entry.palette.len()
        )
    );

    let expected_colors = vec![
        (
            "base00",
            "#282a36",
            ("28".to_string(), "2a".to_string(), "36".to_string()),
            (0.156_862_75, 0.164_705_89, 0.211_764_71),
            (40, 42, 54),
        ),
        (
            "base01",
            "#363447",
            ("36".to_string(), "34".to_string(), "47".to_string()),
            (0.211_764_71, 0.203_921_57, 0.278_431_4),
            (54, 52, 71),
        ),
        (
            "base02",
            "#44475a",
            ("44".to_string(), "47".to_string(), "5a".to_string()),
            (0.266_666_68, 0.278_431_4, 0.352_941_2),
            (68, 71, 90),
        ),
        (
            "base03",
            "#6272a4",
            ("62".to_string(), "72".to_string(), "a4".to_string()),
            (0.384_313_73, 0.447_058_83, 0.643_137_3),
            (98, 114, 164),
        ),
        (
            "base04",
            "#9ea8c7",
            ("9e".to_string(), "a8".to_string(), "c7".to_string()),
            (0.619_607_87, 0.658_823_55, 0.780_392_17),
            (158, 168, 199),
        ),
        (
            "base05",
            "#f8f8f2",
            ("f8".to_string(), "f8".to_string(), "f2".to_string()),
            (0.972_549_, 0.972_549_, 0.949_019_6),
            (248, 248, 242),
        ),
        (
            "base06",
            "#f0f1f4",
            ("f0".to_string(), "f1".to_string(), "f4".to_string()),
            (0.941_176_5, 0.945_098_04, 0.956_862_75),
            (240, 241, 244),
        ),
        (
            "base07",
            "#ffffff",
            ("ff".to_string(), "ff".to_string(), "ff".to_string()),
            (1.0, 1.0, 1.0),
            (255, 255, 255),
        ),
        (
            "base08",
            "#ff5555",
            ("ff".to_string(), "55".to_string(), "55".to_string()),
            (1.0, 0.333_333_34, 0.333_333_34),
            (255, 85, 85),
        ),
        (
            "base09",
            "#ffb86c",
            ("ff".to_string(), "b8".to_string(), "6c".to_string()),
            (1.0, 0.721_568_64, 0.423_529_42),
            (255, 184, 108),
        ),
        (
            "base0A",
            "#f1fa8c",
            ("f1".to_string(), "fa".to_string(), "8c".to_string()),
            (0.945_098_04, 0.980_392_16, 0.549_019_63),
            (241, 250, 140),
        ),
        (
            "base0B",
            "#50fa7b",
            ("50".to_string(), "fa".to_string(), "7b".to_string()),
            (0.313_725_5, 0.980_392_16, 0.482_352_94),
            (80, 250, 123),
        ),
        (
            "base0C",
            "#8be9fd",
            ("8b".to_string(), "e9".to_string(), "fd".to_string()),
            (0.545_098_07, 0.913_725_5, 0.992_156_86),
            (139, 233, 253),
        ),
        (
            "base0D",
            "#80bfff",
            ("80".to_string(), "bf".to_string(), "ff".to_string()),
            (0.501_960_8, 0.749_019_6, 1.0),
            (128, 191, 255),
        ),
        (
            "base0E",
            "#ff79c6",
            ("ff".to_string(), "79".to_string(), "c6".to_string()),
            (1.0, 0.474_509_8, 0.776_470_6),
            (255, 121, 198),
        ),
        (
            "base0F",
            "#bd93f9",
            ("bd".to_string(), "93".to_string(), "f9".to_string()),
            (0.741_176_5, 0.576_470_6, 0.976_470_6),
            (189, 147, 249),
        ),
    ];

    for (color, expected_hex_str, expected_hex, expected_dec, expected_rgb) in expected_colors {
        let palette_color = scheme_entry
            .palette
            .get(color)
            .context(format!("color {color} not found"))?;
        ensure!(
            palette_color.hex_str == expected_hex_str,
            format!(
                "Expected {color}.hex_str to equal {expected_hex_str}, got {}",
                palette_color.hex_str
            )
        );
        ensure!(
            palette_color.hex == expected_hex,
            format!(
                "Expected {color}.hex to equal {expected_hex_str}, got {}",
                palette_color.hex_str
            )
        );
        ensure!(
            palette_color.dec == expected_dec,
            format!(
                "Expected {color}.dec to equal ({}, {}, {}), got ({}, {}, {})",
                expected_dec.0,
                expected_dec.1,
                expected_dec.2,
                palette_color.dec.0,
                palette_color.dec.1,
                palette_color.dec.2,
            )
        );
        ensure!(
            palette_color.rgb == expected_rgb,
            format!(
                "Expected {color}.rgb to equal ({}, {}, {}), got ({}, {}, {})",
                expected_rgb.0,
                expected_rgb.1,
                expected_rgb.2,
                palette_color.rgb.0,
                palette_color.rgb.1,
                palette_color.rgb.2,
            )
        );
    }

    let (foreground, background) = scheme_entry
        .lightness
        .map(|l| (l.foreground, l.background))
        .unwrap();

    ensure!(
        background == 17.336_05,
        format!("Expected lightness.background to be 17.336_05, got {background}")
    );
    ensure!(
        foreground == 97.431,
        format!("Expected lightness.foreground to be 97.431, got {background}")
    );

    Ok(())
}

#[test]
fn test_cli_list_subcommand_as_json_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) =
        setup("test_cli_list_subcommand_as_json_with_setup", "list --json")?;

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(&command_vec, &data_path, true)?;
    let results: Vec<TestSchemeEntry> = serde_json::from_str(&stdout).unwrap();

    ensure!(
        results.len() >= SCHEME_COUNT,
        format!(
            "expected JSON to contain {SCHEME_COUNT} entries, found {}",
            results.len()
        )
    );

    let entry_map: BTreeMap<String, TestSchemeEntry> =
        results.into_iter().map(|e| (e.id.clone(), e)).collect();

    let dracula = entry_map.get("base16-dracula").unwrap().clone();
    let gruvbox = entry_map
        .get("base16-gruvbox-material-dark-hard")
        .unwrap()
        .clone();

    let dracula_json = fs::read_to_string(Path::new("fixtures/base16-dracula.json"))?;
    let gruvbox_json = fs::read_to_string(Path::new("fixtures/gruvbox-material-dark-hard.json"))?;

    let expected_dracula: TestSchemeEntry = serde_json::from_str(&dracula_json).unwrap();
    let expected_gruvbox: TestSchemeEntry = serde_json::from_str(&gruvbox_json).unwrap();

    ensure!(
        expected_dracula == dracula,
        format!("{:?}\ndoes not match:\n{:?}", expected_dracula, dracula)
    );
    ensure!(
        expected_gruvbox == gruvbox,
        format!("{:?}\ndoes not match:\n{:?}", expected_gruvbox, gruvbox)
    );

    cleanup()?;
    Ok(())
}

#[test]
fn test_cli_list_subcommand_as_json_with_custom() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) = setup(
        "test_cli_list_subcommand_as_json_with_custom",
        "list --json",
    )?;
    let scheme_system = "base16";
    let scheme_name_one = "tinted-theming";
    let scheme_name_two = "tinty";
    let custom_scheme_path = data_path.join("custom-schemes");

    fs::create_dir_all(custom_scheme_path.join(scheme_system))?;
    fs::write(
        custom_scheme_path.join(format!("{scheme_system}/{scheme_name_one}.yaml")),
        "",
    )?;

    fs::copy(
        Path::new("fixtures/tinty-city-dark.yaml"),
        custom_scheme_path.join(format!("{scheme_system}/{scheme_name_two}.yaml")),
    )
    .context("failed to copy scheme from fixtures")?;

    let mut command_vec = command_vec;
    command_vec.push("--custom-schemes".to_string());

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(&command_vec, &data_path, true)?;

    let expected_json = fs::read_to_string(Path::new("fixtures/tinty-city-dark.json"))?;
    let expected_entry: TestSchemeEntry = serde_json::from_str(&expected_json).unwrap();

    // ------
    // Assert
    // ------
    let results: Vec<TestSchemeEntry> = serde_json::from_str(&stdout).unwrap();

    // There are two YAMLs in the directory of custom schemes but only one of them
    // contains a YAML body
    ensure!(
        results.len() == 1,
        format!("expected JSON to contain 1 entry, found {}", results.len()),
    );
    ensure!(
        expected_entry == results[0],
        format!("{:?}\ndoes not match:\n{:?}", expected_entry, results[0])
    );

    cleanup()?;
    Ok(())
}
