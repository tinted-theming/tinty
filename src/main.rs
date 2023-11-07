use std::env;
// use clap::Parser;
use std::path::{Path,PathBuf};
use std::fs::{self,File};
use std::io;
use std::fmt;

// #[derive(Parser)]
// struct Cli {
//     theme_name: String,
// }
//

struct PathError {
    message: String,
    path: PathBuf,
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.message, self.path.display())
    }
}

fn ensure_file_exists<P: AsRef<Path>>(file_path: P, is_directory: Option<bool>) -> io::Result<()> {
    let path = file_path.as_ref();

    if is_directory == Some(true) {
        if !path.exists() {
            // Create the file if it does not exist.
            fs::create_dir_all(path)?;
        };
    } else {
        File::create(path)?;
    }

    Ok(())
}

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

fn create_env_vars(config_path: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let base16_config_path = config_path.join("tinted-theming");
    let base16_shell_path: PathBuf = env::current_dir().expect("Failed to get current directory");
    let base16_shell_colorscheme_path = base16_config_path.join("base16_shell_theme");
    let base16_shell_theme_name_path = base16_config_path.join("theme_name");

    (base16_config_path, base16_shell_path, base16_shell_colorscheme_path, base16_shell_theme_name_path)
}

fn main() {
    // let args = Cli::parse();
    let exe_path: Option<PathBuf> = env::current_exe().ok().and_then(|path| path.canonicalize().ok());
    let config_path: PathBuf = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = env::var("HOME").expect("HOME is not set");

            PathBuf::from(home).join(".config")
        });

    let (base16_config_path, base16_shell_path, base16_shell_colorscheme_path, base16_shell_theme_name_path) = create_env_vars(config_path.as_path());

    // let base16_config_path = config_path.join("tinted-theming-test");
    // let base16_shell_colorscheme_path = base16_config_path.join("base16_shell_theme");
    // let base16_shell_theme_name_path = base16_config_path.join("theme_name");

    let base16_shell_path: PathBuf = match env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            println!("Error getting the current directory: {}", e);
            return;
        },
    };


    // if let Some(script_path) = exe_path {
    //     let base16_shell_colorscheme_path_str = base16_shell_colorscheme_path.to_str().expect("invalid path");

    //     println!(
    //         "Config path: {} {} {}", 
    //         base16_shell_colorscheme_path_str,
    //         script_path.display(),
    //         base16_shell_theme_name_path.display()
    //     );
    // }


    let mut base16_shell_hooks_path = env::var("BASE16_SHELL_HOOKS_PATH").unwrap_or_default();

    if base16_shell_hooks_path.is_empty() || !Path::new(&base16_shell_hooks_path).is_dir() {
        base16_shell_hooks_path = format!("{}/hooks", base16_shell_path.display());

        env::set_var("BASE16_SHELL_HOOKS_PATH", &base16_shell_hooks_path);
    }

    // Use base16_shell_hooks_path as needed

    if let Err(e) = ensure_config_files_exist(base16_config_path.as_path(), base16_shell_theme_name_path.as_path()) {
        eprintln!("Error: {}", e);
    }


    println!("script path: {}", base16_shell_path.display());
    println!("BASE16_SHELL_HOOKS_PATH is {}", base16_shell_hooks_path);
    println!("base16_shell_path is {}", base16_shell_path.display());
    println!("base16_config_path is {}", base16_config_path.display());
    println!("base16_shell_colorscheme_path is {}", base16_shell_colorscheme_path.display());
    println!("base16_shell_path is {}", base16_shell_path.display());


    // let user_config_dir_path = user_home_dir_path.unwrap_or_else(|_| default_value.to_string());
    // let base_path = env::var("XDG_CONFIG_HOME")
    //     .or_else(|_| env::var("HOME"))
    //     .unwrap_or_else(|_| panic!("Neither XDG_CONFIG_HOME nor HOME environment variable is set"));

    // // Append '/cli_app' to the base path.
    // let config_path = format!("{}/cli_app", base_path);


    // print!("{username}");
}
