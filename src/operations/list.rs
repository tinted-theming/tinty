use anyhow::Result;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "public/"]
#[prefix = "public/"]
struct Asset;

/// Lists available color schemes in the base16-shell-manager repository.
///
/// This function checks the provided base16-shell-manager repository path to determine if it contains
/// color scheme scripts. It validates that the provided path is a directory, collects the names
/// of available color schemes by inspecting the scripts in the directory, and prints them.
pub fn list() -> Result<()> {
    let asset = Asset::get("public/schemes.txt").unwrap();

    if let Ok(contents) = std::str::from_utf8(asset.data.as_ref()).map_err(anyhow::Error::new) {
        println!("{}", contents);
    }

    Ok(())
}
