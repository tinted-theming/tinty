use anyhow::Result;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "public/"]
#[prefix = "public/"]
struct Asset;

/// Lists available color schemes
///
/// Lists colorschemes file which is updated via scripts/setup by getting a list of schemes
/// available in https://github.com/tinted-theming/schemes/base16
pub fn list() -> Result<()> {
    let asset = Asset::get("public/schemes.txt").unwrap();

    if let Ok(contents) = std::str::from_utf8(asset.data.as_ref()).map_err(anyhow::Error::new) {
        println!("{}", contents);
    }

    Ok(())
}
