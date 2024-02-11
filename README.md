# Tinty - A Tinted Theming CLI tool written in Rust 🦀

Change the theme of your terminal, text editor and anything else with
one command. Immediately switch between over 250 themes!

Tinty is a theming manager for all [Tinted Theming] projects including any
template repository that follows the [Base16 builder specification].

To learn more about [Tinted Theming] and the base16 specification, have
a look at our [home repository] and preview the themes supported by
Tinty have a look at our [Base16 Gallery].

Tinty supports [Base16] and [Base24] scheming systems. 

- [Installation](#installation)
- [Usage](#usage)
  - [CLI structure](#cli-structure)
    - [setup](#setup)
    - [list](#list)
    - [set](#set)
    - [update](#update)
    - [init](#init)
    - [Flags](#flags)
  - [Configuration](#configuration)
    - [config.toml](#configtoml)
  - [Usage examples](#usage-examples)
    - [Set theme](#set-theme)
    - [Select theme using fzf](#select-theme-using-fzf)
    - [Use a different command name](#use-a-different-command-name)
    - [Use a custom config directory](#use-a-custom-config-directory)
    - [`config.toml` with various templates](#configtoml-with-various-templates)

## Installation

### Cargo

```shell
cargo install tinty
```

### Manual

```shell
git clone https://github.com/tinted-theming/tinty path/to/tinty
cd path/to/tinty
cargo build --release
cp target/release/tinty path/to/bin/dir
```

## Usage

### CLI structure

You can use `tinty --help` to get an overview too.

#### `setup`

`tinty setup` performs setup requirements for your config and this is
required to run at least once and whenever a new template is added to
your config file.

#### `list`

Lists all available schemes.

#### `set`

Sets a specific theme. Requires the name of the theme (or scheme) as an
argument.

Replace `<SCHEME_NAME>` with the name of the theme you wish to set.

`tinty set <SCHEME_NAME>`

#### `update`

This updates the templates set in the `config.toml` file with the latest
template and the latest [schemes].

#### `init`

`tinty init` checks to see if you have previously set a theme. If you
have it applies that theme again, otherwise it uses `default_scheme`
value set in your `config.toml` file.

This command is useful when added to your shell `.*rc` file to make sure
your shell and other themes are set correctly.

#### Flags

**`--config` or `-c`**

Path to config directory which contains your `config.toml` file. This
value defaults to `$XDG_CONFIG_HOME` otherwise
`$HOME/.config/tinted-theming/tinty`.

### Configuration

The CLI tool will automatically determine the configuration path and
will fall back to the home directory if necessary. It ensures that the
required directories for data and configuration exist.

#### `config.toml`

- `shell` - Add a shell command which will be used by tinty to execute
  commands. This defaults to `sh -c '{}'`. If you want to use bash or zsh
  the format is similar `bash -c '{}'` and `zsh -c '{}'`
- `default_scheme` - defaults to `default-dark`
- `items` - A toml array of tables. Each item represents a template
  - `name` (Required) - A unique value indicating the name of the item
  - `path` (Required) - A url to the git repository or a path to a local
    copy of the repository. It can start with `~/` which will map to
    your home directory, otherwise it must be an absolute path.
  - `themes_dir` (Required) - The template directory name that contains
    the theme files
  - `hook` - A script that is executed after `tinty set <SCHEME_NAME>`
    has been run. `%f` can be used in the hook which is a variable name
    for the location of the theme file. `hook = ". %f"` will source the
    theme file after the theme has been set
  - system - The scheme system of the template. Defaults to `base16`.
    Currently supports `base16` and `base24` templates.

[Base16] and/or [Base24] templates are added to the `config.toml` file
and Tinty will clone those repositories and the theme file when you run
`tinty set <SCHEME_SYSTEM>-<SCHEME_NAME>`. The theme files are set in
`$XDG_DATA_HOME/tinted-theming/tinty` or
`~/.local/share/tinted-theming/tinty`. The name of the themes are as
follows:
<item.system>-`<item.name>-<item.themes_dir>-file.<FILE_EXTENSION> -
this could look like `base16-tmux-colors-file.conf`. The
`<FILE_EXTENSION>` matches the extension of the original theme. So if
your config looks like the following:
```shell
[[items]]
path = "https://github.com/tinted-theming/base16-shell"
name = "shell"
hook = ". %f"
themes_dir = "scripts"
system = "base16"

[[items]]
path = "~/projects/base16-tmux"
name = "tmux"
hook = "tmux source-file %f"
themes_dir = "colors"
system = "base16"

[[items]]
path = "/home/user/projects/base16-fzf"
name = "fzf"
hook = ". %f"
themes_dir = "bash"
system = "base16"
```

Once `tinty set ocean` is run, the following two files will be generated:

- `~/.local/share/tinted-theming/tinty/base16-shell-scripts-file.sh` with `. ~/.local/share/tinted-theming/tinty/base16-shell-scripts-file.sh` executed afterwards.
- `~/.local/share/tinted-theming/tinty/base16-tmux-scripts-file.conf` with `tmux source-file ~/.local/share/tinted-theming/tinty/base16-tmux-scripts-file.conf` executed afterwards.
- `~/.local/share/tinted-theming/tinty/base16-fzf-scripts-file.config` with `. ~/.local/share/tinted-theming/tinty/base16-fzf-scripts-file.config` executed afterwards.

### Usage examples

Without any `config.toml` file, `tinty` will set your shell theme using
[base16-shell].

#### Set theme

To set the `ocean` theme:

```shell
tinty setup # Required once or when your config file is updated
tinty set ocean
```

#### Select theme using fzf

Requires [fzf]:

```shell
tinty set $(tinty list | fzf)
```

### Use a different command name

Add the alias to your shell `.*rc` file:
```shell

alias fancyname=tinty
```

Then use that to alias:

```shell
fancyname set <SCHEME_NAME>
```

### Use a custom config directory

To do this `--config` must be provided for each command. You can do this
automatically by adding an alias to your `.*rc` shell file:

```shell
alias tinty="$(tinty --config='path/to/config')"
```

### config.toml with various templates

```shell
shell = "zsh -c '{}'"
default_scheme = "ocean"

[[items]]
path = "https://github.com/tinted-theming/base16-shell"
name = "shell"
hook = ". %f"
themes_dir = "scripts"
system = "base16"

[[items]]
path = "https://github.com/tinted-theming/base16-fzf"
name = "fzf"
hook = ". %f"
themes_dir = "bash"
system = "base16"

[[items]]
path = "https://github.com/tinted-theming/base16-tmux"
name = "tmux"
hook = "tmux source-file %f"
themes_dir = "colors"
system = "base16"

[[items]]
path = "https://github.com/tinted-theming/base16-vim"
name = "vim"
themes_dir = "colors"
system = "base16"
```

[Tinted Theming]: https://github.com/tinted-theming/home
[Base16 builder specification]: https://github.com/tinted-theming/home/blob/main/builder.md
[home repository]: https://github.com/tinted-theming/home
[Base16 Gallery]: https://tinted-theming.github.io/base16-gallery
[base16-shell]: https://github.com/tinted-theming/base16-shell
[schemes]: https://github.com/tinted-theming/schemes
[fzf]: https://github.com/junegunn/fzf
[Base16]: https://github.com/tinted-theming/home/blob/main/styling.md
[Base24]: https://github.com/tinted-theming/base24/blob/master/styling.md
