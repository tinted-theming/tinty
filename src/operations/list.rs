use crate::{
    constants::{CUSTOM_SCHEMES_DIR_NAME, REPO_DIR, REPO_NAME, SCHEMES_REPO_NAME},
    utils::{get_all_scheme_file_paths, get_all_scheme_names},
};
use anyhow::{anyhow, Context, Result};
use io::Write;
use rayon::prelude::*;
use serde::Serialize;
use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tinted_builder::{Color, Scheme, SchemeSystem, SchemeVariant};
use tinted_builder_rust::operation_build::utils::SchemeFile;

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

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    if is_json {
        let scheme_files = get_all_scheme_file_paths(&schemes_dir_path, None)?;
        let json = as_json(scheme_files)?;
        if let Err(_) = writeln!(handle, "{}", json) {}
        return Ok(());
    }

    let scheme_vec = get_all_scheme_names(&schemes_dir_path, None)?;
    for scheme in scheme_vec {
        if let Err(_) = writeln!(handle, "{}", scheme) {
            break;
        }
    }

    Ok(())
}

#[derive(Clone, Serialize)]
struct SchemeEntry {
    id: String,
    name: String,
    author: String,
    system: SchemeSystem,
    variant: SchemeVariant,
    slug: String,
    palette: HashMap<String, ColorOut>,
    luminance: Option<Lightness>,
}

#[derive(Clone, Serialize)]
struct ColorOut {
    hex_str: String,
    pub hex: (String, String, String),
    pub rgb: (u8, u8, u8),
    pub dec: (f32, f32, f32),
}

#[derive(Clone, Serialize)]
struct Lightness {
    foreground: f32,
    background: f32,
}

impl SchemeEntry {
    pub fn from_scheme(scheme: &Scheme) -> Self {
        let slug = scheme.get_scheme_slug();
        let system = scheme.get_scheme_system();
        return Self {
            id: format!("{}-{}", system, slug),
            name: scheme.get_scheme_name(),
            system,
            slug,
            author: scheme.get_scheme_author(),
            variant: scheme.get_scheme_variant(),
            luminance: Lightness::from_color(scheme).ok(),
            palette: match scheme.clone() {
                Scheme::Base16(s) | Scheme::Base24(s) => s
                    .palette
                    .into_iter()
                    .map(|(k, v)| (k, ColorOut::from_color(&v)))
                    .collect(),
                _ => HashMap::new(),
            },
        };
    }
}

impl ColorOut {
    pub fn from_color(color: &Color) -> Self {
        return Self {
            hex_str: format!("#{}{}{}", color.hex.0, color.hex.1, color.hex.2),
            hex: color.hex.clone(),
            rgb: color.rgb,
            dec: color.dec,
        };
    }
}

impl Lightness {
    pub fn from_color(scheme: &Scheme) -> Result<Self> {
        let (fg, bg) = match scheme.clone() {
            Scheme::Base16(s) | Scheme::Base24(s) => (
                s.palette.get("base00").context("no fg color")?.clone(),
                s.palette.get("base05").context("no bg color")?.clone(),
            ),
            _ => return Err(anyhow!("no supported palette found")),
        };

        let foreground = Self::lightness(&fg);
        let background = Self::lightness(&bg);

        Ok(Self {
            foreground,
            background,
        })
    }

    fn lightness(color: &Color) -> f32 {
        let r = Self::rgb_to_linear(color.dec.0);
        let g = Self::rgb_to_linear(color.dec.1);
        let b = Self::rgb_to_linear(color.dec.2);
        let luminance = (r * 0.2126) + (g * 0.7152) + (b * 0.0722);
        return Self::luminance_to_lstar(luminance);
    }

    fn rgb_to_linear(channel: f32) -> f32 {
        if channel <= 0.04045 {
            return channel / 12.92;
        }
        let base: f32 = 2.4;
        return base.powf((channel + 0.555) / 1.055);
    }

    fn luminance_to_lstar(luminance: f32) -> f32 {
        if luminance <= (216 as f32 / 24389 as f32) {
            return luminance * (24389 as f32 / 27 as f32);
        }

        let base: f32 = 1 as f32 / 3 as f32;
        return base.powf(luminance) * 116 as f32 - 16 as f32;
    }
}

fn as_json(scheme_files: HashMap<String, (PathBuf, SchemeFile)>) -> Result<String> {
    let mut keys: Vec<String> = scheme_files.keys().cloned().collect();
    // Create a thread-safe HashMap to collect results
    let locked_results: Arc<Mutex<HashMap<String, SchemeEntry>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let mut sorted_results: Vec<SchemeEntry> = Vec::new();
    // We could be parsing hundreds of files. Parallelize with 10 files each arm.
    keys.par_chunks(10).try_for_each(|chunk| -> Result<()> {
        for key in chunk {
            if let Some((_, scheme_file)) = scheme_files.get(key) {
                let scheme = scheme_file.get_scheme()?;
                let mut results_lock = locked_results.lock().unwrap();
                results_lock.insert(key.clone(), SchemeEntry::from_scheme(&scheme));
            }
        }
        Ok(())
    })?;
    keys.sort();
    let results = locked_results.lock().unwrap();
    for k in keys {
        if let Some(v) = results.get(&k) {
            sorted_results.push(v.clone());
        }
    }

    return Ok(serde_json::to_string_pretty(&*sorted_results)?);
}
