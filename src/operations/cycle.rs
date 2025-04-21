use crate::config::Config;
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
    let next_theme = normalized_cycle(&config)
        .as_ref()
        .and_then(|vec| {
            let next_index = vec.iter().position(|scheme| scheme == &current_scheme_slug)
                .map(|i| i + 1)
                .unwrap_or(0);
            Some(vec[(next_index) % vec.len()].clone())
        })
        .unwrap_or(current_scheme_slug);

    if !is_quiet {
        println!("Switching to next theme in cycle: {}", next_theme);
    }

    apply(config_path, data_path, &next_theme, is_quiet, active_operation)
}

fn normalized_cycle(config: &Config) -> Option<Vec<String>> {
    // Return a list of preferred schemes based on presence of this value in the config, and
    // whatever the default scheme is if specified in config also.
    config.preferred_schemes.as_ref().map(|preferred| {
        // If default scheme is defined, add it to the cycle.
        config
            .default_scheme
            .as_ref()
            .filter(|default| !preferred.contains(default))
            .map(|default| {
                let mut result = vec![default.clone()];
                result.extend(preferred.clone());
                result
            })
            .unwrap_or_else(|| preferred.clone())
    }).or_else(|| {
        // If default scheme is defined, use it if preferred schemes is unset.
        config.default_scheme.as_ref().map(|theme| vec![theme.clone()])
    })
}
