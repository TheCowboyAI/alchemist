{
  description = "Information Alchemist";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    # Add rust-overlay for nightly Rust support
    rust-overlay.url = "github:oxalica/rust-overlay";

    # Dev tools
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      imports = [
        inputs.treefmt-nix.flakeModule
      ];

      # Export overlay for external use
      flake = {
        overlays.default = import ./nix/overlay.nix;
      };

      perSystem = { config, self', inputs', pkgs, system, ... }:
        let
          # Apply rust-overlay
          pkgsWithRustOverlay = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
            config.allowUnfree = true;
          };

          # Use the latest nightly toolchain with specific extensions
          rust-toolchain = pkgsWithRustOverlay.rust-bin.nightly.latest.default.override {
            extensions = [
              "rust-src"
              "clippy"
              "rustfmt"
              "rust-analyzer"
            ];
            targets = [ "wasm32-unknown-unknown" ];
          };

          # Dependencies needed for Bevy and our project
          nonRustDeps = import ./nix/rust-deps.nix { inherit pkgs; };

          # Test runner with proper Vulkan support
          test-runner = import ./nix/run-tests.nix {
            inherit pkgs rust-toolchain nonRustDeps;
          };

          # Test runner that mimics production build
          test-runner-prod = import ./nix/test-runner.nix {
            inherit pkgs rust-toolchain nonRustDeps;
          };

          # Main package definition with submodules support
          ia-package = import ./nix/package.nix {
            inherit (pkgs) lib;
            inherit pkgs nonRustDeps;
            srcOverride = inputs.self;
          };

          # Package with tests enabled
          ia-package-with-tests = import ./nix/package-with-tests.nix {
            inherit (pkgs) lib;
            inherit pkgs nonRustDeps;
            srcOverride = inputs.self;
          };

          # App configuration
          ia-app = {
            type = "app";
            program =
              let
                runner = pkgs.writeShellScriptBin "ia-runner" ''
                  export PATH="${rust-toolchain}/bin:${pkgs.pkg-config}/bin:$PATH"
                  export PKG_CONFIG_PATH="${pkgs.lib.makeSearchPath "lib/pkgconfig" nonRustDeps}"
                  export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath nonRustDeps}:$(pwd)/target/debug/deps:$(pwd)/target/release/deps"
                  export RUST_LOG=''${RUST_LOG:-info}
                  export BEVY_ASSET_ROOT="${toString ./.}"
                  export BINDGEN_EXTRA_CLANG_ARGS="-I${pkgs.alsa-lib}/include"

                  # Run cargo from the current directory
                  exec cargo run --bin ia "$@"
                '';
              in
              "${runner}/bin/ia-runner";
          };
        in
        {
          packages = {
            # Main application package
            default = ia-package;

            # Alias for clarity
            ia = ia-package;

            # Package with tests enabled
            ia-with-tests = ia-package-with-tests;
          };

          # Make 'nix run' work
          apps = {
            default = ia-app;
            ia = ia-app;
          };

          # Development shell
          devShells.default = pkgs.mkShell {
            inputsFrom = [
              (import ./nix/devshell.nix {
                inherit pkgs rust-toolchain nonRustDeps;
              })
            ];
            # Removed test runners from packages to prevent automatic test execution
            # packages = [ test-runner test-runner-prod test-runner-build ];
          };

          # Formatting configuration
          treefmt.config = {
            projectRootFile = "flake.nix";
            programs = {
              rustfmt.enable = true;
              nixpkgs-fmt.enable = true;
            };
          };
        };
    };
}
