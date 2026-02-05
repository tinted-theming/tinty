{
  config,
  lib,
  ...
}: let
  cfg = config.programs.tinty;
in {
  options.programs.tinty.templates.shell = {
    enable = lib.mkEnableOption "Shell template";
  };

  config = lib.mkIf (cfg.enable && cfg.templates.shell.enable) {
    programs.tinty._items = [
      {
        name = "tinted-shell";
        path = "https://github.com/tinted-theming/tinted-shell";
        themes-dir = "scripts";
        supported-systems = ["base16" "base24"];
        hook = ". %f";
      }
    ];
  };
}
