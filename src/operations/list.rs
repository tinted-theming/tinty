use crate::{
    constants::{REPO_NAME, REPO_URL},
    utils::read_lines_to_vec,
};
use anyhow::Result;
use std::path::Path;

/// Lists available color schemes in the base16-shell-manager repository.
///
/// This function checks the provided base16-shell-manager repository path to determine if it contains
/// color scheme scripts. It validates that the provided path is a directory, collects the names
/// of available color schemes by inspecting the scripts in the directory, and prints them.
pub fn list(schemes_list_path: &Path) -> Result<()> {
    if !schemes_list_path.exists() {
        println!("Unable to retrieve the schemes list. Please run `{} setup` again. If this error persists, please file an issue at {}/issues", REPO_NAME, REPO_URL);
        return Ok(());
    }

    let schemes_list: Vec<String> = read_lines_to_vec(schemes_list_path)
        .map_err(anyhow::Error::new)
        .unwrap();

    for scheme in &schemes_list {
        println!("{}", scheme);
    }

    Ok(())
}
