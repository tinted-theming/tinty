{
  config,
  lib,
  ...
}: let
  cfg = config.programs.tinty;
in {
  options.programs.tinty.templates.delta = {
    enable = lib.mkEnableOption "Delta template";

    revision = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Git revision of the template repository to use";
    };
  };

  config = lib.mkIf (cfg.enable && cfg.templates.delta.enable) {
    programs.tinty._items = [
      ({
          name = "tinted-delta";
          path = "https://github.com/tinted-theming/tinted-delta";
          themes-dir = "configs";
          supported-systems = ["base16" "base24"];
        }
        // lib.optionalAttrs (cfg.templates.delta.revision != null) {
          revision = cfg.templates.delta.revision;
        })
    ];

    programs.git.includes = [
      {path = "${config.xdg.dataHome}/tinted-theming/tinty/tinted-delta-configs-file.gitconfig";}
    ];
  };
}
