# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_tinty_global_optspecs
	string join \n c/config= d/data-dir= h/help V/version
end

function __fish_tinty_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_tinty_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_tinty_using_subcommand
	set -l cmd (__fish_tinty_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c tinty -n "__fish_tinty_needs_command" -s c -l config -d 'Optional path to the tinty config.toml file' -r
complete -c tinty -n "__fish_tinty_needs_command" -s d -l data-dir -d 'Optional path to the tinty data directory' -r
complete -c tinty -n "__fish_tinty_needs_command" -s h -l help -d 'Print help'
complete -c tinty -n "__fish_tinty_needs_command" -s V -l version -d 'Print version'
complete -c tinty -n "__fish_tinty_needs_command" -f -a "build" -d 'Builds the target theme template'
complete -c tinty -n "__fish_tinty_needs_command" -f -a "current" -d 'Prints the last scheme name applied or specific values from the current scheme'
complete -c tinty -n "__fish_tinty_needs_command" -f -a "generate-completion" -d 'Generates a shell completion script'
complete -c tinty -n "__fish_tinty_needs_command" -f -a "generate-scheme" -d 'Generates a scheme based on an image'
complete -c tinty -n "__fish_tinty_needs_command" -f -a "info" -d 'Shows scheme colors for all schemes matching <scheme_system>-<scheme_name> (Eg: tinty info base16-mocha)'
complete -c tinty -n "__fish_tinty_needs_command" -f -a "init" -d 'Initializes with the exising config. Used to Initialize exising theme for when your shell starts up'
complete -c tinty -n "__fish_tinty_needs_command" -f -a "list" -d 'Lists available schemes'
complete -c tinty -n "__fish_tinty_needs_command" -f -a "config" -d 'Provides config related information'
complete -c tinty -n "__fish_tinty_needs_command" -f -a "apply" -d 'Applies a theme based on the chosen scheme'
complete -c tinty -n "__fish_tinty_needs_command" -f -a "install" -d 'Install the environment needed for tinty'
complete -c tinty -n "__fish_tinty_needs_command" -f -a "update" -d 'Update to the latest themes'
complete -c tinty -n "__fish_tinty_needs_command" -f -a "sync" -d 'Install missing templates in tinty/config.toml and update existing templates'
complete -c tinty -n "__fish_tinty_needs_command" -f -a "cycle" -d 'Cycle through your preferred themes'
complete -c tinty -n "__fish_tinty_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c tinty -n "__fish_tinty_using_subcommand build" -s c -l config -d 'Optional path to the tinty config.toml file' -r
complete -c tinty -n "__fish_tinty_using_subcommand build" -s d -l data-dir -d 'Optional path to the tinty data directory' -r
complete -c tinty -n "__fish_tinty_using_subcommand build" -s q -l quiet -d 'Silence stdout'
complete -c tinty -n "__fish_tinty_using_subcommand build" -s h -l help -d 'Print help'
complete -c tinty -n "__fish_tinty_using_subcommand current" -s c -l config -d 'Optional path to the tinty config.toml file' -r
complete -c tinty -n "__fish_tinty_using_subcommand current" -s d -l data-dir -d 'Optional path to the tinty data directory' -r
complete -c tinty -n "__fish_tinty_using_subcommand current" -s h -l help -d 'Print help'
complete -c tinty -n "__fish_tinty_using_subcommand generate-completion" -s c -l config -d 'Optional path to the tinty config.toml file' -r
complete -c tinty -n "__fish_tinty_using_subcommand generate-completion" -s d -l data-dir -d 'Optional path to the tinty data directory' -r
complete -c tinty -n "__fish_tinty_using_subcommand generate-completion" -s h -l help -d 'Print help'
complete -c tinty -n "__fish_tinty_using_subcommand generate-scheme" -l author -d 'Scheme author info (name, email, etc) to write, defaults to \'Tinty\'' -r -f
complete -c tinty -n "__fish_tinty_using_subcommand generate-scheme" -l description -d 'Scheme description info' -r -f
complete -c tinty -n "__fish_tinty_using_subcommand generate-scheme" -l name -d 'Scheme display name (can include spaces and capitalization). Defaults to \'Tinty Generated\'' -r -f
complete -c tinty -n "__fish_tinty_using_subcommand generate-scheme" -l slug -d 'Scheme slug (the name you specify when applying schemes). Can not contain white-space or capitalization. Defaults to \'tinty-generated\'' -r -f
complete -c tinty -n "__fish_tinty_using_subcommand generate-scheme" -l system -d 'Whether to generate a base16 or base24 scheme' -r -f -a "base16\t''
base24\t''"
complete -c tinty -n "__fish_tinty_using_subcommand generate-scheme" -l variant -d 'Whether to generate a dark or light scheme' -r -f -a "dark\t''
light\t''"
complete -c tinty -n "__fish_tinty_using_subcommand generate-scheme" -s c -l config -d 'Optional path to the tinty config.toml file' -r
complete -c tinty -n "__fish_tinty_using_subcommand generate-scheme" -s d -l data-dir -d 'Optional path to the tinty data directory' -r
complete -c tinty -n "__fish_tinty_using_subcommand generate-scheme" -l save -d 'Whether to add the scheme to the installed schemes.'
complete -c tinty -n "__fish_tinty_using_subcommand generate-scheme" -s h -l help -d 'Print help'
complete -c tinty -n "__fish_tinty_using_subcommand info" -s c -l config -d 'Optional path to the tinty config.toml file' -r
complete -c tinty -n "__fish_tinty_using_subcommand info" -s d -l data-dir -d 'Optional path to the tinty data directory' -r
complete -c tinty -n "__fish_tinty_using_subcommand info" -l custom-schemes -d 'Lists availabile custom schemes'
complete -c tinty -n "__fish_tinty_using_subcommand info" -s h -l help -d 'Print help'
complete -c tinty -n "__fish_tinty_using_subcommand init" -s c -l config -d 'Optional path to the tinty config.toml file' -r
complete -c tinty -n "__fish_tinty_using_subcommand init" -s d -l data-dir -d 'Optional path to the tinty data directory' -r
complete -c tinty -n "__fish_tinty_using_subcommand init" -l verbose -d 'Print to stdout'
complete -c tinty -n "__fish_tinty_using_subcommand init" -s h -l help -d 'Print help'
complete -c tinty -n "__fish_tinty_using_subcommand list" -s c -l config -d 'Optional path to the tinty config.toml file' -r
complete -c tinty -n "__fish_tinty_using_subcommand list" -s d -l data-dir -d 'Optional path to the tinty data directory' -r
complete -c tinty -n "__fish_tinty_using_subcommand list" -l custom-schemes -d 'Lists availabile custom schemes'
complete -c tinty -n "__fish_tinty_using_subcommand list" -l json -d 'Output as JSON'
complete -c tinty -n "__fish_tinty_using_subcommand list" -s h -l help -d 'Print help'
complete -c tinty -n "__fish_tinty_using_subcommand config" -s c -l config -d 'Optional path to the tinty config.toml file' -r
complete -c tinty -n "__fish_tinty_using_subcommand config" -s d -l data-dir -d 'Optional path to the tinty data directory' -r
complete -c tinty -n "__fish_tinty_using_subcommand config" -l config-path -d 'Returns path to the tinty config file'
complete -c tinty -n "__fish_tinty_using_subcommand config" -l data-dir-path -d 'Returns path to the tinty data directory'
complete -c tinty -n "__fish_tinty_using_subcommand config" -s h -l help -d 'Print help'
complete -c tinty -n "__fish_tinty_using_subcommand apply" -s c -l config -d 'Optional path to the tinty config.toml file' -r
complete -c tinty -n "__fish_tinty_using_subcommand apply" -s d -l data-dir -d 'Optional path to the tinty data directory' -r
complete -c tinty -n "__fish_tinty_using_subcommand apply" -s q -l quiet -d 'Silence stdout'
complete -c tinty -n "__fish_tinty_using_subcommand apply" -s h -l help -d 'Print help'
complete -c tinty -n "__fish_tinty_using_subcommand install" -s c -l config -d 'Optional path to the tinty config.toml file' -r
complete -c tinty -n "__fish_tinty_using_subcommand install" -s d -l data-dir -d 'Optional path to the tinty data directory' -r
complete -c tinty -n "__fish_tinty_using_subcommand install" -s q -l quiet -d 'Silence stdout'
complete -c tinty -n "__fish_tinty_using_subcommand install" -s h -l help -d 'Print help'
complete -c tinty -n "__fish_tinty_using_subcommand update" -s c -l config -d 'Optional path to the tinty config.toml file' -r
complete -c tinty -n "__fish_tinty_using_subcommand update" -s d -l data-dir -d 'Optional path to the tinty data directory' -r
complete -c tinty -n "__fish_tinty_using_subcommand update" -s q -l quiet -d 'Silence stdout'
complete -c tinty -n "__fish_tinty_using_subcommand update" -s h -l help -d 'Print help'
complete -c tinty -n "__fish_tinty_using_subcommand sync" -s c -l config -d 'Optional path to the tinty config.toml file' -r
complete -c tinty -n "__fish_tinty_using_subcommand sync" -s d -l data-dir -d 'Optional path to the tinty data directory' -r
complete -c tinty -n "__fish_tinty_using_subcommand sync" -s q -l quiet -d 'Silence stdout'
complete -c tinty -n "__fish_tinty_using_subcommand sync" -s h -l help -d 'Print help'
complete -c tinty -n "__fish_tinty_using_subcommand cycle" -s c -l config -d 'Optional path to the tinty config.toml file' -r
complete -c tinty -n "__fish_tinty_using_subcommand cycle" -s d -l data-dir -d 'Optional path to the tinty data directory' -r
complete -c tinty -n "__fish_tinty_using_subcommand cycle" -s q -l quiet -d 'Silence stdout'
complete -c tinty -n "__fish_tinty_using_subcommand cycle" -s h -l help -d 'Print help'
complete -c tinty -n "__fish_tinty_using_subcommand help; and not __fish_seen_subcommand_from build current generate-completion generate-scheme info init list config apply install update sync cycle help" -f -a "build" -d 'Builds the target theme template'
complete -c tinty -n "__fish_tinty_using_subcommand help; and not __fish_seen_subcommand_from build current generate-completion generate-scheme info init list config apply install update sync cycle help" -f -a "current" -d 'Prints the last scheme name applied or specific values from the current scheme'
complete -c tinty -n "__fish_tinty_using_subcommand help; and not __fish_seen_subcommand_from build current generate-completion generate-scheme info init list config apply install update sync cycle help" -f -a "generate-completion" -d 'Generates a shell completion script'
complete -c tinty -n "__fish_tinty_using_subcommand help; and not __fish_seen_subcommand_from build current generate-completion generate-scheme info init list config apply install update sync cycle help" -f -a "generate-scheme" -d 'Generates a scheme based on an image'
complete -c tinty -n "__fish_tinty_using_subcommand help; and not __fish_seen_subcommand_from build current generate-completion generate-scheme info init list config apply install update sync cycle help" -f -a "info" -d 'Shows scheme colors for all schemes matching <scheme_system>-<scheme_name> (Eg: tinty info base16-mocha)'
complete -c tinty -n "__fish_tinty_using_subcommand help; and not __fish_seen_subcommand_from build current generate-completion generate-scheme info init list config apply install update sync cycle help" -f -a "init" -d 'Initializes with the exising config. Used to Initialize exising theme for when your shell starts up'
complete -c tinty -n "__fish_tinty_using_subcommand help; and not __fish_seen_subcommand_from build current generate-completion generate-scheme info init list config apply install update sync cycle help" -f -a "list" -d 'Lists available schemes'
complete -c tinty -n "__fish_tinty_using_subcommand help; and not __fish_seen_subcommand_from build current generate-completion generate-scheme info init list config apply install update sync cycle help" -f -a "config" -d 'Provides config related information'
complete -c tinty -n "__fish_tinty_using_subcommand help; and not __fish_seen_subcommand_from build current generate-completion generate-scheme info init list config apply install update sync cycle help" -f -a "apply" -d 'Applies a theme based on the chosen scheme'
complete -c tinty -n "__fish_tinty_using_subcommand help; and not __fish_seen_subcommand_from build current generate-completion generate-scheme info init list config apply install update sync cycle help" -f -a "install" -d 'Install the environment needed for tinty'
complete -c tinty -n "__fish_tinty_using_subcommand help; and not __fish_seen_subcommand_from build current generate-completion generate-scheme info init list config apply install update sync cycle help" -f -a "update" -d 'Update to the latest themes'
complete -c tinty -n "__fish_tinty_using_subcommand help; and not __fish_seen_subcommand_from build current generate-completion generate-scheme info init list config apply install update sync cycle help" -f -a "sync" -d 'Install missing templates in tinty/config.toml and update existing templates'
complete -c tinty -n "__fish_tinty_using_subcommand help; and not __fish_seen_subcommand_from build current generate-completion generate-scheme info init list config apply install update sync cycle help" -f -a "cycle" -d 'Cycle through your preferred themes'
complete -c tinty -n "__fish_tinty_using_subcommand help; and not __fish_seen_subcommand_from build current generate-completion generate-scheme info init list config apply install update sync cycle help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
