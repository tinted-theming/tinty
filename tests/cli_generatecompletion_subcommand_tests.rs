mod utils;

use crate::utils::setup;
use anyhow::Result;

fn generate_shell_completion_test(shell_name: &str) -> Result<String> {
    // -------
    // Arrange
    // -------
    let (_, data_path, command_vec, cleanup) = setup(
        "test_cli_generatecompletion_subcommand",
        format!("generate-completion {shell_name}").as_str(),
    )?;

    // ---
    // Act
    // ---
    let (stdout, _) = utils::run_command(command_vec, &data_path, true).unwrap();

    cleanup()?;
    Ok(stdout)
}

#[test]
fn test_cli_generatecompletion_subcommand_bash() -> Result<()> {
    // ---
    // Act
    // ---
    let shell_name = "bash";
    let stdout = generate_shell_completion_test(shell_name)?;

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains(
            r#"_tinty() {
    local i cur prev opts cmd
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}"#
        ),
        "stdout does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_generatecompletion_subcommand_elvish() -> Result<()> {
    // ---
    // Act
    // ---
    let shell_name = "elvish";
    let stdout = generate_shell_completion_test(shell_name)?;

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains(
            r#"
use builtin;
use str;

set edit:completion:arg-completer[tinty] = {|@words|"#
        ),
        "stdout does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_generatecompletion_subcommand_fish() -> Result<()> {
    // ---
    // Act
    // ---
    let shell_name = "fish";
    let stdout = generate_shell_completion_test(shell_name)?;

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains(r#"
complete -c tinty -n "__fish_tinty_needs_command" -s c -l config -d 'Optional path to the tinty config.toml file' -r
complete -c tinty -n "__fish_tinty_needs_command" -s d -l data-dir -d 'Optional path to the tinty data directory' -r
complete -c tinty -n "__fish_tinty_needs_command" -s h -l help -d 'Print help'
"#),
        "stdout does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_generatecompletion_subcommand_powershell() -> Result<()> {
    // ---
    // Act
    // ---
    let shell_name = "powershell";
    let stdout = generate_shell_completion_test(shell_name)?;

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains(
            r#"
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'tinty' -ScriptBlock {"#
        ),
        "stdout does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_cli_generatecompletion_subcommand_zsh() -> Result<()> {
    // ---
    // Act
    // ---
    let shell_name = "zsh";
    let stdout = generate_shell_completion_test(shell_name)?;

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains(
            r#"#compdef tinty

autoload -U is-at-least

_tinty() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1"#
        ),
        "stdout does not contain the expected output"
    );

    Ok(())
}
