# Changelog

## 0.5.0 (2024-02-14)

- Change `config.toml` `items.system` to `items.supported_systems` which
  now accepts an array of strings instead of a string. This allows for
  using a single template repo for setting both base16 and base24
  themes.

## 0.4.0 (2024-02-11)

- Enforces config.toml `[[items]]` `name` property is a unique value to
  prevent dirname conflicts
- Removes config.toml `[[items]]` `git_url` property
- Adds config.toml `[[items]]` `path` property which supports git URLs
  as well as path to local repo dir

## 0.3.0 (2024-02-10)

- Add support for base24 templates

## 0.2.1 (2024-02-08)

- Fix bug where `tinty --version` displays incorrect version number

## 0.2.0 (2024-02-07)

- Generate `tinty list` from local version of
  https://github.com/tinted-theming/schemes/base16
- `tinty list` now displays schemes prepended by their `system`.
  `ocean` -> `base16-ocean`

## 0.1.0 (2024-02-06)

- Initial release