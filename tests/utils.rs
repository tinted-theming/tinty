use anyhow::{anyhow, Result};
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

#[allow(dead_code)]
pub const REPO_NAME: &str = env!("CARGO_PKG_NAME");
#[allow(dead_code)]
pub const ORG_NAME: &str = "tinted-theming";
pub const COMMAND_NAME: &str = env!("CARGO_BIN_EXE_tinty");
#[allow(dead_code)]
pub const CURRENT_SCHEME_FILE_NAME: &str = "current_scheme";
#[allow(dead_code)]
pub const REPO_DIR: &str = "repos";
#[allow(dead_code)]
pub const SCHEMES_REPO_NAME: &str = "schemes";
#[allow(dead_code)]
pub const CUSTOM_SCHEMES_DIR_NAME: &str = "custom-schemes";

pub fn run_command(command_vec: Vec<String>) -> Result<(String, String), Box<dyn Error>> {
    let output = Command::new(&command_vec[0])
        .args(&command_vec[1..])
        .output()
        .expect("Failed to execute command");

    if !output.stderr.is_empty() {
        println!(
            "tests::utils::run_command stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = strip_ansi_escapes::strip(String::from_utf8(output.stdout)?);
    let stderr = strip_ansi_escapes::strip(String::from_utf8(output.stderr)?);

    Ok((String::from_utf8(stdout)?, String::from_utf8(stderr)?))
}

#[allow(dead_code)]
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

pub fn cleanup(config_path: impl AsRef<Path>, data_path: impl AsRef<Path>) -> Result<()> {
    if config_path.as_ref().is_file() {
        fs::remove_file(config_path)?;
    }

    if data_path.as_ref().is_dir() {
        fs::remove_dir_all(data_path)?;
    }

    Ok(())
}

pub fn write_to_file(path: impl AsRef<Path>, contents: &str) -> Result<()> {
    if path.as_ref().exists() {
        fs::remove_file(&path)?;
    }

    if path.as_ref().parent().is_some() && !path.as_ref().parent().unwrap().exists() {
        fs::create_dir_all(path.as_ref().parent().unwrap())?;
    }

    let mut file = File::create(path)?;

    file.write_all(contents.as_bytes())?;

    Ok(())
}

#[allow(clippy::type_complexity)]
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

#[allow(dead_code)]
pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.as_ref().join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}
