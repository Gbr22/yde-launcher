{
  description = "yde-launcher";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, crane, ... }:
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

        rustToolchain = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

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
          libxcb
          libxkbcommon
          libGL
          vulkan-loader
          wayland
        ];
        nativeBuildInputs = with pkgs; [
          pkg-config
        ];

        libPath = pkgs.lib.makeLibraryPath (buildInputs ++ nativeBuildInputs);

        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          src = src;
          nativeBuildInputs = nativeBuildInputs;
          buildInputs = buildInputs;
          LD_LIBRARY_PATH = libPath;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        releaseBuild = craneLib.buildPackage (commonArgs // {
          cargoArtifacts = cargoArtifacts;
          name = exeName;

          nativeBuildInputs = commonArgs.nativeBuildInputs ++ [ pkgs.copyDesktopItems ];

          postFixup = ''
            patchelf --set-rpath ${libPath} $out/bin/${exeName}
          '';

          meta = {
            description = appName;
            maintainers = [];
          };

          desktopItems = [ desktopItem ];
        });

        runScript = pkgs.writeShellScriptBin exeName ''
          #!/usr/bin/env bash
          nix develop --command cargo run
        '';
        shell = craneLib.devShell {
          inputsFrom = [ releaseBuild ];
          packages = [ rustToolchain ];

          shellHook = ''
            export LD_LIBRARY_PATH="${libPath}:$LD_LIBRARY_PATH";
          '';
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
