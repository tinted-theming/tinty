self: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.programs.tinty;
  tomlFormat = pkgs.formats.toml {};

  createConfigFile =
    tomlFormat.generate "tinty-config.toml"
    (lib.filterAttrs (_: v: v != null && v != []) {
      shell = cfg.shell;
      default-scheme = cfg.default-scheme;
      preferred-schemes = cfg.preferred-schemes;
      hooks =
        (
          if builtins.isList cfg.hooks
          then cfg.hooks
          else []
        )
        ++ cfg._extraHooks;
      items = cfg._items;
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

    # Internal accumulation points for submodules
    _items = lib.mkOption {
      type = lib.types.listOf lib.types.attrs;
      default = [];
      internal = true;
      description = "Internal: Items contributed by app modules.";
    };

    _extraHooks = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [];
      internal = true;
      description = "Internal: Extra hooks contributed by app modules.";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [cfg.package];
    xdg.configFile."tinted-theming/tinty/config.toml".source = createConfigFile;
  };
}
