# Tinty - A Tinted Theming CLI tool written in Rust 🦀

[![Matrix Chat](https://img.shields.io/matrix/tinted-theming:matrix.org)](https://matrix.to/#/#tinted-theming:matrix.org)
[![Crates.io](https://img.shields.io/crates/v/tinty.svg)](https://crates.io/crates/tinty)
[![Tests](https://github.com/tinted-theming/tinty/actions/workflows/ci.yml/badge.svg)](https://github.com/tinted-theming/tinty/actions/workflows/ci.yml)
[![LICENSE](https://img.shields.io/crates/l/tinty)](./LICENSE)

<p align="center">
  <img
    src="https://raw.githubusercontent.com/tinted-theming/tinty/main/mascot.png"
    alt="Tinty mascot" height="200" width="200" />
</p>

Change the theme of your terminal, text editor and anything else with
one command. Immediately switch between over 250 themes!

Tinty is a [Base16] and [Base24] theming manager for all [Tinted
Theming] projects including any third-party template repository that
follows the [Base16 builder specification].

To learn more about [Tinted Theming] and the base16 specification, have
a look at our [home repository] and preview the themes supported by
Tinty have a look at our [Tinted Gallery].

## Table of contents

- [How it works](#how-it-works)
- [Installation](#installation)
- [Basic usage](#basic-usage)
- [Usage](#usage)
- [Configuration](#configuration)
- [Migration from Flavours](#migration-from-flavours)
- [Debugging Tinty issues](#debugging-tinty-issues)
- [Contributing](#contributing)
- [License](#license)

## How it Works

At its core, Tinty is designed to simplify the theming process across
different tools and environments by leveraging the power of [Base16] and
[Base24] template themes. Here's a closer look at how Tinty operates:

### Theme Application Process

When you setup Tinty and `apply` a theme, Tinty performs several steps
to ensure that your selected theme is seamlessly integrated across your
specified applications:

1. **Configuration Loading**: Tinty starts by reading your `config.toml`
   file to understand your settings and the specific theming components
   (`[[items]]`) you've defined. This configuration dictates everything
   from which shell to use for executing hooks to what themes and templates
   are applied.

2. **Theme Repository Management**: For each item in your configuration,
   Tinty checks if the necessary theme template repository is already
   cloned to your local machine. If not, it clones the repository to
   `~/.local/share/tinted-theming/tinty`, ensuring that the latest themes
   are always at your fingertips.

3. **Theme Copying**: Once the repositories are set up, Tinty copies the
   relevant theme files from each template repository based on the
   scheme you've chosen to apply. This step gathers all necessary template
   theme files in one place.

4. **Executed Hooks**: With all theme files ready, Tinty then executes
   the optional shell hooks specified in your `config.toml`. These hooks
   might apply the theme directly (e.g., by sourcing a shell script) or
   perform additional actions like copying theme files to specific
   locations. This is where the actual theme application occurs, affecting
   your terminal, text editor, and any other tools you've configured.

### Beyond Basic Theming

Tinty's functionality extends beyond applying themes:

- **Scheme Listing and Information**: Tinty can list all available
  schemes and provide detailed information about them. This feature helps
  you explore and choose from over 250 themes supported by Tinty,
  including those from third-party repositories following the [Base16
  builder specification].

- **Dynamic Updates**: The `tinty update` executes a `git pull` command
  on your local copy of theme template repositories. This ensures that you
  have access to the latest themes and updates from the community.

- **Initialization and Persistence**: Using `tinty init`, the tool can
  reapply the last used theme or a default scheme at startup, making your
  theming preferences persistent across sessions.

### Designed for Flexibility

Tinty is built with flexibility in mind, accommodating a wide range of
theming needs and preferences. Whether you're looking to quickly switch
themes across multiple tools, explore new color schemes, or ensure a
consistent look and feel in your development environment, Tinty provides
the necessary mechanisms to make it happen.

By understanding the sequence of actions Tinty performs and the options
available to you, you can tailor the theming process to suit your
workflow, making your experience more enjoyable and productive.

## Installation

### Cargo

```sh
cargo install tinty
```

### Homebrew

```sh
brew tap tinted-theming/tinted
brew install tinty
```

### Arch Linux

```sh
paru -S tinty-git
```

### Binaries

Download the relevant binary from the [repository releases] page.

### Manual

```sh
git clone https://github.com/tinted-theming/tinty path/to/tinty
cd path/to/tinty
make install
make build
cp target/release/tinty /path/to/bin/dir
```

## Basic usage

For the most basic usage without configuration, install Tinty and run
the following to apply `base16-mocha`:

```sh
tinty sync # Required initially for setup and when your `tinty/config.toml` is updated
tinty apply base16-mocha
```

To get a list of [schemes] and more information about the colors:

```sh
tinty list
tinty info base16-oceanicnext
```

Without any `config.toml` file, `tinty` will apply your shell theme
using [tinted-shell] using `sh` shell.

## Usage

For advanced usage and setup, have a look at the [USAGE.md] document.

### CLI

The following is a table of the available subcommands for the CLI tool (Tinty), including the descriptions and any notable arguments.

| Subcommand | Description                                         | Arguments            | Example Usage                              |
|------------|-----------------------------------------------------|----------------------|--------------------------------------------|
| `sync`     | Installs and updates schemes and templates defined in `tinty/config.toml` | - | `tinty sync` |
| `list`     | Lists all available themes. | Optional argument `--custom-schemes` to list saved custom theme files using `tinty generate-scheme`.<br>Optional argument `--json` to output more info about each scheme in JSON form | `tinty list` |
| `apply`    | Applies a specific theme. | `<scheme_system>-<scheme_name>`: Name of the system and scheme to apply. | `tinty apply base16-mocha` |
| `cycle`    | Applies the next theme among your preferred ones. See [Configuration](#configuration).  |  | `tinty cycle` |
| `init`     | Initializes the tool with the last applied theme otherwise `default-scheme` from `config.toml`. | - | `tinty init` |
| `current`  | Displays the currently applied theme or current theme values. | `<scheme_property_name>` (Optional argument with the following supported values: `author` \| `description` \| `name` \| `slug` \| `system` \| `variant`) | `tinty current` |
| `config`   | Displays config related information currently in use by Tinty. Without flags it returns `config.yml` content. | - | `tinty config` |
| `info`     | Provides information about themes. | `[<scheme_system>-<scheme_name>]`: Optional argument `--custom-schemes` to provide information on any custom schemes | `tinty info base16-mocha` |
| `build`    | Builds the provided base16 or base24 template using [tinted-builder-rust]. | `<DIR>`: Path to the base16 or base24 template directory. | `tinty build path/to/tinted-tmux` |
| `generate-completion` | Generates a shell completion file to source in your shell startup file (`*rc`). | `<shell_name>`: Name of the shell to generate a completion script for. Supports `bash`, `elvish`, `fish`, `powershell`, `zsh` | `tinty generate-completion bash` |
| `generate-scheme` | Generates a yaml scheme file with colors inferred from provided image. | `<image_path>`: Path to image. Prints to stdout unless `--save` is provided which saves to `~/.local/share/tinted-theming/tinty/custom-schemes` for use within Tinty | `tinty generate-scheme --system=base16 --save /path/to/image.png` |
| `install`  | Installs requirements for the configuration. (Use `tinty sync`) | - | `tinty install` |
| `update`   | Updates the templates and schemes. (Use `tinty sync`) | - | `tinty update` |

Some subcommands support additional flags and options to modify their behavior:

| Flag/Option        | Description                             | Applicable Subcommands | Default Value | Example Usage                             |
|--------------------|-----------------------------------------|------------------------|---------------|-------------------------------------------|
| `--config` `-c`    | Specifies a custom configuration file path. | All | If not provided tinty looks for `config.toml` at `$XDG_CONFIG_HOME/tinted-theming/tinty/config.toml` otherwise `~/.config/tinted-theming/tinty/config.toml` | `tinty apply base16-ayu-dark --config /path/to/custom/config.toml` |
| `--data-dir`       | Specifies a custom path for the data directory. | All | If not provided tinty looks for the data directory at `$XDG_DATA_HOME/tinted-theming/tinty` otherwise `~/.local/share/tinted-theming/tinty` | `tinty install --data-dir /path/to/custom/data-dir` |
| `--help` `-h`      | Displays help information for the subcommand. | All | - | `tinty --help`, `tinty apply --help`, etc. |
| `--version` `-V`   | Shows the version of tinty. | All | - | `tinty --version` |
| `--config-path`    | Shows the config.yml path. | `config` | - | `tinty config --config-path` |
| `--data-dir-path`  | Shows the data directory path. | `config` | - | `tinty config --data-dir-path` |
| `--custom-schemes` | Lists saved custom theme files manually created or generated by `tinty generate-scheme` | `list` | - | `tinty list --custom-schemes` |
| `--quiet`          | Boolean flag which silences stdout prints | `apply`, `build`, `install`, `update`, `sync` | `false` | `tinty build . --quiet` |

## Configuration

The `config.toml` file allows you to customize and configure the
behavior of the application, enabling personalized themes and
functionalities. This file specifies shell settings, the default theme
scheme, and configurations for various items such as terminal, editor
themes, or any other supported application.

### Global `config.toml` table Schema

Below, you'll find the global configuration options for `config.toml`.
These settings apply to the overall operation of Tinty and include
essential configurations such as the default shell command and the
default theme scheme. Setting these options tailors the Tinty experience
to your preferences and environment.

| Key               | Type               | Required | Description                                                                            | Default | Example |
|-------------------|--------------------|----------|----------------------------------------------------------------------------------------|---------|---------|
| `shell`           | `string`           | Optional | Specifies the shell command used to execute hooks. | `"sh -c '{}'"` | `shell = "bash -c '{}'"` |
| `default-scheme`  | `string`           | Optional | Defines the default theme scheme to be applied if no specific scheme is set. | None | `default-scheme = "base16-mocha"` |
| `preferred-schemes`  | `array<string>`           | Optional | A list of your favorite theme schemes you'd like to cycle through  | `[]` | `preferred-schemes = ["base16-gruvbox-dark", "base16-gruvbox-light"]` |
| `hooks`           | `array<string>`    | Optional | A list of strings which are executed after every `tinty apply` | None | `hooks = ["echo \"The current scheme is: $(tinty current)\""]` |
| `[[items]]`       | `array<items>`     | Required | An array of `items` configurations. Each item represents a themeable component. Detailed structure provided in the next section. | - | - |

#### hooks

**New in Tinty 0.29+**: Theme & color values are now available to hooks as environment variables:


| Variable name | Description |
|---------------|-------------|
| `TINTY_THEME_FILE_PATH` | Path to the theme file for that `[[items]]` entry<br>e.g. `/home/user/.local/share/tinted-theming/tinty/tinted-alacritty-colors-file.toml` |
| `TINTY_THEME_OPERATION` | The command operation that is running the hook e.g. `apply` or `init`  |
| `TINTY_SCHEME_ID` | The unique name of the applied theme e.g. `base16-ayu-dark` |
| `TINTY_SCHEME_SYSTEM` | The system-part of the theme ID e.g. `base16` or `base24` |
| `TINTY_SCHEME_SLUG` | The slug-part of the theme ID e.g. `ayu-dark` |
| `TINTY_SCHEME_VARIANT` | Whether the theme is a light or dark variant e.g. `light` or `dark` |
| `TINTY_SCHEME_PALETTE_BASE{*}_HEX_{R,G,B}` | The R, G, or B value of the given `base*` color, in hex<br>e.g. `$TINTY_SCHEME_PALETTE_BASE01_HEX_R` is the red color hex value of the applied theme's `base01` color  |
| `TINTY_SCHEME_PALETTE_BASE{*}_RGB_{R,G,B}` | The R, G, or B value of the given `base*` color, in `0-255` range<br>e.g. `$TINTY_SCHEME_PALETTE_BASE08_RGB_G` is the green color `{0...255}` value of the applied theme's `base08` color  |
| `TINTY_SCHEME_PALETTE_BASE{*}_DEC_{R,G,B}` | The R, G, or B value of the given `base*` color, in `0-1.0` floating-point range<br>e.g. `$TINTY_SCHEME_PALETTE_BASE0C_DEC_B` is the blue color `{0...1.0}` float value of the applied theme's `base0C` color   |
| `TINTY_SCHEME_LIGHTNESS_FOREGROUND` | The lightness value of the applied theme's foreground color e.g. `89.47651`  |
| `TINTY_SCHEME_LIGHTNESS_BACKGROUND` | The lightness value of the applied theme's background color e.g. `6.0706663`    |


##### Examples

Copy an applied theme to a path on your system:

  ```toml
  # Copy the generated Alacritty theme file to the expected destination:
  hook = "cp -f \"$TINTY_THEME_FILE_PATH ~/.config/alacritty/colors.yml"
  ```

Send info about the applied theme somewhere:

  ```toml
  hook = "echo 'Theme $TINTY_SCHEME_ID applied' | tee -a /var/log/tinty-theme-history.log | xargs notify-send"
  ```

Conditionally run hooks based on the operation:

  ```toml
  # Only change Alacritty theme on tinty apply, not on init.
  hook = "[ \"$TINTY_THEME_OPERATION\" = \"apply\" ] && cp -f \"$TINTY_THEME_FILE_PATH\" ~/.config/alacritty/colors.yml"
  ```

Run programs that read colorscheme info

```toml
hook = "ruby /path/to/my/tinty_hook.rb" # Reads stuff from ENV['TINTY_****']
```

#### `printf`-style formatting

Only a limited set of variables are available to Tinty 0.28 & lower. They came in the form of `%` format specifiers:

* `%f` - Equivalent to `$TINTY_THEME_FILE_PATH`
* `%n` - Equivalent to `$TINTY_SCHEME_ID`
* `%o` - Equivalent to `$TINTY_THEME_OPERATION`

These still work, but are now deprecated & subject to removal in a future Tinty version.


### Items table `config.toml` Schema

The `[[items]]` section within `config.toml` allows for detailed
customization of individual themeable components. Each item represents a
specific element you can theme, such as a text editor or terminal. The
table below outlines the structure for these items, including how to
specify templates, directories for theme files, and hooks for applying
themes. Configuring items effectively enables you to manage multiple
themes across different applications seamlessly.

| Key                    | Type     | Required | Description                                                   | Default | Example                                    |
|------------------------|----------|----------|---------------------------------------------------------------|---------|--------------------------------------------|
| `name`                 | `string` | Required | A unique name for the item being configured.                  | - | `name = "vim"`                             |
| `path`                 | `string` | Required | The file system path or URL to the theme template repository. Paths beginning with `~/` map to home dir. | - | `path = "https://github.com/tinted-tmux"` |
| `revision`             | `string` | Optional | The Git revision to use.<br> Accepts a branch name, a tag, or a commit SHA1 | `main` | `revision = "1.2.0"` |
| `themes-dir`           | `string` | Required | The directory within the repository where theme files are located. | - | `themes-dir = "colors"`                    |
| `hook`                 | `string` | Optional | A command to be executed after the theme is applied. Useful for reloading configurations. | None    | `hook = "source ~/.vimrc"` |
| `theme-file-extension` | `string` | Optional | Define a custom theme file extension that isn't `/\.*$/`. Tinty looks for themes named `base16-uwunicorn.*` (for example), but when the theme file isn't structured that way, this option can help specify the pattern. | - | `theme-file-extension = ".module.css"` |
| `supported-systems`    | `array<"base16" or "base24">` | Optional | Defines which theming systems ("base16" and or "base24") are supported by the item. | `["base16"]` | `supported-systems = ["base16", "base24"]` |

#### Note on `supported-systems`

The `supported-systems` key within an `[[items]]` table allows for
specifying compatibility with different theming systems. The available
options are `"base16"` and `"base24"`, indicating support for [Base16]
and [Base24] theming systems, respectively. If the template repository
does not support a system, it should not be included in this property.

The `[[items]]` configuration allows defining multiple themeable
components, each with its own set of configurations as described above.
Here's how you might define multiple items in your `config.toml`:

### Full Configuration Example

Here's a complete `config.toml` example demonstrating how to configure
multiple items along with global settings:

```toml
# Global settings
shell = "zsh -c '{}'"
default-scheme = "base16-mocha"
preferred-schemes = ["base16-gruvbox-dark", "base16-gruvbox-light", "base16-github-dark"]

# Item configurations
[[items]]
name = "tinted-shell"
path = "https://github.com/tinted-theming/tinted-shell"
themes-dir = "scripts"
hook = "source \"$TINTY_THEME_FILE_PATH\""
supported-systems = ["base16", "base24"]

[[items]]
name = "base16-vim"
path = "https://github.com/tinted-theming/base16-vim"
themes-dir = "colors"
supported-systems = ["base16"]
# Look at USAGE.md for more information on sourcing the generated file in vim

[[items]]
name = "tmux"
path = "~/path/path/to/base16-tmux"
themes-dir = "colors"
hook = "tmux run 2> /dev/null && tmux source-file \"$TINTY_THEME_FILE_PATH\""
supported-systems = ["base16", "base24"]
```

### Select and apply a scheme using fzf

Note: Requires [fzf]

```sh
tinty apply $(tinty list | fzf)
```

## Migration from Flavours

[Flavours] is a great base16 manager written in Rust and it's where
Tinty has gotten a lot of its inspiration. Flavours isn't actively
maintained anymore and that's the reason I continued to build and
develop Tinty.

Tinty doesn't include base16 builder (Flavours does) and therefore Tinty
copies theme files from template directories instead of generating them.
Since a builder is not included in Tinty, generating a scheme based on
image colors is not functionality included.

If you're looking for a base16 or base24 builder, have a look at
[builder-go].

### CLI mapping

- `flavours apply mocha` -> `tinty apply base16-mocha`
- `flavours info mocha` -> `tinty info base16-mocha`
- `flavours current` -> `tinty current`
- `flavours update` -> `tinty sync`

### config.toml mapping

#### Flavours

```toml
# ~/.config/flavours/config.toml
[[item]]
template = "alacritty"
file = "~/.config/alacritty/colors.toml"

[[items]]
file = "~/.config/waybar/colors.css"
template = "waybar"
rewrite = true

[[items]]
file = "~/.config/sway/config"
template = "sway"
subtemplate = "colors"
hook = "swaymsg reload"
light = false
```

#### Tinty

```toml
# ~/.config/tinted-theming/tinty/config.toml
[[items]]
path = "https://github.com/tinted-theming/tinted-alacritty"
name = "tinted-alacritty"
themes-dir = "colors"
hook = "cp -f \"$TINTY_THEME_FILE_PATH\" ~/.config/alacritty/colors.toml"

[[items]]
path = "https://github.com/tinted-theming/base16-waybar"
name = "base16-waybar"
themes-dir = "colors"
hook = "cp -f \"$TINTY_THEME_FILE_PATH\" ~/.config/waybar/colors.css"

[[items]]
path = "https://github.com/rkubosz/base16-sway"
name = "base16-sway"
themes-dir = "themes"
hook = "cp -f \"$TINTY_THEME_FILE_PATH\" ~/.config/sway/config.d/theme && swaymsg reload"
```

- `path`: A `path` to the repository is provided in the Tinty
  `config.toml`. In Flavours this path was determined using the
  `template` property
- `themes-dir`: This is the directory the themes are in within the
  repository provided in `path`
- `name`: A unique name used to set theme filename
- `hook`: This property exists in Flavours too, but Tinty offloads a bit
  of work from the Rust codebase to this hook.

## Debugging Tinty issues

`tinty config` with the relevant flags, `--config-path` and
`--data-dir-path`, can be useful when attempting to debug an issue with
Tinty. These commands can help to make sure that the expected config
file is applied and the expected data directory are being used by Tinty.

## Contributing

Contributions are welcome. Have a look at [CONTRIBUTING.md] for more
information.

## License

Like most other [Tinted Theming] projects, Tinty falls under the MIT
license. Have a look at the [LICENSE] document for more information.

### Third-Party Licenses

This project includes third-party code licensed under the MPL-2.0,
BSD-2-Clause and BSD-3-Clause License. See the [LICENSES-THIRD-PARTY.md]
file for details.

[Tinted Theming]: https://github.com/tinted-theming/home
[Base16 builder specification]: https://github.com/tinted-theming/home/blob/main/builder.md
[home repository]: https://github.com/tinted-theming/home
[Tinted Gallery]: https://tinted-theming.github.io/tinted-gallery
[tinted-shell]: https://github.com/tinted-theming/tinted-shell
[schemes]: https://github.com/tinted-theming/schemes
[fzf]: https://github.com/junegunn/fzf
[Base16]: https://github.com/tinted-theming/home/blob/main/styling.md
[Base24]: https://github.com/tinted-theming/base24/blob/master/styling.md
[Flavours]: https://github.com/Misterio77/flavours
[builder-go]: https://github.com/tinted-theming/base16-builder-go
[repository releases]: https://github.com/tinted-theming/tinty/releases/latest
[CONTRIBUTING.md]: CONTRIBUTING.md
[LICENSE]: LICENSE
[USAGE.md]: USAGE.md
[tinted-builder-rust]: https://github.com/tinted-theming/tinted-builder-rust
[LICENSES-THIRD-PARTY.md]: LICENSES-THIRD-PARTY.md
