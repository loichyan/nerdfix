{
  inputs = {
    nixpkgs.url = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, flake-utils, fenix, ... }:
    let
      mkPkgs = pkgs:
        rec {
          rustToolchain = (with pkgs.fenix; combine [
            stable.defaultToolchain
            stable.rust-src
          ]);
          rustPlatform = (pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          });
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
        inherit (mkPkgs pkgs) rustToolchain nerdfix;
      in
      {
        packages.default = nerdfix;
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            rustToolchain
          ];
        };
      }
    )) // {
      overlays.default = _: prev: {
        inherit (mkPkgs prev) nerdfix;
      };
    }
  ;
}
