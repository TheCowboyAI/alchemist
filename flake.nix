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

          # Test runner using buildRustPackage
          test-runner-build = import ./nix/test-runner-build.nix {
            inherit (pkgs) lib;
            inherit pkgs nonRustDeps;
          };

          # Main package definition
          ia-package = import ./nix/package.nix {
            inherit (pkgs) lib;
            inherit pkgs nonRustDeps;
          };

          # App configuration
          ia-app = {
            type = "app";
            program = "${ia-package}/bin/ia";
            meta.description = ia-package.meta.description;
          };
        in
        {
          packages = {
            # Main application package
            default = ia-package;

            # Alias for clarity
            ia = ia-package;

            # Test runner using buildRustPackage
            test-runner-build = test-runner-build;
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
            packages = [ test-runner test-runner-prod test-runner-build ];
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
