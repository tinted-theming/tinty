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
    _arguments "${_arguments_options[@]}" : \
'-c+[Optional path to the tinty config.toml file]:FILE:_default' \
'--config=[Optional path to the tinty config.toml file]:FILE:_default' \
'-d+[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY:_default' \
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
            (build)
_arguments "${_arguments_options[@]}" : \
'-c+[Optional path to the tinty config.toml file]:FILE:_default' \
'--config=[Optional path to the tinty config.toml file]:FILE:_default' \
'-d+[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY:_default' \
'-q[Silence stdout]' \
'--quiet[Silence stdout]' \
'-h[Print help]' \
'--help[Print help]' \
':template-dir -- Local path to the theme template you want to build:_default' \
&& ret=0
;;
(current)
_arguments "${_arguments_options[@]}" : \
'-c+[Optional path to the tinty config.toml file]:FILE:_default' \
'--config=[Optional path to the tinty config.toml file]:FILE:_default' \
'-d+[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY:_default' \
'-h[Print help]' \
'--help[Print help]' \
'::property_name -- Optional field to retrieve scheme information for\: author, description, name, etc.:(author description name slug system variant)' \
&& ret=0
;;
(generate-completion)
_arguments "${_arguments_options[@]}" : \
'-c+[Optional path to the tinty config.toml file]:FILE:_default' \
'--config=[Optional path to the tinty config.toml file]:FILE:_default' \
'-d+[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY:_default' \
'-h[Print help]' \
'--help[Print help]' \
':shell_name -- The name of the shell you want to generate a completion script for:(bash elvish fish powershell zsh)' \
&& ret=0
;;
(generate-scheme)
_arguments "${_arguments_options[@]}" : \
'--author=[Scheme author info (name, email, etc) to write, defaults to '\''Tinty'\'']: :' \
'--description=[Scheme description info]: :' \
'--name=[Scheme display name (can include spaces and capitalization). Defaults to '\''Tinty Generated'\'']: :' \
'--slug=[Scheme slug (the name you specify when applying schemes). Can not contain white-space or capitalization. Defaults to '\''tinty-generated'\'']: :' \
'--system=[Whether to generate a base16 or base24 scheme]: :(base16 base24)' \
'--variant=[Whether to generate a dark or light scheme]: :(dark light)' \
'-c+[Optional path to the tinty config.toml file]:FILE:_default' \
'--config=[Optional path to the tinty config.toml file]:FILE:_default' \
'-d+[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--save[Whether to add the scheme to the installed schemes.]' \
'-h[Print help]' \
'--help[Print help]' \
':image_path -- Which image file to use.:_files' \
&& ret=0
;;
(info)
_arguments "${_arguments_options[@]}" : \
'-c+[Optional path to the tinty config.toml file]:FILE:_default' \
'--config=[Optional path to the tinty config.toml file]:FILE:_default' \
'-d+[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--custom-schemes[Lists availabile custom schemes]' \
'-h[Print help]' \
'--help[Print help]' \
'::scheme_name -- The scheme you want to get information about:_default' \
&& ret=0
;;
(init)
_arguments "${_arguments_options[@]}" : \
'-c+[Optional path to the tinty config.toml file]:FILE:_default' \
'--config=[Optional path to the tinty config.toml file]:FILE:_default' \
'-d+[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--verbose[Print to stdout]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
'-c+[Optional path to the tinty config.toml file]:FILE:_default' \
'--config=[Optional path to the tinty config.toml file]:FILE:_default' \
'-d+[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--custom-schemes[Lists availabile custom schemes]' \
'--json[Output as JSON]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(config)
_arguments "${_arguments_options[@]}" : \
'-c+[Optional path to the tinty config.toml file]:FILE:_default' \
'--config=[Optional path to the tinty config.toml file]:FILE:_default' \
'-d+[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY:_default' \
'(--data-dir-path)--config-path[Returns path to the tinty config file]' \
'(--config-path)--data-dir-path[Returns path to the tinty data directory]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(apply)
_arguments "${_arguments_options[@]}" : \
'-c+[Optional path to the tinty config.toml file]:FILE:_default' \
'--config=[Optional path to the tinty config.toml file]:FILE:_default' \
'-d+[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY:_default' \
'-q[Silence stdout]' \
'--quiet[Silence stdout]' \
'-h[Print help]' \
'--help[Print help]' \
':scheme_name -- The scheme you want to apply:_default' \
&& ret=0
;;
(install)
_arguments "${_arguments_options[@]}" : \
'-c+[Optional path to the tinty config.toml file]:FILE:_default' \
'--config=[Optional path to the tinty config.toml file]:FILE:_default' \
'-d+[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY:_default' \
'-q[Silence stdout]' \
'--quiet[Silence stdout]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
'-c+[Optional path to the tinty config.toml file]:FILE:_default' \
'--config=[Optional path to the tinty config.toml file]:FILE:_default' \
'-d+[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY:_default' \
'-q[Silence stdout]' \
'--quiet[Silence stdout]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(sync)
_arguments "${_arguments_options[@]}" : \
'-c+[Optional path to the tinty config.toml file]:FILE:_default' \
'--config=[Optional path to the tinty config.toml file]:FILE:_default' \
'-d+[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY:_default' \
'-q[Silence stdout]' \
'--quiet[Silence stdout]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(cycle)
_arguments "${_arguments_options[@]}" : \
'-c+[Optional path to the tinty config.toml file]:FILE:_default' \
'--config=[Optional path to the tinty config.toml file]:FILE:_default' \
'-d+[Optional path to the tinty data directory]:DIRECTORY:_default' \
'--data-dir=[Optional path to the tinty data directory]:DIRECTORY:_default' \
'-q[Silence stdout]' \
'--quiet[Silence stdout]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_tinty__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:tinty-help-command-$line[1]:"
        case $line[1] in
            (build)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(current)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(generate-completion)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(generate-scheme)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(info)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(init)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(config)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(apply)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(install)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(sync)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(cycle)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
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
'build:Builds the target theme template' \
'current:Prints the last scheme name applied or specific values from the current scheme' \
'generate-completion:Generates a shell completion script' \
'generate-scheme:Generates a scheme based on an image' \
'info:Shows scheme colors for all schemes matching <scheme_system>-<scheme_name> (Eg\: tinty info base16-mocha)' \
'init:Initializes with the exising config. Used to Initialize exising theme for when your shell starts up' \
'list:Lists available schemes' \
'config:Provides config related information' \
'apply:Applies a theme based on the chosen scheme' \
'install:Install the environment needed for tinty' \
'update:Update to the latest themes' \
'sync:Install missing templates in tinty/config.toml and update existing templates' \
'cycle:Cycle through your preferred themes' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'tinty commands' commands "$@"
}
(( $+functions[_tinty__apply_commands] )) ||
_tinty__apply_commands() {
    local commands; commands=()
    _describe -t commands 'tinty apply commands' commands "$@"
}
(( $+functions[_tinty__build_commands] )) ||
_tinty__build_commands() {
    local commands; commands=()
    _describe -t commands 'tinty build commands' commands "$@"
}
(( $+functions[_tinty__config_commands] )) ||
_tinty__config_commands() {
    local commands; commands=()
    _describe -t commands 'tinty config commands' commands "$@"
}
(( $+functions[_tinty__current_commands] )) ||
_tinty__current_commands() {
    local commands; commands=()
    _describe -t commands 'tinty current commands' commands "$@"
}
(( $+functions[_tinty__cycle_commands] )) ||
_tinty__cycle_commands() {
    local commands; commands=()
    _describe -t commands 'tinty cycle commands' commands "$@"
}
(( $+functions[_tinty__generate-completion_commands] )) ||
_tinty__generate-completion_commands() {
    local commands; commands=()
    _describe -t commands 'tinty generate-completion commands' commands "$@"
}
(( $+functions[_tinty__generate-scheme_commands] )) ||
_tinty__generate-scheme_commands() {
    local commands; commands=()
    _describe -t commands 'tinty generate-scheme commands' commands "$@"
}
(( $+functions[_tinty__help_commands] )) ||
_tinty__help_commands() {
    local commands; commands=(
'build:Builds the target theme template' \
'current:Prints the last scheme name applied or specific values from the current scheme' \
'generate-completion:Generates a shell completion script' \
'generate-scheme:Generates a scheme based on an image' \
'info:Shows scheme colors for all schemes matching <scheme_system>-<scheme_name> (Eg\: tinty info base16-mocha)' \
'init:Initializes with the exising config. Used to Initialize exising theme for when your shell starts up' \
'list:Lists available schemes' \
'config:Provides config related information' \
'apply:Applies a theme based on the chosen scheme' \
'install:Install the environment needed for tinty' \
'update:Update to the latest themes' \
'sync:Install missing templates in tinty/config.toml and update existing templates' \
'cycle:Cycle through your preferred themes' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'tinty help commands' commands "$@"
}
(( $+functions[_tinty__help__apply_commands] )) ||
_tinty__help__apply_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help apply commands' commands "$@"
}
(( $+functions[_tinty__help__build_commands] )) ||
_tinty__help__build_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help build commands' commands "$@"
}
(( $+functions[_tinty__help__config_commands] )) ||
_tinty__help__config_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help config commands' commands "$@"
}
(( $+functions[_tinty__help__current_commands] )) ||
_tinty__help__current_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help current commands' commands "$@"
}
(( $+functions[_tinty__help__cycle_commands] )) ||
_tinty__help__cycle_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help cycle commands' commands "$@"
}
(( $+functions[_tinty__help__generate-completion_commands] )) ||
_tinty__help__generate-completion_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help generate-completion commands' commands "$@"
}
(( $+functions[_tinty__help__generate-scheme_commands] )) ||
_tinty__help__generate-scheme_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help generate-scheme commands' commands "$@"
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
(( $+functions[_tinty__help__init_commands] )) ||
_tinty__help__init_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help init commands' commands "$@"
}
(( $+functions[_tinty__help__install_commands] )) ||
_tinty__help__install_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help install commands' commands "$@"
}
(( $+functions[_tinty__help__list_commands] )) ||
_tinty__help__list_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help list commands' commands "$@"
}
(( $+functions[_tinty__help__sync_commands] )) ||
_tinty__help__sync_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help sync commands' commands "$@"
}
(( $+functions[_tinty__help__update_commands] )) ||
_tinty__help__update_commands() {
    local commands; commands=()
    _describe -t commands 'tinty help update commands' commands "$@"
}
(( $+functions[_tinty__info_commands] )) ||
_tinty__info_commands() {
    local commands; commands=()
    _describe -t commands 'tinty info commands' commands "$@"
}
(( $+functions[_tinty__init_commands] )) ||
_tinty__init_commands() {
    local commands; commands=()
    _describe -t commands 'tinty init commands' commands "$@"
}
(( $+functions[_tinty__install_commands] )) ||
_tinty__install_commands() {
    local commands; commands=()
    _describe -t commands 'tinty install commands' commands "$@"
}
(( $+functions[_tinty__list_commands] )) ||
_tinty__list_commands() {
    local commands; commands=()
    _describe -t commands 'tinty list commands' commands "$@"
}
(( $+functions[_tinty__sync_commands] )) ||
_tinty__sync_commands() {
    local commands; commands=()
    _describe -t commands 'tinty sync commands' commands "$@"
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
