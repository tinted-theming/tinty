use include_dir::{include_dir, Dir};

pub const DEFAULT_BASE16_SHELL_PATH: &str = "~/.config/tinted-theming/shell";
pub const BASE16_SHELL_PATH_ENV: &str = "BASE16_SHELL_PATH";
pub const BASE16_SHELL_THEME_NAME_PATH_ENV: &str = "BASE16_SHELL_THEME_NAME_PATH";
pub const BASE16_SHELL_CONFIG_PATH_ENV: &str = "BASE16_CONFIG_PATH";
pub const BASE16_THEME_ENV: &str = "BASE16_THEME";
pub const XDG_CONFIG_HOME_ENV: &str = "XDG_CONFIG_HOME";
pub const HOME_ENV: &str = "HOME";
pub const REPO_URL: &str = "https://github.com/tinted-theming/base16-shell";

static EMBEDDED_HOOKS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/hooks");
static EMBEDDED_COLORSCHEME_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/scripts");

pub fn get_embedded_colorscheme_dir() -> &'static Dir<'static> {
    &EMBEDDED_COLORSCHEME_DIR
}
pub fn get_embedded_hooks_dir() -> &'static Dir<'static> {
    &EMBEDDED_HOOKS_DIR
}
