use std::path::PathBuf;

use anyhow::Result;
use tinted_builder::{SchemeSystem, SchemeVariant};
use tinted_scheme_extractor::{create_scheme_from_image, SchemeParams};

use crate::utils::write_to_file;

#[allow(clippy::too_many_arguments)]
pub fn generate_scheme(
    image_path: PathBuf,
    output_file_path_option: Option<PathBuf>,
    author: String,
    description: Option<String>,
    name: String,
    slug: String,
    system: SchemeSystem,
    variant: SchemeVariant,
) -> Result<()> {
    let params = SchemeParams {
        author,
        description,
        image_path,
        name,
        slug,
        system,
        variant,
        verbose: false,
    };
    let scheme = create_scheme_from_image(params)?;

    match output_file_path_option {
        Some(path) => {
            let contents = serde_yaml::to_string(&scheme)?;

            write_to_file(&path, &contents)?;

            println!("Scheme created: {}", path.display());
        }
        None => print!("{scheme}"), // Scheme .display() already ends with a newline
    }

    Ok(())
}
