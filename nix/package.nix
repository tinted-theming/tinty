{pkgs, ...}: let
  assets = {
    "x86_64-linux" = {
      name = "tinty-x86_64-unknown-linux-gnu.tar.gz";
      hash = "sha256-iOUdOo9qS/M7Ent9JkfaVfQU2GYxKWdOf6SNTAPc5Tc=";
    };
    "aarch64-linux" = {
      name = "tinty-aarch64-unknown-linux-gnu.tar.gz";
      hash = "sha256-SMne8As3MwqBHHZ1BxGp43XUy6gwUXVBPupvLFdsFGg=";
    };
    "x86_64-darwin" = {
      name = "tinty-x86_64-apple-darwin.tar.gz";
      hash = "sha256-G+bzFDhYEYZG7Kefx9iQbWqNy6eZLinq+vtB2HHFByo=";
    };
    "aarch64-darwin" = {
      name = "tinty-aarch64-apple-darwin.tar.gz";
      hash = "sha256-e4UurpRwWeCLEfNDfEzrWMraW24it08vftg0Vth/MXg=";
    };
  };
  version = "0.35.0";
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
