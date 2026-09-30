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
        appName = "YDE Launcher";
        appId = "sh.kess.yde.launcher";
        exeName = "yde-launcher";
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        desktopItem = pkgs.makeDesktopItem {
          name = appId;
          desktopName = appName;
          exec = exeName;
          icon = "app-launcher";
          categories = [ "Utility" ];
          startupWMClass = appId;
          terminal = false;
          noDisplay = true;
        };

        buildInputs = with pkgs; [
          pkg-config
          rust-bin.stable.latest.default
          libxcb
          libxkbcommon
          libGL
          vulkan-loader
          wayland
        ];
        nativeBuildInputs = with pkgs; [
          pkg-config
          copyDesktopItems
        ];
        libPath = pkgs.lib.makeLibraryPath (buildInputs ++ nativeBuildInputs);
        runScript = pkgs.writeShellScriptBin exeName ''
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
          name = exeName;
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = nativeBuildInputs;
          buildInputs = buildInputs;

          LD_LIBRARY_PATH = libPath;

          postFixup = ''
            patchelf --set-rpath ${libPath} $out/bin/${exeName}
          '';

          meta = {
            description = appName;
            maintainers = [];
          };

          desktopItems = [ desktopItem ];
        };
      in
      {
        packages.default = releaseBuild;
        devShells.default = shell;
        apps.default = {
          type = "app";
          program = "${runScript}/bin/${exeName}";
        };
      }
    );
}
