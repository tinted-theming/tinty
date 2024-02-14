use crate::constants::REPO_NAME;
use crate::utils::read_file_to_string;
use anyhow::{anyhow, Context, Result};
use serde::de::{self, Deserializer, Unexpected, Visitor};
use serde::Deserialize;
use std::collections::HashSet;
use std::fmt;
use std::path::Path;
use url::Url;

pub const DEFAULT_CONFIG_SHELL: &str = "sh -c '{}'";
pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const BASE16_SHELL_REPO_URL: &str = "https://github.com/tinted-theming/base16-shell";
pub const BASE16_SHELL_REPO_NAME: &str = "base16-shell";
pub const BASE16_SHELL_THEMES_DIR: &str = "scripts";
pub const BASE16_SHELL_HOOK: &str = ". %f";

#[derive(Debug, Default, Clone, PartialEq)]
pub enum SupportedSchemeSystems {
    #[default]
    Base16,
    Base24,
}

impl<'de> Deserialize<'de> for SupportedSchemeSystems {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SupportedSystemVisitor;

        impl<'de> Visitor<'de> for SupportedSystemVisitor {
            type Value = SupportedSchemeSystems;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("`base16` or `base24`")
            }

            fn visit_str<E>(self, value: &str) -> Result<SupportedSchemeSystems, E>
            where
                E: de::Error,
            {
                match value {
                    "base16" => Ok(SupportedSchemeSystems::Base16),
                    "base24" => Ok(SupportedSchemeSystems::Base24),
                    _ => Err(E::invalid_value(Unexpected::Str(value), &self)),
                }
            }
        }

        deserializer.deserialize_str(SupportedSystemVisitor)
    }
}

impl SupportedSchemeSystems {
    pub fn to_str(&self) -> &'static str {
        match self {
            SupportedSchemeSystems::Base16 => "base16",
            SupportedSchemeSystems::Base24 => "base24",
        }
    }

    pub fn from_str(system_string: &str) -> SupportedSchemeSystems {
        match system_string {
            "base16" => SupportedSchemeSystems::Base16,
            "base24" => SupportedSchemeSystems::Base24,
            _ => SupportedSchemeSystems::Base16,
        }
    }

    pub fn variants() -> &'static [SupportedSchemeSystems] {
        static VARIANTS: &[SupportedSchemeSystems] = &[
            SupportedSchemeSystems::Base16,
            SupportedSchemeSystems::Base24,
        ];
        VARIANTS
    }
}

/// Structure for configuration apply items
#[derive(Deserialize, Debug)]
pub struct ConfigItem {
    pub name: String,
    pub path: String,
    pub hook: Option<String>,
    pub themes_dir: String,
    pub supported_systems: Option<Vec<SupportedSchemeSystems>>,
}

impl fmt::Display for ConfigItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hook = self.hook.clone().unwrap_or_default();
        let mut system_text = String::new();

        for system in self
            .supported_systems
            .as_ref()
            .unwrap_or(&vec![SupportedSchemeSystems::default()])
        {
            system_text += system.to_str();
        }

        // You can format the output however you like
        writeln!(f, "  - Item")?;
        writeln!(f, "    - name: {}", self.name)?;
        writeln!(f, "    - path: {}", self.path)?;
        if !hook.is_empty() {
            writeln!(f, "    - hook: {}", hook)?;
        }
        writeln!(f, "    - system: {}", system_text)?;
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

fn ensure_item_name_is_unique(items: &[ConfigItem]) -> Result<()> {
    let mut names = HashSet::new();

    for item in items.iter() {
        if !names.insert(&item.name) {
            return Err(anyhow!("config.toml item.name should be unique values, but \"{}\" is used for more than 1 item.name. Please change this to a unique value.", item.name));
        }
    }

    Ok(())
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

        // Create default `item`
        let shell = config
            .shell
            .clone()
            .unwrap_or_else(|| DEFAULT_CONFIG_SHELL.into());
        let base16_shell_config_item = ConfigItem {
            path: BASE16_SHELL_REPO_URL.to_string(),
            name: BASE16_SHELL_REPO_NAME.to_string(),
            themes_dir: BASE16_SHELL_THEMES_DIR.to_string(),
            hook: Some(BASE16_SHELL_HOOK.to_string()),
            supported_systems: Some(vec![SupportedSchemeSystems::Base16]), // DEFAULT_SCHEME_SYSTEM
        };

        // Add default `item` if no items exist
        match config.items.as_ref() {
            Some(items) => {
                ensure_item_name_is_unique(items)?;
            }
            None => {
                config.items = Some(vec![base16_shell_config_item]);
            }
        }

        // Set default `system` property for missing systems
        if let Some(ref mut items) = config.items {
            for item in items.iter_mut() {
                if item.supported_systems.is_none() {
                    item.supported_systems = Some(vec![SupportedSchemeSystems::default()]);
                }

                // Replace `~/` with absolute home path
                let trimmed_path = item.path.trim();
                if trimmed_path.starts_with("~/") {
                    match dirs::home_dir() {
                        Some(home_dir) => {
                            item.path = trimmed_path.replacen(
                                "~/",
                                format!("{}/", home_dir.display()).as_str(),
                                1,
                            );
                        }
                        None => {
                            return Err(anyhow!("Unable to determine a home directory for \"{}\", please use an absolute path instead", item.path));
                        }
                    }
                }

                // Return Err if path is not a valid url or an existing directory path
                if Url::parse(item.path.as_str()).is_err()
                    && !Path::new(item.path.as_str()).is_dir()
                {
                    return Err(anyhow!("One of your config.toml items has an invalid `path` value. \"{}\" is not a valid url and is not a path to an existing local directory", item.path));
                }
            }
        }

        if !shell.contains("{}") {
            let msg = "The configured shell does not contain the required command placeholder '{}'. Check the default file or github for config examples.";
            return Err(anyhow!(msg));
        }

        config.shell = Some(shell);

        Ok(config)
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
