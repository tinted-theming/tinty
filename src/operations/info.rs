use crate::constants::{CUSTOM_SCHEMES_DIR_NAME, REPO_DIR, REPO_NAME, REPO_URL, SCHEMES_REPO_NAME};
use crate::operations::current::get_current_scheme_slug;
use anyhow::{anyhow, Result};
use hex_color::HexColor;
use serde::Deserialize;
use std::str::FromStr;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tinted_builder::SchemeSystem;

#[derive(Debug, Deserialize)]
struct Base16Scheme {
    name: String,
    author: String,
    palette: Base16Palette,
}

#[derive(Debug, Deserialize)]
struct Base16Palette {
    base00: String,
    base01: String,
    base02: String,
    base03: String,
    base04: String,
    base05: String,
    base06: String,
    base07: String,
    base08: String,
    base09: String,
    #[serde(rename = "base0A")]
    base0_a: String,
    #[serde(rename = "base0B")]
    base0_b: String,
    #[serde(rename = "base0C")]
    base0_c: String,
    #[serde(rename = "base0D")]
    base0_d: String,
    #[serde(rename = "base0E")]
    base0_e: String,
    #[serde(rename = "base0F")]
    base0_f: String,
}

#[derive(Debug, Deserialize)]
struct Base24Scheme {
    name: String,
    author: String,
    palette: Base24Palette,
}

#[derive(Debug, Deserialize)]
struct Base24Palette {
    base00: String,
    base01: String,
    base02: String,
    base03: String,
    base04: String,
    base05: String,
    base06: String,
    base07: String,
    base08: String,
    base09: String,
    #[serde(rename = "base0A")]
    base0_a: String,
    #[serde(rename = "base0B")]
    base0_b: String,
    #[serde(rename = "base0C")]
    base0_c: String,
    #[serde(rename = "base0D")]
    base0_d: String,
    #[serde(rename = "base0E")]
    base0_e: String,
    #[serde(rename = "base0F")]
    base0_f: String,
    base10: String,
    base11: String,
    base12: String,
    base13: String,
    base14: String,
    base15: String,
    base16: String,
    base17: String,
}

#[allow(clippy::too_many_lines)]
fn print_scheme(scheme_path: &Path) -> Result<()> {
    let dir_name = scheme_path
        .parent()
        .and_then(|p| p.file_name())
        .map_or_else(String::new, |f| f.to_string_lossy().into_owned()); // Ensures ownership
    let system = dir_name.as_str();
    let mut palette: Vec<(String, String, String)> = vec![];
    let str = fs::read_to_string(scheme_path)?;
    let slug = scheme_path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let author;
    let name;

    // ANSI values based on base16 0.4.2 and base24 0.1.3
    if let Ok(scheme_system) = SchemeSystem::from_str(system) {
        match scheme_system {
            SchemeSystem::Base16 => {
                let scheme: Base16Scheme = serde_yaml::from_str(&str)?;
                author = scheme.author;
                name = scheme.name;
                palette.push(("base00".to_string(), scheme.palette.base00, "0".to_string()));
                palette.push((
                    "base01".to_string(),
                    scheme.palette.base01,
                    "18".to_string(),
                ));
                palette.push((
                    "base02".to_string(),
                    scheme.palette.base02,
                    "19".to_string(),
                ));
                palette.push(("base03".to_string(), scheme.palette.base03, "8".to_string()));
                palette.push((
                    "base04".to_string(),
                    scheme.palette.base04,
                    "20".to_string(),
                ));
                palette.push(("base05".to_string(), scheme.palette.base05, "7".to_string()));
                palette.push((
                    "base06".to_string(),
                    scheme.palette.base06,
                    "21".to_string(),
                ));
                palette.push((
                    "base07".to_string(),
                    scheme.palette.base07,
                    "15".to_string(),
                ));
                palette.push((
                    "base08".to_string(),
                    scheme.palette.base08,
                    "1 and 9".to_string(),
                ));
                palette.push((
                    "base09".to_string(),
                    scheme.palette.base09,
                    "16".to_string(),
                ));
                palette.push((
                    "base0A".to_string(),
                    scheme.palette.base0_a,
                    "3 and 11".to_string(),
                ));
                palette.push((
                    "base0B".to_string(),
                    scheme.palette.base0_b,
                    "2 and 10".to_string(),
                ));
                palette.push((
                    "base0C".to_string(),
                    scheme.palette.base0_c,
                    "6 and 14".to_string(),
                ));
                palette.push((
                    "base0D".to_string(),
                    scheme.palette.base0_d,
                    "4 and 12".to_string(),
                ));
                palette.push((
                    "base0E".to_string(),
                    scheme.palette.base0_e,
                    "5 and 13".to_string(),
                ));
                palette.push((
                    "base0F".to_string(),
                    scheme.palette.base0_f,
                    "17".to_string(),
                ));
            }
            SchemeSystem::Base24 => {
                let scheme: Base24Scheme = serde_yaml::from_str(&str)?;
                author = scheme.author;
                name = scheme.name;
                palette.push(("base00".to_string(), scheme.palette.base00, "0".to_string()));
                palette.push((
                    "base01".to_string(),
                    scheme.palette.base01,
                    "18".to_string(),
                ));
                palette.push((
                    "base02".to_string(),
                    scheme.palette.base02,
                    "19".to_string(),
                ));
                palette.push(("base03".to_string(), scheme.palette.base03, "8".to_string()));
                palette.push((
                    "base04".to_string(),
                    scheme.palette.base04,
                    "20".to_string(),
                ));
                palette.push(("base05".to_string(), scheme.palette.base05, "7".to_string()));
                palette.push((
                    "base06".to_string(),
                    scheme.palette.base06,
                    "21".to_string(),
                ));
                palette.push((
                    "base07".to_string(),
                    scheme.palette.base07,
                    "15".to_string(),
                ));
                palette.push(("base08".to_string(), scheme.palette.base08, "1".to_string()));
                palette.push((
                    "base09".to_string(),
                    scheme.palette.base09,
                    "16".to_string(),
                ));
                palette.push((
                    "base0A".to_string(),
                    scheme.palette.base0_a,
                    "3".to_string(),
                ));
                palette.push((
                    "base0B".to_string(),
                    scheme.palette.base0_b,
                    "2".to_string(),
                ));
                palette.push((
                    "base0C".to_string(),
                    scheme.palette.base0_c,
                    "6".to_string(),
                ));
                palette.push((
                    "base0D".to_string(),
                    scheme.palette.base0_d,
                    "4".to_string(),
                ));
                palette.push((
                    "base0E".to_string(),
                    scheme.palette.base0_e,
                    "5".to_string(),
                ));
                palette.push((
                    "base0F".to_string(),
                    scheme.palette.base0_f,
                    "17".to_string(),
                ));
                palette.push(("base10".to_string(), scheme.palette.base10, "-".to_string()));
                palette.push(("base11".to_string(), scheme.palette.base11, "-".to_string()));
                palette.push(("base12".to_string(), scheme.palette.base12, "9".to_string()));
                palette.push((
                    "base13".to_string(),
                    scheme.palette.base13,
                    "11".to_string(),
                ));
                palette.push((
                    "base14".to_string(),
                    scheme.palette.base14,
                    "10".to_string(),
                ));
                palette.push((
                    "base15".to_string(),
                    scheme.palette.base15,
                    "14".to_string(),
                ));
                palette.push((
                    "base16".to_string(),
                    scheme.palette.base16,
                    "12".to_string(),
                ));
                palette.push((
                    "base17".to_string(),
                    scheme.palette.base17,
                    "13".to_string(),
                ));
            }
            _ => {
                return Err(anyhow!(
                    "Scheme system is not supported \"{}\": {}",
                    system,
                    scheme_path.display()
                ))
            }
        }
    } else {
        return Err(anyhow!(
            "Scheme system is not supported \"{}\": {}",
            system,
            scheme_path.display()
        ));
    }

    let ansi_col_header_spacing: (String, String) = SchemeSystem::from_str(system).map_or_else(
        |_| (String::new(), String::new()),
        |scheme_system| match scheme_system {
            SchemeSystem::Base16 => ("    ".to_string(), "----".to_string()),
            _ => (String::new(), String::new()),
        },
    );
    println!("System: {system}",);
    println!("Slug: {slug}",);
    println!("Name: {name}",);
    println!("Author: {author}");
    println!("Scheme path: {}", scheme_path.to_string_lossy(),);
    println!(
        "| Color       | Name   | Hex     | ANSI {}|",
        ansi_col_header_spacing.0
    );
    println!(
        "|-------------|--------|---------|------{}|",
        ansi_col_header_spacing.1
    );

    let reset = "\x1B[0m";
    for (name, hex, ansi) in palette {
        let hex_text = format!("#{}", hex.strip_prefix('#').unwrap_or(&hex));
        let hex = HexColor::parse(&hex_text)?;
        let bg_ansi = format!("\x1B[48;2;{};{};{}m", hex.r, hex.g, hex.b);
        let fg_ansi = format!("\x1B[38;2;{};{};{}m", hex.r, hex.g, hex.b);
        let ansi_col_whitespace: String = SchemeSystem::from_str(system).map_or_else(
            |_| String::new(),
            |scheme_system| {
                match scheme_system {
                    SchemeSystem::Base16 => (ansi.len()..8).map(|_| " ").collect(), // 4 is length of "ANSI"
                    SchemeSystem::Base24 => (ansi.len()..4).map(|_| " ").collect(), // 4 is length of "ANSI"
                    _ => String::new(),
                }
            },
        );

        println!("| {bg_ansi}{fg_ansi}           {reset} | {name} | {hex_text} | {ansi} {ansi_col_whitespace}|");
    }

    println!();

    Ok(())
}

fn print_single_schemes(files: &[PathBuf], scheme_name: &str) -> Result<()> {
    let scheme_system_name = scheme_name.split('-').next().unwrap_or_default();

    if !SchemeSystem::variants()
        .iter()
        .map(SchemeSystem::as_str)
        .any(|x| x == scheme_system_name)
    {
        return Err(anyhow!(
            r#"Invalid scheme system: "{}" from scheme name "{}"
Make sure to add the system prefix to the theme name. Eg: {}-oceanicnext
Run `{} list` to get a list of scheme names"#,
            scheme_system_name,
            scheme_name,
            SchemeSystem::default(),
            REPO_NAME
        ));
    }

    let scheme_name_without_system = scheme_name
        .split('-')
        .skip(1)
        .collect::<Vec<&str>>()
        .join("-");

    match files.iter().find(|path|
        path.parent().is_some_and(|parent_path|
        path.file_stem().unwrap_or_default().to_string_lossy() == scheme_name_without_system
            && parent_path.file_name().unwrap_or_default() == scheme_system_name)
    ) {
        Some(scheme_path) => {
            print_scheme(scheme_path)?;
        }
        None => return Err(anyhow!("Scheme file does not exist. Perhaps schemes are outdated, try running `{REPO_NAME} update`\nIf the problem persist please create an issue at {REPO_URL}/issues")),
    }

    Ok(())
}

fn print_all_schemes(files: Vec<PathBuf>) -> Result<()> {
    for file_path in files {
        let extension = file_path.extension().unwrap_or_default();
        let filename = file_path.file_name().unwrap_or_default();

        if filename.is_empty() || extension != "yaml" {
            continue;
        }

        print_scheme(&file_path)?;
    }

    Ok(())
}

pub fn info(
    data_path: &Path,
    scheme_name_option: Option<&String>,
    is_custom: bool,
    exhaustive_list: bool,
) -> Result<()> {
    let schemes_dir_path = if is_custom {
        data_path.join(CUSTOM_SCHEMES_DIR_NAME)
    } else {
        data_path.join(format!("{REPO_DIR}/{SCHEMES_REPO_NAME}"))
    };

    match (schemes_dir_path.exists(), is_custom) {
        (false, true) => {
            return Err(anyhow!(
                "You don't have any local custom schemes at: {}",
                schemes_dir_path.display(),
            ))
        }
        (false, false) => {
            return Err(anyhow!(
                "Scheme repo path does not exist: {}\nRun `{} install` and try again",
                schemes_dir_path.display(),
                REPO_NAME
            ))
        }
        _ => {}
    }

    let files_entries = fs::read_dir(schemes_dir_path.join(SchemeSystem::default().as_str()))?;
    let mut files: Vec<PathBuf> = files_entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    let scheme_systems_without_default: Vec<&str> = SchemeSystem::variants()
        .iter()
        .filter(|s| s.as_str() != SchemeSystem::default().as_str())
        .map(SchemeSystem::as_str)
        .collect();

    // Add other scheme_system schemes to vec
    for scheme_system in scheme_systems_without_default {
        if schemes_dir_path.join(scheme_system).exists() {
            files.extend(
                fs::read_dir(schemes_dir_path.join(scheme_system))?
                    .filter_map(|entry| entry.ok().map(|e| e.path())),
            );
        }
    }

    files.sort();

    if scheme_name_option.is_some() || !exhaustive_list {
        let scheme_name = scheme_name_option
            .cloned()
            .unwrap_or_else(|| get_current_scheme_slug(data_path));
        print_single_schemes(&files, &scheme_name)?;
    } else {
        print_all_schemes(files)?;
    }

    Ok(())
}
