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
    let expected_output = format!(
        "Schemes are missing, run install and then try again: `{} install`",
        REPO_NAME
    );

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
        "You don't have any local custom schemes at: data_path_{}/custom-schemes",
        test_name
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
    let expected_output = format!(
        "{}-{}\n{}-{}",
        scheme_system, scheme_name_one, scheme_system, scheme_name_two
    );
    let custom_scheme_path = data_path.join("custom-schemes");

    fs::create_dir_all(custom_scheme_path.join(scheme_system))?;
    fs::write(
        custom_scheme_path.join(format!("{}/{}.yaml", scheme_system, scheme_name_one)),
        "",
    )?;
    fs::write(
        custom_scheme_path.join(format!("{}/{}.yml", scheme_system, scheme_name_two)),
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
    let github_json = fs::read_to_string(Path::new("fixtures/github.json"))?;
    // ---
    // Act
    // ---
    let github: TestSchemeEntry = serde_json::from_str(&github_json).unwrap();

    assert!(
        github.name == "Github",
        "Expected name to be Github, got {}",
        github.name
    );
    assert!(
        github.system == SchemeSystem::Base16,
        "Expected system to be base16, got {}",
        github.system
    );
    assert!(
        github.variant == SchemeVariant::Light,
        "Expected variant to be base16, got {}",
        github.variant
    );
    assert!(
        github.author == "Tinted Theming (https://github.com/tinted-theming)",
        "Expected author to be 'Tinted Theming (https://github.com/tinted-theming)', got {}",
        github.author
    );
    assert!(
        github.slug == "github",
        "Expected slug to be github, got {}",
        github.slug
    );
    assert!(
        github.id == "base16-github",
        "Expected id to be base16-github, got {}",
        github.id
    );
    assert!(
        github.palette.len() == 16,
        "Expected 16 colors in palette, got {}",
        github.palette.len()
    );

    let expected_colors = vec![
        (
            "base00",
            "#eaeef2",
            ("ea".to_string(), "ee".to_string(), "f2".to_string()),
            (0.91764706, 0.93333334, 0.9490196),
            (234, 238, 242),
        ),
        (
            "base01",
            "#d0d7de",
            ("d0".to_string(), "d7".to_string(), "de".to_string()),
            (0.8156863, 0.84313726, 0.87058824),
            (208, 215, 222),
        ),
        (
            "base02",
            "#afb8c1",
            ("af".to_string(), "b8".to_string(), "c1".to_string()),
            (0.6862745, 0.72156864, 0.75686276),
            (175, 184, 193),
        ),
        (
            "base03",
            "#8c959f",
            ("8c".to_string(), "95".to_string(), "9f".to_string()),
            (0.54901963, 0.58431375, 0.62352943),
            (140, 149, 159),
        ),
        (
            "base04",
            "#6e7781",
            ("6e".to_string(), "77".to_string(), "81".to_string()),
            (0.43137255, 0.46666667, 0.5058824),
            (110, 119, 129),
        ),
        (
            "base05",
            "#424a53",
            ("42".to_string(), "4a".to_string(), "53".to_string()),
            (0.25882354, 0.2901961, 0.3254902),
            (66, 74, 83),
        ),
        (
            "base06",
            "#32383f",
            ("32".to_string(), "38".to_string(), "3f".to_string()),
            (0.19607843, 0.21960784, 0.24705882),
            (50, 56, 63),
        ),
        (
            "base07",
            "#1f2328",
            ("1f".to_string(), "23".to_string(), "28".to_string()),
            (0.12156863, 0.13725491, 0.15686275),
            (31, 35, 40),
        ),
        (
            "base08",
            "#fa4549",
            ("fa".to_string(), "45".to_string(), "49".to_string()),
            (0.98039216, 0.27058825, 0.28627452),
            (250, 69, 73),
        ),
        (
            "base09",
            "#e16f24",
            ("e1".to_string(), "6f".to_string(), "24".to_string()),
            (0.88235295, 0.43529412, 0.14117648),
            (225, 111, 36),
        ),
        (
            "base0A",
            "#bf8700",
            ("bf".to_string(), "87".to_string(), "00".to_string()),
            (0.7490196, 0.5294118, 0.0),
            (191, 135, 0),
        ),
        (
            "base0B",
            "#2da44e",
            ("2d".to_string(), "a4".to_string(), "4e".to_string()),
            (0.1764706, 0.6431373, 0.30588236),
            (45, 164, 78),
        ),
        (
            "base0C",
            "#339d9b",
            ("33".to_string(), "9d".to_string(), "9b".to_string()),
            (0.2, 0.6156863, 0.60784316),
            (51, 157, 155),
        ),
        (
            "base0D",
            "#218bff",
            ("21".to_string(), "8b".to_string(), "ff".to_string()),
            (0.12941177, 0.54509807, 1.0),
            (33, 139, 255),
        ),
        (
            "base0E",
            "#a475f9",
            ("a4".to_string(), "75".to_string(), "f9".to_string()),
            (0.6431373, 0.45882353, 0.9764706),
            (164, 117, 249),
        ),
        (
            "base0F",
            "#4d2d00",
            ("4d".to_string(), "2d".to_string(), "00".to_string()),
            (0.3019608, 0.1764706, 0.0),
            (77, 45, 0),
        ),
    ];

    for (color, expected_hex_str, expected_hex, expected_dec, expected_rgb) in expected_colors {
        let palette_color = github
            .palette
            .get(color)
            .context(format!("color {} not found", color))?;
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

    let (foreground, background) = github
        .lightness
        .map(|l| (l.foreground, l.background))
        .unwrap();

    assert!(
        background == 93.90565,
        "Expected lightness.background to be 93.90565, got {}",
        background
    );
    assert!(
        foreground == 31.067837,
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

    let github = entry_map.get("base16-github").unwrap().clone();
    let gruvbox = entry_map
        .get("base16-gruvbox-material-dark-hard")
        .unwrap()
        .clone();

    let github_json = fs::read_to_string(Path::new("fixtures/github.json"))?;
    let gruvbox_json = fs::read_to_string(Path::new("fixtures/gruvbox-material-dark-hard.json"))?;

    let expected_github: TestSchemeEntry = serde_json::from_str(&github_json).unwrap();
    let expected_gruvbox: TestSchemeEntry = serde_json::from_str(&gruvbox_json).unwrap();

    assert!(
        expected_github == github,
        "{:?} does not match {:?}",
        expected_github,
        github
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
