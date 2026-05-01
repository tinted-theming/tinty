{pkgs, ...}: let
  assets = {
    "x86_64-linux" = {
      name = "tinty-x86_64-unknown-linux-gnu.tar.gz";
      hash = "sha256-V3ETM48hG9NJVctx4Tvfyl1qJK5uEJDLsLIPw42agKA=";
    };
    "aarch64-linux" = {
      name = "tinty-aarch64-unknown-linux-gnu.tar.gz";
      hash = "sha256-bCuSU6RaO3L+Uw14UOOqbGrBt3aNrb3Yep5UL59oGw8=";
    };
    "x86_64-darwin" = {
      name = "tinty-universal-apple-darwin.tar.gz";
      hash = "sha256-vLoCumQGVsj+0V/rtOav1G+mbSzgIAItQWgLOyKtt7w=";
    };
    "aarch64-darwin" = {
      name = "tinty-universal-apple-darwin.tar.gz";
      hash = "sha256-6qy1+gINOwP3m2Etk2pz2h8ZlI4j8cAKFhuYN5a8PxU=";
    };
  };
  version = "0.32.0";
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
