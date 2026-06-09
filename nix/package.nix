{pkgs, ...}: let
  assets = {
    "x86_64-linux" = {
      name = "tinty-x86_64-unknown-linux-gnu.tar.gz";
      hash = "sha256-i3wI5nMjXwX+H5JipqlES8RgeqTvLxkRIYxelsgHeSQ=";
    };
    "aarch64-linux" = {
      name = "tinty-aarch64-unknown-linux-gnu.tar.gz";
      hash = "sha256-Kz4LBRJfnBo70enK7/h9njEPOvEBIG7mPEiKxAR8XHY=";
    };
    "x86_64-darwin" = {
      name = "tinty-x86_64-apple-darwin.tar.gz";
      hash = "sha256-pHVCfHG7cR+kD40ZE73vuUGOaJmorXiu9itkr2dINxg=";
    };
    "aarch64-darwin" = {
      name = "tinty-aarch64-apple-darwin.tar.gz";
      hash = "sha256-PyoEdbWr0vU3qtJKcSMG7AD5S6MJ2IeP7k30zwsGSV4=";
    };
  };
  version = "0.33.0";
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
