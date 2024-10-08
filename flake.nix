{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, flake-utils, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ inputs.fenix.overlays.default ];
        };
        inherit (pkgs) fenix lib;

        # Rust toolchain
        rustToolchain = fenix.toolchainOf {
          channel = (lib.importTOML ./rust-toolchain.toml).toolchain.channel;
          sha256 = "sha256-gdYqng0y9iHYzYPAdkC/ka3DRny3La/S5G8ASj0Ayyc=";
        };

        # For development
        rust-dev = fenix.combine (
          with rustToolchain;
          [
            defaultToolchain
            rust-src
            rust-analyzer
          ]
        );

        # For building packages
        rust-minimal = rustToolchain.minimalToolchain;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rust-minimal;
          rustc = rust-minimal;
        };
      in
      {
        packages.default = rustPlatform.buildRustPackage {
          pname = "nerdfix";
          version = "0.4.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
        devShells.default = with pkgs; mkShell { packages = [ rust-dev ]; };
      }
    );
}
