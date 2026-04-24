use crate::config::Config;
use crate::operations::apply::apply;
use crate::operations::current::get_current_scheme_slug;
use crate::utils::{cycle_scheme_list, next_scheme_in_cycle};
use anyhow::Result;
use std::path::Path;

/// Cycle to next scheme in a configured ring.
pub fn cycle(
    config_path: &Path,
    data_path: &Path,
    is_quiet: bool,
    ring_name: Option<&str>,
    active_operation: Option<&str>,
) -> Result<()> {
    let config = Config::read(config_path)?;

    let current_scheme_slug = get_current_scheme_slug(data_path);

    let schemes = cycle_scheme_list(&config, ring_name)?;
    let next_theme = next_scheme_in_cycle(&current_scheme_slug, &schemes);

    if !is_quiet {
        println!("Applying next theme in cycle: {next_theme}");
    }

    apply(
        config_path,
        data_path,
        &next_theme,
        is_quiet,
        active_operation,
    )
}
