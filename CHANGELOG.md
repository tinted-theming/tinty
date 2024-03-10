# Changelog

## [0.11.0] - 2024-03-10

### Added

- Add config.toml root-level property "hooks" (`Array<String>`) which
  executes the provided strings after a scheme is applied
- Add colour styling to `--help` text
- Add instructions to install via Homebrew

### Fixed

- Fix bug where item `hook` template variable `%f` returns a path to
  theme file in the repository template instead of the copied version
  under `~/.local/share/tinted-theming/tinty`

### Updated

- Remove unnecessary helper function and optimise code

## [0.10.1] - 2024-02-20

- Fix bug where spaces in config or data directory paths would cause
  `install` and `update` to fail
- Fix bug so now tinty works without a `config.toml` file being provided

## [0.10.0] - 2024-02-19

- **Breaking**: Change `--config` flag to accept a path to config file
  and not a directory containing a `config.toml`
- Add `--data-dir` flag to allow for manually setting data directory

## [0.9.0] - 2024-02-18

- **Breaking**: `set` subcommand renamed to `apply`
- **Breaking**: `setup` subcommand renamed to `install`

## [0.8.1] - 2024-02-17

- Fix visual `tinty info` spacing bug

## [0.8.0] - 2024-02-17

- Add `info` subcommand to list scheme metadata as well as scheme colors

## [0.7.0] - 2024-02-16

- Add `current` subcommand to print the last scheme name set.

## [0.6.0] - 2024-02-15

- Change config.toml properties to use dashes instead of underscores.
  Properties changes are:
  - `default_theme` => `default_theme`
  - `themes_dir` => `themes-dir`
  - `supported_systems` => `supported-systems`

## [0.5.0] - 2024-02-14

- Change `config.toml` `items.system` to `items.supported_systems` which
  now accepts an array of strings instead of a string. This allows for
  using a single template repo for setting both base16 and base24
  themes.

## [0.4.0] - 2024-02-11

- Enforces config.toml `[[items]]` `name` property is a unique value to
  prevent dirname conflicts
- Removes config.toml `[[items]]` `git_url` property
- Adds config.toml `[[items]]` `path` property which supports git URLs
  as well as path to local repo dir

## [0.3.0] - 2024-02-10

- Add support for base24 templates

## [0.2.1] - 2024-02-08

- Fix bug where `tinty --version` displays incorrect version number

## [0.2.0] - 2024-02-07

- Generate `tinty list` from local version of
  https://github.com/tinted-theming/schemes/base16
- `tinty list` now displays schemes prepended by their `system`.
  `ocean` -> `base16-ocean`

## [0.1.0] - 2024-02-06

- Initial release

[0.10.1]: https://github.com/tinted-theming/tinty/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/tinted-theming/tinty/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/tinted-theming/tinty/compare/v0.8.1...v0.9.0
[0.8.1]: https://github.com/tinted-theming/tinty/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/tinted-theming/tinty/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/tinted-theming/tinty/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/tinted-theming/tinty/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/tinted-theming/tinty/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/tinted-theming/tinty/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/tinted-theming/tinty/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/tinted-theming/tinty/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/tinted-theming/tinty/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/tinted-theming/tinty/releases/tag/v0.1.0
