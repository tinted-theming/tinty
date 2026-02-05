{
  config,
  lib,
  ...
}: let
  cfg = config.programs.tinty;
in {
  options.programs.tinty.templates.fzf = {
    enable = lib.mkEnableOption "Fzf template";
  };

  config = lib.mkIf (cfg.enable && cfg.templates.fzf.enable) {
    programs.tinty._items = [
      {
        name = "tinted-fzf";
        path = "https://github.com/tinted-theming/tinted-fzf";
        themes-dir = "sh";
        supported-systems = ["base16" "base24"];
      }
    ];

    programs.tinty._extraHooks = [
      "${config.xdg.dataHome}/tinted-theming/tinty/repos/tinted-fzf/ansi/ansi.sh"
    ];
  };
}
