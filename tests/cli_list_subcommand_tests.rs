mod utils;

use crate::utils::REPO_NAME;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
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
    let (_, stderr) = utils::run_command(command_vec, &data_path, false).unwrap();

    // ------
    // Assert
    // ------
    assert!(
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
    let (_, stderr) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------
    assert!(
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
    let (stdout, _) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------
    // The sort order of the schemes differ slightly so do an assert on each line instead of the
    // whole file
    let lines: Vec<&str> = expected_output.lines().collect();
    for line in lines {
        assert!(
            stdout.contains(line),
            "stdout does not contain the expected output"
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

    let mut command_vec = command_vec.clone();
    command_vec.push("--custom-schemes".to_string());

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(command_vec, &data_path, true).unwrap();

    // ------
    // Assert
    // ------
    // The sort order of the schemes differ slightly so do an assert on each line instead of the
    // whole file
    let lines: Vec<&str> = expected_output.lines().collect();
    for line in lines {
        assert!(
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
    pub palette: HashMap<String, TestColorOut>,
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
fn test_cli_list_subcommand_deserialize_fixture_scheme_entry() -> Result<()> {
    // -------
    // Arrange
    // -------
    let scheme_json = fs::read_to_string(Path::new("fixtures/base16-dracula.json"))?;

    // ---
    // Act
    // ---
    let scheme_entry: TestSchemeEntry = serde_json::from_str(&scheme_json).unwrap();

    assert!(
        scheme_entry.name == "Dracula",
        "Expected name to be Dracula, got {}",
        scheme_entry.name
    );
    assert!(
        scheme_entry.system == SchemeSystem::Base16,
        "Expected system to be base16, got {}",
        scheme_entry.system
    );
    assert!(
        scheme_entry.variant == SchemeVariant::Dark,
        "Expected variant to be base16, got {}",
        scheme_entry.variant
    );
    assert!(
        scheme_entry.author == "Jamy Golden (http://github.com/JamyGolden), based on Dracula Theme (http://github.com/dracula)",
        "Expected author to be 'Tinted Theming (https://github.com/tinted-theming)', got {}",
        scheme_entry.author
    );
    assert!(
        scheme_entry.slug == "dracula",
        "Expected slug to be dracula, got {}",
        scheme_entry.slug
    );
    assert!(
        scheme_entry.id == "base16-dracula",
        "Expected id to be base16-dracula, got {}",
        scheme_entry.id
    );
    assert!(
        scheme_entry.palette.len() == 16,
        "Expected 16 colors in palette, got {}",
        scheme_entry.palette.len()
    );

    let expected_colors = vec![
        (
            "base00",
            "#282a36",
            ("28".to_string(), "2a".to_string(), "36".to_string()),
            (0.15686275, 0.16470589, 0.21176471),
            (40, 42, 54),
        ),
        (
            "base01",
            "#363447",
            ("36".to_string(), "34".to_string(), "47".to_string()),
            (0.21176471, 0.20392157, 0.2784314),
            (54, 52, 71),
        ),
        (
            "base02",
            "#44475a",
            ("44".to_string(), "47".to_string(), "5a".to_string()),
            (0.26666668, 0.2784314, 0.3529412),
            (68, 71, 90),
        ),
        (
            "base03",
            "#6272a4",
            ("62".to_string(), "72".to_string(), "a4".to_string()),
            (0.38431373, 0.44705883, 0.6431373),
            (98, 114, 164),
        ),
        (
            "base04",
            "#9ea8c7",
            ("9e".to_string(), "a8".to_string(), "c7".to_string()),
            (0.61960787, 0.65882355, 0.78039217),
            (158, 168, 199),
        ),
        (
            "base05",
            "#f8f8f2",
            ("f8".to_string(), "f8".to_string(), "f2".to_string()),
            (0.972549, 0.972549, 0.9490196),
            (248, 248, 242),
        ),
        (
            "base06",
            "#f0f1f4",
            ("f0".to_string(), "f1".to_string(), "f4".to_string()),
            (0.9411765, 0.94509804, 0.95686275),
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
            (1.0, 0.33333334, 0.33333334),
            (255, 85, 85),
        ),
        (
            "base09",
            "#ffb86c",
            ("ff".to_string(), "b8".to_string(), "6c".to_string()),
            (1.0, 0.72156864, 0.42352942),
            (255, 184, 108),
        ),
        (
            "base0A",
            "#f1fa8c",
            ("f1".to_string(), "fa".to_string(), "8c".to_string()),
            (0.94509804, 0.98039216, 0.54901963),
            (241, 250, 140),
        ),
        (
            "base0B",
            "#50fa7b",
            ("50".to_string(), "fa".to_string(), "7b".to_string()),
            (0.3137255, 0.98039216, 0.48235294),
            (80, 250, 123),
        ),
        (
            "base0C",
            "#8be9fd",
            ("8b".to_string(), "e9".to_string(), "fd".to_string()),
            (0.54509807, 0.9137255, 0.99215686),
            (139, 233, 253),
        ),
        (
            "base0D",
            "#80bfff",
            ("80".to_string(), "bf".to_string(), "ff".to_string()),
            (0.5019608, 0.7490196, 1.0),
            (128, 191, 255),
        ),
        (
            "base0E",
            "#ff79c6",
            ("ff".to_string(), "79".to_string(), "c6".to_string()),
            (1.0, 0.4745098, 0.7764706),
            (255, 121, 198),
        ),
        (
            "base0F",
            "#bd93f9",
            ("bd".to_string(), "93".to_string(), "f9".to_string()),
            (0.7411765, 0.5764706, 0.9764706),
            (189, 147, 249),
        ),
    ];

    for (color, expected_hex_str, expected_hex, expected_dec, expected_rgb) in expected_colors {
        let palette_color = scheme_entry
            .palette
            .get(color)
            .context(format!("color {color} not found"))?;
        assert!(
            palette_color.hex_str == expected_hex_str,
            "Exoected {}.hex_str to equal {}, got {}",
            color,
            expected_hex_str,
            palette_color.hex_str
        );
        assert!(
            palette_color.hex == expected_hex,
            "Exoected {}.hex to equal {}, got {}",
            color,
            expected_hex_str,
            palette_color.hex_str
        );
        assert!(
            palette_color.dec == expected_dec,
            "Exoected {}.dec to equal ({}, {}, {}), got ({}, {}, {})",
            color,
            expected_dec.0,
            expected_dec.1,
            expected_dec.2,
            palette_color.dec.0,
            palette_color.dec.1,
            palette_color.dec.2,
        );
        assert!(
            palette_color.rgb == expected_rgb,
            "Exoected {}.rgb to equal ({}, {}, {}), got ({}, {}, {})",
            color,
            expected_rgb.0,
            expected_rgb.1,
            expected_rgb.2,
            palette_color.rgb.0,
            palette_color.rgb.1,
            palette_color.rgb.2,
        );
    }

    let (foreground, background) = scheme_entry
        .lightness
        .map(|l| (l.foreground, l.background))
        .unwrap();

    assert!(
        background == 17.336052,
        "Expected lightness.background to be 93.90565, got {}",
        background
    );
    assert!(
        foreground == 97.431,
        "Expected lightness.foreground to be 31.067837, got {}",
        foreground
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
    let (stdout, _) = utils::run_command(command_vec, &data_path, true).unwrap();
    let results: Vec<TestSchemeEntry> = serde_json::from_str(&stdout).unwrap();

    assert!(
        results.len() >= SCHEME_COUNT,
        "expected JSON to contain {} entries, found {}",
        SCHEME_COUNT,
        results.len()
    );

    let entry_map: HashMap<String, TestSchemeEntry> =
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

    assert!(
        expected_dracula == dracula,
        "{:?} does not match {:?}",
        expected_dracula,
        dracula
    );
    assert!(
        expected_gruvbox == gruvbox,
        "{:?} does not match {:?}",
        expected_gruvbox,
        gruvbox
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
        custom_scheme_path.join(format!("{}/{}.yaml", scheme_system, scheme_name_one)),
        "",
    )?;

    fs::copy(
        Path::new("fixtures/tinty-city-dark.yaml"),
        custom_scheme_path.join(format!("{}/{}.yaml", scheme_system, scheme_name_two)),
    )
    .context("failed to copy scheme from fixtures")?;

    let mut command_vec = command_vec.clone();
    command_vec.push("--custom-schemes".to_string());

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(command_vec, &data_path, true).unwrap();

    let expected_json = fs::read_to_string(Path::new("fixtures/tinty-city-dark.json"))?;
    let expected_entry: TestSchemeEntry = serde_json::from_str(&expected_json).unwrap();

    // ------
    // Assert
    // ------
    let results: Vec<TestSchemeEntry> = serde_json::from_str(&stdout).unwrap();
    // There are two YAMLs in the directory of custom schemes but only one of them
    // contains a YAML body
    assert!(
        results.len() == 1,
        "expected JSON to contain 1 entry, found {}",
        results.len()
    );
    assert!(
        expected_entry == results[0],
        "{:?} does not match {:?}",
        expected_entry,
        results[0],
    );

    cleanup()?;
    Ok(())
}
