use std::env;
use std::fmt;
use std::fs::{self,File};
use std::io::{self,Read};
use std::os::unix::fs as unix_fs;
use std::path::{Path,PathBuf};
use std::process::Command;

struct PathError {
    message: String,
    path: PathBuf,
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.message, self.path.display())
    }
}

// Creates a file if it does not exist
fn ensure_file_exists<P: AsRef<Path>>(file_path: P, is_directory: Option<bool>) -> io::Result<()> {
    let path = file_path.as_ref();

    if is_directory == Some(true) {
        if !path.exists() {
            fs::create_dir_all(path)?;
        };
    } else {
        if !path.exists() {
            File::create(path)?;
        };
    }

    Ok(())
}

// Create config files if they don't exist
fn ensure_config_files_exist(base16_config_path: &Path, base16_shell_theme_name_path: &Path) -> Result<(), PathError> {
    ensure_file_exists(base16_config_path, Some(true)).map_err(|_| PathError {
        message: "Failed to create directory. Check if parent directory exists.".to_string(),
        path: base16_config_path.to_path_buf()
    })?;
    ensure_file_exists(base16_shell_theme_name_path, Some(false)).map_err(|_| PathError {
        message: "Failed to create directory. Check if parent directory exists.".to_string(),
        path: base16_shell_theme_name_path.to_path_buf()
    })?;

    Ok(())
}

// Convert file contents to string
fn read_file_to_string(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    Ok(contents)
}

fn set_theme(
    theme_name: &String,
    base16_config_path: &Path,
    base16_shell_path: &Path,
    theme_script_path: &Path,
    base16_shell_colorscheme_path: &Path,
    base16_shell_theme_name_path: &Path
) -> std::io::Result<()> {
    let current_theme_name = match read_file_to_string(base16_shell_theme_name_path) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Failed to read from file: {}", e);
            return Err(e);
        }
    };

    if theme_name.to_string() == current_theme_name {
        eprintln!("Theme \"{}\" is already set", theme_name);
        std::process::exit(1);
    }

    // Remove symlink file and create colorscheme symlink
    // -----------------------------------------------------------------
    if base16_shell_colorscheme_path.exists() {
        fs::remove_file(base16_shell_colorscheme_path)?;
    }

    unix_fs::symlink(theme_script_path, base16_shell_colorscheme_path)?;

    // Write theme name to file
    // -----------------------------------------------------------------
    fs::write(base16_shell_theme_name_path, theme_name)?;

    // Run colorscheme script
    // -----------------------------------------------------------------

    // Source colorscheme script
    // Wait for script to fully execute before continuing
    let mut child = Command::new("/bin/bash").arg(base16_shell_colorscheme_path).spawn()?;
    let status = child.wait()?;
    if !status.success() {
        eprintln!("Command finished with a non-zero status.");
        return Err(
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Script execution failed"
            )
        );
    }

    // Hooks
    // Set env variables for hooks and then execute .sh hooks
    // -----------------------------------------------------------------
    let mut base16_shell_hooks_path = env::var("BASE16_SHELL_HOOKS_PATH").unwrap_or_default();
    if base16_shell_hooks_path.is_empty() || !Path::new(&base16_shell_hooks_path).is_dir() {
        base16_shell_hooks_path = format!("{}/hooks", base16_shell_path.display());

        env::set_var("BASE16_SHELL_HOOKS_PATH", &base16_shell_hooks_path);
    }

    env::set_var("BASE16_SHELL_THEME_NAME_PATH", base16_shell_theme_name_path);
    env::set_var("BASE16_CONFIG_PATH", base16_config_path);

    let base16_shell_hooks_path = base16_shell_path.join("hooks");
    if base16_shell_hooks_path.is_dir() {
        for entry in fs::read_dir(base16_shell_hooks_path)? {
            let entry = entry?;
            let path = entry.path();

            // Check if the file name ends with .sh
            if path.extension().and_then(|ext| ext.to_str()) == Some("sh") {
                Command::new("/bin/bash").arg(path).status()?;
            }
        }
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("You didn't provide a theme name argument.");
        std::process::exit(1);
    }

    let config_path: PathBuf = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = env::var("HOME").expect("HOME is not set");

            PathBuf::from(home).join(".config")
        });
    let config_path_something = config_path.as_path();
    let base16_config_path = config_path_something.join("tinted-theming");
    let base16_shell_path: PathBuf = env::current_dir().expect("Failed to get current directory");
    let base16_shell_colorscheme_path = base16_config_path.join("base16_shell_theme");
    let base16_shell_theme_name_path = base16_config_path.join("theme_name");
    let theme_name = &args[1];
    let theme_script_path = base16_shell_path.join(format!("scripts/base16-{}.sh", theme_name));

    if !theme_script_path.exists() {
        eprintln!("Theme \"{}\" does not exist, try a different theme", theme_name);
        std::process::exit(1);
    }

    if let Err(e) = ensure_config_files_exist(base16_config_path.as_path(), base16_shell_theme_name_path.as_path()) {
        eprintln!("Error: {}", e);
    }

    match set_theme(
        &theme_name, 
        base16_config_path.as_path(),
        base16_shell_path.as_path(),
        theme_script_path.as_path(),
        base16_shell_colorscheme_path.as_path(),
        base16_shell_theme_name_path.as_path()
    ) {
        Ok(()) => {
            eprintln!("Theme set to: {}", theme_name);
        },
        Err(e) => {
            eprintln!("Failed to set theme \"{}\": {}", theme_name, e);
            std::process::exit(1);
        }
    }
}
