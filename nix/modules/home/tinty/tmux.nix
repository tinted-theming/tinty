{
  config,
  lib,
  ...
}: let
  cfg = config.programs.tinty;
in {
  options.programs.tinty.templates.tmux = {
    enable = lib.mkEnableOption "Tmux template";

    revision = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Git revision of the template repository to use";
    };

    hook = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = "tmux run 2> /dev/null && tmux source-file %f";
      description = "Hook to source the generated tmux theme file";
    };
  };

  config = lib.mkIf (cfg.enable && cfg.templates.tmux.enable) {
    programs.tinty._items = [
      ({
          name = "tinted-tmux";
          path = "https://github.com/tinted-theming/tinted-tmux";
          themes-dir = "colors";
          supported-systems = ["base16" "base24"];
          hook = cfg.templates.tmux.hook;
        }
        // lib.optionalAttrs (cfg.templates.tmux.revision != null) {
          revision = cfg.templates.tmux.revision;
        })
    ];

    programs.tmux.extraConfig = ''
      source-file ${config.xdg.dataHome}/tinted-theming/tinty/tinted-tmux-colors-file.conf
    '';
  };
}
