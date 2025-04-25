use crate::config::Config;
use crate::utils::{next_scheme_in_cycle, user_curated_scheme_list};
use anyhow::Result;
use std::path::Path;
use crate::operations::current::get_current_scheme_slug;
use crate::operations::apply::apply;

/// Cycle to next preferred scheme
pub fn cycle(
    config_path: &Path,
    data_path: &Path,
    is_quiet: bool,
    active_operation: Option<&str>,
) -> Result<()> {

    let config = Config::read(config_path)?;

    let current_scheme_slug = get_current_scheme_slug(data_path);

    // Figure out what the next theme should be given current theme & preferred schemes.
    let next_theme = user_curated_scheme_list(&config)
        .as_ref()
        .map(|vec| {
            next_scheme_in_cycle(&current_scheme_slug, vec.to_vec())
        })
        .unwrap_or(current_scheme_slug);

    if !is_quiet {
        println!("Applying next theme in cycle: {}", next_theme);
    }

    apply(config_path, data_path, &next_theme, is_quiet, active_operation)
}

