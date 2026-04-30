#![allow(clippy::suboptimal_flops)]
use crate::{
    constants::{CUSTOM_SCHEMES_DIR_NAME, REPO_DIR, REPO_NAME, SCHEMES_REPO_NAME},
    utils::{get_all_scheme_file_paths, get_all_scheme_names},
};
use anyhow::{anyhow, Context, Result};
use io::Write;
use rayon::{iter::ParallelIterator, slice::ParallelSlice};
use serde::Serialize;
use std::{
    collections::{BTreeMap, HashMap},
    io,
    path::Path,
    sync::{Arc, Mutex},
};
use tinted_builder::tinted8::{SyntaxKey, UiKey};
use tinted_builder::{Color, Scheme, SchemeSystem, SchemeVariant};
use tinted_builder_rust::operation_build::utils::SchemeFile;

/// Lists available color schemes
///
/// Lists colorschemes file which is updated via scripts/install by getting a list of schemes
/// available in <https://github.com/tinted-theming/schemes>
pub fn list(data_path: &Path, is_custom: bool, is_json: bool) -> Result<()> {
    let schemes_dir_path = schemes_dir_path(data_path, is_custom)?;

    let stdout = io::stdout();
    if is_json {
        let json = scheme_entries_json(&schemes_dir_path)?;
        let mut handle = stdout.lock();
        let _ = writeln!(handle, "{json}");
        return Ok(());
    }

    let scheme_vec = get_all_scheme_names(&schemes_dir_path, None)?;
    let mut handle = stdout.lock();
    for scheme in scheme_vec {
        if writeln!(handle, "{scheme}").is_err() {
            break;
        }
    }

    Ok(())
}

pub fn schemes_dir_path(data_path: &Path, is_custom: bool) -> Result<std::path::PathBuf> {
    let schemes_dir_path = if is_custom {
        data_path.join(CUSTOM_SCHEMES_DIR_NAME)
    } else {
        data_path.join(format!("{REPO_DIR}/{SCHEMES_REPO_NAME}"))
    };

    match (schemes_dir_path.exists(), is_custom) {
        (false, true) => Err(anyhow!(
            "You don't have any local custom schemes at: {}",
            schemes_dir_path.display(),
        )),
        (false, false) => Err(anyhow!(
            "Schemes are missing, run install and then try again: `{REPO_NAME} install`",
        )),
        _ => Ok(schemes_dir_path),
    }
}

pub fn scheme_entries_json(schemes_dir_path: &Path) -> Result<String> {
    let scheme_files = get_all_scheme_file_paths(schemes_dir_path, None)?;
    let entries = scheme_entries(scheme_files)?;

    Ok(serde_json::to_string(&entries)?)
}

#[derive(Clone, Serialize)]
pub struct SchemeEntry {
    id: String,
    name: String,
    author: String,
    system: SchemeSystem,
    variant: SchemeVariant,
    slug: String,
    palette: BTreeMap<String, ColorOut>,
    lightness: Option<Lightness>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ui: Option<BTreeMap<String, ColorOut>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    syntax: Option<BTreeMap<String, ColorOut>>,
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
        let (palette, ui, syntax) = match scheme.clone() {
            Scheme::Base16(s) => (
                s.palette
                    .into_iter()
                    .map(|(k, v)| (k, ColorOut::from_color(&v)))
                    .collect(),
                None,
                None,
            ),
            Scheme::Base24(s) => (
                s.palette
                    .into_iter()
                    .map(|(k, v)| (k, ColorOut::from_color(&v)))
                    .collect(),
                None,
                None,
            ),
            Scheme::Tinted8(s) => (
                tinted8_palette(&s),
                Some(tinted8_ui(&s)),
                Some(tinted8_syntax(&s)),
            ),
            _ => (BTreeMap::new(), None, None),
        };
        Self {
            id: format!("{system}-{slug}"),
            name: scheme.get_scheme_name(),
            system,
            slug,
            author: scheme.get_scheme_author(),
            variant: scheme.get_scheme_variant(),
            lightness: Lightness::from_color(scheme).ok(),
            palette,
            ui,
            syntax,
        }
    }

    pub(crate) fn to_envs(&self) -> Vec<(String, String)> {
        let basic_info = [
            ("TINTY_SCHEME_ID".to_string(), self.id.clone()),
            ("TINTY_SCHEME_NAME".to_string(), self.name.clone()),
            ("TINTY_SCHEME_SLUG".to_string(), self.slug.clone()),
            ("TINTY_SCHEME_SYSTEM".to_string(), self.system.to_string()),
            ("TINTY_SCHEME_VARIANT".to_string(), self.variant.to_string()),
        ]
        .to_vec();

        let lightness_info = self
            .lightness
            .clone()
            .map(|l| {
                [
                    (
                        "TINTY_SCHEME_LIGHTNESS_FOREGROUND".to_string(),
                        l.foreground.to_string(),
                    ),
                    (
                        "TINTY_SCHEME_LIGHTNESS_BACKGROUND".to_string(),
                        l.background.to_string(),
                    ),
                ]
                .to_vec()
            })
            .unwrap_or_default();

        let color_info: Vec<(String, String)> = self
            .palette
            .clone()
            .iter()
            .flat_map(|(color, color_out)| {
                let c = color.to_uppercase();
                [
                    (
                        format!("TINTY_SCHEME_PALETTE_{}_HEX_R", c.to_uppercase()),
                        color_out.hex.0.clone(),
                    ),
                    (
                        format!("TINTY_SCHEME_PALETTE_{}_HEX_G", c.to_uppercase()),
                        color_out.hex.1.clone(),
                    ),
                    (
                        format!("TINTY_SCHEME_PALETTE_{}_HEX_B", c.to_uppercase()),
                        color_out.hex.2.clone(),
                    ),
                    (
                        format!("TINTY_SCHEME_PALETTE_{}_RGB_R", c.to_uppercase()),
                        color_out.rgb.0.to_string(),
                    ),
                    (
                        format!("TINTY_SCHEME_PALETTE_{}_RGB_G", c.to_uppercase()),
                        color_out.rgb.1.to_string(),
                    ),
                    (
                        format!("TINTY_SCHEME_PALETTE_{}_RGB_B", c.to_uppercase()),
                        color_out.rgb.2.to_string(),
                    ),
                    (
                        format!("TINTY_SCHEME_PALETTE_{}_DEC_R", c.to_uppercase()),
                        color_out.dec.0.to_string(),
                    ),
                    (
                        format!("TINTY_SCHEME_PALETTE_{}_DEC_G", c.to_uppercase()),
                        color_out.dec.1.to_string(),
                    ),
                    (
                        format!("TINTY_SCHEME_PALETTE_{}_DEC_B", c.to_uppercase()),
                        color_out.dec.2.to_string(),
                    ),
                ]
            })
            .collect();

        let mut envs: Vec<(String, String)> = Vec::new();
        envs.extend(basic_info);
        envs.extend(lightness_info);
        envs.extend(color_info);
        envs
    }
}

impl ColorOut {
    pub fn from_color(color: &Color) -> Self {
        Self {
            hex_str: format!("#{}{}{}", color.hex.0, color.hex.1, color.hex.2),
            hex: color.hex.clone(),
            rgb: color.rgb,
            dec: color.dec,
        }
    }
}

fn tinted8_palette(scheme: &tinted_builder::tinted8::Scheme) -> BTreeMap<String, ColorOut> {
    let mut map = BTreeMap::new();
    for (color_name, color_variant) in tinted_builder::tinted8::Palette::get_color_list() {
        if let Some(color) = scheme.palette.get_color(&color_name, &color_variant) {
            let key = format!("{color_name}-{color_variant}");
            map.insert(key, ColorOut::from_color(color));
        }
    }
    map
}

fn tinted8_syntax(scheme: &tinted_builder::tinted8::Scheme) -> BTreeMap<String, ColorOut> {
    SyntaxKey::variants()
        .iter()
        .map(|key| {
            (
                key.to_string(),
                ColorOut::from_color(scheme.syntax.get_color(key)),
            )
        })
        .collect()
}

fn tinted8_ui(scheme: &tinted_builder::tinted8::Scheme) -> BTreeMap<String, ColorOut> {
    UiKey::variants()
        .iter()
        .map(|key| {
            (
                key.to_string(),
                ColorOut::from_color(scheme.ui.get_color(key)),
            )
        })
        .collect()
}

impl Lightness {
    pub fn from_color(scheme: &Scheme) -> Result<Self> {
        let (fg, bg) = match scheme.clone() {
            Scheme::Base16(s) => (
                s.palette.get("base05").context("no fg color")?.clone(),
                s.palette.get("base00").context("no bg color")?.clone(),
            ),
            Scheme::Base24(s) => (
                s.palette.get("base05").context("no fg color")?.clone(),
                s.palette.get("base00").context("no bg color")?.clone(),
            ),
            Scheme::Tinted8(s) => (
                s.ui.global.foreground.normal.clone(),
                s.ui.global.background.normal.clone(),
            ),
            _ => return Err(anyhow!("no supported palette found")),
        };

        let fg_luminance = Self::luminance(&fg);
        let bg_luminance = Self::luminance(&bg);
        let foreground = Self::luminance_to_lstar(fg_luminance);
        let background = Self::luminance_to_lstar(bg_luminance);

        Ok(Self {
            foreground,
            background,
        })
    }

    fn gamma_corrected_to_linear(channel: f32) -> f32 {
        if channel <= 0.04045 {
            return channel / 12.92;
        }
        let base = (channel + 0.055) / 1.055;
        base.powf(2.4)
    }

    fn luminance_to_lstar(luminance: f32) -> f32 {
        if luminance <= (216.0 / 24389.0) {
            return luminance * (24389.0 / 27.0);
        }

        luminance.cbrt().mul_add(116.0, -16.0)
    }

    fn luminance(color: &Color) -> f32 {
        let r = Self::gamma_corrected_to_linear(color.dec.0);
        let g = Self::gamma_corrected_to_linear(color.dec.1);
        let b = Self::gamma_corrected_to_linear(color.dec.2);
        (r * 0.2126) + (g * 0.7152) + (b * 0.0722)
    }
}

fn scheme_entries(scheme_files: HashMap<String, SchemeFile>) -> Result<Vec<SchemeEntry>> {
    let mut keys: Vec<String> = scheme_files.keys().cloned().collect();
    // Create a thread-safe HashMap to collect results
    let mutex = Arc::new(Mutex::new(HashMap::new()));
    let mut sorted_results: Vec<SchemeEntry> = Vec::new();
    scheme_files
        .into_iter()
        .collect::<Vec<_>>()
        // We could be parsing hundreds of files. Parallelize with 10 files each arm.
        .par_chunks(10)
        .map(|chunk| {
            chunk
                .iter()
                .filter_map(|(k, sf)| {
                    sf.get_scheme()
                        .ok()
                        .map(|scheme| (k.clone(), SchemeEntry::from_scheme(&scheme)))
                })
                .collect::<HashMap<String, SchemeEntry>>()
        })
        .for_each(|map| {
            // Each batch will produce a HashMap<String, SchemaFile>
            // Merge them into the final HashMap.
            if let Ok(mut accum) = mutex.lock() {
                accum.extend(map);
            }
        });

    keys.sort();
    let Ok(results) = mutex.lock() else {
        return Err(anyhow!("Unable to unlock mutex"));
    };

    for k in keys {
        if let Some(v) = results.get(&k) {
            sorted_results.push(v.clone());
        }
    }

    Ok(sorted_results)
}
