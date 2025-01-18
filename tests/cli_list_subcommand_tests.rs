mod utils;

use crate::utils::REPO_NAME;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tinted_builder::{SchemeSystem, SchemeVariant};
use utils::setup;

const SCHEME_COUNT: usize = 287;

#[test]
fn test_cli_list_subcommand_without_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (_, _, command_vec, cleanup) = setup("test_cli_list_subcommand_without_setup", "list")?;
    let expected_output = format!(
        "Schemes are missing, run install and then try again: `{} install`",
        REPO_NAME
    );

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(command_vec).unwrap();

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
    let (_, _, command_vec, cleanup) = setup(test_name, "list --custom-schemes")?;
    let expected_output = format!(
        "You don't have any local custom schemes at: data_path_{}/custom-schemes",
        test_name
    );

    // ---
    // Act
    // ---
    let (_, stderr) = utils::run_command(command_vec).unwrap();

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
    let (config_path, data_path, command_vec, cleanup) =
        setup("test_cli_list_subcommand_with_setup", "list")?;
    let expected_output = fs::read_to_string(Path::new("fixtures/schemes.txt"))?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (stdout, _) = utils::run_command(command_vec).unwrap();

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
        custom_scheme_path.join(format!("{}/{}.yaml", scheme_system, scheme_name_two)),
        "",
    )?;

    let mut command_vec = command_vec.clone();
    command_vec.push("--custom-schemes".to_string());

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(command_vec).unwrap();

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

#[test]
fn test_cli_list_subcommand_as_json_with_setup() -> Result<()> {
    // -------
    // Arrange
    // -------
    let (config_path, data_path, command_vec, cleanup) =
        setup("test_cli_list_subcommand_as_json_with_setup", "list --json")?;

    // ---
    // Act
    // ---
    utils::run_install_command(&config_path, &data_path)?;
    let (stdout, _) = utils::run_command(command_vec).unwrap();

    let results: Vec<TestSchemeEntry> = serde_json::from_str(&stdout).unwrap();

    assert!(
        results.len() == SCHEME_COUNT,
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
    let (stdout, _) = utils::run_command(command_vec).unwrap();

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
