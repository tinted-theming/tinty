{
  description = "Tinty flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flake-parts,
    rust-overlay,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];

      perSystem = {
        pkgs,
        system,
        ...
      }: {
        _module.args.pkgs = import nixpkgs {
          overlays = [rust-overlay.overlays.default];
          inherit system;
        };

        formatter = pkgs.alejandra;

        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.just
            pkgs.alejandra
            pkgs.cargo-deny
            pkgs.cargo-about
            pkgs.rust-bin.stable.latest.default
          ];
        };
      };
    };
}
