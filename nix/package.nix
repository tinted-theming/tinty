{pkgs, ...}: let
  assets = {
    "x86_64-linux" = {
      name = "tinty-x86_64-unknown-linux-gnu.tar.gz";
      hash = "sha256-ao6EP5dKT244hi5/65acJrqn5uFJgUz4z/IBhLvLqgQ=";
    };
    "aarch64-linux" = {
      name = "tinty-aarch64-unknown-linux-gnu.tar.gz";
      hash = "sha256-w67ZcqAM7rTPgkf+/+b7MFxJNpFBM6WEFDtiSshRanA=";
    };
    "x86_64-darwin" = {
      name = "tinty-universal-apple-darwin.tar.gz";
      hash = "sha256-4ccaz3ooaK+imRTlxS56SzU9ZOBnoeW6WSXc2RYQ8+0=";
    };
    "aarch64-darwin" = {
      name = "tinty-universal-apple-darwin.tar.gz";
      hash = "sha256-4ccaz3ooaK+imRTlxS56SzU9ZOBnoeW6WSXc2RYQ8+0=";
    };
  };
  version = "0.31.0";
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
