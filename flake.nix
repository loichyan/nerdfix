{
  inputs = {
    nixpkgs.url = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = { nixpkgs, flake-utils, fenix, crane, ... }:
    let
      mkPkgs = pkgs:
        rec {
          rustToolchain = (with pkgs.fenix; combine [
            stable.defaultToolchain
            stable.rust-src
          ]);
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
          nerdfix =
            let
              craneFilter = path: type:
                path != "cache-version"
                || (craneLib.filterCargoSources path type);
              inherit (pkgs.lib) cleanSourceWith;
            in
            craneLib.buildPackage {
              src = cleanSourceWith {
                src = (craneLib.path ./.);
                filter = craneFilter;
              };
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
