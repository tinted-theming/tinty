use crate::hooks;
use anyhow::Result;
use std::path::Path;

/// Initializes the base16 colorscheme and runs the associated colorscheme script.
///
/// This function sets up the base16 colorscheme by executing a shell script specified by
/// `theme_path`. It also checks if the necessary configuration files exist
/// and if not, it attempts to read the theme name from `theme_name_path`.
pub fn init(
    app_config_path: &Path,
    theme_name_path: &Path,
    default_theme_name: &str,
) -> Result<()> {
    let (base16_shell_init_theme_response, is_base16_shell_init_theme_success) =
        hooks::base16_shell::init_theme(app_config_path, theme_name_path, default_theme_name)?;

    let hooks = [(
        base16_shell_init_theme_response,
        is_base16_shell_init_theme_success,
    )];

    for (init_theme_response, is_init_theme_success) in hooks {
        if !is_init_theme_success && !init_theme_response.is_empty() {
            println!("{}", init_theme_response);
        }
    }

    Ok(())
}
