{
  description = "Home-manager integration tests for tinty";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    tinty.url = "../..";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flake-parts,
    home-manager,
    tinty,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];

      perSystem = {
        pkgs,
        lib,
        system,
        ...
      }: let
        mkTestConfig = modules:
          home-manager.lib.homeManagerConfiguration {
            inherit pkgs;
            modules =
              [
                tinty.homeManagerModules.default
                {
                  home.username = "test";
                  home.homeDirectory = "/home/test";
                  home.stateVersion = "24.05";
                }
              ]
              ++ modules;
          };

        getConfigToml = config:
          config.xdg.configFile."tinted-theming/tinty/config.toml".source;

        mkConfigCheck = name: config: expected: let
          expectedFile = pkgs.writeText "expected-${name}" (builtins.concatStringsSep "\n" expected);
        in
          pkgs.runCommand "check-${name}" {} ''
            config="${getConfigToml config}"
            while IFS= read -r line; do
              if ! grep -qF "$line" "$config"; then
                echo "Expected '$line' not found in config.toml"
                echo "Actual content:"
                cat "$config"
                exit 1
              fi
            done < ${expectedFile}
            echo "All checks passed for ${name}"
            touch $out
          '';

        rootConfig = mkTestConfig [
          {
            programs.tinty = {
              enable = true;
              default-scheme = "gruvbox-dark-medium";
              shell = "bash -c '{}'";
            };
          }
        ];

        deltaConfig = mkTestConfig [
          {
            programs.tinty = {
              enable = true;
              templates.delta.enable = true;
            };
          }
        ];

        fzfConfig = mkTestConfig [
          {
            programs.tinty = {
              enable = true;
              templates.fzf.enable = true;
            };
          }
        ];

        shellConfig = mkTestConfig [
          {
            programs.tinty = {
              enable = true;
              templates.shell.enable = true;
            };
          }
        ];

        terminalAlacrittyConfig = mkTestConfig [
          {
            programs.tinty = {
              enable = true;
              templates.terminal.enable = true;
              templates.terminal.type = "alacritty";
            };
          }
        ];

        terminalRioConfig = mkTestConfig [
          {
            programs.tinty = {
              enable = true;
              templates.terminal.enable = true;
              templates.terminal.type = "rio";
            };
          }
        ];

        tmuxConfig = mkTestConfig [
          {
            programs.tinty = {
              enable = true;
              templates.tmux.enable = true;
            };
          }
        ];
      in {
        checks = {
          hm-root = rootConfig.activationPackage;
          hm-delta = deltaConfig.activationPackage;
          hm-fzf = fzfConfig.activationPackage;
          hm-shell = shellConfig.activationPackage;
          hm-terminal-alacritty = terminalAlacrittyConfig.activationPackage;
          hm-terminal-rio = terminalRioConfig.activationPackage;
          hm-tmux = tmuxConfig.activationPackage;

          config-root = mkConfigCheck "root" rootConfig.config [
            ''default-scheme = "gruvbox-dark-medium"''
            ''shell = "bash -c '{}'"''
          ];

          config-delta = mkConfigCheck "delta" deltaConfig.config [
            ''path = "https://github.com/tinted-theming/tinted-delta"''
            ''name = "tinted-delta"''
            ''themes-dir = "configs"''
            ''systems = ["base16", "base24"]''
          ];

          config-fzf = mkConfigCheck "fzf" fzfConfig.config [
            ''hooks = ["/home/test/.local/share/tinted-theming/tinty/repos/tinted-fzf/ansi/ansi.sh"]''
            ''path = "https://github.com/tinted-theming/tinted-fzf"''
            ''name = "tinted-fzf"''
            ''themes-dir = "sh"''
            ''systems = ["base16", "base24"]''
          ];

          config-shell = mkConfigCheck "shell" shellConfig.config [
            ''path = "https://github.com/tinted-theming/tinted-shell"''
            ''name = "tinted-shell"''
            ''themes-dir = "scripts"''
            ''systems = ["base16", "base24"]''
          ];

          config-terminal-alacritty = mkConfigCheck "terminal-alacritty" terminalAlacrittyConfig.config [
            ''path = "https://github.com/tinted-theming/tinted-terminal"''
            ''name = "tinted-alacritty"''
            ''themes-dir = "themes/alacritty"''
            ''systems = ["base16", "base24"]''
          ];

          config-terminal-rio = mkConfigCheck "terminal-rio" terminalRioConfig.config [
            ''path = "https://github.com/tinted-theming/tinted-terminal"''
            ''name = "tinted-rio"''
            ''themes-dir = "themes-16/rio"''
            ''systems = ["base16", "base24"]''
          ];

          config-tmux = mkConfigCheck "tmux" tmuxConfig.config [
            ''path = "https://github.com/tinted-theming/tinted-tmux"''
            ''name = "tinted-tmux"''
            ''themes-dir = "colors"''
            ''hook = "tmux run 2> /dev/null && tmux source-file %f"''
            ''systems = ["base16", "base24"]''
          ];
        };
      };
    };
}
