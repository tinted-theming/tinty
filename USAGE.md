# Usage

There are many specific usage situations which this document will cover.
For more general usage, look at the [Usage section] in [README.md].

## Table of contents

- [How it works](#how-it-works)
- [Vim or Neovim](#vim-or-neovim)
- [tmux](#tmux)
- [fzf](#fzf)
- [bat](#bat)

## How it works

There are some concepts which some of the following instructions will
make use of.

### current_scheme

`~/.local/share/tinted-theming/tinty/current_scheme` is a file which
contains the name of the system prefix and name of the scheme
(`<system>-<scheme_name>`), eg: `base16-mocha`. Whenever a scheme is
applied through Tinty, this file is updated.

### What does `tinty apply` do?

1. `tinty apply` sets `current_scheme`.
1. It then runs through the `[[items]]` in your `config.toml
1. For each `[[items]]`, or theme template, it copies the relevant theme
   to `~/.local/shared/tinted-theming/tinty` and executes the the `hook`
   property of the `[[items]]`. `%f` is a template variable that can be
   used in the hook, eg: `hook = "cp -f %f
   ~/.config/alacritty/colors.yml`.

Once you understand the functionality and the lifecycle, you can do a
lot with it.

## Vim or Neovim

There are two different ways you could have vim hooked up to Tiny:

1. Have base16-vim installed in your vim setup and run
   `:colorscheme <THEME_NAME>` when Tinty applies a scheme
2. Have vim source the `.vim` theme file when Tinty applies a scheme

### With base16-vim setup in Vim/Neovim

This doesn't require any setup in your Tinty `config.toml`.

1. Follow the [base16-vim] setup installation instructions.
2. Have vim read the `current_scheme` file and set the vim colorscheme
   with `:colorscheme <DATA_IN_CURRENT_SCHEME>` by adding the following
   to your vim setup. The following Lua or VimScript reads the
   `current_scheme` file when you set your focus to vim and if the theme
   is different to the one already set, it sets it:

**Neovim (Lua)**

```lua
local default_theme = "base16-oceanicnext"

local function get_tinty_theme()
  local theme_name = vim.fn.system("tinty current &> /dev/null && tinty current")

  if vim.v.shell_error ~= 0 then
    return default_theme
  else
    return theme_name
  end
end


local function handle_focus_gained()
  local new_theme_name = get_tinty_theme()
  local current_theme_name = vim.g.colors_name

  if current_theme_name ~= new_theme_name then
    vim.cmd("colorscheme " .. new_theme_name)
  end
end

local function main()
  vim.o.termguicolors = true
  vim.g.tinted_colorspace = 256
  local current_theme_name = get_tinty_theme()

  vim.cmd("colorscheme " .. current_theme_name)

  vim.api.nvim_create_autocmd("FocusGained", {
    callback = handle_focus_gained,
  })
end

main()
```

**Vim**

```vim
let g:default_theme = "base16-oceanicnext"

function! GetTintyTheme()
  let l:theme_name = system("tinty current &> /dev/null && tinty current")

  if v:shell_error != 0
    return g:default_theme
  else
    return l:theme_name
  endif
endfunction

function! HandleFocusGained()
  let l:new_theme_name = GetTintyTheme()
  let l:current_theme_name = g:colors_name

  if l:current_theme_name != l:new_theme_name
    execute "colorscheme " . l:new_theme_name
  endif
endfunction

function! Main()
  set termguicolors
  let g:tinted_colorspace = 256
  let l:current_theme_name = GetTintyTheme()

  execute "colorscheme " . l:current_theme_name

  augroup TintyThemeChange
    autocmd!
    autocmd FocusGained * call HandleFocusGained()
  augroup END
endfunction

call Main()
```

### Without base16-vim setup in Vim/Neovim

1. Add base16-vim to Tinty `config.toml`
2. Have vim source the `.vim` colorscheme file when you focus
   Vim/Neovim.

```toml
[[items]]
path = "https://github.com/tinted-theming/base16-vim"
name = "base16-vim"
themes-dir = "colors"
```

**Neovim (Lua)**

```lua
local theme_script_path = vim.fn.expand("~/.local/share/tinted-theming/tinty/base16-vim-colors-file.vim")

local function file_exists(file_path)
  return vim.fn.filereadable(file_path) == 1 and true or false
end

local function handle_focus_gained()
  if file_exists(theme_script_path) then
      vim.cmd("source " .. theme_script_path)
  end
end

if file_exists(theme_script_path) then
  vim.o.termguicolors = true
  vim.g.tinted_colorspace = 256

  vim.cmd("source " .. theme_script_path)

  vim.api.nvim_create_autocmd("FocusGained", {
    callback = handle_focus_gained,
  })
end
```

**Vim**

```vim
let theme_script_path = expand("~/.local/share/tinted-theming/tinty/base16-vim-colors-file.vim")

function! FileExists(file_path)
  return filereadable(a:file_path) == 1
endfunction

function! HandleFocusGained()
  if FileExists(g:theme_script_path)
    execute 'source ' . g:theme_script_path
  endif
endfunction

if FileExists(theme_script_path)
  set termguicolors
  let g:tinted_colorspace = 256
  execute 'source ' . theme_script_path
  autocmd FocusGained * call HandleFocusGained()
endif
```

## tmux

### Add to Tinty config.toml

```toml
[[items]]
path = "https://github.com/tinted-theming/base16-tmux"
name = "base16-tmux"
# Check if tmux is in use and if it is, reload the config file
hook = "test -n \"$TMUX\" && tmux source-file %f"
themes-dir = "colors"
```

## fzf

### Add to Tinty config.toml

```toml
[[items]]
path = "https://github.com/tinted-theming/base16-fzf"
name = "base16-fzf"
hook = ". %f"
themes-dir = "sh"
# Or for fish shell
# themes-dir = "fish"
```

## bat

[bat has an integration] with [base16-shell] and another option to allow
ANSI colors to be used. The available `bat` theme names for this are
`base16-256` and `ansi`.

- `bat --theme="base16-256"` if you're using the default Tinty or
base16-shell with Tinty.
- `bat --theme="ansi"` if you're using another shell template theme with
Tinty.

Set the alias in your `.*rc` file to make sure this is run by default
whenever `bat` is executed.

```shell
alias bat="bat --theme='base16-256'"
```

[Usage section]: https://github.com/tinted-theming/tinty?tab=readme-ov-file#usage
[README.md]: https://github.com/tinted-theming/tinty/blob/main/README.md
[bat]: https://github.com/sharkdp/bat
[bat has an integration]: https://github.com/sharkdp/bat?tab=readme-ov-file#highlighting-theme
[base16-shell]: https://github.com/tinted-theming/base16-shell
