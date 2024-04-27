
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'tinty' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'tinty'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'tinty' {
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Optional path to the tinty config.toml file')
            [CompletionResult]::new('--config', 'config', [CompletionResultType]::ParameterName, 'Optional path to the tinty config.toml file')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Optional path to the tinty data directory')
            [CompletionResult]::new('--data-dir', 'data-dir', [CompletionResultType]::ParameterName, 'Optional path to the tinty data directory')
            [CompletionResult]::new('--generate-completion', 'generate-completion', [CompletionResultType]::ParameterName, 'Generate completion scripts')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', 'V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('current', 'current', [CompletionResultType]::ParameterValue, 'Prints the last scheme name applied')
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'Shows scheme colors for all schemes matching <scheme_system>-<scheme_name> (Eg: tinty info base16-mocha)')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initializes with the exising config. Used to Initialize exising theme for when your shell starts up')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'Lists available schemes')
            [CompletionResult]::new('apply', 'apply', [CompletionResultType]::ParameterValue, 'Applies a theme based on the chosen scheme')
            [CompletionResult]::new('install', 'install', [CompletionResultType]::ParameterValue, 'Install the environment needed for tinty')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Update to the latest themes')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'tinty;current' {
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Optional path to the tinty config.toml file')
            [CompletionResult]::new('--config', 'config', [CompletionResultType]::ParameterName, 'Optional path to the tinty config.toml file')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Optional path to the tinty data directory')
            [CompletionResult]::new('--data-dir', 'data-dir', [CompletionResultType]::ParameterName, 'Optional path to the tinty data directory')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'tinty;info' {
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Optional path to the tinty config.toml file')
            [CompletionResult]::new('--config', 'config', [CompletionResultType]::ParameterName, 'Optional path to the tinty config.toml file')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Optional path to the tinty data directory')
            [CompletionResult]::new('--data-dir', 'data-dir', [CompletionResultType]::ParameterName, 'Optional path to the tinty data directory')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'tinty;init' {
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Optional path to the tinty config.toml file')
            [CompletionResult]::new('--config', 'config', [CompletionResultType]::ParameterName, 'Optional path to the tinty config.toml file')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Optional path to the tinty data directory')
            [CompletionResult]::new('--data-dir', 'data-dir', [CompletionResultType]::ParameterName, 'Optional path to the tinty data directory')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'tinty;list' {
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Optional path to the tinty config.toml file')
            [CompletionResult]::new('--config', 'config', [CompletionResultType]::ParameterName, 'Optional path to the tinty config.toml file')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Optional path to the tinty data directory')
            [CompletionResult]::new('--data-dir', 'data-dir', [CompletionResultType]::ParameterName, 'Optional path to the tinty data directory')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'tinty;apply' {
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Optional path to the tinty config.toml file')
            [CompletionResult]::new('--config', 'config', [CompletionResultType]::ParameterName, 'Optional path to the tinty config.toml file')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Optional path to the tinty data directory')
            [CompletionResult]::new('--data-dir', 'data-dir', [CompletionResultType]::ParameterName, 'Optional path to the tinty data directory')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'tinty;install' {
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Optional path to the tinty config.toml file')
            [CompletionResult]::new('--config', 'config', [CompletionResultType]::ParameterName, 'Optional path to the tinty config.toml file')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Optional path to the tinty data directory')
            [CompletionResult]::new('--data-dir', 'data-dir', [CompletionResultType]::ParameterName, 'Optional path to the tinty data directory')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'tinty;update' {
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Optional path to the tinty config.toml file')
            [CompletionResult]::new('--config', 'config', [CompletionResultType]::ParameterName, 'Optional path to the tinty config.toml file')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Optional path to the tinty data directory')
            [CompletionResult]::new('--data-dir', 'data-dir', [CompletionResultType]::ParameterName, 'Optional path to the tinty data directory')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'tinty;help' {
            [CompletionResult]::new('current', 'current', [CompletionResultType]::ParameterValue, 'Prints the last scheme name applied')
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'Shows scheme colors for all schemes matching <scheme_system>-<scheme_name> (Eg: tinty info base16-mocha)')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initializes with the exising config. Used to Initialize exising theme for when your shell starts up')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'Lists available schemes')
            [CompletionResult]::new('apply', 'apply', [CompletionResultType]::ParameterValue, 'Applies a theme based on the chosen scheme')
            [CompletionResult]::new('install', 'install', [CompletionResultType]::ParameterValue, 'Install the environment needed for tinty')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Update to the latest themes')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'tinty;help;current' {
            break
        }
        'tinty;help;info' {
            break
        }
        'tinty;help;init' {
            break
        }
        'tinty;help;list' {
            break
        }
        'tinty;help;apply' {
            break
        }
        'tinty;help;install' {
            break
        }
        'tinty;help;update' {
            break
        }
        'tinty;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
