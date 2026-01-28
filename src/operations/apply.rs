use crate::config::Config;
use crate::constants::{
    ARTIFACTS_DIR, CURRENT_SCHEME_FILE_NAME, CUSTOM_SCHEMES_DIR_NAME, DEFAULT_SCHEME_SYSTEM,
    LOCK_FILE, REPO_DIR, REPO_NAME, REPO_URL, SCHEMES_REPO_NAME,
};
use crate::utils::{
    create_theme_filename_without_extension, get_all_scheme_file_paths,
    get_shell_command_from_string, write_to_file,
};
use anyhow::{anyhow, Context, Error, Result};
use fs2::FileExt;
use regex::{self, Regex};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::str::FromStr;
use std::{fs, io};
use tinted_builder::SchemeSystem;
use tinted_builder_rust::operation_build::build;
use tinted_builder_rust::operation_build::utils::SchemeFile;

use super::list::SchemeEntry;

fn str_matches_scheme_system(value: &str) -> bool {
    match value {
        _ if value == SchemeSystem::Base16.as_str() => true,
        _ if value == SchemeSystem::Base24.as_str() => true,
        _ => false,
    }
}

/// Apply theme
///
/// For each of the provided config items, copy the theme to the `data_dir` based on the provided
/// `scheme_name`
#[allow(clippy::too_many_lines)]
pub fn apply(
    config_path: &Path,
    data_path: &Path,
    full_scheme_name: &str,
    is_quiet: bool,
    active_operation: Option<&str>,
) -> Result<()> {
    let scheme_name_arr: Vec<String> = full_scheme_name
        .split('-')
        .map(ToString::to_string)
        .collect();
    let scheme_system_option = scheme_name_arr.first().map(String::to_string);

    // Check provided scheme exists
    if scheme_name_arr.len() < 2 {
        return Err(anyhow!(
            "Invalid scheme name. Make sure the scheme system is prefixed <SCHEME_SYSTEM>-<SCHEME_NAME>, eg: `{DEFAULT_SCHEME_SYSTEM}-ayu-dark`",
        ));
    }

    // Check provided scheme is valid
    if !str_matches_scheme_system(scheme_system_option.clone().unwrap_or_default().as_str()) {
        return Err(anyhow!(
            "Invalid scheme name. Make sure your scheme is prefixed with a supprted system (\"{}\" or \"{}\"), eg: {}-{}",
            SchemeSystem::Base16.as_str(),
            SchemeSystem::Base24.as_str(),
            DEFAULT_SCHEME_SYSTEM,
            full_scheme_name
        ));
    }

    // Create a temporary data directory
    let staging_data_dir = tempfile::Builder::new()
        .prefix(format!("{ARTIFACTS_DIR}-").as_str())
        .tempdir_in(data_path)?;
    let staging_data_path = staging_data_dir.path();

    // Go through custom schemes
    let scheme_system =
        SchemeSystem::from_str(&scheme_system_option.unwrap_or_else(|| "base16".to_string()))?;
    let schemes_path = &data_path.join(format!("{REPO_DIR}/{SCHEMES_REPO_NAME}"));
    let custom_schemes_path = &data_path.join(CUSTOM_SCHEMES_DIR_NAME);
    let builtin_scheme_files = get_all_scheme_file_paths(schemes_path, None)?;
    let custom_scheme_files = get_all_scheme_file_paths(custom_schemes_path, None).ok();
    let config = Config::read(config_path)?;
    let builtin_scheme = builtin_scheme_files.get(full_scheme_name);
    let custom_scheme = custom_scheme_files
        .as_ref()
        .and_then(|m| m.get(full_scheme_name));

    let Some(scheme_file) = builtin_scheme.xor(custom_scheme) else {
        // We expect the scheme to be a built-in scheme or a custom schemes, not both.
        if builtin_scheme.is_none() {
            return Err(anyhow!("Scheme does not exist: {full_scheme_name}"));
        }

        if let Some(scheme_partial_arr) = scheme_name_arr.get(1..) {
            let scheme_partial_name = scheme_partial_arr.join("-");

            return Err(anyhow!(
                "You have a Tinty generated scheme named the same as an official tinted-theming/schemes name, please rename or remove it: {}/{scheme_partial_name}.yaml",
                 custom_schemes_path.display(),
            ));
        }

        return Err(anyhow!(
            "You have a Tinty generated scheme named the same as an official tinted-theming/schemes name, please rename or remove it",
        ));
    };

    if custom_scheme.is_some() {
        build_and_get_custom_scheme_file(custom_schemes_path, data_path, &config)?;
    }

    write_to_file(
        staging_data_path.join(CURRENT_SCHEME_FILE_NAME),
        full_scheme_name,
    )?;

    let system_items = config
        .items
        .map(|f| {
            f.into_iter()
                .filter(|f| {
                    f.supported_systems
                        .clone()
                        .map(|s| s.contains(&scheme_system))
                        .is_some()
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut hook_commands: Vec<Hook> = Vec::new();

    // Run through provided items in config.toml
    for item in system_items {
        let repo_path = data_path.join(REPO_DIR).join(&item.name);
        let themes_path = repo_path.join(&item.themes_dir);

        if !themes_path.exists() {
            return Err(anyhow!(format!(
                "Provided theme path for {} does not exist: {}\nTry running `{REPO_NAME} install` or `{REPO_NAME} update` or check your config.toml file and try again.",
                item.name,
                themes_path.display(),
            )));
        }

        // Find the corresponding theme file for the provided item
        let theme_dir = fs::read_dir(&themes_path)
            .map_err(Error::new)
            .with_context(|| format!("Themes are missing from {}, try running `{REPO_NAME} install` or `{REPO_NAME} update` and try again.", item.name))?;
        let theme_option = &theme_dir.filter_map(Result::ok).find(|entry| {
            let path = entry.path();
            item.theme_file_extension.as_ref().map_or_else(
                || {
                    let filename = path.file_stem().and_then(|name| name.to_str());
                    full_scheme_name == filename.unwrap_or_default()
                },
                |extension| {
                    let filename = path.file_name().and_then(|name| name.to_str());
                    format!("{full_scheme_name}{extension}") == filename.unwrap_or_default()
                },
            )
        });

        // Copy that theme to the data_path or log a message that it isn't found
        match theme_option {
            Some(theme_file) => {
                let theme_file_path = &theme_file.path();
                let extension = theme_file_path.extension().map_or_else(String::new, |ext| {
                    format!(".{}", ext.to_str().unwrap_or_default())
                });
                let filename = format!(
                    "{}{extension}",
                    create_theme_filename_without_extension(&item),
                );
                let data_theme_path = staging_data_path.join(&filename);
                let theme_content = fs::read_to_string(theme_file.path())?;

                write_to_file(&data_theme_path, theme_content.as_str())?;

                // Gather the hook commands, we will run them after we've committed all items onto
                // the final artifacts directory.
                if let Some(hook_text) = &item.hook {
                    let hook_parts = Hook {
                        name: item.name.clone(),
                        command_template: hook_text.clone(),
                        operation: active_operation.unwrap_or("apply").to_string(),
                        relative_file_path: PathBuf::from(filename),
                    };
                    hook_commands.push(hook_parts);
                }

                // Run config.items.write_to_file
                if let Some(write_to_file_vec) = item.write_to_file {
                    match write_to_file_vec.as_slice() {
                        [target_filepath, start_marker, end_marker] => {
                            let expanded_filepath = expand_tilde(target_filepath);
                            let target_content = read_to_string(&expanded_filepath)?;
                            let rendered_content = generate_file_contents(
                                &theme_content,
                                &target_content,
                                Some(start_marker),
                                Some(end_marker),
                            )?;

                            write_to_file(&expanded_filepath, &rendered_content)?;

                            Ok(())
                        }
                        [target_filepath, start_marker] => {
                            let expanded_filepath = expand_tilde(target_filepath);
                            let target_content = read_to_string(&expanded_filepath)?;
                            let rendered_content = generate_file_contents(
                                &theme_content,
                                &target_content,
                                Some(start_marker),
                                None,
                            )?;

                            write_to_file(&expanded_filepath, &rendered_content)?;

                            Ok(())
                        }
                        [target_filepath] => {
                            let expanded_filepath = expand_tilde(target_filepath);

                            write_to_file(&expanded_filepath, &theme_content)?;

                            Ok(())
                        }
                        _ => Err(anyhow!(
                            "tinty.toml requires has invalid values in `write_to_file` property"
                        )),
                    }?;
                }
            }
            None => {
                if !is_quiet {
                    println!(
                        "Theme does not exists for {} in {}. Try running `{REPO_NAME} update` or submit an issue on {REPO_URL}",
                        item.name, themes_path.display(),
                    );
                }
            }
        }
    }

    let lock_path = data_path.join(LOCK_FILE);
    let lock_file = fs::File::create(&lock_path).context(format!(
        "Failed to create lock file: {}",
        lock_path.display()
    ))?;
    lock_file.lock_exclusive().context(format!(
        "Failed to acquire exclusive lock on {}",
        lock_path.display()
    ))?;

    let target_path = data_path.join(ARTIFACTS_DIR);
    if target_path.exists() {
        // Replace the existing artifacts directory with the staging one.
        fs::remove_dir_all(&target_path)?;
    }
    fs::rename(staging_data_path, &target_path)?;
    std::mem::forget(staging_data_dir);

    for hook in hook_commands {
        hook.run_command(&target_path, config_path, full_scheme_name, scheme_file)?;
    }

    create_symlinks_for_backwards_compat(&target_path, data_path)?;

    // Run global tinty/config.toml hooks
    if let Some(hooks_vec) = config.hooks {
        for hook in &hooks_vec {
            let hook_command_vec = get_shell_command_from_string(config_path, hook.as_str())?;
            let Some(command) = hook_command_vec.first() else {
                return Err(anyhow!("Unable to extract cli command"));
            };
            let Some(args) = hook_command_vec.get(1..) else {
                return Err(anyhow!("Unable to extract cli args"));
            };
            Command::new(command)
                .args(args)
                .envs(SchemeEntry::from_scheme(&scheme_file.get_scheme()?).to_envs())
                .status()
                .with_context(|| format!("Failed to execute global hook: {hook}"))?;
        }
    }

    Ok(())
}

fn build_and_get_custom_scheme_file(
    custom_schemes_path: &Path,
    data_path: &Path,
    config: &Config,
) -> Result<()> {
    if let Some(items) = &config.items {
        let item_name_vec: Vec<String> = items.iter().map(|p| p.name.clone()).collect();
        for item_name in item_name_vec {
            let item_template_path: PathBuf = data_path.join(format!("{REPO_DIR}/{item_name}"));
            build(&item_template_path, custom_schemes_path, true)?;
        }
    }

    Ok(())
}

fn create_symlinks_for_backwards_compat(source_path: &PathBuf, target_path: &Path) -> Result<()> {
    for entry in fs::read_dir(source_path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            let file_name = entry.file_name();
            let src_file = entry.path();
            let dst_file = target_path.join(file_name);
            // Delete existing destination file or symlink if it exists
            if dst_file.exists() {
                fs::remove_file(&dst_file)?;
            }
            symlink_any(&src_file, &dst_file)?;
        }
    }
    delete_non_dirs_and_broken_symlinks(target_path)?;

    Ok(())
}

struct Hook {
    name: String,
    command_template: String,
    operation: String,
    relative_file_path: PathBuf,
}

impl Hook {
    fn run_command(
        &self,
        artifacts_path: &Path,
        config_path: &Path,
        full_scheme_name: &str,
        scheme_file: &SchemeFile,
    ) -> Result<Child, Error> {
        let theme_file_path = artifacts_path
            .join(self.relative_file_path.clone())
            .display()
            .to_string();
        let hook_script = self
            .command_template
            .replace("%o", self.operation.as_str())
            .replace("%f", theme_file_path.as_str())
            .replace("%n", full_scheme_name);
        let command_vec = get_shell_command_from_string(config_path, hook_script.as_str())?;
        let Some(command) = command_vec.first() else {
            return Err(anyhow!("Unable to extract cli command"));
        };
        let Some(args) = command_vec.get(1..) else {
            return Err(anyhow!("Unable to extract cli args"));
        };
        Command::new(command)
            .args(args)
            .env("TINTY_THEME_FILE_PATH", theme_file_path)
            .env("TINTY_THEME_OPERATION", self.operation.as_str())
            .envs(SchemeEntry::from_scheme(&scheme_file.get_scheme()?).to_envs())
            .spawn()
            .with_context(|| {
                format!(
                    "Failed to execute {} hook: {}",
                    self.name, self.command_template
                )
            })
    }
}

fn symlink_any(src: &Path, dst: &Path) -> Result<(), Error> {
    std::os::unix::fs::symlink(src, dst)?;
    Ok(())
}

fn delete_non_dirs_and_broken_symlinks(dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(name) = path.file_name() {
            if name == LOCK_FILE {
                continue;
            }
        }

        let metadata = fs::symlink_metadata(&path)?; // Don't follow symlinks

        let file_type = metadata.file_type();

        if file_type.is_dir() {
            continue;
        }

        if file_type.is_symlink() {
            // Try to follow the symlink
            if fs::metadata(&path).is_err_and(|e| e.kind() == io::ErrorKind::NotFound) {
                fs::remove_file(&path)?;
            }
        } else {
            // Regular file
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

/// Replaces content between markers in a target string with new content.
///
/// This function finds sections delimited by `start_marker` and `end_marker` (or from
/// `start_marker` to end of string if no end marker) and replaces the content between
/// them with `insertion_content`. The markers themselves are preserved in the output.
///
/// # Arguments
/// * `insertion_content` - The new content to insert between the markers
/// * `target_content` - The original string containing the markers
/// * `start_marker` - The marker indicating the start of the replaceable section (required)
/// * `end_marker` - The marker indicating the end of the replaceable section (optional)
///
/// # Returns
/// * `Ok(String)` - The target content with the marked sections replaced
/// * `Err` - If `start_marker` is `None`
///
/// # Behavior
/// - With both markers: replaces all occurrences of `start_marker...end_marker` (non-greedy)
/// - With start marker only: replaces from `start_marker` to end of string
/// - If markers are not found in target, returns target unchanged
fn generate_file_contents(
    insertion_content: &str,
    source_content: &str,
    start_marker: Option<&str>,
    end_marker: Option<&str>,
) -> Result<String> {
    match (start_marker, end_marker) {
        (Some(start_marker), Some(end_marker)) => {
            let re_start = regex::escape(start_marker);
            let re_end = regex::escape(end_marker);
            let re = Regex::new(&format!(r"(?s){re_start}.*?{re_end}"))?;

            Ok(re
                .replace_all(
                    source_content,
                    format!("{start_marker}{insertion_content}{end_marker}"),
                )
                .to_string())
        }
        (Some(start_marker), None) => {
            let re_start = regex::escape(start_marker);
            let re = Regex::new(&format!(r"(?s){re_start}.*$"))?;

            Ok(re
                .replace_all(source_content, format!("{start_marker}{insertion_content}"))
                .to_string())
        }
        _ => Err(anyhow!("Unable to get file contents")),
    }
}

fn expand_tilde(path: impl AsRef<Path>) -> PathBuf {
    if let Ok(stripped) = path.as_ref().strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }

    PathBuf::from(path.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_file_contents_with_start_and_end_markers() {
        let insertion = "new-content";
        let source_content = "before\n<!-- START -->old<!-- END -->\nafter";
        let result = generate_file_contents(
            insertion,
            source_content,
            Some("<!-- START -->"),
            Some("<!-- END -->"),
        )
        .unwrap();

        assert_eq!(
            result,
            "before\n<!-- START -->new-content<!-- END -->\nafter"
        );
    }

    #[test]
    fn generate_file_contents_with_start_marker_only() {
        let insertion = "new-content";
        let source_content = "before\n# START\nold trailing stuff";
        let result =
            generate_file_contents(insertion, source_content, Some("# START\n"), None).unwrap();

        assert_eq!(result, "before\n# START\nnew-content");
    }

    #[test]
    fn generate_file_contents_with_multiline_replacement() {
        let insertion = "line1\nline2\nline3";
        let source_content = "header\n<!-- START -->\nold\nmulti\nline\n<!-- END -->\nfooter";
        let result = generate_file_contents(
            insertion,
            source_content,
            Some("<!-- START -->"),
            Some("<!-- END -->"),
        )
        .unwrap();

        assert_eq!(
            result,
            "header\n<!-- START -->line1\nline2\nline3<!-- END -->\nfooter"
        );
    }

    #[test]
    fn generate_file_contents_with_multiple_marker_pairs() {
        let insertion = "X";
        let source_content = "a<!-- S -->1<!-- E -->b<!-- S -->2<!-- E -->c";
        let result = generate_file_contents(
            insertion,
            source_content,
            Some("<!-- S -->"),
            Some("<!-- E -->"),
        )
        .unwrap();

        assert_eq!(result, "a<!-- S -->X<!-- E -->b<!-- S -->X<!-- E -->c");
    }

    #[test]
    fn generate_file_contents_with_special_regex_chars_in_markers() {
        let insertion = "content";
        let source_content = "before[START]old[END]after";
        let result =
            generate_file_contents(insertion, source_content, Some("[START]"), Some("[END]"))
                .unwrap();

        assert_eq!(result, "before[START]content[END]after");
    }

    #[test]
    fn generate_file_contents_no_markers_returns_error() {
        let result = generate_file_contents("content", "source_content", None, None);

        assert!(result.is_err());
    }

    #[test]
    fn generate_file_contents_end_marker_only_returns_error() {
        let result =
            generate_file_contents("content", "source_content", None, Some("<!-- END -->"));

        assert!(result.is_err());
    }

    #[test]
    fn generate_file_contents_markers_not_found_leaves_unchanged() {
        let insertion = "new";
        let source_content = "no markers here";
        let result = generate_file_contents(
            insertion,
            source_content,
            Some("<!-- START -->"),
            Some("<!-- END -->"),
        )
        .unwrap();

        assert_eq!(result, "no markers here");
    }

    #[test]
    fn generate_file_contents_empty_insertion() {
        let insertion = "";
        let source_content = "before<!-- S -->old<!-- E -->after";
        let result = generate_file_contents(
            insertion,
            source_content,
            Some("<!-- S -->"),
            Some("<!-- E -->"),
        )
        .unwrap();

        assert_eq!(result, "before<!-- S --><!-- E -->after");
    }

    #[test]
    fn generate_file_contents_insertion_contains_markers() {
        let insertion = "theme with <!-- START --> and <!-- END --> inside";
        let source_content = "before\n<!-- START -->old<!-- END -->\nafter";
        let result = generate_file_contents(
            insertion,
            source_content,
            Some("<!-- START -->"),
            Some("<!-- END -->"),
        )
        .unwrap();

        assert_eq!(
            result,
            "before\n<!-- START -->theme with <!-- START --> and <!-- END --> inside<!-- END -->\nafter"
        );
    }

    #[test]
    fn generate_file_contents_reapply_with_markers_in_content() {
        let insertion = "new-theme";
        let source_content =
            "before\n<!-- START -->theme with <!-- START --> and <!-- END --> inside<!-- END -->\nafter";
        let result = generate_file_contents(
            insertion,
            source_content,
            Some("<!-- START -->"),
            Some("<!-- END -->"),
        )
        .unwrap();

        assert_eq!(
            result,
            "before\n<!-- START -->new-theme<!-- END --> inside<!-- END -->\nafter"
        );
    }

    #[test]
    fn generate_file_contents_whitespace_around_markers() {
        let insertion = "content";
        let source_content = "before  <!-- START -->  old  <!-- END -->  after";
        let result = generate_file_contents(
            insertion,
            source_content,
            Some("<!-- START -->"),
            Some("<!-- END -->"),
        )
        .unwrap();

        assert_eq!(result, "before  <!-- START -->content<!-- END -->  after");
    }

    #[test]
    fn generate_file_contents_newlines_adjacent_to_markers() {
        let insertion = "content";
        let source_content = "before\n<!-- START -->\nold\n<!-- END -->\nafter";
        let result = generate_file_contents(
            insertion,
            source_content,
            Some("<!-- START -->"),
            Some("<!-- END -->"),
        )
        .unwrap();

        assert_eq!(result, "before\n<!-- START -->content<!-- END -->\nafter");
    }

    #[test]
    fn generate_file_contents_preserves_newlines_in_insertion() {
        let insertion = "\nline1\nline2\n";
        let source_content = "before<!-- S -->old<!-- E -->after";
        let result = generate_file_contents(
            insertion,
            source_content,
            Some("<!-- S -->"),
            Some("<!-- E -->"),
        )
        .unwrap();

        assert_eq!(result, "before<!-- S -->\nline1\nline2\n<!-- E -->after");
    }
}
