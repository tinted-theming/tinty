pub const REPO_URL: &str = env!("CARGO_PKG_HOMEPAGE");
pub const REPO_NAME: &str = env!("CARGO_PKG_NAME");
pub const REPO_DIR: &str = "repos";
/// Subdirectory of the data dir that holds scheme repositories: the built-in
/// `schemes` repo plus any `[[schemes.extras]]` the user configures. Kept
/// separate from `repos/` (which holds template `[[items]]`) so scheme
/// collections and item templates never share a namespace.
pub const SCHEME_REPO_DIR: &str = "scheme-repos";
pub const ARTIFACTS_DIR: &str = "artifacts";
pub const LOCK_FILE: &str = ".tinty.lock";
pub const SCHEMES_REPO_URL: &str = "https://github.com/tinted-theming/schemes";
pub const SCHEMES_REPO_NAME: &str = "schemes";
pub const CUSTOM_SCHEMES_DIR_NAME: &str = "custom-schemes";
pub const CURRENT_SCHEME_FILE_NAME: &str = "current_scheme";
pub const DEFAULT_SCHEME_SYSTEM: &str = "base16";
pub const SCHEMES_REPO_REVISION: &str = "spec-0.11";
/// Fallback Git revision used when a repository has no configured `revision`.
pub const DEFAULT_REVISION: &str = "main";
