use crate::constants::{REPO_NAME, SCHEMES_REPO_NAME, SCHEMES_REPO_REVISION, SCHEMES_REPO_URL};
use crate::utils::replace_tilde_slash_with_home;
use anyhow::{anyhow, Context, Result};
use home::home_dir;
use serde::Deserialize;
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::Path;
use tinted_builder::SchemeSystem;
use url::Url;

pub const DEFAULT_CONFIG_SHELL: &str = "sh -c '{}'";
pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const ORG_NAME: &str = "tinted-theming";
pub const BASE16_SHELL_REPO_URL: &str = "https://github.com/tinted-theming/tinted-shell";
pub const BASE16_SHELL_REPO_NAME: &str = "tinted-shell";
pub const BASE16_SHELL_THEMES_DIR: &str = "scripts";
pub const BASE16_SHELL_HOOK: &str = ". %f";

/// Structure for configuration apply items
#[derive(Deserialize, Debug)]
pub struct ConfigItem {
    pub name: String,
    pub path: String,
    pub hook: Option<String>,
    #[serde(rename = "themes-dir")]
    pub themes_dir: String,
    #[serde(rename = "supported-systems")]
    pub supported_systems: Option<Vec<SchemeSystem>>,
    #[serde(rename = "theme-file-extension")]
    pub theme_file_extension: Option<String>,
    pub revision: Option<String>,
    #[serde(rename = "write-to-file")]
    pub write_to_file: Option<Vec<String>>,
    /// When `true`, `tinty update` is allowed to proceed even if this item's
    /// local copy has uncommitted changes. Non-overlapping local edits are
    /// carried forward; an update that would overwrite local changes is
    /// refused without touching the working tree. Defaults to `false`.
    #[serde(default, rename = "allow-dirty-update")]
    pub allow_dirty_update: bool,
}

impl fmt::Display for ConfigItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hook = self.hook.clone().unwrap_or_default();
        let revision = self.revision.clone().unwrap_or_default();
        let default_supported_systems = vec![SchemeSystem::default()];
        let system_text = self
            .supported_systems
            .clone()
            .unwrap_or(default_supported_systems)
            .into_iter()
            .map(|system| format!("\"{system}\""))
            .collect::<Vec<String>>()
            .join(", ");

        // You can format the output however you like
        writeln!(f)?;
        writeln!(f, "[[items]]")?;
        writeln!(f, "name = \"{}\"", self.name)?;
        writeln!(f, "path = \"{}\"", self.path)?;
        if !hook.is_empty() {
            writeln!(f, "hook = \"{hook}\"")?;
        }
        if !revision.is_empty() {
            writeln!(f, "revision = \"{revision}\"")?;
        }
        if self.allow_dirty_update {
            writeln!(f, "allow-dirty-update = true")?;
        }
        writeln!(f, "supported-systems = [{system_text}]")?;
        write!(f, "themes-dir = \"{}\"", self.themes_dir)
    }
}

/// Structure for configuration cycle rings
#[derive(Deserialize, Debug)]
pub struct ConfigRing {
    pub name: String,
    pub schemes: Vec<String>,
}

impl fmt::Display for ConfigRing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let schemes_text = self
            .schemes
            .iter()
            .map(|scheme| format!("\"{scheme}\""))
            .collect::<Vec<String>>()
            .join(", ");

        writeln!(f)?;
        writeln!(f, "[[rings]]")?;
        writeln!(f, "name = \"{}\"", self.name)?;
        write!(f, "schemes = [{schemes_text}]")
    }
}

/// An additional scheme repository declared under `[[schemes.extras]]`. Extra
/// repos are merged with the built-in `schemes` repo into a single scheme
/// collection. They follow the same source mechanics as `[[items]]`: `path` is
/// a Git URL (cloned) or a local directory (symlinked), with an optional
/// `revision`. Unlike an item, an extra is a collection of `<system>/<slug>.yaml`
/// scheme files, not a template — so it has no `themes-dir`, `hook`, or
/// `supported-systems`.
#[derive(Deserialize, Debug, Clone)]
pub struct SchemeRepoConfig {
    /// Unique name for the repo; also its directory under `scheme-repos/`.
    pub name: String,
    /// Git URL (cloned) or local directory (symlinked). A leading `~/` is
    /// expanded to the home directory during config read.
    pub path: String,
    /// Git revision (branch, tag, or commit SHA) to check out. Ignored for a
    /// local-directory `path`.
    pub revision: Option<String>,
    /// When `true`, `tinty update` may proceed even if this repo's local copy
    /// has uncommitted changes. Mirrors an item's `allow-dirty-update`.
    #[serde(default, rename = "allow-dirty-update")]
    pub allow_dirty_update: bool,
}

impl fmt::Display for SchemeRepoConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        writeln!(f, "[[schemes.extras]]")?;
        writeln!(f, "name = \"{}\"", self.name)?;
        write!(f, "path = \"{}\"", self.path)?;
        if let Some(revision) = &self.revision {
            write!(f, "\nrevision = \"{revision}\"")?;
        }
        if self.allow_dirty_update {
            write!(f, "\nallow-dirty-update = true")?;
        }
        Ok(())
    }
}

/// Settings for the built-in schemes repository, which has no `[[items]]`
/// entry of its own. Grouped under a `[schemes]` table so more schemes-repo
/// specific options can be added here in the future.
#[derive(Deserialize, Debug, Default)]
pub struct SchemesConfig {
    /// When `true`, `tinty update` is allowed to proceed even if the schemes
    /// repo has uncommitted changes. Behaves like an item's `allow-dirty-update`.
    /// Defaults to `false`, preserving the strict skip-if-dirty behavior.
    #[serde(default, rename = "allow-dirty-update")]
    pub allow_dirty_update: bool,
    /// Overrides the source of the built-in schemes repository. May be a Git
    /// remote URL (cloned into `repos/schemes`) or a local directory (symlinked
    /// as `repos/schemes`). When unset, the built-in
    /// `tinted-theming/schemes` repo is used. A local directory need not be a
    /// Git repository.
    #[serde(default)]
    pub path: Option<String>,
    /// Git revision (branch, tag, or commit SHA) to check out for the schemes
    /// repo, mirroring an item's `revision`. Ignored when `path` points at a
    /// local directory. When unset and `path` is unset, the built-in pinned
    /// revision is used.
    #[serde(default)]
    pub revision: Option<String>,
    /// Additional scheme repositories merged with the built-in `schemes` repo.
    /// Declared as `[[schemes.extras]]` array-of-tables.
    #[serde(default)]
    pub extras: Vec<SchemeRepoConfig>,
}

/// Rejects a `[schemes].path` (a local directory) that resolves to tinty's own
/// managed schemes directory (`repos/schemes`). Symlinking that slot to itself,
/// or cloning it into itself, is a circular reference. Git URL sources can never
/// name the local slot, so they are always accepted here.
pub fn ensure_schemes_path_not_circular(source: &str, schemes_repo_path: &Path) -> Result<()> {
    if Url::parse(source).is_ok() {
        return Ok(());
    }

    // Identity of the schemes-repo slot itself, computed without dereferencing a
    // symlink that may already occupy it (canonicalizing the slot directly would
    // follow such a symlink and produce a false positive on repeat runs).
    let slot_identity = schemes_repo_path
        .parent()
        .and_then(|parent| parent.canonicalize().ok())
        .zip(schemes_repo_path.file_name())
        .map(|(parent, name)| parent.join(name));

    if let (Ok(source_canon), Some(slot)) = (Path::new(source).canonicalize(), slot_identity) {
        if source_canon == slot {
            return Err(anyhow!(
                "config.toml [schemes].path points at {REPO_NAME}'s own managed schemes directory ({}). This would create a circular reference; point it at a different directory or a Git URL.",
                schemes_repo_path.display()
            ));
        }
    }

    Ok(())
}

/// Structure for configuration
#[derive(Deserialize, Debug)]
pub struct Config {
    pub shell: Option<String>,
    #[serde(rename = "default-scheme")]
    pub default_scheme: Option<String>,
    #[serde(rename = "preferred-schemes")]
    pub preferred_schemes: Option<Vec<String>>,
    pub rings: Option<Vec<ConfigRing>>,
    #[serde(rename = "default-cycle-ring")]
    pub default_cycle_ring: Option<String>,
    pub items: Option<Vec<ConfigItem>>,
    pub hooks: Option<Vec<String>>,
    #[serde(default)]
    pub schemes: SchemesConfig,
}

fn ensure_item_name_is_unique(items: &[ConfigItem]) -> Result<()> {
    let mut names = HashSet::new();

    for item in items {
        if item.name == SCHEMES_REPO_NAME {
            return Err(anyhow!("config.toml item.name \"{SCHEMES_REPO_NAME}\" is reserved for the built-in schemes repository and cannot be used for a custom item. Please rename this item."));
        }

        if !names.insert(&item.name) {
            return Err(anyhow!("config.toml item.name should be unique values, but \"{}\" is used for more than 1 item.name. Please change this to a unique value.", item.name));
        }
    }

    Ok(())
}

/// Validates `[[schemes.extras]]` names: each must be non-empty, unique among
/// extras, and must not be the reserved `schemes` name (which addresses the
/// built-in repo). Names double as directory names under `scheme-repos/`, so a
/// collision would make two repos fight over one slot.
fn ensure_scheme_extras_are_valid(extras: &[SchemeRepoConfig]) -> Result<()> {
    let mut names = HashSet::new();

    for extra in extras {
        if extra.name.trim().is_empty() {
            return Err(anyhow!(
                "config.toml schemes.extras.name should not be empty"
            ));
        }

        if extra.name == SCHEMES_REPO_NAME {
            return Err(anyhow!("config.toml schemes.extras.name \"{SCHEMES_REPO_NAME}\" is reserved for the built-in schemes repository. Please rename this extra scheme repo."));
        }

        if !names.insert(&extra.name) {
            return Err(anyhow!("config.toml schemes.extras.name should be unique values, but \"{}\" is used for more than 1 extra. Please change this to a unique value.", extra.name));
        }
    }

    Ok(())
}

fn ensure_ring_names_are_valid(rings: &[ConfigRing]) -> Result<()> {
    let mut names = HashSet::new();

    for ring in rings {
        if ring.name.trim().is_empty() {
            return Err(anyhow!("config.toml rings.name should not be empty"));
        }

        if !names.insert(&ring.name) {
            return Err(anyhow!("config.toml rings.name should be unique values, but \"{}\" is used for more than 1 rings.name. Please change this to a unique value.", ring.name));
        }
    }

    Ok(())
}

impl Config {
    /// Resolves the effective source and revision for the built-in schemes
    /// repository from the `[schemes]` table.
    ///
    /// Returns the source (a Git URL or local directory) and the revision to
    /// check out (`None` lets the repository backend pick its default, matching
    /// `[[items]]`). Backwards-compatible: with neither `path` nor `revision`
    /// set, this is exactly the built-in repo pinned at its default revision.
    /// A `revision` may be set on its own to re-pin the built-in repo, and a
    /// `path` may be set on its own to swap the source while defaulting the
    /// revision.
    pub fn schemes_source(&self) -> (String, Option<String>) {
        self.schemes.path.as_ref().map_or_else(
            || {
                (
                    SCHEMES_REPO_URL.to_string(),
                    Some(
                        self.schemes
                            .revision
                            .clone()
                            .unwrap_or_else(|| SCHEMES_REPO_REVISION.to_string()),
                    ),
                )
            },
            |path| (path.clone(), self.schemes.revision.clone()),
        )
    }

    #[allow(clippy::too_many_lines)]
    pub fn read(path: &Path) -> Result<Self> {
        if path.exists() && !path.is_file() {
            return Err(anyhow!(
                "The provided config path is a directory and not a file: {}",
                path.display()
            ));
        }

        let contents = fs::read_to_string(path).unwrap_or_default();
        let mut config: Self = toml::from_str(contents.as_str()).with_context(|| {
            format!(
                "Couldn't parse {REPO_NAME} configuration file ({}). Check if it's syntactically correct",
                path.display()
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
            supported_systems: Some(vec![SchemeSystem::Base16]), // DEFAULT_SCHEME_SYSTEM
            theme_file_extension: None,
            revision: None,
            write_to_file: None,
            allow_dirty_update: false,
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

        if let Some(rings) = config.rings.as_ref() {
            ensure_ring_names_are_valid(rings)?;

            if let Some(default_cycle_ring) = config.default_cycle_ring.as_ref() {
                if !rings.iter().any(|ring| ring.name == *default_cycle_ring) {
                    return Err(anyhow!(
                        "config.toml default-cycle-ring is set to \"{default_cycle_ring}\", but no ring with that name exists"
                    ));
                }
            }
        } else if let Some(default_cycle_ring) = config.default_cycle_ring.as_ref() {
            return Err(anyhow!(
                "config.toml default-cycle-ring is set to \"{default_cycle_ring}\", but no rings are configured"
            ));
        }

        // Set default `system` property for missing systems
        if let Some(ref mut items) = config.items {
            for item in items.iter_mut() {
                if item.supported_systems.is_none() {
                    item.supported_systems = Some(vec![SchemeSystem::default()]);
                }

                // Replace `~/` with absolute home path
                let trimmed_path = item.path.trim();
                if trimmed_path.starts_with("~/") {
                    match home_dir() {
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

        // Normalize and validate the optional `[schemes].path` the same way item
        // paths are handled: expand a leading `~/`, then require it be a valid
        // URL or an existing local directory. Unlike an item, a local directory
        // need not be a Git repository.
        if let Some(raw_path) = config.schemes.path.clone() {
            let expanded = replace_tilde_slash_with_home(&raw_path)?
                .to_string_lossy()
                .into_owned();

            if Url::parse(&expanded).is_err() && !Path::new(&expanded).is_dir() {
                return Err(anyhow!("config.toml [schemes].path \"{expanded}\" is not a valid url and is not a path to an existing local directory"));
            }

            config.schemes.path = Some(expanded);
        }

        // Validate `[[schemes.extras]]` names, then normalize each extra's path
        // exactly like an item path: expand a leading `~/` and require a valid
        // URL or an existing local directory.
        ensure_scheme_extras_are_valid(&config.schemes.extras)?;
        for extra in &mut config.schemes.extras {
            let expanded = replace_tilde_slash_with_home(&extra.path)?
                .to_string_lossy()
                .into_owned();

            if Url::parse(&expanded).is_err() && !Path::new(&expanded).is_dir() {
                return Err(anyhow!("config.toml schemes.extras \"{}\" has an invalid `path` value. \"{expanded}\" is not a valid url and is not a path to an existing local directory", extra.name));
            }

            extra.path = expanded;
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
        let none_value = String::from("None");
        writeln!(
            f,
            "shell = \"{}\"",
            self.shell.as_ref().unwrap_or(&none_value)
        )?;

        if let Some(default_scheme) = &self.default_scheme {
            writeln!(f, "default-scheme = \"{default_scheme}\"")?;
        }

        if let Some(default_cycle_ring) = &self.default_cycle_ring {
            writeln!(f, "default-cycle-ring = \"{default_cycle_ring}\"")?;
        }

        if let Some(items) = &self.preferred_schemes {
            let preferred_schemes_text = items
                .clone()
                .into_iter()
                .map(|t| format!("\"{t}\""))
                .collect::<Vec<String>>()
                .join(", ");
            writeln!(f, "preferred-schemes = [{preferred_schemes_text}]")?;
        }

        if let Some(hooks) = &self.hooks {
            writeln!(f, "hooks = [")?;
            for hook in hooks {
                writeln!(f, "  \"{hook}\"")?;
            }
            writeln!(f, "]")?;
        }

        // Emitted before the `[[rings]]`/`[[items]]` array-of-tables so its
        // keys are not mis-parsed as belonging to the last array entry. The
        // `[schemes]` scalar keys must precede the `[[schemes.extras]]`
        // array-of-tables for the same reason.
        if self.schemes.path.is_some()
            || self.schemes.revision.is_some()
            || self.schemes.allow_dirty_update
        {
            writeln!(f, "\n[schemes]")?;
            if let Some(path) = &self.schemes.path {
                writeln!(f, "path = \"{path}\"")?;
            }
            if let Some(revision) = &self.schemes.revision {
                writeln!(f, "revision = \"{revision}\"")?;
            }
            if self.schemes.allow_dirty_update {
                writeln!(f, "allow-dirty-update = true")?;
            }
        }

        for extra in &self.schemes.extras {
            writeln!(f, "{extra}")?;
        }

        if let Some(rings) = &self.rings {
            for ring in rings {
                writeln!(f, "{ring}")?;
            }
        }

        if let Some(items) = &self.items {
            for item in items {
                writeln!(f, "{item}")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, ConfigItem, SchemeRepoConfig};

    fn item_with(allow_dirty_update: bool) -> ConfigItem {
        ConfigItem {
            name: "example".to_string(),
            path: "https://example.com/repo".to_string(),
            hook: None,
            themes_dir: "themes".to_string(),
            supported_systems: None,
            theme_file_extension: None,
            revision: None,
            write_to_file: None,
            allow_dirty_update,
        }
    }

    #[test]
    fn item_allow_dirty_update_parses_when_present() {
        let toml = r#"
name = "example"
path = "https://example.com/repo"
themes-dir = "themes"
allow-dirty-update = true
"#;
        let item: ConfigItem = toml::from_str(toml).unwrap();
        assert!(item.allow_dirty_update);
    }

    #[test]
    fn item_allow_dirty_update_defaults_to_false_when_absent() {
        let toml = r#"
name = "example"
path = "https://example.com/repo"
themes-dir = "themes"
"#;
        let item: ConfigItem = toml::from_str(toml).unwrap();
        assert!(!item.allow_dirty_update);
    }

    #[test]
    fn schemes_allow_dirty_update_parses_and_defaults_to_false() {
        let with: Config = toml::from_str("[schemes]\nallow-dirty-update = true\n").unwrap();
        assert!(with.schemes.allow_dirty_update);

        // `[schemes]` table present but the key omitted.
        let table_only: Config = toml::from_str("[schemes]\n").unwrap();
        assert!(!table_only.schemes.allow_dirty_update);

        // No `[schemes]` table at all.
        let without: Config = toml::from_str("shell = \"sh -c '{}'\"\n").unwrap();
        assert!(!without.schemes.allow_dirty_update);
    }

    #[test]
    fn item_display_emits_allow_dirty_update_only_when_true() {
        assert!(item_with(true)
            .to_string()
            .contains("allow-dirty-update = true"));
        assert!(!item_with(false).to_string().contains("allow-dirty-update"));
    }

    #[test]
    fn config_display_emits_schemes_table_only_when_set() {
        let mut config: Config = toml::from_str("[schemes]\nallow-dirty-update = true\n").unwrap();
        config.shell = Some("sh -c '{}'".to_string());
        let rendered = config.to_string();
        assert!(rendered.contains("[schemes]"));
        assert!(rendered.contains("allow-dirty-update = true"));

        let off: Config = toml::from_str("shell = \"sh -c '{}'\"\n").unwrap();
        assert!(!off.to_string().contains("[schemes]"));
    }

    #[test]
    fn schemes_source_defaults_to_builtin_repo_and_revision() {
        // Backwards-compatible default: no `[schemes]` table at all.
        let config: Config = toml::from_str("shell = \"sh -c '{}'\"\n").unwrap();
        let (source, revision) = config.schemes_source();
        assert_eq!(source, super::SCHEMES_REPO_URL);
        assert_eq!(revision.as_deref(), Some(super::SCHEMES_REPO_REVISION));
    }

    #[test]
    fn schemes_source_revision_only_repins_builtin_repo() {
        let config: Config = toml::from_str("[schemes]\nrevision = \"main\"\n").unwrap();
        let (source, revision) = config.schemes_source();
        assert_eq!(source, super::SCHEMES_REPO_URL);
        assert_eq!(revision.as_deref(), Some("main"));
    }

    #[test]
    fn schemes_source_path_overrides_source_and_defaults_revision() {
        let config: Config =
            toml::from_str("[schemes]\npath = \"https://example.com/schemes\"\n").unwrap();
        let (source, revision) = config.schemes_source();
        assert_eq!(source, "https://example.com/schemes");
        // No revision given for a custom source => backend default (None).
        assert_eq!(revision, None);
    }

    #[test]
    fn schemes_source_path_and_revision_are_both_honored() {
        let config: Config = toml::from_str(
            "[schemes]\npath = \"https://example.com/schemes\"\nrevision = \"v1.0.0\"\n",
        )
        .unwrap();
        let (source, revision) = config.schemes_source();
        assert_eq!(source, "https://example.com/schemes");
        assert_eq!(revision.as_deref(), Some("v1.0.0"));
    }

    #[test]
    fn schemes_path_and_revision_parse_and_render() {
        let mut config: Config = toml::from_str(
            "[schemes]\npath = \"https://example.com/schemes\"\nrevision = \"dev\"\n",
        )
        .unwrap();
        config.shell = Some("sh -c '{}'".to_string());
        assert_eq!(
            config.schemes.path.as_deref(),
            Some("https://example.com/schemes")
        );
        assert_eq!(config.schemes.revision.as_deref(), Some("dev"));

        let rendered = config.to_string();
        assert!(rendered.contains("[schemes]"));
        assert!(rendered.contains("path = \"https://example.com/schemes\""));
        assert!(rendered.contains("revision = \"dev\""));
    }

    #[test]
    fn schemes_extras_parse_as_array_of_tables() {
        let config: Config = toml::from_str(concat!(
            "[[schemes.extras]]\nname = \"community\"\npath = \"https://example.com/community\"\nrevision = \"main\"\n\n",
            "[[schemes.extras]]\nname = \"work\"\npath = \"/some/dir\"\nallow-dirty-update = true\n",
        ))
        .unwrap();
        let extras = &config.schemes.extras;
        assert_eq!(extras.len(), 2);
        assert_eq!(extras[0].name, "community");
        assert_eq!(extras[0].path, "https://example.com/community");
        assert_eq!(extras[0].revision.as_deref(), Some("main"));
        assert!(!extras[0].allow_dirty_update);
        assert_eq!(extras[1].name, "work");
        assert!(extras[1].allow_dirty_update);
        assert_eq!(extras[1].revision, None);
    }

    #[test]
    fn schemes_extras_absent_defaults_to_empty() {
        let config: Config = toml::from_str("shell = \"sh -c '{}'\"\n").unwrap();
        assert!(config.schemes.extras.is_empty());
    }

    #[test]
    fn ensure_scheme_extras_rejects_reserved_name() {
        let extras = vec![SchemeRepoConfig {
            name: super::SCHEMES_REPO_NAME.to_string(),
            path: "https://example.com/x".to_string(),
            revision: None,
            allow_dirty_update: false,
        }];
        let err = super::ensure_scheme_extras_are_valid(&extras).unwrap_err();
        assert!(err.to_string().contains("reserved"));
    }

    #[test]
    fn ensure_scheme_extras_rejects_duplicate_names() {
        let extra = |name: &str| SchemeRepoConfig {
            name: name.to_string(),
            path: "https://example.com/x".to_string(),
            revision: None,
            allow_dirty_update: false,
        };
        let err = super::ensure_scheme_extras_are_valid(&[extra("dup"), extra("dup")]).unwrap_err();
        assert!(err.to_string().contains("unique"));
    }

    #[test]
    fn ensure_scheme_extras_rejects_empty_name() {
        let extras = vec![SchemeRepoConfig {
            name: "   ".to_string(),
            path: "https://example.com/x".to_string(),
            revision: None,
            allow_dirty_update: false,
        }];
        let err = super::ensure_scheme_extras_are_valid(&extras).unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn config_display_round_trips_scheme_extras() {
        let mut config: Config = toml::from_str(concat!(
            "[[schemes.extras]]\nname = \"community\"\npath = \"https://example.com/community\"\nrevision = \"main\"\n",
        ))
        .unwrap();
        config.shell = Some("sh -c '{}'".to_string());
        let rendered = config.to_string();
        assert!(rendered.contains("[[schemes.extras]]"));
        assert!(rendered.contains("name = \"community\""));
        assert!(rendered.contains("path = \"https://example.com/community\""));
        assert!(rendered.contains("revision = \"main\""));

        // The rendered config must itself parse back to the same extras.
        let reparsed: Config = toml::from_str(&rendered).unwrap();
        assert_eq!(reparsed.schemes.extras.len(), 1);
        assert_eq!(reparsed.schemes.extras[0].name, "community");
    }

    #[test]
    fn ensure_schemes_path_not_circular_allows_urls_and_other_dirs() {
        use std::path::Path;
        // A URL source is never circular.
        assert!(super::ensure_schemes_path_not_circular(
            "https://example.com/schemes",
            Path::new("/does/not/matter/repos/schemes"),
        )
        .is_ok());

        // A local dir that differs from the slot is fine. Use the temp dir,
        // which exists, as the source and a distinct slot path.
        let tmp = std::env::temp_dir();
        assert!(super::ensure_schemes_path_not_circular(
            tmp.to_str().unwrap(),
            &tmp.join("some-other-data/repos/schemes"),
        )
        .is_ok());
    }

    #[test]
    fn ensure_schemes_path_not_circular_rejects_self_reference() {
        let tmp = tempfile::tempdir().unwrap();
        let repos = tmp.path().join("repos");
        let slot = repos.join("schemes");
        std::fs::create_dir_all(&slot).unwrap();

        // Pointing `[schemes].path` at the slot itself is circular.
        let err = super::ensure_schemes_path_not_circular(slot.to_str().unwrap(), &slot)
            .expect_err("expected a circular-reference error");
        assert!(err.to_string().contains("circular reference"));
    }
}
