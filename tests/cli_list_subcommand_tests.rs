mod common;

extern crate dirs;

use crate::common::{cleanup, COMMAND_NAME};
use anyhow::Result;
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::Path;

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

#[test]
fn test_cli_list_subcommand() -> Result<()> {
    // -------
    // Arrange
    // -------
    let config_path = Path::new("test_cli_list_subcommand");
    let expected_output = fs::read_to_string(Path::new("public/schemes.txt"))?;
    let command = format!(
        "{} list --config=\"{}\"",
        COMMAND_NAME,
        config_path.display()
    );
    let command_vec = shell_words::split(command.as_str()).map_err(anyhow::Error::new)?;

    // // ---
    // // Act
    // // ---
    let (stdout, _) = common::run_command(command_vec).unwrap();

    // // ------
    // // Assert
    // // ------
    assert!(
        stdout.contains(&expected_output),
        "stdout does not contain the expected output"
    );

    cleanup(config_path)?;
    Ok(())
}
