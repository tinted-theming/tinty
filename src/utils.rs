#![allow(clippy::arithmetic_side_effects)]
use crate::config::{Config, ConfigItem, ConfigRing, DEFAULT_CONFIG_SHELL};
use crate::constants::REPO_NAME;
use anyhow::{anyhow, Context, Result};
use home::home_dir;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tinted_builder::SchemeSystem;
use tinted_builder_rust::operation_build::utils::SchemeFile;

/// Ensures that a directory exists, creating it if it does not.
pub fn ensure_directory_exists<P: AsRef<Path>>(dir_path: P) -> Result<()> {
    let path = dir_path.as_ref();

    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory at {}", path.display()))?;
    }

    Ok(())
}

pub fn write_to_file(path: impl AsRef<Path>, contents: &str) -> Result<()> {
    let mut file = File::create(path.as_ref())
        .map_err(anyhow::Error::new)
        .with_context(|| format!("Unable to create file: {}", path.as_ref().display()))?;

    file.write_all(contents.as_bytes())?;

    Ok(())
}

pub fn get_shell_command_from_string(config_path: &Path, command: &str) -> Result<Vec<String>> {
    let config = Config::read(config_path)?;
    let shell = config
        .shell
        .unwrap_or_else(|| DEFAULT_CONFIG_SHELL.to_string());
    let full_command = shell.replace("{}", command);

    if shell.contains("{}") {
        shell_words::split(&full_command).map_err(anyhow::Error::new)
    } else {
        // This error is handled earlier so should never get here
        Err(anyhow!(
            "The configured shell property does not contain the required command placeholder '{{}}'"
        ))
    }
}

pub fn create_theme_filename_without_extension(item: &ConfigItem) -> String {
    format!(
        "{}-{}-file",
        item.name.clone(),
        item.themes_dir.clone().replace('/', "-"), // Flatten path/to/dir to path-to-dir
    )
}

pub fn get_all_scheme_names(
    schemes_path: &Path,
    scheme_systems_option: Option<SchemeSystem>,
) -> Result<Vec<String>> {
    let file_paths = get_all_scheme_file_paths(schemes_path, scheme_systems_option)?;
    let mut scheme_vec: Vec<String> = file_paths.into_keys().collect();
    scheme_vec.sort();

    Ok(scheme_vec)
}

pub fn get_all_scheme_file_paths(
    schemes_path: &Path,
    scheme_systems_option: Option<SchemeSystem>,
) -> Result<HashMap<String, SchemeFile>> {
    if !schemes_path.exists() {
        return Err(anyhow!(
            "Schemes do not exist, run install and try again: `{REPO_NAME} install`",
        ));
    }

    let mut scheme_files: HashMap<String, SchemeFile> = HashMap::new();

    // For each supported scheme system, add schemes to vec
    let scheme_systems =
        scheme_systems_option.map_or_else(|| SchemeSystem::variants().to_vec(), |s| vec![s]);
    for scheme_system in scheme_systems {
        let scheme_system_dir = schemes_path.join(scheme_system.as_str());
        if !scheme_system_dir.exists() {
            continue;
        }

        let files = fs::read_dir(&scheme_system_dir)?
            // Discard failed read results
            .filter_map(Result::ok)
            .collect::<Vec<_>>()
            .into_iter()
            .filter_map(|file| {
                // Convert batch of files into a HashMap<String, SchemeFile>, where
                // the key is the scheme's <system>-<slug> e.g. base16-github
                // Map each entry into a (<String, SchemaFile) tuple that
                // we can collect() into this batch's HashMap<String, SchemaFile>
                let name = format!("{scheme_system}-{}", file.path().file_stem()?.to_str()?);
                let scheme_file = SchemeFile::new(file.path().as_path()).ok()?;

                Some((name, scheme_file))
            })
            .collect::<HashMap<String, SchemeFile>>();
        scheme_files.extend(files);
    }
    Ok(scheme_files)
}

pub fn replace_tilde_slash_with_home(path_str: &str) -> Result<PathBuf> {
    let trimmed_path_str = path_str.trim();
    if trimmed_path_str.starts_with("~/") {
        home_dir().map_or_else(|| Err(anyhow!("Unable to determine a home directory for \"{trimmed_path_str}\", please use an absolute path instead")), |home_dir| Ok(PathBuf::from(trimmed_path_str.replacen(
                   "~/",
                   format!("{}/", home_dir.display()).as_str(),
                   1,
               ))))
    } else {
        Ok(PathBuf::from(trimmed_path_str))
    }
}

pub fn next_scheme_in_cycle(current: &String, schemes: &[String]) -> String {
    if schemes
        .iter()
        .position(|scheme| scheme == current)
        .unwrap_or(0)
        < usize::MAX
    {
        let next_index = schemes
            .iter()
            .position(|scheme| scheme == current)
            .map_or(0, |i| i + 1_usize);

        let next_item = schemes.get((next_index) % schemes.len());

        if let Some(next_item) = next_item {
            return next_item.clone();
        }

        current.clone()
    } else {
        schemes.first().cloned().unwrap_or_else(|| current.clone())
    }
}

fn ring_names(rings: &[ConfigRing]) -> String {
    rings
        .iter()
        .map(|ring| ring.name.clone())
        .collect::<Vec<String>>()
        .join(", ")
}

pub fn preferred_schemes_migration_message(config: &Config) -> String {
    let mut migration_schemes = config.preferred_schemes.clone().unwrap_or_default();

    if let Some(default_scheme) = config
        .default_scheme
        .as_ref()
        .filter(|default_scheme| !migration_schemes.contains(default_scheme))
    {
        migration_schemes.insert(0, default_scheme.clone());
    }

    let schemes = migration_schemes
        .iter()
        .map(|scheme| format!("\"{scheme}\""))
        .collect::<Vec<String>>()
        .join(", ");

    format!(
        "`preferred-schemes` is no longer supported by `tinty cycle`.\n\
Remove `preferred-schemes` from your config and add this instead:\n\n\
default-cycle-ring = \"default\"\n\n\
[[rings]]\n\
name = \"default\"\n\
schemes = [{schemes}]"
    )
}

pub fn cycle_scheme_list(config: &Config, requested_ring: Option<&str>) -> Result<Vec<String>> {
    if config.preferred_schemes.is_some() {
        return Err(anyhow!(preferred_schemes_migration_message(config)));
    }

    let ring_name = requested_ring
        .or(config.default_cycle_ring.as_deref())
        .ok_or_else(|| {
            anyhow!(
                "`tinty cycle` requires either `default-cycle-ring` in config.toml or `--ring <name>`"
            )
        })?;

    let rings = config
        .rings
        .as_ref()
        .ok_or_else(|| anyhow!("`tinty cycle` requires at least one configured `[[rings]]`"))?;

    let ring = rings
        .iter()
        .find(|ring| ring.name == ring_name)
        .ok_or_else(|| {
            let available_rings = ring_names(rings);
            if available_rings.is_empty() {
                anyhow!("No ring named \"{ring_name}\" exists")
            } else {
                anyhow!("No ring named \"{ring_name}\" exists. Available rings: {available_rings}")
            }
        })?;

    if ring.schemes.is_empty() {
        return Err(anyhow!(
            "Ring \"{}\" does not contain any schemes and cannot be cycled",
            ring.name
        ));
    }

    Ok(ring.schemes.clone())
}
