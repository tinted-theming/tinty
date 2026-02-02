self: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.programs.tinty;

  tomlFormat = pkgs.formats.toml {};

  mkItem = {
    name,
    path,
    themes-dir,
    supported-systems,
    revision ? null,
  }:
    {
      inherit name path themes-dir supported-systems;
    }
    // lib.optionalAttrs (revision != null) {
      inherit revision;
    };

  alacrittyItem =
    mkItem {
      name = "tinted-alacritty";
      path = "https://github.com/tinted-theming/tinted-alacritty";
      themes-dir = "themes/alacritty";
      supported-systems = ["base16" "base24"];
    }
    // lib.optionalAttrs (cfg.templates.alacritty.revision != null) {
      revision = cfg.templates.alacritty.revision;
    };

  tmuxItem =
    mkItem {
      name = "tinted-tmux";
      path = "https://github.com/tinted-theming/tinted-tmux";
      themes-dir = "colors";
      supported-systems = ["base16" "base24"];
      revision = cfg.templates.tmux.revision;
    }
    // lib.optionalAttrs (cfg.templates.tmux.revision != null) {
      revision = cfg.templates.tmux.revision;
    }
    // lib.optionalAttrs (cfg.templates.tmux.hook != null) {
      hook = cfg.templates.tmux.hook;
    };

  configFile = tomlFormat.generate "tinty-config.toml" (lib.filterAttrs (_: v: v != null && v != []) {
    shell = cfg.shell;
    default-scheme = cfg.default-scheme;
    preferred-schemes = cfg.preferred-schemes;
    hooks = cfg.hooks;
    items =
      lib.optional cfg.templates.alacritty.enable alacrittyItem
      ++ lib.optional cfg.templates.tmux.enable tmuxItem;
  });
in {
  options.programs.tinty = {
    enable = lib.mkEnableOption "Tinty";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.system}.default;
      description = "The tinty package to use";
    };

    shell = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "The command that executes all hooks, eg `zsh -c '{}'`";
    };

    default-scheme = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "The default scheme to apply if nothing exists in the cache from previous sessions";
    };

    preferred-schemes = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [];
      description = "List of schemes which can be easily cycled through";
    };

    hooks = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [];
      description = "List of shell commands to execute after a scheme is applied";
    };

    templates = {
      alacritty = {
        enable = lib.mkEnableOption "Alacritty template";
        revision = lib.mkOption {
          type = lib.types.nullOr lib.types.str;
          default = null;
          description = "Git revision of the template repository to use";
        };
      };

      tmux = {
        enable = lib.mkEnableOption "Tmux template";
        revision = lib.mkOption {
          type = lib.types.nullOr lib.types.str;
          default = null;
          description = "Git revision of the template repository to use";
        };
        hook = lib.mkOption {
          type = lib.types.nullOr lib.types.str;
          default = "tmux run 2> /dev/null && tmux source-file %f";
          description = "Git revision of the template repository to use";
        };
      };
    };
  };

  config = lib.mkMerge [
    (lib.mkIf cfg.enable {
      home.packages = [cfg.package];
      xdg.configFile."tinted-theming/tinty/config.toml".source = configFile;
    })

    (lib.mkIf (cfg.enable && cfg.templates.alacritty.enable) {
      programs.alacritty.settings.general.import = [
        "${config.xdg.dataHome}/tinted-theming/tinty/tinted-alacritty-themes-alacritty-file.toml"
      ];
    })

    (lib.mkIf (cfg.enable && cfg.templates.tmux.enable) {
      programs.tmux.extraConfig = ''
        source-file ${config.xdg.dataHome}/tinted-theming/tinty/tinted-tmux-colors-file.conf
      '';
    })
  ];
}
