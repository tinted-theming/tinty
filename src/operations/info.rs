use crate::{
    config::SupportedSchemeSystems,
    constants::{REPO_DIR, REPO_NAME, REPO_URL, SCHEMES_REPO_NAME},
    utils::read_file_to_string,
};
use anyhow::{anyhow, Result};
use hex_color::HexColor;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

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

fn print_scheme(scheme_path: &Path) -> Result<()> {
    let dir_name = scheme_path
        .parent()
        .and_then(|p| p.file_name())
        .map_or_else(|| "".to_string(), |f| f.to_string_lossy().into_owned()); // Ensures ownership
    let system = dir_name.as_str();
    let mut palette: Vec<String> = vec![];
    let str = read_file_to_string(scheme_path)?;
    let slug = scheme_path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let author;
    let name;

    match system {
        "base16" => {
            let scheme: Base16Scheme = serde_yaml::from_str(&str)?;
            author = scheme.author;
            name = scheme.name;
            palette.push(scheme.palette.base00);
            palette.push(scheme.palette.base01);
            palette.push(scheme.palette.base02);
            palette.push(scheme.palette.base03);
            palette.push(scheme.palette.base04);
            palette.push(scheme.palette.base05);
            palette.push(scheme.palette.base06);
            palette.push(scheme.palette.base07);
            palette.push(scheme.palette.base08);
            palette.push(scheme.palette.base09);
            palette.push(scheme.palette.base0_a);
            palette.push(scheme.palette.base0_b);
            palette.push(scheme.palette.base0_c);
            palette.push(scheme.palette.base0_d);
            palette.push(scheme.palette.base0_e);
            palette.push(scheme.palette.base0_f);
        }
        "base24" => {
            let scheme: Base24Scheme = serde_yaml::from_str(&str)?;
            author = scheme.author;
            name = scheme.name;
            palette.push(scheme.palette.base00);
            palette.push(scheme.palette.base01);
            palette.push(scheme.palette.base02);
            palette.push(scheme.palette.base03);
            palette.push(scheme.palette.base04);
            palette.push(scheme.palette.base05);
            palette.push(scheme.palette.base06);
            palette.push(scheme.palette.base07);
            palette.push(scheme.palette.base08);
            palette.push(scheme.palette.base09);
            palette.push(scheme.palette.base0_a);
            palette.push(scheme.palette.base0_b);
            palette.push(scheme.palette.base0_c);
            palette.push(scheme.palette.base0_d);
            palette.push(scheme.palette.base0_e);
            palette.push(scheme.palette.base0_f);
            palette.push(scheme.palette.base10);
            palette.push(scheme.palette.base11);
            palette.push(scheme.palette.base12);
            palette.push(scheme.palette.base13);
            palette.push(scheme.palette.base14);
            palette.push(scheme.palette.base15);
            palette.push(scheme.palette.base16);
            palette.push(scheme.palette.base17);
        }
        _ => {
            return Err(anyhow!(
                "Scheme system is not supported \"{}\": {}",
                system,
                scheme_path.display()
            ))
        }
    };

    println!(
        "{} ({}-{}) @ {}",
        name,
        system,
        slug,
        scheme_path.to_string_lossy()
    );
    println!("by {}", author);

    let reset = "\x1B[0m";
    for color in palette {
        let hex_text = format!("#{}", color);
        let hex = HexColor::parse(&hex_text)?;
        let bg_ansi = format!("\x1B[48;2;{};{};{}m", hex.r, hex.g, hex.b);
        let fg_ansi = format!("\x1B[38;2;{};{};{}m", hex.r, hex.g, hex.b);

        print!("{} {} {}", bg_ansi, hex_text, reset);
        print!("  {}{}{}", fg_ansi, hex_text, reset);
        println!();
    }

    Ok(())
}

fn print_single_schemes(files: Vec<PathBuf>, scheme_name: &str) -> Result<()> {
    let scheme_system_name = scheme_name.split('-').next().unwrap_or_default();

    if !SupportedSchemeSystems::variants()
        .iter()
        .map(|s| s.to_str())
        .collect::<Vec<&str>>()
        .contains(&scheme_system_name)
    {
        return Err(anyhow!(
            r##"Invalid scheme system: "{}" from scheme name "{}"
Make sure to add the system prefix to the theme name. Eg: {}-oceanicnext
Run `{} list` to get a list of scheme names"##,
            scheme_system_name,
            scheme_name,
            SupportedSchemeSystems::default(),
            REPO_NAME
        ));
    }

    let scheme_name_without_system = scheme_name
        .split('-')
        .skip(1)
        .collect::<Vec<&str>>()
        .join("-");

    match files.iter().find(|path| {
        path.file_stem().unwrap_or_default().to_string_lossy() == scheme_name_without_system
            && path.parent().unwrap().file_name().unwrap_or_default() == scheme_system_name
    }) {
        Some(scheme_path) => {
            print_scheme(scheme_path)?;
        }
        None => return Err(anyhow!("Scheme file does not exist. Perhaps schemes are outdated, try running `{} update`\nIf the problem persist please create an issue at {}/issues", REPO_NAME, REPO_URL)),
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

pub fn info(data_path: &Path, scheme_name_option: Option<&String>) -> Result<()> {
    let schemes_repo_path = data_path.join(format!("{}/{}", REPO_DIR, SCHEMES_REPO_NAME));

    if !schemes_repo_path
        .join(SupportedSchemeSystems::default().to_string())
        .exists()
    {
        return Err(anyhow!(
            "Scheme repo path does not exist: {}\nRun `{} setup` and try again",
            schemes_repo_path.display(),
            REPO_NAME
        ));
    }

    let files_entries =
        fs::read_dir(schemes_repo_path.join(SupportedSchemeSystems::default().to_str()))?;
    let mut files: Vec<PathBuf> = files_entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    let scheme_systems_without_default: Vec<&str> = SupportedSchemeSystems::variants()
        .iter()
        .filter(|s| s.to_str() != SupportedSchemeSystems::default().to_str())
        .map(|s| s.to_str())
        .collect();

    // Add other scheme_system schemes to vec
    for scheme_system in scheme_systems_without_default {
        if schemes_repo_path.join(scheme_system).exists() {
            files.extend(
                fs::read_dir(schemes_repo_path.join(scheme_system))?
                    .filter_map(|entry| entry.ok().map(|e| e.path())),
            );
        }
    }

    files.sort();

    match scheme_name_option {
        Some(scheme_name) => {
            print_single_schemes(files, scheme_name)?;
        }
        None => {
            print_all_schemes(files)?;
        }
    }

    Ok(())
}
