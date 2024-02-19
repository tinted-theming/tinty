use anyhow::{anyhow, Result};
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

pub const REPO_NAME: &str = env!("CARGO_PKG_NAME");
pub const COMMAND_NAME: &str = "./target/release/tinty";
pub const CURRENT_SCHEME_FILE_NAME: &str = "current_scheme";

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

pub fn run_install_command(config_path: &Path, data_path: &Path) -> Result<()> {
    let output_install = Command::new(COMMAND_NAME)
        .args([
            "install",
            format!("--config={}", config_path.display()).as_str(),
            format!("--data-dir={}", data_path.display()).as_str(),
        ])
        .status()
        .expect("Failed to execute install command");

    if output_install.success() {
        Ok(())
    } else {
        Err(anyhow!("Install command stderr: {}", output_install))
    }
}

pub fn cleanup(config_path: &Path, data_path: &Path) -> Result<()> {
    if config_path.is_file() {
        fs::remove_file(config_path)?;
    }

    if data_path.is_dir() {
        fs::remove_dir_all(&data_path)?;
    }

    Ok(())
}

pub fn write_to_file(path: &Path, contents: &str) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }

    if path.parent().is_some() && !path.parent().unwrap().exists() {
        fs::create_dir_all(path.parent().unwrap())?;
    }

    let mut file = File::create(path)?;

    file.write_all(contents.as_bytes())?;

    Ok(())
}

pub fn read_file_to_string(path: &Path) -> Result<String> {
    if !path.exists() {
        return Err(anyhow!("File does not exist: {}", path.display()));
    }

    let mut file = File::open(path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn setup(
    name: &str,
    command: &str,
) -> Result<(
    PathBuf,
    PathBuf,
    Vec<String>,
    Box<dyn FnOnce() -> Result<()>>,
)> {
    let config_path = PathBuf::from(format!("config_path_{}.toml", name).as_str());
    let data_path = PathBuf::from(format!("data_path_{}", name).as_str());
    let command = format!(
        "{} --config=\"{}\" --data-dir=\"{}\" {}",
        COMMAND_NAME,
        config_path.display(),
        data_path.display(),
        command
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;

    cleanup(&config_path, &data_path)?;
    write_to_file(&config_path, "")?;

    let config_path_clone = config_path.clone();
    let data_path_clone = data_path.clone();

    Ok((
        config_path,
        data_path,
        command_vec,
        Box::new(move || cleanup(&config_path_clone, &data_path_clone)),
    ))
}
