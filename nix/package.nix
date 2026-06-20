{pkgs, ...}: let
  assets = {
    "x86_64-linux" = {
      name = "tinty-x86_64-unknown-linux-gnu.tar.gz";
      hash = "sha256-MRmF5LJcRjwJ2wDn/YyeUKBV4H12THwJFrfn4xrdD8M=";
    };
    "aarch64-linux" = {
      name = "tinty-aarch64-unknown-linux-gnu.tar.gz";
      hash = "sha256-5NpY5xO1oh9ZQzqVAJLZaWf/4mlibNiAwy3XHGJpkKo=";
    };
    "x86_64-darwin" = {
      name = "tinty-x86_64-apple-darwin.tar.gz";
      hash = "sha256-58967a+prTf4WQUlzmcpVZ+16NAByAIa8XUWdl8lLN4=";
    };
    "aarch64-darwin" = {
      name = "tinty-aarch64-apple-darwin.tar.gz";
      hash = "sha256-vKUpuJDuuFtLosh6Ss8IkfGp37qe1n88QuK3RhpTepI=";
    };
  };
  version = "0.34.1";
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
