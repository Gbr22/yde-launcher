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
            outputHashes = {
              "cryoglyph-0.1.0" = "1rqk1bz8l1nn3m1qh4ax50ca8lhl07xj2vc0d7xq813r6y3spkr5";
              "dpi-0.1.1" = "0h4dcyl8s0wiwki7r1l29zkfcy9h6c0q5s7ia4iwj92j46aga2d5";
              "iced-0.14.0-dev" = "0lp4qqd0zgd0cjy1lmnq8jixha2sb3q3bhw1v29k03k2sfnw4z7b";
            };
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
