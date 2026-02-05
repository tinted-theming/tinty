{
  config,
  lib,
  ...
}: let
  cfg = config.programs.tinty;
  terminalNames = [
    "alacritty"
    "conemu"
    "foot"
    "ghostty"
    "iterm2"
    "kermit"
    "kitty"
    "konsole"
    "putty"
    "rio"
    "qterminal"
    "st"
    "termite"
    "wezterm"
    "xfce4"
  ];
in {
  options.programs.tinty.templates.terminal = {
    enable = lib.mkEnableOption "Terminal template integration via tinted-terminal monorepo";
    type = lib.mkOption {
      type = lib.types.nullOr (lib.types.enum terminalNames);
      default = null;
      description = "Terminal type within tinted-terminal (e.g., 'alacritty', 'kitty').";
    };
    revision = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Git revision of the tinted-terminal repository to use (optional).";
    };
  };

  config = lib.mkMerge [
    (lib.mkIf (cfg.enable && cfg.templates.terminal.enable) {
      programs.tinty._items = [
        ({
            path = "https://github.com/tinted-theming/tinted-terminal";
            supported-systems = ["base16" "base24"];
          }
          // lib.optionalAttrs (cfg.templates.terminal.type != null) {
            name = "tinted-${cfg.templates.terminal.type}";
          }
          // lib.optionalAttrs (cfg.templates.terminal.revision != null) {
            revision = cfg.templates.terminal.revision;
          }
          // lib.optionalAttrs (cfg.templates.terminal.type != null) {
            themes-dir = "themes/${cfg.templates.terminal.type}";
          }
          # For terminals that don't support 256 colors
          // lib.optionalAttrs (cfg.templates.terminal.type == "kermit" || cfg.templates.terminal.type == "rio") {
            themes-dir = "themes-16/${cfg.templates.terminal.type}";
          })
      ];
    })

    # Alacritty HM integration: import generated TOML theme file
    (lib.mkIf (cfg.enable && cfg.templates.terminal.enable && cfg.templates.terminal.type == "alacritty") {
      programs.alacritty.settings.general.import = [
        "${config.xdg.dataHome}/tinted-theming/tinty/tinted-terminal-themes-alacritty-file.toml"
      ];
    })
  ];
}
