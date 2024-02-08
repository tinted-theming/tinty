extern crate strip_ansi_escapes;

use anyhow::{anyhow, Result};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

pub const REPO_NAME: &str = "tinty";
pub const COMMAND_NAME: &str = "./target/release/tinty";

pub fn run_command(command_vec: Vec<String>) -> Result<(String, String), Box<dyn Error>> {
    let output = Command::new(&command_vec[0])
        .args(&command_vec[1..])
        .output()
        .expect("Failed to execute command");

    if !output.stderr.is_empty() {
        println!(
            "Init command stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = strip_ansi_escapes::strip(String::from_utf8(output.stdout)?);
    let stderr = strip_ansi_escapes::strip(String::from_utf8(output.stderr)?);

    Ok((String::from_utf8(stdout)?, String::from_utf8(stderr)?))
}

pub fn run_setup_command(config_path: &Path) -> Result<()> {
    let output_setup = Command::new(COMMAND_NAME)
        .args([
            "setup",
            format!("--config={}", config_path.display()).as_str(),
        ])
        .status()
        .expect("Failed to execute setup command");

    if output_setup.success() {
        Ok(())
    } else {
        anyhow::bail!("Setup command stderr: {}", output_setup);
    }
}

pub fn get_data_path() -> Result<PathBuf> {
    let system_data_path =
        dirs::data_dir().ok_or_else(|| anyhow!("Error getting data directory"))?;
    let data_path = system_data_path.join(format!("tinted-theming/{}", REPO_NAME));

    return Ok(data_path);
}

pub fn cleanup(config_path: &Path) -> Result<()> {
    let data_path = get_data_path()?;

    if data_path.is_dir() {
        fs::remove_dir_all(&data_path)?;
    }

    if config_path.is_dir() {
        fs::remove_dir_all(config_path)?;
    }

    Ok(())
}
