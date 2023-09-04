{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, flake-utils, fenix, ... }:
    let
      mkPkgs = pkgs:
        let
          inherit (pkgs) lib;
        in
        rec {
          rustToolchain = pkgs.fenix.toolchainOf {
            channel = (lib.importTOML ./rust-toolchain).toolchain.channel;
            sha256 = "sha256-DzNEaW724O8/B8844tt5AVHmSjSQ3cmzlU4BP90oRlY=";
          };
          rust = (
            with pkgs.fenix;
            with rustToolchain;
            combine [
              defaultToolchain
              rust-src
              rust-analyzer
            ]
          );
          rustPlatform = (pkgs.makeRustPlatform { cargo = rust; rustc = rust; });
          nerdfix =
            rustPlatform.buildRustPackage {
              pname = "nerdfix";
              version = "0.2.3";
              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;
            };
        }
      ;
    in
    (flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default ];
        };
        inherit (mkPkgs pkgs) rust nerdfix;
      in
      {
        packages.default = nerdfix;
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [ rust ];
        };
      }
    )) // {
      overlays.default = _: prev: {
        inherit (mkPkgs prev) nerdfix;
      };
    }
  ;
}
