use clap::{Arg, ArgAction, Command};

use crate::config::REPO_NAME;

/// Builds the command-line interface for the application.
pub fn build_cli() -> Command {
    Command::new("base16_shell")
        .version("1.0.0")
        .author("Tinted Theming")
        .about("A tool to switch base16 colorschemes")
        // Define a global argument for specifying the repository directory
        .arg(
            Arg::new("repo-dir")
                .short('d')
                .help(format!("Optional path to the {} repository. This is used to run the colorschemes and hooks if you don't want to use the compiled versions.", REPO_NAME))
                .value_name("DIR")
                .long("repo-dir")
                .global(true)
                .action(ArgAction::Set)
        )
        // Define a global argument for specifying the repository directory
        .arg(
            Arg::new("config")
                .short('c')
                .help(format!("Optional path to the {} config directory.", REPO_NAME))
                .value_name("CONFIG")
                .long("config")
                .global(true)
                .action(ArgAction::Set)
        )
        // Define subcommands
        .subcommand(
            Command::new("init").about("Initializes base16 with the exising config. Used to Initialize exising theme for when your shell starts up.")
        )
        .subcommand(Command::new("list").about("Lists available base16 colorschemes"))
        .subcommand(
            Command::new("set").about("Sets a base16 colorscheme").arg(
                Arg::new("theme_name")
                    .help("The base16 colorscheme you want to set")
                    .required(true),
            ),
        )
}
