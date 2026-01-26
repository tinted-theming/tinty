#![allow(clippy::arithmetic_side_effects)]
use crate::config::{Config, ConfigItem, DEFAULT_CONFIG_SHELL};
use crate::constants::REPO_NAME;
use anyhow::{anyhow, Context, Error, Result};
use home::home_dir;
use rand::Rng;
use regex::bytes::Regex;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str;
use tinted_builder::SchemeSystem;
use tinted_builder_rust::operation_build::utils::SchemeFile;

/// Ensures that a directory exists, creating it if it does not.
pub fn ensure_directory_exists<P: AsRef<Path>>(dir_path: P) -> Result<()> {
    let path = dir_path.as_ref();

    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory at {}", path.display()))?;
    }

    Ok(())
}

pub fn write_to_file(path: impl AsRef<Path>, contents: &str) -> Result<()> {
    let mut file = File::create(path.as_ref())
        .map_err(anyhow::Error::new)
        .with_context(|| format!("Unable to create file: {}", path.as_ref().display()))?;

    file.write_all(contents.as_bytes())?;

    Ok(())
}

pub fn get_shell_command_from_string(config_path: &Path, command: &str) -> Result<Vec<String>> {
    let config = Config::read(config_path)?;
    let shell = config
        .shell
        .unwrap_or_else(|| DEFAULT_CONFIG_SHELL.to_string());
    let full_command = shell.replace("{}", command);

    if shell.contains("{}") {
        shell_words::split(&full_command).map_err(anyhow::Error::new)
    } else {
        // This error is handled earlier so should never get here
        Err(anyhow!(
            "The configured shell property does not contain the required command placeholder '{{}}'"
        ))
    }
}

pub fn git_clone(repo_url: &str, target_dir: &Path, revision: Option<&str>) -> Result<()> {
    if target_dir.exists() {
        return Err(anyhow!(
            "Error cloning {}. Target directory '{}' already exists",
            repo_url,
            target_dir.display()
        ));
    }

    let git_command = format!("git clone \"{repo_url}\" \"{}\"", target_dir.display());
    let command_vec = shell_words::split(git_command.as_str()).map_err(anyhow::Error::new)?;

    let Some(command) = command_vec.first() else {
        return Err(anyhow!("Unable to extract cli command"));
    };
    let Some(args) = command_vec.get(1..) else {
        return Err(anyhow!("Unable to extract cli args"));
    };

    Command::new(command)
        .args(args)
        .stdout(Stdio::null())
        .status()
        .with_context(|| format!("Failed to clone repository from {repo_url}"))?;

    if let Some(revision_str) = revision {
        let result = git_to_revision(target_dir, "origin", revision_str);
        if let Err(e) = result {
            // Cleanup! If we cannot checkout the revision, remove the directory.
            fs::remove_dir_all(target_dir)
                .with_context(|| format!("Failed to remove directory {}", target_dir.display()))?;
            return Err(e);
        }
    }

    Ok(())
}

pub fn git_update(repo_path: &Path, repo_url: &str, revision: Option<&str>) -> Result<()> {
    if !repo_path.is_dir() {
        return Err(anyhow!(
            "Error with updating. {} is not a directory",
            repo_path.display()
        ));
    }

    // To make this operation atomic, we'll satisfy the remote & revision in this sequence:
    // 1.) add the remote URL as a new temporary remote.
    // 2.) check if the revision exists in the temporary remote.
    // 3.) checkout the revision from temporary remote
    // 4.) On success:
    //      4.1) replace the origin remote URL
    //      4.2) remove the temporary remote
    // 5.) On error, remove temporary remote
    //
    // Note that this sequence works even if the directory is already on that remote & revision.
    //
    let tmp_remote_name = random_remote_name();

    // Create a temporary remote
    safe_command(
        format!("git remote add \"{tmp_remote_name}\" \"{repo_url}\"").as_str(),
        repo_path,
    )?
    .current_dir(repo_path)
    .stdout(Stdio::null())
    .status()
    .with_context(|| {
        format!(
            "Error with adding {} as a remote named {} in {}",
            repo_url,
            tmp_remote_name,
            repo_path.display()
        )
    })?;

    let revision_str = revision.unwrap_or("main");
    let res = git_to_revision(repo_path, &tmp_remote_name, revision_str);

    if let Err(e) = res {
        // Failed to switch to the desired revision. Cleanup!
        safe_command(
            format!("git remote rm \"{tmp_remote_name}\"").as_str(),
            repo_path,
        )?
        .stdout(Stdio::null())
        .status()
        .with_context(|| {
            format!(
                "Failed to remove temporary remote {} in {}",
                tmp_remote_name,
                repo_path.display()
            )
        })?;
        return Err(e);
    }

    safe_command(
        format!("git remote set-url origin \"{repo_url}\"").as_str(),
        repo_path,
    )?
    .stdout(Stdio::null())
    .status()
    .with_context(|| {
        format!(
            "Failed to set origin remote to {repo_url} in {}",
            repo_path.display()
        )
    })?;
    safe_command(
        format!("git remote rm \"{tmp_remote_name}\"").as_str(),
        repo_path,
    )?
    .stdout(Stdio::null())
    .status()
    .with_context(|| {
        format!(
            "Failed to remove temporary remote {tmp_remote_name} in {}",
            repo_path.display()
        )
    })?;

    Ok(())
}

fn random_remote_name() -> String {
    let mut rng = rand::thread_rng();
    let random_number: u32 = rng.gen();

    format!("tinty-remote-{random_number}")
}

// Resolvees the SHA1 of revision at remote_name.
// revision can be a tag, a branch, or a commit SHA1.
#[allow(clippy::too_many_lines)]
fn git_resolve_revision(repo_path: &Path, remote_name: &str, revision: &str) -> Result<String> {
    // 1.) Check if its a tag.
    let expected_tag_ref = format!("refs/tags/{revision}");
    let mut command = safe_command(
        format!("git ls-remote --quiet --tags \"{remote_name}\" \"{expected_tag_ref}\"").as_str(),
        repo_path,
    )?;
    let mut child = command
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| "Failed to spawn".to_string())?;

    let Some(stdout) = child.stdout.take() else {
        return Err(anyhow!("failed to capture stdout"));
    };
    let reader = BufReader::new(stdout);

    if let Some(parts) = reader
        .lines()
        .map_while(Result::ok)
        .map(|line| line.split('\t').map(String::from).collect::<Vec<String>>())
        .filter(|parts| parts.len() == 2)
        .find(|parts| {
            parts
                .get(1)
                .map_or_else(|| false, |second_part| *second_part == expected_tag_ref)
        })
    {
        if let Some(first_part) = parts.first() {
            // we found a tag that matches
            child.kill()?; // Abort the child process.
            child.wait()?; // Cleanup
            return Ok(first_part.clone()); // Return early.
        }
    }

    child
        .wait()
        .with_context(|| format!("Failed to list remote tags from {remote_name}"))?;

    // 2.) Check if its a branch
    let expected_branch_ref = format!("refs/heads/{revision}");
    let mut command = safe_command(
        format!("git ls-remote --quiet \"{remote_name}\" \"{expected_branch_ref}\"").as_str(),
        repo_path,
    )?;
    let mut child = command
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| "Failed to spawn".to_string())?;

    let Some(stdout) = child.stdout.take() else {
        return Err(anyhow!("failed to capture stdout"));
    };
    let reader = BufReader::new(stdout);

    if let Some(parts) = reader
        .lines()
        .map_while(Result::ok)
        .map(|line| line.split('\t').map(String::from).collect::<Vec<String>>())
        .filter(|parts| parts.len() == 2)
        .find(|parts| {
            parts
                .get(1)
                .map_or_else(|| false, |second_part| *second_part == expected_branch_ref)
        })
    {
        if let Some(first_part) = parts.first() {
            // we found a branch that matches.
            child.kill()?; // Abort the child process.
            child.wait()?; // Cleanup
            return Ok(first_part.clone()); // Return early.
        }
    }

    child
        .wait()
        .with_context(|| format!("Failed to list branches tags from {remote_name}"))?;

    // We are here because revision isn't a tag or a branch.
    // First, we'll check if revision itself *could* be a SHA1.
    // If it doesn't look like one, we'll return early.
    let pattern = r"^[0-9a-f]{1,40}$";
    let Ok(re) = Regex::new(pattern) else {
        return Err(anyhow!("Invalid regex"));
    };
    if !re.is_match(revision.as_bytes()) {
        return Err(anyhow!("cannot resolve {revision} into a Git SHA1",));
    }

    safe_command(
        format!("git fetch --quiet \"{remote_name}\"").as_str(),
        repo_path,
    )?
    .stdout(Stdio::null())
    .status()
    .with_context(|| format!("unable to fetch objects from remote {remote_name}"))?;

    // 3.) Check if any branch in remote contains the SHA1:
    // It seems that the only way to do this is to list the branches that contain the SHA1
    // and check if it belongs in the remote.
    let remote_branch_prefix = format!("refs/remotes/{remote_name}/");
    let mut command = safe_command(
        format!("git branch --format=\"%(refname)\" -a --contains \"{revision}\"").as_str(),
        repo_path,
    )?;
    let mut child = command.stdout(Stdio::piped()).spawn().with_context(|| {
        format!("Failed to find branches containing commit {revision} from {remote_name}",)
    })?;
    let Some(stdout) = child.stdout.take() else {
        return Err(anyhow!("failed to capture stdout"));
    };
    let reader = BufReader::new(stdout);

    if reader
        .lines()
        .map_while(Result::ok)
        .any(|line| line.starts_with(&remote_branch_prefix))
    {
        // we found a remote ref that contains the commit sha
        child.kill()?; // Abort the child process.
        child.wait()?; // Cleanup
        return Ok(revision.to_string()); // Return early.
    }

    child.wait().with_context(|| {
        format!("Failed to list branches from {remote_name} containing SHA1 {revision}",)
    })?;

    Err(anyhow!(
        "cannot find revision {revision} in remote {remote_name}",
    ))
}

fn safe_command(command_str: &str, cwd: &Path) -> Result<Command, Error> {
    let command_vec = shell_words::split(command_str).map_err(anyhow::Error::new)?;
    let Some(command) = command_vec.first() else {
        return Err(anyhow!("Unable to extract cli command"));
    };
    let Some(args) = command_vec.get(1..) else {
        return Err(anyhow!("Unable to extract cli args"));
    };
    let mut command = Command::new(command);

    command.args(args).current_dir(cwd);
    Ok(command)
}

fn git_to_revision(repo_path: &Path, remote_name: &str, revision: &str) -> Result<()> {
    // Download the object from the remote
    safe_command(
        format!("git fetch --quiet \"{remote_name}\" \"{revision}\"").as_str(),
        repo_path,
    )?
    .status()
    .with_context(|| {
        format!(
            "Error with fetching revision {revision} in {}",
            repo_path.display()
        )
    })?;

    // Normalize the revision into the SHA.
    let commit_sha = git_resolve_revision(repo_path, remote_name, revision)?;

    safe_command(
        format!("git -c advice.detachedHead=false checkout --quiet \"{commit_sha}\"").as_str(),
        repo_path,
    )?
    .stdout(Stdio::null())
    .current_dir(repo_path)
    .status()
    .with_context(|| {
        format!(
            "Failed to checkout SHA {commit_sha} in {}",
            repo_path.display()
        )
    })?;

    Ok(())
}

pub fn git_is_working_dir_clean(target_dir: &Path) -> Result<bool> {
    // We use the Git plumbing `status --porcelain` command to tell us of files that has changed,
    // both staged and unstaged.
    let output = safe_command("git status --porcelain", target_dir)?
        .output()
        .with_context(|| format!("Failed to execute process in {}", target_dir.display()))?;

    // With the --quiet flag, it will return a 0 exit-code if no files has changed.
    Ok(output.stdout.is_empty())
}

pub fn create_theme_filename_without_extension(item: &ConfigItem) -> String {
    format!(
        "{}-{}-file",
        item.name.clone(),
        item.themes_dir.clone().replace('/', "-"), // Flatten path/to/dir to path-to-dir
    )
}

pub fn get_all_scheme_names(
    schemes_path: &Path,
    scheme_systems_option: Option<SchemeSystem>,
) -> Result<Vec<String>> {
    let file_paths = get_all_scheme_file_paths(schemes_path, scheme_systems_option)?;
    let mut scheme_vec: Vec<String> = file_paths.into_keys().collect();
    scheme_vec.sort();

    Ok(scheme_vec)
}

pub fn get_all_scheme_file_paths(
    schemes_path: &Path,
    scheme_systems_option: Option<SchemeSystem>,
) -> Result<HashMap<String, SchemeFile>> {
    if !schemes_path.exists() {
        return Err(anyhow!(
            "Schemes do not exist, run install and try again: `{REPO_NAME} install`",
        ));
    }

    let mut scheme_files: HashMap<String, SchemeFile> = HashMap::new();

    // For each supported scheme system, add schemes to vec
    let scheme_systems =
        scheme_systems_option.map_or_else(|| SchemeSystem::variants().to_vec(), |s| vec![s]);
    for scheme_system in scheme_systems {
        let scheme_system_dir = schemes_path.join(scheme_system.as_str());
        if !scheme_system_dir.exists() {
            continue;
        }

        let files = fs::read_dir(&scheme_system_dir)?
            // Discard failed read results
            .filter_map(Result::ok)
            .collect::<Vec<_>>()
            .into_iter()
            .filter_map(|file| {
                // Convert batch of files into a HashMap<String, SchemeFile>, where
                // the key is the scheme's <system>-<slug> e.g. base16-github
                // Map each entry into a (<String, SchemaFile) tuple that
                // we can collect() into this batch's HashMap<String, SchemaFile>
                let name = format!("{scheme_system}-{}", file.path().file_stem()?.to_str()?,);
                let scheme_file = SchemeFile::new(file.path().as_path()).ok()?;

                Some((name, scheme_file))
            })
            .collect::<HashMap<String, SchemeFile>>();
        scheme_files.extend(files);
    }
    Ok(scheme_files)
}

pub fn replace_tilde_slash_with_home(path_str: &str) -> Result<PathBuf> {
    let trimmed_path_str = path_str.trim();
    if trimmed_path_str.starts_with("~/") {
        home_dir().map_or_else(|| Err(anyhow!("Unable to determine a home directory for \"{trimmed_path_str}\", please use an absolute path instead")), |home_dir| Ok(PathBuf::from(trimmed_path_str.replacen(
                   "~/",
                   format!("{}/", home_dir.display()).as_str(),
                   1,
               ))))
    } else {
        Ok(PathBuf::from(trimmed_path_str))
    }
}

pub fn next_scheme_in_cycle(current: &String, schemes: &[String]) -> String {
    if schemes
        .iter()
        .position(|scheme| scheme == current)
        .unwrap_or(0)
        < usize::MAX
    {
        let next_index = schemes
            .iter()
            .position(|scheme| scheme == current)
            .map_or(0, |i| i + 1_usize);

        let next_item = schemes.get((next_index) % schemes.len());

        if let Some(next_item) = next_item {
            return next_item.clone();
        }

        current.clone()
    } else {
        schemes.first().cloned().unwrap_or_else(|| current.clone())
    }
}

pub fn user_curated_scheme_list(config: &Config) -> Option<Vec<String>> {
    // Return a list of preferred schemes based on presence of this value in the config, and
    // whatever the default scheme is if specified in config also.
    config
        .preferred_schemes
        .as_ref()
        .map(|preferred| {
            // If default scheme is defined, add it to the cycle.
            config
                .default_scheme
                .as_ref()
                .filter(|default| !preferred.contains(default))
                .map_or_else(
                    || preferred.clone(),
                    |default| {
                        let mut result = vec![default.clone()];
                        result.extend(preferred.clone());
                        result
                    },
                )
        })
        .or_else(|| {
            // If default scheme is defined, use it if preferred schemes is unset.
            config
                .default_scheme
                .as_ref()
                .map(|theme| vec![theme.clone()])
        })
}
