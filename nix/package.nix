{pkgs, ...}: let
  assets = {
    "x86_64-linux" = {
      name = "tinty-x86_64-unknown-linux-gnu.tar.gz";
      hash = "sha256-9xZLUfx3xSA8NoFoH25meL3ngUNwDqwLa5Hyfr4pDVA=";
    };
    "aarch64-linux" = {
      name = "tinty-aarch64-unknown-linux-gnu.tar.gz";
      hash = "sha256-uZpk4CSdEJvK+5KxGYKTPEYTkxuxdvkkNm5oqqj9gEY=";
    };
    "x86_64-darwin" = {
      name = "tinty-universal-apple-darwin.tar.gz";
      hash = "sha256-6U+/UIMcajB7ImVAUcjqE3sHKFQXNqRsxDS1S19y2qI=";
    };
    "aarch64-darwin" = {
      name = "tinty-universal-apple-darwin.tar.gz";
      hash = "sha256-qYffUI4rddKI/2b97PS+3hTO+OKp3bOXVhs0b215Zdk=";
    };
  };
  version = "0.32.1";
  asset = assets.${pkgs.system};
  isLinux = pkgs.stdenv.isLinux;
in
  pkgs.stdenv.mkDerivation {
    pname = "tinty";
    inherit version;

    src = pkgs.fetchurl {
      url = "https://github.com/tinted-theming/tinty/releases/download/v${version}/${asset.name}";
      inherit (asset) hash;
    };

    sourceRoot = ".";

    nativeBuildInputs = pkgs.lib.optionals isLinux [pkgs.autoPatchelfHook];
    buildInputs = pkgs.lib.optionals isLinux [pkgs.gcc-unwrapped];

    installPhase = ''
      install -Dm755 tinty $out/bin/tinty
    '';
  }
