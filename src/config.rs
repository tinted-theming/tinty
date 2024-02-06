use crate::constants::REPO_NAME;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fmt;
use std::path::Path;

use crate::utils::read_file_to_string;

pub const DEFAULT_CONFIG_SHELL: &str = "sh -c '{}'";
pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const BASE16_SHELL_REPO_URL: &str = "https://github.com/tinted-theming/base16-shell";
pub const BASE16_SHELL_REPO_NAME: &str = "base16-shell";
pub const BASE16_SHELL_THEMES_DIR: &str = "scripts";
pub const BASE16_SHELL_HOOK: &str = ". %f";

/// Structure for configuration apply items
#[derive(Deserialize, Debug)]
pub struct ConfigItem {
    pub name: String,
    pub git_url: String,
    pub hook: Option<String>,
    pub themes_dir: String,
}

impl fmt::Display for ConfigItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hook = self.hook.clone().unwrap_or_default();
        // You can format the output however you like
        writeln!(f, "  - Item")?;
        writeln!(f, "    - name: {}", self.name)?;
        writeln!(f, "    - git_url: {}", self.git_url)?;
        if !hook.is_empty() {
            writeln!(f, "    - hook: {}", hook)?;
        }
        writeln!(f, "    - themes_dir: {}", self.themes_dir)
    }
}

/// Structure for configuration
#[derive(Deserialize, Debug)]
pub struct Config {
    pub shell: Option<String>,
    pub default_scheme: Option<String>,
    pub items: Option<Vec<ConfigItem>>,
}

impl Config {
    pub fn read(path: &Path) -> Result<Config> {
        let contents =
            read_file_to_string(&path.join(CONFIG_FILE_NAME)).unwrap_or(String::from(""));
        let mut config: Config = toml::from_str(contents.as_str()).with_context(|| {
            format!(
                "Couldn't parse {} configuration file ({:?}). Check if it's syntactically correct",
                REPO_NAME, path
            )
        })?;

        // Add default values
        let shell = config
            .shell
            .clone()
            .unwrap_or_else(|| DEFAULT_CONFIG_SHELL.into());
        let base16_shell_config_item = ConfigItem {
            git_url: BASE16_SHELL_REPO_URL.to_string(),
            name: BASE16_SHELL_REPO_NAME.to_string(),
            themes_dir: BASE16_SHELL_THEMES_DIR.to_string(),
            hook: Some(BASE16_SHELL_HOOK.to_string()),
        };

        if config.items.is_none() {
            config.items = Some(vec![base16_shell_config_item]);
        }

        if !shell.contains("{}") {
            // Hide {} in this error message from the formatting machinery in anyhow macro
            let msg = "The configured shell does not contain the required command placeholder '{}'. Check the default file or github for config examples.";
            return Err(anyhow!(msg));
        }

        config.shell = Some(shell);

        Ok(config)
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // You can format the output however you like
        writeln!(f, "Config")?;
        writeln!(
            f,
            "- Shell: {}",
            self.shell.as_ref().unwrap_or(&"None".to_string())
        )?;

        match &self.items {
            Some(items) => {
                for item in items {
                    writeln!(f, "- Items\n{}", item)?;
                }
            }
            None => {}
        }

        Ok(())
    }
}
