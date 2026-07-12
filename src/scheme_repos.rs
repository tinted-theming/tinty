//! Scheme repositories: the built-in `schemes` repo plus any user-configured
//! `[[schemes.extras]]`.
//!
//! All scheme repositories live under `<data>/scheme-repos/`, kept separate from
//! template `[[items]]` (which live under `<data>/repos/`). Each repo is a
//! directory of `<system>/<slug>.yaml` scheme files, exactly like
//! `tinted-theming/schemes`. When more than one repo defines the same
//! `<system>-<slug>` key, the collection here resolves the conflict
//! deterministically by treating the repos as an ordered overlay stack
//! `[built-in, extra1, extra2, …]` where the last one wins: extras override the
//! built-in repo, and later-listed extras override earlier ones. This lets a
//! user override a built-in (or earlier-extra) scheme by declaring their own
//! lower in `config.toml`.

use crate::config::Config;
use crate::constants::{REPO_DIR, REPO_NAME, SCHEMES_REPO_NAME, SCHEME_REPO_DIR};
use crate::utils::ensure_directory_exists;
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tinted_builder_rust::operation_build::utils::{get_scheme_files_by_name, SchemeFile};

/// A scheme repository tinty knows about: the built-in `schemes` repo or a
/// configured extra. `path` is the on-disk directory that holds the
/// `<system>/<slug>.yaml` tree.
#[derive(Debug, Clone)]
pub struct SchemeRepoRef {
    /// The repo's name (`"schemes"` for the built-in, otherwise the extra's
    /// configured `name`). Used to label duplicate-scheme conflicts.
    pub name: String,
    /// The on-disk directory holding this repo's `<system>/` scheme tree.
    pub path: PathBuf,
}

/// Directory that holds every scheme repository: `<data>/scheme-repos`.
#[must_use]
pub fn scheme_repos_dir(data_path: &Path) -> PathBuf {
    data_path.join(SCHEME_REPO_DIR)
}

/// On-disk location of the built-in schemes repo: `<data>/scheme-repos/schemes`.
#[must_use]
pub fn builtin_schemes_repo_path(data_path: &Path) -> PathBuf {
    scheme_repos_dir(data_path).join(SCHEMES_REPO_NAME)
}

/// On-disk location of an extra scheme repo: `<data>/scheme-repos/<name>`.
#[must_use]
pub fn extra_repo_path(data_path: &Path, name: &str) -> PathBuf {
    scheme_repos_dir(data_path).join(name)
}

/// Pre-relocation location of the built-in schemes repo: `<data>/repos/schemes`.
/// Only referenced by the one-time migration.
#[must_use]
pub fn legacy_schemes_repo_path(data_path: &Path) -> PathBuf {
    data_path.join(REPO_DIR).join(SCHEMES_REPO_NAME)
}

/// Every scheme repository configured for this invocation, in precedence order:
/// the built-in `schemes` repo first, then the `[[schemes.extras]]` in config
/// order. Existence on disk is not checked here — callers that read schemes skip
/// missing directories; callers that install/update create them.
#[must_use]
pub fn scheme_repo_refs(data_path: &Path, config: &Config) -> Vec<SchemeRepoRef> {
    let mut refs = vec![SchemeRepoRef {
        name: SCHEMES_REPO_NAME.to_string(),
        path: builtin_schemes_repo_path(data_path),
    }];

    for extra in &config.schemes.extras {
        refs.push(SchemeRepoRef {
            name: extra.name.clone(),
            path: extra_repo_path(data_path, &extra.name),
        });
    }

    refs
}

/// A scheme key defined by more than one repo. The occurrence from `kept_repo`
/// is used; the one from `shadowed_repo` is discarded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemeConflict {
    /// The `<system>-<slug>` key defined in both repos.
    pub key: String,
    /// The repo whose scheme is used (higher precedence).
    pub kept_repo: String,
    /// The repo whose scheme is shadowed (lower precedence).
    pub shadowed_repo: String,
}

/// The merged scheme collection across all configured repos.
pub struct MergedSchemes {
    /// `<system>-<slug>` → the winning `SchemeFile`.
    pub files: HashMap<String, SchemeFile>,
    /// Keys defined by more than one repo, in discovery order.
    pub conflicts: Vec<SchemeConflict>,
}

/// Collects and merges the schemes from every configured repo that exists on
/// disk, applying last-listed-wins precedence (extras override the built-in,
/// later extras override earlier ones) — the repos are an overlay stack with
/// the built-in at the bottom.
///
/// Returns an error only when *no* scheme repository exists on disk yet, keeping
/// tinty's long-standing "run install" guidance. Configured-but-not-yet-installed
/// extras are skipped silently; `install`/`update` are what create them.
pub fn collect_merged_schemes(refs: &[SchemeRepoRef]) -> Result<MergedSchemes> {
    let mut files: HashMap<String, SchemeFile> = HashMap::new();
    // Which repo currently owns each key, so a later occurrence can name the
    // repo it overrides.
    let mut key_source: HashMap<String, String> = HashMap::new();
    let mut conflicts: Vec<SchemeConflict> = Vec::new();
    let mut any_repo_present = false;

    for repo in refs {
        if !repo.path.is_dir() {
            continue;
        }
        any_repo_present = true;

        let repo_files = get_scheme_files_by_name(&repo.path, None)
            .with_context(|| format!("Failed to read schemes from {}", repo.path.display()))?;

        for (key, scheme_file) in repo_files {
            if let Some(shadowed_repo) = key_source.get(&key).cloned() {
                // Last-listed wins: this later repo overrides the one that held
                // the key. Record the shadowing so callers can surface it.
                conflicts.push(SchemeConflict {
                    key: key.clone(),
                    kept_repo: repo.name.clone(),
                    shadowed_repo,
                });
            }
            key_source.insert(key.clone(), repo.name.clone());
            files.insert(key, scheme_file);
        }
    }

    if !any_repo_present {
        return Err(anyhow!(
            "Schemes are missing, run install and then try again: `{REPO_NAME} install`",
        ));
    }

    // Stable, source-order-independent ordering for the user-facing notice.
    conflicts.sort_by(|a, b| {
        a.key
            .cmp(&b.key)
            .then_with(|| a.shadowed_repo.cmp(&b.shadowed_repo))
    });

    Ok(MergedSchemes { files, conflicts })
}

/// Convenience wrapper returning the merged scheme collection for `data_path`
/// and `config`. See [`collect_merged_schemes`].
pub fn merged_schemes(data_path: &Path, config: &Config) -> Result<MergedSchemes> {
    collect_merged_schemes(&scheme_repo_refs(data_path, config))
}

/// Formats a one-line warning for each shadowed scheme, or `None` when there are
/// no conflicts. Callers print this to stderr so scriptable stdout is unaffected.
#[must_use]
pub fn conflict_warning(conflicts: &[SchemeConflict]) -> Option<String> {
    if conflicts.is_empty() {
        return None;
    }
    let mut lines = vec![format!(
        "Note: {} duplicate scheme(s) across scheme repos; keeping the higher-precedence copy:",
        conflicts.len()
    )];
    for c in conflicts {
        lines.push(format!(
            "  \"{}\" from \"{}\" is used; the copy in \"{}\" is ignored",
            c.key, c.kept_repo, c.shadowed_repo
        ));
    }
    Some(lines.join("\n"))
}

/// One-time relocation of the built-in schemes repo from its pre-relocation home
/// (`<data>/repos/schemes`) to `<data>/scheme-repos/schemes`.
///
/// Runs at startup for every command. It is a no-op once migrated (or on a fresh
/// install with nothing at the old location), so the common case costs a couple
/// of `stat`s. When it does move something it returns a human-facing message so
/// the relocation is seamless but not invisible; callers print it to stderr.
///
/// The legacy slot may be a real clone *or* a symlink (a local-path schemes
/// source). `rename` moves either in place; a symlink keeps pointing at its
/// absolute target.
pub fn migrate_legacy_schemes_repo(data_path: &Path) -> Result<Option<String>> {
    let legacy = legacy_schemes_repo_path(data_path);
    let new = builtin_schemes_repo_path(data_path);

    // `symlink_metadata` does not follow a symlink, so a legacy symlink counts as
    // "present" and is migrated rather than dereferenced.
    if fs::symlink_metadata(&legacy).is_err() {
        return Ok(None); // Nothing at the old location.
    }
    if fs::symlink_metadata(&new).is_ok() {
        return Ok(None); // New location already occupied; leave both as-is.
    }

    ensure_directory_exists(scheme_repos_dir(data_path))?;
    fs::rename(&legacy, &new).with_context(|| {
        format!(
            "Failed to relocate the schemes repository from {} to {}",
            legacy.display(),
            new.display()
        )
    })?;

    Ok(Some(format!(
        "Moved the schemes repository to its new home:\n  {}  ->  {}\n\
         Scheme repositories now live under `{SCHEME_REPO_DIR}/`, separate from template items in `{REPO_DIR}/`.",
        legacy.display(),
        new.display()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(extras_toml: &str) -> Config {
        toml::from_str(extras_toml).expect("valid config")
    }

    #[test]
    fn refs_put_builtin_first_then_extras_in_order() {
        let config = cfg(concat!(
            "[[schemes.extras]]\nname = \"a\"\npath = \"https://example.com/a\"\n\n",
            "[[schemes.extras]]\nname = \"b\"\npath = \"https://example.com/b\"\n",
        ));
        let refs = scheme_repo_refs(Path::new("/data"), &config);
        let names: Vec<&str> = refs.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["schemes", "a", "b"]);
        assert_eq!(refs[1].path, Path::new("/data/scheme-repos/a"));
        assert_eq!(refs[0].path, Path::new("/data/scheme-repos/schemes"));
    }

    #[test]
    fn migration_is_noop_when_nothing_at_legacy_location() {
        let tmp = tempfile::tempdir().unwrap();
        let result = migrate_legacy_schemes_repo(tmp.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn migration_moves_legacy_clone_to_new_location() {
        let tmp = tempfile::tempdir().unwrap();
        let legacy = legacy_schemes_repo_path(tmp.path());
        fs::create_dir_all(legacy.join("base16")).unwrap();
        fs::write(legacy.join("base16").join("x.yaml"), "system: base16\n").unwrap();

        let msg = migrate_legacy_schemes_repo(tmp.path()).unwrap();
        assert!(msg.is_some(), "expected a migration message");

        let new = builtin_schemes_repo_path(tmp.path());
        assert!(new.join("base16").join("x.yaml").exists());
        assert!(!legacy.exists(), "legacy location should be gone");
    }

    #[test]
    fn migration_leaves_new_location_untouched_when_already_present() {
        let tmp = tempfile::tempdir().unwrap();
        let legacy = legacy_schemes_repo_path(tmp.path());
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join("legacy-marker"), "old").unwrap();
        let new = builtin_schemes_repo_path(tmp.path());
        fs::create_dir_all(&new).unwrap();
        fs::write(new.join("new-marker"), "new").unwrap();

        let msg = migrate_legacy_schemes_repo(tmp.path()).unwrap();
        assert!(msg.is_none(), "must not migrate when new location exists");
        assert!(legacy.join("legacy-marker").exists());
        assert!(new.join("new-marker").exists());
    }

    #[test]
    fn merge_prefers_extra_over_builtin_on_duplicate() {
        let tmp = tempfile::tempdir().unwrap();
        let builtin = builtin_schemes_repo_path(tmp.path());
        let extra = extra_repo_path(tmp.path(), "extra");
        for dir in [&builtin, &extra] {
            fs::create_dir_all(dir.join("base16")).unwrap();
        }
        let yaml = "system: base16\nname: Dup\nslug: dup\nauthor: t\nvariant: dark\npalette:\n  base00: '#000000'\n  base01: '#111111'\n  base02: '#222222'\n  base03: '#333333'\n  base04: '#444444'\n  base05: '#555555'\n  base06: '#666666'\n  base07: '#777777'\n  base08: '#888888'\n  base09: '#999999'\n  base0A: '#aaaaaa'\n  base0B: '#bbbbbb'\n  base0C: '#cccccc'\n  base0D: '#dddddd'\n  base0E: '#eeeeee'\n  base0F: '#ffffff'\n";
        fs::write(builtin.join("base16").join("dup.yaml"), yaml).unwrap();
        fs::write(extra.join("base16").join("dup.yaml"), yaml).unwrap();

        let refs = vec![
            SchemeRepoRef {
                name: "schemes".to_string(),
                path: builtin,
            },
            SchemeRepoRef {
                name: "extra".to_string(),
                path: extra,
            },
        ];
        let merged = collect_merged_schemes(&refs).unwrap();
        assert!(merged.files.contains_key("base16-dup"));
        assert_eq!(merged.conflicts.len(), 1);
        // The extra is listed after the built-in, so it wins (last-listed wins).
        assert_eq!(merged.conflicts[0].kept_repo, "extra");
        assert_eq!(merged.conflicts[0].shadowed_repo, "schemes");
    }

    #[test]
    fn merge_errors_when_no_repo_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let refs = scheme_repo_refs(tmp.path(), &cfg(""));
        let err = collect_merged_schemes(&refs).map(|_| ()).unwrap_err();
        assert!(err.to_string().contains("run install"));
    }
}
