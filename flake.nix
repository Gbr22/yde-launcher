{
  description = "yde-launcher";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        buildInputs = with pkgs; [
          pkg-config
          rust-bin.stable.latest.default
          libxkbcommon
          vulkan-loader
          wayland
        ];
        libPath = pkgs.lib.makeLibraryPath buildInputs;
        runScript = pkgs.writeShellScriptBin "yde-launcher" ''
          #!/usr/bin/env bash
          export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${libPath}";
          cargo run
        '';
        shell = pkgs.mkShell {
          shellHook = ''
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${libPath}";
          '';
        };
        releaseBuild = pkgs.rustPlatform.buildRustPackage {
          name = "yde-launcher";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = buildInputs;

          meta = {
            description = "YDE Launcher";
            maintainers = [];
          };
        };
      in
      {
        packages.default = releaseBuild;
        devShells.default = shell;
        apps.default = {
          type = "app";
          program = "${runScript}/bin/yde-launcher";
        };
      }
    );
}
