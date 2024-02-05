extern crate shell_words;

use crate::config::{Config, DEFAULT_CONFIG_SHELL};
use anyhow::{anyhow, Context, Result};
use std::fs::{self, File};
use std::io::{self, BufRead, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;

/// Ensures that a directory exists, creating it if it does not.
pub fn ensure_directory_exists<P: AsRef<Path>>(dir_path: P) -> Result<()> {
    let path = dir_path.as_ref();

    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory at {:?}", path))?;
    }

    Ok(())
}

/// Reads the contents of a file and returns it as a string.
pub fn read_file_to_string(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn write_to_file(path: &Path, contents: &str) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }

    let mut file = File::create(path)?;
    file.write_all(contents.as_bytes())?;

    Ok(())
}

pub fn read_lines_to_vec(file_path: &Path) -> io::Result<Vec<String>> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let mut lines = Vec::new();
    for line in reader.lines() {
        let line = line?;
        lines.push(line);
    }

    Ok(lines)
}

pub fn get_shell_command_from_string(config_path: &Path, command: &str) -> Result<Vec<String>> {
    let config = Config::read(config_path)?;
    let shell = config
        .shell
        .unwrap_or_else(|| DEFAULT_CONFIG_SHELL.to_string());
    let full_command = shell.replace("{}", command);

    shell_words::split(&full_command).map_err(anyhow::Error::new)
}

pub fn git_clone(repo_url: &str, target_dir: &Path) -> Result<()> {
    if target_dir.exists() {
        return Err(anyhow!(
            "Error cloning {}. Target directory '{}' already exists",
            repo_url,
            target_dir.display()
        ));
    }

    let command = format!("git clone {} {}", repo_url, target_dir.display());
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;

    Command::new(&command_vec[0])
        .args(&command_vec[1..])
        .stdout(Stdio::null())
        .status()
        .with_context(|| format!("Failed to clone repository from {}", repo_url))?;

    Ok(())
}

pub fn git_pull(repo_path: &Path) -> Result<()> {
    if !repo_path.is_dir() {
        return Err(anyhow!(
            "Error with git pull. {} is not a directory",
            repo_path.display()
        ));
    }

    let command = "git pull";
    let command_vec = shell_words::split(command).map_err(anyhow::Error::new)?;

    let status = Command::new(&command_vec[0])
        .args(&command_vec[1..])
        .current_dir(repo_path)
        .stdout(Stdio::null())
        .status()
        .with_context(|| format!("Failed to execute process in {}", repo_path.display()))?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("Error wth git pull in {}", repo_path.display()))
    }
}

pub fn git_diff(target_dir: &Path) -> Result<bool> {
    let command = "git status --porcelain";
    let command_vec = shell_words::split(command).map_err(anyhow::Error::new)?;
    let output = Command::new(&command_vec[0])
        .args(&command_vec[1..])
        .current_dir(target_dir)
        .output()
        .with_context(|| format!("Failed to execute process in {}", target_dir.display()))?;
    let stdout = str::from_utf8(&output.stdout).expect("Not valid UTF-8");

    if stdout.is_empty() {
        Ok(false)
    } else {
        Ok(true)
    }
}
