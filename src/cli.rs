use clap::{Arg, ArgAction, Command};

pub fn build_cli() -> Command {
    Command::new("base16_shell")
        .version("1.0.0")
        .author("Tinted Theming")
        .about("A tool to switch base16 colorschemes")
        .arg(
            Arg::new("repo-dir")
                .short('d')
                .help("Optional path to the base16-shell repository. This is used to run the colorschemes and hooks if you don't want to use the compiled versions.")
                .value_name("DIR")
                .long("repo-dir")
                .global(true)
                .action(ArgAction::Set)
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
