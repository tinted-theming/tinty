use crate::{
    config::{REPO_NAME, REPO_URL},
    utils::read_lines_to_vec,
};
use anyhow::Result;
use std::path::Path;

pub fn get_themes(schemes_list_path: &Path) -> Option<Vec<String>> {
    if !schemes_list_path.exists() {
        return None;
    }

    let schemes_list: Vec<String> = read_lines_to_vec(schemes_list_path)
        .map_err(anyhow::Error::new)
        .unwrap();

    Some(schemes_list)
}

pub fn has_theme(theme_name: &str, app_data_path: &Path) -> Result<bool> {
    let local_repo_path = app_data_path.join(REPO_NAME);
    let schemes_list_path = local_repo_path.join("schemes.txt");
    let schemes_list: Vec<String> = read_lines_to_vec(&schemes_list_path)?;
    let mut theme_exists = false;

    for scheme in &schemes_list {
        if scheme == theme_name {
            theme_exists = true;
        }
    }

    Ok(theme_exists)
}

pub fn setup_hook(app_data_path: &Path) -> Result<(&str, bool)> {
    let local_repo_path = app_data_path.join(REPO_NAME);

    crate::hooks::utils::setup_hook(REPO_NAME, REPO_URL, &local_repo_path)
}

pub fn update_hook(app_data_path: &Path) -> Result<(&str, bool)> {
    let local_repo_path = app_data_path.join(REPO_NAME);

    crate::hooks::utils::update_hook(REPO_NAME, REPO_URL, &local_repo_path)
}
