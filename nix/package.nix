{pkgs, ...}: let
  assets = {
    "x86_64-linux" = {
      name = "tinty-x86_64-unknown-linux-gnu.tar.gz";
      hash = "sha256-u78yEr1XYYF8tYLps1eFhVLG5ksoTDTlhZS9DUhn5lA=";
    };
    "aarch64-linux" = {
      name = "tinty-aarch64-unknown-linux-gnu.tar.gz";
      hash = "sha256-Dc0pTA/vjSL1MgF4JV3+FHNOKFmLIRIeFzpyhePSeLs=";
    };
    "x86_64-darwin" = {
      name = "tinty-universal-apple-darwin.tar.gz";
      hash = "sha256-jV0jXFD3rAVA2NpKc5zvFwxITBFdG9WtjxRRcntoW9o=";
    };
    "aarch64-darwin" = {
      name = "tinty-universal-apple-darwin.tar.gz";
      hash = "sha256-jV0jXFD3rAVA2NpKc5zvFwxITBFdG9WtjxRRcntoW9o=";
    };
  };
  version = "0.30.0";
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
