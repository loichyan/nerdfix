{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
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
        inherit (pkgs) lib mkShell fenix;

        toolchainFile = (lib.importTOML ./rust-toolchain.toml);
        cargoManifest = (lib.importTOML ./Cargo.toml);
        crate = cargoManifest.package;

        # Rust toolchain for development
        rustChannel = {
          channel = toolchainFile.toolchain.channel;
          sha256 = "sha256-SXRtAuO4IqNOQq+nLbrsDFbVk+3aVA8NNpSZsKlVH/8=";
        };
        rustToolchain = fenix.toolchainOf rustChannel;
        rust-dev = fenix.combine (
          with rustToolchain;
          [
            defaultToolchain
            rust-src
          ]
        );

        # Rust toolchain of MSRV
        rust-msrv =
          if crate.rust-version == rustChannel.channel then
            # Reuse the development toolchain if possible
            rust-dev
          else
            (fenix.toolchainOf {
              channel = crate.rust-version;
              sha256 = "";
            }).minimalToolchain;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = crate.name;
          version = crate.version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          meta = {
            description = crate.description;
            homepage = crate.repository;
            license = with lib.licenses; [
              mit
              asl20
            ];
          };
        };

        devShells.default = mkShell {
          packages = [ rust-dev ];
        };
        devShells.with-rust-analyzer = mkShell {
          packages = [
            rust-dev
            rustToolchain.rust-analyzer
          ];
        };
        devShells.msrv = mkShell {
          packages = [ rust-msrv ];
        };
      }
    );
}
