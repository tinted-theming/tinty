use clap::{
    builder::{styling, PossibleValue},
    Arg, ArgAction, ArgGroup, ArgMatches, Command, ValueHint,
};
use clap_complete::Shell;

use crate::constants::REPO_NAME;

/// Builds the command-line interface for the application.
pub fn build_cli() -> Command {
    Command::new(REPO_NAME)
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("config")
                .short('c')
                .help(format!("Optional path to the {} config.toml file", REPO_NAME))
                .value_name("FILE")
                .long("config")
                .global(true)
                .action(ArgAction::Set)
        )
        .arg(
            Arg::new("data-dir")
                .short('d')
                .help(format!("Optional path to the {} data directory", REPO_NAME))
                .value_name("DIRECTORY")
                .long("data-dir")
                .global(true)
                .action(ArgAction::Set)
        )
        .subcommand(
            Command::new("build")
                .about("Builds the target theme template")
                .arg(
                    Arg::new("template-dir")
                        .help("Local path to the theme template you want to build")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("current").about("Prints the last scheme name applied")
        )
        .subcommand(
            Command::new("generate-completion").about("Generates a shell completion script").arg(
                Arg::new("shell_name")
                    .value_parser(clap::value_parser!(Shell))
                    .help("The name of the shell you want to generate a completion script for")
                    .required(true),
            ),
        )
        .subcommand(
            Command::new("generate-scheme")
                .about("Generates a scheme based on an image")
                .arg(
                    Arg::new("image_path")
                    .help("Which image file to use.")
                    .required(true)
                    .value_name("INFILE")
                    .value_hint(ValueHint::FilePath)
                )
                .arg(
                    Arg::new("outfile")
                    .help("Output path to save the <slug>.yaml file to. Use '-' for stdout")
                    .value_name("OUTFILE")
                    .value_hint(ValueHint::FilePath)
                )
                .arg(
                    Arg::new("author")
                    .long("author")
                    .help("Scheme author info (name, email, etc) to write, defaults to 'Tinty'")
                    .value_hint(ValueHint::Other)
                )
                .arg(
                    Arg::new("name")
                    .long("name")
                    .help("Scheme display name (can include spaces and capitalization). Defaults to 'Tinty Generated'")
                    .value_hint(ValueHint::Other)
                )
                .arg(
                    Arg::new("slug")
                    .long("slug")
                    .help("Scheme slug (the name you specify when applying schemes). Can not contain white-space or capitalization. Defaults to 'tinty-generated'")
                    .value_hint(ValueHint::Other)
                )
                .arg(
                    Arg::new("system")
                    .long("system")
                    .help("Whether to generate a base16 or base24 scheme")
                    .value_parser([
                        PossibleValue::new("base16"),
                        PossibleValue::new("base24"),
                    ])
                    .value_hint(ValueHint::Other)
                )
                .arg(
                    Arg::new("save")
                    .long("save")
                    .help("Whether to add the scheme to the installed schemes.")
                    .action(ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("variant")
                    .long("variant")
                    .help("Whether to generate a dark or light scheme")
                    .value_parser([
                        PossibleValue::new("dark"),
                        PossibleValue::new("light"),
                    ])
                    .value_hint(ValueHint::Other)
                )
                .group(ArgGroup::new("required_flags")
                    .args(["outfile", "save"])
                    .required(true)),
        )
        .subcommand(
            Command::new("info").about(format!("Shows scheme colors for all schemes matching <scheme_system>-<scheme_name> (Eg: {} info base16-mocha)", REPO_NAME)).arg(
                Arg::new("scheme_name")
                    .help("The scheme you want to get information about")
                    .required(false),
            )
        )
        .subcommand(
            Command::new("init").about("Initializes with the exising config. Used to Initialize exising theme for when your shell starts up")
        )
        .subcommand(Command::new("list").about("Lists available schemes")
                .arg(
                    Arg::new("custom-schemes")
                        .help("Lists availabile custom schemes")
                        .long("custom-schemes")
                        .action(ArgAction::SetTrue)
                ))
        .subcommand(
            Command::new("config").about("Provides config related information")
                .arg(
                    Arg::new("config-path")
                        .help(format!("Returns path to the {} config file", REPO_NAME))
                        .value_name("FILE")
                        .long("config-path")
                        .conflicts_with("data-dir-path")
                        .action(ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("data-dir-path")
                        .help(format!("Returns path to the {} data directory", REPO_NAME))
                        .value_name("DIRECTORY")
                        .long("data-dir-path")
                        .conflicts_with("config-path")
                        .action(ArgAction::SetTrue)
                )
        )
        .subcommand(
            Command::new("apply").about("Applies a theme based on the chosen scheme").arg(
                Arg::new("scheme_name")
                    .help("The scheme you want to apply")
                    .required(true),
            ),
        )
        .subcommand(
            Command::new("install").about(format!("Install the environment needed for {}", REPO_NAME))
        )
        .subcommand(
            Command::new("update").about("Update to the latest themes")
        )
}

// Parse the command line arguments with styling
pub fn get_matches() -> ArgMatches {
    let styles = styling::Styles::styled()
        .header(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
        .usage(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
        .literal(styling::AnsiColor::Blue.on_default() | styling::Effects::BOLD)
        .placeholder(styling::AnsiColor::Cyan.on_default());

    build_cli().styles(styles).get_matches()
}
