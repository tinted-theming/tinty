use clap::{Arg, ArgAction, Command};

use crate::constants::REPO_NAME;

/// Builds the command-line interface for the application.
pub fn build_cli() -> Command {
    Command::new(REPO_NAME)
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
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
            Command::new("current").about("Prints the last scheme name set")
        )
        .subcommand(
            Command::new("init").about("Initializes base16 with the exising config. Used to Initialize exising theme for when your shell starts up.")
        )
        .subcommand(Command::new("list").about("Lists available base16 colorschemes"))
        .subcommand(
            Command::new("set").about("Sets a base16 colorscheme").arg(
                Arg::new("scheme_name")
                    .help("The base16 colorscheme you want to set")
                    .required(true),
            ),
        )
        .subcommand(
            Command::new("setup").about(format!("Setup the environment needed for {}.", REPO_NAME))
        )
        .subcommand(
            Command::new("update").about("Update to the latest themes.")
        )
}
