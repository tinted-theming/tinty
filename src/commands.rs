use crate::config::{
    get_embedded_colorscheme_dir, get_embedded_hooks_dir, BASE16_SHELL_CONFIG_PATH_ENV,
    BASE16_SHELL_THEME_NAME_PATH_ENV, BASE16_THEME_ENV,
};
use anyhow::{Context, Result};
use include_dir::Dir;
use std::fs::{self, File};
use std::io::Read;
use std::io::Write;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::from_utf8;
use tempfile::NamedTempFile;

/// Ensures that a directory exists, creating it if it does not.
fn ensure_directory_exists<P: AsRef<Path>>(dir_path: P) -> Result<()> {
    let path = dir_path.as_ref();

    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory at {:?}", path))?;
    }

    Ok(())
}

/// Ensures that a file exists, creating it if it does not.
fn ensure_file_exists<P: AsRef<Path>>(file_path: P) -> Result<()> {
    let path = file_path.as_ref();

    if !path.exists() {
        File::create(path).with_context(|| format!("Failed to create file at {:?}", path))?;
    };

    Ok(())
}

/// Ensures the existence of configuration files required by the application.
pub fn ensure_config_files_exist(
    base16_config_path: &Path,
    base16_shell_theme_name_path: &Path,
) -> Result<()> {
    ensure_directory_exists(base16_config_path).with_context(|| {
        format!(
            "Failed to create config directory at {:?}",
            base16_config_path
        )
    })?;
    ensure_file_exists(base16_shell_theme_name_path).with_context(|| {
        format!(
            "Failed to create config file at {:?}",
            base16_shell_theme_name_path
        )
    })?;

    Ok(())
}

/// Reads the contents of a file and returns it as a string.
fn read_file_to_string(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    Ok(contents)
}

fn write_to_file(path: &Path, contents: &str) -> Result<()> {
    if path.exists() {
        fs::remove_file(&path)?;
    }

    let mut file = File::create(&path)?;
    file.write_all(contents.as_bytes())?;

    Ok(())
}

fn set_colorscheme_provided(
    theme_name: &String,
    base16_shell_repo_path: &PathBuf,
    base16_shell_colorscheme_path: &Path,
    base16_shell_theme_name_path: &Path,
) -> Result<()> {
    let theme_script_path =
        base16_shell_repo_path.join(format!("scripts/base16-{}.sh", theme_name));
    if !theme_script_path.exists() {
        anyhow::bail!(
            "Theme \"{}\" does not exist at \"{}\", try a different theme",
            theme_name,
            theme_script_path.display()
        )
    }

    // Remove symlink file and create colorscheme symlink
    if base16_shell_colorscheme_path.exists() {
        fs::remove_file(base16_shell_colorscheme_path)?;
    }

    let theme_script_path =
        base16_shell_repo_path.join(format!("scripts/base16-{}.sh", theme_name));
    unix_fs::symlink(theme_script_path, base16_shell_colorscheme_path)?;

    // Write theme name to file
    fs::write(base16_shell_theme_name_path, theme_name)?;

    // Source colorscheme script
    // Wait for script to fully execute before continuing
    let mut child = Command::new("/bin/bash")
        .arg(base16_shell_colorscheme_path)
        .spawn()
        .with_context(|| {
            format!(
                "Failed to execute script: {:?}",
                base16_shell_colorscheme_path
            )
        })?;
    let status = child.wait().context("Failed to wait on bash status")?;
    if !status.success() {
        anyhow::bail!("Command finished with a non-zero status: {}", status)
    }

    Ok(())
}

fn set_colorscheme_embedded(
    theme_name: &String,
    base16_shell_colorscheme_path: &Path,
    base16_shell_theme_name_path: &Path,
) -> Result<()> {
    let embedded_colorscheme_dir = get_embedded_colorscheme_dir();

    let theme_script_file = embedded_colorscheme_dir
        .get_file(format!("base16-{}.sh", theme_name))
        .unwrap();
    let theme_script_contents = theme_script_file.contents();

    // Create new colorscheme script file
    write_to_file(
        base16_shell_colorscheme_path,
        from_utf8(theme_script_contents)?,
    )
    .with_context(|| {
        format!(
            "Unable to write to file: {}",
            base16_shell_colorscheme_path.display()
        )
    })?;

    // Write theme name to file
    write_to_file(base16_shell_theme_name_path, theme_name).with_context(|| {
        format!(
            "Unable to write to file: {}",
            base16_shell_colorscheme_path.display()
        )
    })?;

    // Source colorscheme script
    // Wait for script to fully execute before continuing
    let mut child = Command::new("/bin/bash")
        .arg(base16_shell_colorscheme_path)
        .spawn()
        .with_context(|| {
            format!(
                "Failed to execute script: {:?}",
                base16_shell_colorscheme_path
            )
        })?;
    let status = child.wait().context("Failed to wait on bash status")?;
    if !status.success() {
        anyhow::bail!("Command finished with a non-zero status: {}", status)
    }

    Ok(())
}

fn set_colorscheme(
    theme_name: &String,
    base16_shell_repo_path_option: &Option<PathBuf>,
    base16_shell_colorscheme_path: &Path,
    base16_shell_theme_name_path: &Path,
) -> Result<()> {
    // Read value from file
    let current_theme_name =
        read_file_to_string(base16_shell_theme_name_path).context("Failed to read from file")?;

    if theme_name.as_str() == current_theme_name {
        anyhow::bail!("Theme \"{}\" is already set", theme_name)
    }

    if let Some(base16_shell_repo_path) = base16_shell_repo_path_option {
        set_colorscheme_provided(
            &theme_name,
            &base16_shell_repo_path,
            &base16_shell_colorscheme_path,
            &base16_shell_theme_name_path,
        )
        .context("Failed to set user provided colorscheme")?;
    } else {
        set_colorscheme_embedded(
            &theme_name,
            &base16_shell_colorscheme_path,
            &base16_shell_theme_name_path,
        )
        .context("Failed to set embedded colorscheme")?;
    }

    Ok(())
}

fn run_hooks_provided(
    base16_shell_repo_path: &PathBuf,
    env_vars_to_set: Vec<(&str, &str)>,
) -> Result<()> {
    let base16_shell_hooks_path = base16_shell_repo_path.join("hooks");

    if !base16_shell_hooks_path.exists() {
        anyhow::bail!(
            "Provided hooks path does not exist: \"{}\"",
            base16_shell_hooks_path.display()
        )
    }

    for entry in fs::read_dir(base16_shell_hooks_path)? {
        let entry = entry?;

        let path = entry.path();

        // Check if the file name ends with .sh
        if path.extension().and_then(|ext| ext.to_str()) == Some("sh") {
            let mut command = Command::new("/bin/bash");

            command.arg(&path);

            // Set each environment variable for the script
            for (key, value) in &env_vars_to_set {
                command.env(key, value);
            }

            command
                .status()
                .with_context(|| format!("Failed to execute shell hook script: {:?}", path))?;
        }
    }

    Ok(())
}

fn run_hooks_embedded(
    embedded_dir: &'static Dir<'static>,
    env_vars_to_set: Vec<(&str, &str)>,
) -> Result<()> {
    for dir_entry in embedded_dir.find("*.sh").unwrap() {
        let file = dir_entry.as_file().unwrap();
        let contents = file.contents_utf8().unwrap();
        let mut temp_file = NamedTempFile::new()?;
        let entry = dir_entry;
        let path = entry.path();

        write!(temp_file, "{}", contents)
            .map_err(anyhow::Error::new)
            .context("Unable to write to temporary colorscheme file")?;

        let mut command = Command::new("/bin/bash");

        command.arg(temp_file.path());

        // Set each environment variable for the script
        for (key, value) in &env_vars_to_set {
            command.env(key, value);
        }

        command
            .status()
            .with_context(|| format!("Failed to execute shell hook script: {:?}", path))?;
    }

    Ok(())
}

// Set env variables for hooks and then execute .sh hooks
fn run_hooks(
    theme_name: &String,
    base16_config_path: &Path,
    base16_shell_repo_path_option: Option<PathBuf>,
    base16_shell_theme_name_path: &Path,
) -> Result<()> {
    let env_vars_to_set: Vec<(&str, &str)> = vec![
        (
            BASE16_SHELL_THEME_NAME_PATH_ENV,
            base16_shell_theme_name_path.to_str().unwrap(),
        ),
        (
            BASE16_SHELL_CONFIG_PATH_ENV,
            base16_config_path.to_str().unwrap(),
        ),
        (BASE16_THEME_ENV, theme_name),
    ];

    if let Some(base16_shell_repo_path) = base16_shell_repo_path_option {
        run_hooks_provided(&base16_shell_repo_path, env_vars_to_set).context(format!(
            "Error executing colorscheme scripts in dir: {}",
            base16_shell_repo_path.join("scripts").display()
        ))?;
    } else {
        let embedded_hooks_dir = get_embedded_hooks_dir();

        run_hooks_embedded(embedded_hooks_dir, env_vars_to_set).context(format!(
            "Error executing scripts in embedded dir: {}",
            embedded_hooks_dir.path().display()
        ))?;
    }

    Ok(())
}

/// Sets the selected colorscheme and runs associated hook scripts.
///
/// This function sets the desired colorscheme based on the provided theme name.
/// It determines whether to use the provided repository path or embedded resources
/// to locate the colorscheme script. After setting the colorscheme, it runs the hook
/// scripts to apply the colorscheme to the current environment.
///
/// # Arguments
///
/// * `theme_name` - A reference to a string containing the name of the theme to set.
/// * `base16_config_path` - A reference to a `PathBuf` pointing to the base16 configuration directory.
/// * `base16_shell_repo_path_option` - An `Option` wrapping a `PathBuf` that may contain a path to
///   the base16-shell repository. If `Some`, it uses the scripts from this path; otherwise, it falls back
///   to the embedded scripts.
/// * `base16_shell_colorscheme_path` - A reference to a `PathBuf` pointing to the file where the
///   colorscheme script should be written or linked.
/// * `base16_shell_theme_name_path` - A reference to a `PathBuf` pointing to the file where the
///   theme name should be stored.
///
/// # Errors
///
/// Returns an error if the colorscheme cannot be set, which could occur if the theme script does not
/// exist in the specified path or in the embedded resources, or if there's an issue executing the hook scripts.
///
/// # Examples
///
/// ```
/// # use std::path::PathBuf;
/// # fn run_example() -> anyhow::Result<()> {
/// let theme_name = "default".to_string();
/// let base16_config_path = PathBuf::from("/path/to/base16/config");
/// let base16_shell_repo_path_option = Some(PathBuf::from("/path/to/base16-shell/repo"));
/// let base16_shell_colorscheme_path = PathBuf::from("/path/to/base16/colorscheme/script");
/// let base16_shell_theme_name_path = PathBuf::from("/path/to/base16/theme/name");
///
/// commands::set_command(
///     &theme_name,
///     &base16_config_path,
///     base16_shell_repo_path_option,
///     &base16_shell_colorscheme_path,
///     &base16_shell_theme_name_path,
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn set_command(
    theme_name: &String,
    base16_config_path: &PathBuf,
    base16_shell_repo_path_option: Option<PathBuf>,
    base16_shell_colorscheme_path: &PathBuf,
    base16_shell_theme_name_path: &PathBuf,
) -> Result<()> {
    set_colorscheme(
        theme_name,
        &base16_shell_repo_path_option,
        base16_shell_colorscheme_path,
        base16_shell_theme_name_path,
    )
    .with_context(|| format!("Failed to set colorscheme \"{:?}\"", theme_name))?;

    run_hooks(
        theme_name,
        base16_config_path,
        base16_shell_repo_path_option,
        base16_shell_theme_name_path,
    )
    .context("Failed to run hooks")?;

    Ok(())
}

/// Lists available base16 color schemes.
///
/// This function looks for color schemes either in a provided directory
/// or, if not provided, in an embedded location. It then prints the
/// names of all found color schemes to standard output.
///
/// # Arguments
/// * `base16_shell_repo_path_option` - An optional `PathBuf` to the base16 shell scripts directory.
///
/// # Errors
/// Returns an error if the specified path does not exist or is not a directory,
/// or if there is an issue reading the directory.
///
/// # Examples
/// ```
/// // To list color schemes from a specified path
/// list_command(Some(PathBuf::from("/path/to/base16/scripts")));
///
/// // To list color schemes from the default embedded location
/// list_command(None);
/// ```
pub fn list_command(base16_shell_repo_path_option: Option<PathBuf>) -> Result<()> {
    // Check if a custom path to base16 shell repository is provided
    if let Some(base16_shell_repo_path) = base16_shell_repo_path_option {
        let scripts_path = base16_shell_repo_path.join("scripts");

        // Validate that the provided scripts path is a directory
        if !scripts_path.is_dir() {
            anyhow::bail!(
                "Scripts directory does not exist or is not a directory: {:?}",
                scripts_path
            );
        }

        // Collect color scheme names from the scripts directory
        let colorschemes: Vec<String> = fs::read_dir(&scripts_path)
            .with_context(|| format!("Failed to read directory: {:?}", &scripts_path))?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();

                // Filter for files and extract the color scheme name
                if path.is_file() {
                    return path
                        .file_stem()
                        .and_then(|name| name.to_str())
                        .and_then(|name| name.strip_prefix("base16-"))
                        .map(|name| name.to_string());
                }

                None
            })
            .collect();

        // Print the found color schemes or a message if none are found
        if colorschemes.is_empty() {
            println!("No themes found in the scripts directory.");
        } else {
            for colorscheme in colorschemes {
                println!("{}", colorscheme);
            }
        }
    } else {
        // Use the embedded colorscheme directory if no path is provided
        let colorschemes_dir = get_embedded_colorscheme_dir();

        // Process each file in the directory
        for file in colorschemes_dir.files() {
            let filepath_name = file.path().file_name().and_then(|os_str| os_str.to_str());

            match filepath_name {
                Some(colorscheme_filename) => {
                    // Strip the "base16-" prefix and print the color scheme name
                    let without_prefix = if colorscheme_filename.starts_with("base16-") {
                        &colorscheme_filename["base16-".len()..]
                    } else {
                        colorscheme_filename
                    };

                    // Extract and print the color scheme name
                    let colorscheme = without_prefix.split('.').next().unwrap_or("");

                    println!("{}", colorscheme);
                }
                None => println!("File path does not have a valid UTF-8 file name"),
            }
        }
    }

    Ok(())
}
