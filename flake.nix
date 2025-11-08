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
          libxcb
          libxkbcommon
          vulkan-loader
          wayland
        ];
        nativeBuildInputs = with pkgs; [
          pkg-config
        ];
        libPath = pkgs.lib.makeLibraryPath (buildInputs ++ nativeBuildInputs);
        runScript = pkgs.writeShellScriptBin "yde-launcher" ''
          #!/usr/bin/env bash
          nix develop --command cargo run
        '';
        shell = pkgs.mkShell {
          packages = buildInputs ++ nativeBuildInputs;

          shellHook = ''
            export LD_LIBRARY_PATH="${libPath}:$LD_LIBRARY_PATH";
          '';
        };
        releaseBuild = pkgs.rustPlatform.buildRustPackage {
          name = "yde-launcher";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = nativeBuildInputs;
          buildInputs = buildInputs;

          LD_LIBRARY_PATH = libPath;

          postFixup = ''
            patchelf --set-rpath ${libPath} $out/bin/yde-launcher
          '';

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
