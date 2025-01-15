use crate::{
    constants::{CUSTOM_SCHEMES_DIR_NAME, REPO_DIR, REPO_NAME, SCHEMES_REPO_NAME},
    utils::{get_all_scheme_file_paths, get_all_scheme_names},
};
use anyhow::{anyhow, Result};
use std::{collections::HashMap, fs, path::{Path, PathBuf}, sync::{Arc,Mutex}};
use serde_json::Value;
use serde::Serialize;
use rayon::prelude::*;

/// Lists available color schemes
///
/// Lists colorschemes file which is updated via scripts/install by getting a list of schemes
/// available in https://github.com/tinted-theming/schemes
pub fn list(data_path: &Path, is_custom: bool, is_json: bool) -> Result<()> {
    let schemes_dir_path = if is_custom {
        data_path.join(CUSTOM_SCHEMES_DIR_NAME)
    } else {
        data_path.join(format!("{}/{}", REPO_DIR, SCHEMES_REPO_NAME))
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
                "Schemes are missing, run install and then try again: `{} install`",
                REPO_NAME
            ))
        }
        _ => {}
    }

    if is_json {
        print_as_json(get_all_scheme_file_paths(&schemes_dir_path, None)?);
        return Ok(());
    }

    let scheme_vec = get_all_scheme_names(&schemes_dir_path, None)?;
    for scheme in scheme_vec {
        println!("{}", scheme);
    }

    Ok(())
}

#[derive(Clone, Serialize)]
struct SchemeEntry {
    key: String,
    scheme_data: Value,
}

fn print_as_json(file_paths: HashMap<String,PathBuf>) -> () {
    let mut keys: Vec<String> = file_paths.keys().cloned().collect();
    // Create a thread-safe HashMap to collect results
    let locked_results: Arc<Mutex<HashMap<String, SchemeEntry>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut sorted_results: Vec<SchemeEntry> = Vec::new();
    keys.par_chunks(10)
        .for_each(|chunk| {
            for key in chunk {
                if let Some(path) = file_paths.get(key) {
                    let mut results_lock = locked_results.lock().unwrap();
                    let entry = SchemeEntry {
                        key: key.clone(),
                        scheme_data: read_yaml_into_json(path.to_path_buf()).expect(""),
                    };
                    results_lock.insert(key.clone(), entry);
                }
            }
        });
    keys.sort();
    let results = locked_results.lock().unwrap();
    for k in keys {
        if let Some(v) = results.get(&k) {
            sorted_results.push(v.clone());
        }
    }
    let json_string = serde_json::to_string_pretty(&*sorted_results).unwrap();

    println!("{}", json_string);
    return ()
}

fn read_yaml_into_json(file_path: PathBuf) -> Result<Value> {
    let yaml_content = fs::read_to_string(file_path)?; // Read the YAML file
    let json_value: Value = serde_yaml::from_str(&yaml_content)?; // Parse YAML to JSON
    Ok(json_value)
}
