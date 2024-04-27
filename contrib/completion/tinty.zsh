#compdef tinty

autoload -U is-at-least

_tinty() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" \
'-c+[Optional path to the tinty config.toml file]:FILE: ' \
'--config=[Optional path to the tinty config.toml file]:FILE: ' \
'-d+[Optional path to the tinty data directory]:DIRECTORY: ' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY: ' \
'--generate-completion=[Generate completion scripts]:SHELL:(bash elvish fish powershell zsh)' \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
":: :_tinty_commands" \
"*::: :->tinty" \
&& ret=0
    case $state in
    (tinty)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:tinty-command-$line[1]:"
        case $line[1] in
            (current)
_arguments "${_arguments_options[@]}" \
'-c+[Optional path to the tinty config.toml file]:FILE: ' \
'--config=[Optional path to the tinty config.toml file]:FILE: ' \
'-d+[Optional path to the tinty data directory]:DIRECTORY: ' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY: ' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(info)
_arguments "${_arguments_options[@]}" \
'-c+[Optional path to the tinty config.toml file]:FILE: ' \
'--config=[Optional path to the tinty config.toml file]:FILE: ' \
'-d+[Optional path to the tinty data directory]:DIRECTORY: ' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY: ' \
'-h[Print help]' \
'--help[Print help]' \
'::scheme_name -- The scheme you want to get information about:' \
&& ret=0
;;
(init)
_arguments "${_arguments_options[@]}" \
'-c+[Optional path to the tinty config.toml file]:FILE: ' \
'--config=[Optional path to the tinty config.toml file]:FILE: ' \
'-d+[Optional path to the tinty data directory]:DIRECTORY: ' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY: ' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" \
'-c+[Optional path to the tinty config.toml file]:FILE: ' \
'--config=[Optional path to the tinty config.toml file]:FILE: ' \
'-d+[Optional path to the tinty data directory]:DIRECTORY: ' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY: ' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(apply)
_arguments "${_arguments_options[@]}" \
'-c+[Optional path to the tinty config.toml file]:FILE: ' \
'--config=[Optional path to the tinty config.toml file]:FILE: ' \
'-d+[Optional path to the tinty data directory]:DIRECTORY: ' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY: ' \
'-h[Print help]' \
'--help[Print help]' \
':scheme_name -- The scheme you want to apply:' \
&& ret=0
;;
(install)
_arguments "${_arguments_options[@]}" \
'-c+[Optional path to the tinty config.toml file]:FILE: ' \
'--config=[Optional path to the tinty config.toml file]:FILE: ' \
'-d+[Optional path to the tinty data directory]:DIRECTORY: ' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY: ' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" \
'-c+[Optional path to the tinty config.toml file]:FILE: ' \
'--config=[Optional path to the tinty config.toml file]:FILE: ' \
'-d+[Optional path to the tinty data directory]:DIRECTORY: ' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY: ' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" \
":: :_tinty__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:tinty-help-command-$line[1]:"
        case $line[1] in
            (current)
_arguments "${_arguments_options[@]}" \
&& ret=0
;;
(info)
_arguments "${_arguments_options[@]}" \
&& ret=0
;;
(init)
_arguments "${_arguments_options[@]}" \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" \
&& ret=0
;;
(apply)
_arguments "${_arguments_options[@]}" \
&& ret=0
;;
(install)
_arguments "${_arguments_options[@]}" \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
}

(( $+functions[_tinty_commands] )) ||
_tinty_commands() {
    local commands; commands=(
'current:Prints the last scheme name applied' \
'info:Shows scheme colors for all schemes matching <scheme_system>-<scheme_name> (Eg\: tinty info base16-mocha)' \
'init:Initializes with the exising config. Used to Initialize exising theme for when your shell starts up' \
'list:Lists available schemes' \
'apply:Applies a theme based on the chosen scheme' \
'install:Install the environment needed for tinty' \
'update:Update to the latest themes' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'tinty commands' commands "$@"
}
(( $+functions[_tinty__apply_commands] )) ||
_tinty__apply_commands() {
    local commands; commands=()
    _describe -t commands 'tinty apply commands' commands "$@"
}
(( $+functions[_tinty__help__apply_commands] )) ||
_tinty__help__apply_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help apply commands' commands "$@"
}
(( $+functions[_tinty__current_commands] )) ||
_tinty__current_commands() {
    local commands; commands=()
    _describe -t commands 'tinty current commands' commands "$@"
}
(( $+functions[_tinty__help__current_commands] )) ||
_tinty__help__current_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help current commands' commands "$@"
}
(( $+functions[_tinty__help_commands] )) ||
_tinty__help_commands() {
    local commands; commands=(
'current:Prints the last scheme name applied' \
'info:Shows scheme colors for all schemes matching <scheme_system>-<scheme_name> (Eg\: tinty info base16-mocha)' \
'init:Initializes with the exising config. Used to Initialize exising theme for when your shell starts up' \
'list:Lists available schemes' \
'apply:Applies a theme based on the chosen scheme' \
'install:Install the environment needed for tinty' \
'update:Update to the latest themes' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'tinty help commands' commands "$@"
}
(( $+functions[_tinty__help__help_commands] )) ||
_tinty__help__help_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help help commands' commands "$@"
}
(( $+functions[_tinty__help__info_commands] )) ||
_tinty__help__info_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help info commands' commands "$@"
}
(( $+functions[_tinty__info_commands] )) ||
_tinty__info_commands() {
    local commands; commands=()
    _describe -t commands 'tinty info commands' commands "$@"
}
(( $+functions[_tinty__help__init_commands] )) ||
_tinty__help__init_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help init commands' commands "$@"
}
(( $+functions[_tinty__init_commands] )) ||
_tinty__init_commands() {
    local commands; commands=()
    _describe -t commands 'tinty init commands' commands "$@"
}
(( $+functions[_tinty__help__install_commands] )) ||
_tinty__help__install_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help install commands' commands "$@"
}
(( $+functions[_tinty__install_commands] )) ||
_tinty__install_commands() {
    local commands; commands=()
    _describe -t commands 'tinty install commands' commands "$@"
}
(( $+functions[_tinty__help__list_commands] )) ||
_tinty__help__list_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help list commands' commands "$@"
}
(( $+functions[_tinty__list_commands] )) ||
_tinty__list_commands() {
    local commands; commands=()
    _describe -t commands 'tinty list commands' commands "$@"
}
(( $+functions[_tinty__help__update_commands] )) ||
_tinty__help__update_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help update commands' commands "$@"
}
(( $+functions[_tinty__update_commands] )) ||
_tinty__update_commands() {
    local commands; commands=()
    _describe -t commands 'tinty update commands' commands "$@"
}

if [ "$funcstack[1]" = "_tinty" ]; then
    _tinty "$@"
else
    compdef _tinty tinty
fi
