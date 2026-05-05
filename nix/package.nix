{pkgs, ...}: let
  assets = {
    "x86_64-linux" = {
      name = "tinty-x86_64-unknown-linux-gnu.tar.gz";
      hash = "sha256-cMSEYJD9KnJiYhwfUZuF2KifUcli6mAyNulY0ZStp3o=";
    };
    "aarch64-linux" = {
      name = "tinty-aarch64-unknown-linux-gnu.tar.gz";
      hash = "sha256-cQocG57iTWGpQWPgaVSoV0m6SD3NzwGGKNhfHlV/2ko=";
    };
    "x86_64-darwin" = {
      name = "tinty-universal-apple-darwin.tar.gz";
      hash = "sha256-S0e9+QxQ076YUVfeNzlmvvF8OIHp1Il0F2AnVhxUDMI=";
    };
    "aarch64-darwin" = {
      name = "tinty-universal-apple-darwin.tar.gz";
      hash = "sha256-dIEzlB8P04De2SPC/ajFPPlbg7eGaQ8wP8AbBk82C+U=";
    };
  };
  version = "0.32.2";
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
