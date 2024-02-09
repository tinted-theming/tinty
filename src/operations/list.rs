use crate::{
    constants::{REPO_DIR, REPO_NAME, SCHEMES_REPO_NAME},
    utils::get_all_scheme_names,
};
use anyhow::{anyhow, Result};
use std::path::Path;

/// Lists available color schemes
///
/// Lists colorschemes file which is updated via scripts/setup by getting a list of schemes
/// available in https://github.com/tinted-theming/schemes/base16
pub fn list(data_path: &Path) -> Result<()> {
    let schemes_repo_path = data_path.join(format!("{}/{}", REPO_DIR, SCHEMES_REPO_NAME));

    if !schemes_repo_path.exists() {
        return Err(anyhow!(
            "Schemes are missing, run setup and then try again: `{} setup`",
            REPO_NAME
        ));
    }

    let scheme_vec = get_all_scheme_names(data_path)?;
    for scheme in scheme_vec {
        println!("{}", scheme);
    }

    Ok(())
}
