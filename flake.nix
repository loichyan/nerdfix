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
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default ];
        };
      in
      with pkgs; {
        devShells.default = mkShell {
          nativeBuildInputs = [
            (with pkgs.fenix; combine [
              stable.defaultToolchain
              stable.rust-src
            ])
          ];
        };
      }
    )
  ;
}
