{pkgs, ...}: let
  assets = {
    "x86_64-linux" = {
      name = "tinty-x86_64-unknown-linux-gnu.tar.gz";
      hash = "sha256-ebyglFAGVT5iRp82OktQ7MMoTIJjwaIj8A+V4zXuCQc=";
    };
    "aarch64-linux" = {
      name = "tinty-aarch64-unknown-linux-gnu.tar.gz";
      hash = "sha256-zq0ngY5mBFmjneJsxV1GkHOoDV4LX+UldvPpG/ngrOY=";
    };
    "x86_64-darwin" = {
      name = "tinty-x86_64-apple-darwin.tar.gz";
      hash = "sha256-dxoFMMeHfa7bIEVErjIjmLHYbwmAcBWqVwYy+DafFak=";
    };
    "aarch64-darwin" = {
      name = "tinty-aarch64-apple-darwin.tar.gz";
      hash = "sha256-1D6hfynzOZqIto+zTJ7ggWulF74+vKOFUOR+Rr33L/Y=";
    };
  };
  version = "0.34.0";
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
