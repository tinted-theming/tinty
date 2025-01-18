# Changelog

## Unreleased

### Added

- Support `--json` option in `tinty list`.


## [0.25.0] - 2025-01-14

### Added
 
- Support `[[items]] revision` property that accepts a tag, branch, or commit SHA1.

### Changed

- `tinty update` now keeps the items' `origin` remote URLs up-to-date.
- The item repositories are now checked out to a specific commit SHA1 in "detached HEAD" mode.

### Fixed

- Fix bug where period preceeding extension is still added to generated files even when an extension doesn't exist

## [0.24.0] - 2024-12-20

### Changed

- Update Cargo dependencies
- Update to latest tinted-scheme-extractor which alters how it adjusts
  colors slightly to ensure >= `base08` aren't too dark compared to
  `base00`

### Fixed

- In tests, use `CARGO_BIN_EXE_tinty` env var for the binary path
  instead of assuming it exists in `./target/release` directory
- Upgrade `url` crate dependency due to security vulnerability

## [0.23.0] - 2024-11-17

### Changed

- Change `tinty generate-scheme` API by removing the `OUTFILE` option
  and only printing to stdout or saving to the tinty data directory with
  the `--save` flag

### Fixed

- Fix bug where `tinty info` doesn't list schemes correctly when the
  `palette` hex values contain a `#` prefix

## [0.22.0] - 2024-10-09

### Added

- Support arguments with `current` subcommand to allow consumers to get
  specific current scheme data
- Add hook string template variable (`%o`) to print the current
  command operation executing the hook

## [0.21.1] - 2024-10-02

### Fixed

- Fix bug where `tinty build` subcommand doesn't support `filename`
  config.yaml property

## [0.21.0] - 2024-10-01

### Changed

- BREAKING: MacOS breaking change only since Tinty now uses `XDG` paths
  for config and data directories while falling back to `XDG` defaults 
  for these dirs if the environment variables aren't set. This is how 
  Tinty functions on Linux already

## [0.20.1] - 2024-09-25

### Fixed

- Fix bug where `tinted-builder-rust` displays build information by
  default when `tinty apply` is run

## [0.20.0] - 2024-09-25

### Added

- Add `--quiet` flag for `apply`, `install` and `update` subcommands
- Add `sync` quality-of-life subcommand combining `install` and `update`
  subcommands

### Fixed

- Fix bug where Tinty won't update after custom schemes have been built
  in local templates

## [0.19.0] - 2024-09-23

### Added

- Add `--quiet` flag for `build` subcommand
- Add `--verbose` flag for `init` subcommand

### Changed

- Update dependencies

### Fixed

- Fix bug where `scheme_partial_name` doesn't render correctly in all
  situations when a scheme name exists in schemes and custom schemes

## [0.18.0] - 2024-07-12

### Added

- Add `--custom-schemes` flag support for `tinty info`

### Fixed

- Fix bug where `tinty generate-scheme` would generate schemes without
  scheme values being wrapped in double quotes
- Fix error message when `tinty list --custom-schemes` when no custom
  schemes exist

## [0.17.0] - 2024-07-03

### Added

- Add `tinty generate-scheme` command to generate a scheme based on
  provided image input, with the included functionality of saving this
  scheme file and applying it with `tinty apply`.
- Add `--custom-schemes` flag for `tinty list` subcommand to list custom
  schemes saved with `tinnty generate-scheme`.

## [0.16.0] - 2024-07-02

### Added

- Add `theme-file-extension` option in item config to allow users to
  define a custom theme extension that isn't `/\.*$/`

## [0.15.0] - 2024-06-11

### Changed

- Remove stderr comment output for `tinty generate-completion`
- Update to latest tinted-builder using newly created ribboncurls
  mustache template rendering engine.

## [0.14.0] - 2024-05-07

### Added

- Add `tinty build` subcommand to build a base16 or base24 template
  directory

## [0.13.0] - 2024-05-07

### Added

- Add `tinty config` subcommand with flags to return config related
  information

## [0.12.0] - 2024-04-29

### Added

- Add shell completions functionality with `generate-completion`
  subcommand

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

[0.25.0]: https://github.com/tinted-theming/tinty/compare/v0.24.0...v0.25.0
[0.24.0]: https://github.com/tinted-theming/tinty/compare/v0.23.0...v0.24.0
[0.23.0]: https://github.com/tinted-theming/tinty/compare/v0.22.0...v0.23.0
[0.22.0]: https://github.com/tinted-theming/tinty/compare/v0.21.1...v0.22.0
[0.21.1]: https://github.com/tinted-theming/tinty/compare/v0.21.0...v0.21.1
[0.21.0]: https://github.com/tinted-theming/tinty/compare/v0.20.1...v0.21.0
[0.20.1]: https://github.com/tinted-theming/tinty/compare/v0.20.0...v0.20.1
[0.20.0]: https://github.com/tinted-theming/tinty/compare/v0.19.0...v0.20.0
[0.19.0]: https://github.com/tinted-theming/tinty/compare/v0.18.0...v0.19.0
[0.18.0]: https://github.com/tinted-theming/tinty/compare/v0.17.0...v0.18.0
[0.17.0]: https://github.com/tinted-theming/tinty/compare/v0.16.0...v0.17.0
[0.16.0]: https://github.com/tinted-theming/tinty/compare/v0.15.0...v0.16.0
[0.15.0]: https://github.com/tinted-theming/tinty/compare/v0.14.0...v0.15.0
[0.14.0]: https://github.com/tinted-theming/tinty/compare/v0.13.0...v0.14.0
[0.13.0]: https://github.com/tinted-theming/tinty/compare/v0.12.0...v0.13.0
[0.12.0]: https://github.com/tinted-theming/tinty/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/tinted-theming/tinty/compare/v0.10.1...v0.11.0
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
