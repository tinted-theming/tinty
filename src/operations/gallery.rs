use crate::{
    constants::ARTIFACTS_DIR,
    operations::list::{scheme_entries_json, schemes_dir_path},
    utils::{ensure_directory_exists, write_to_file},
};
use anyhow::{Context, Result};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

const GALLERY_DIR_NAME: &str = "gallery";
const INDEX_HTML: &str = include_str!("gallery/index.html");
const GALLERY_CSS: &str = include_str!("gallery/gallery.css");
const GALLERY_JS: &str = include_str!("gallery/gallery.js");
const LOGO_BYTES: &[u8] = include_bytes!("../../assets/tinted-theming-logo.png");
const FAVICON_BYTES: &[u8] = include_bytes!("../../assets/favicon.png");

pub fn gallery(
    data_path: &Path,
    is_custom: bool,
    dump_dir: Option<&str>,
    should_open: bool,
) -> Result<PathBuf> {
    let schemes_path = schemes_dir_path(data_path, is_custom)?;
    let schemes_json = scheme_entries_json(&schemes_path)?;
    let output_dir = dump_dir.map_or_else(
        || data_path.join(ARTIFACTS_DIR).join(GALLERY_DIR_NAME),
        PathBuf::from,
    );

    write_gallery_files(&output_dir, &schemes_json)?;

    let index_path = output_dir.join("index.html");
    if should_open {
        open_in_browser(&index_path)?;
    }

    println!("Gallery written to {}", index_path.display());

    Ok(index_path)
}

fn write_gallery_files(output_dir: &Path, schemes_json: &str) -> Result<()> {
    let assets_dir = output_dir.join("assets");

    ensure_directory_exists(output_dir)?;
    ensure_directory_exists(&assets_dir)?;

    write_to_file(output_dir.join("index.html"), INDEX_HTML)?;
    write_to_file(assets_dir.join("gallery.css"), GALLERY_CSS)?;
    let gallery_js = GALLERY_JS.replace("__TINTY_SCHEMES__", schemes_json);
    write_to_file(assets_dir.join("gallery.js"), &gallery_js)?;
    write_binary_file(assets_dir.join("tinted-theming-logo.png"), LOGO_BYTES)?;
    write_binary_file(assets_dir.join("favicon.png"), FAVICON_BYTES)?;

    Ok(())
}

fn write_binary_file(path: impl AsRef<Path>, contents: &[u8]) -> Result<()> {
    let mut file = File::create(path.as_ref())
        .map_err(anyhow::Error::new)
        .with_context(|| format!("Unable to create file: {}", path.as_ref().display()))?;

    file.write_all(contents)?;

    Ok(())
}

fn open_in_browser(index_path: &Path) -> Result<()> {
    let index_path = index_path
        .canonicalize()
        .with_context(|| format!("Unable to resolve {}", index_path.display()))?;

    let mut command = browser_command(&index_path);
    let status = command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .with_context(|| format!("Unable to open gallery at {}", index_path.display()))?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Unable to open gallery at {}",
            index_path.display()
        ));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn browser_command(path: &Path) -> Command {
    let mut command = Command::new("open");
    command.arg(path);
    command
}

#[cfg(target_os = "windows")]
fn browser_command(path: &Path) -> Command {
    let mut command = Command::new("cmd");
    command.args(["/C", "start", ""]).arg(path);
    command
}

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
fn browser_command(path: &Path) -> Command {
    let mut command = Command::new("xdg-open");
    command.arg(path);
    command
}
