{
  description = "CIM Domain Nix - Domain module for Nix ecosystem operations";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane, advisory-db }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs = with pkgs; [
            # Add runtime dependencies here
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs
            pkgs.libiconv
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
            rustToolchain
          ];
        };

        # Build *just* the cargo dependencies
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the actual crate itself
        cim-domain-nix = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          doCheck = false; # We'll run tests separately
        });

        # Run tests
        cim-domain-nix-tests = craneLib.cargoNextest (commonArgs // {
          inherit cargoArtifacts;
          partitions = 1;
          partitionType = "count";
        });
      in
      {
        checks = {
          inherit cim-domain-nix cim-domain-nix-tests;

          # Run clippy
          cim-domain-nix-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          # Check formatting
          cim-domain-nix-fmt = craneLib.cargoFmt {
            inherit src;
          };

          # Audit dependencies
          cim-domain-nix-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };
        };

        packages = {
          default = cim-domain-nix;
          inherit cim-domain-nix;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = cim-domain-nix;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks.${system};

          # Extra inputs for development
          nativeBuildInputs = with pkgs; [
            rustToolchain
            rust-analyzer
            cargo-watch
            cargo-nextest
            cargo-edit
            cargo-outdated
            cargo-audit
            cargo-license
            cargo-tarpaulin
            
            # Nix development tools
            nix-prefetch-git
            nixpkgs-fmt
            nixfmt
            statix
            deadnix
            nil
            
            # General development tools
            just
            bacon
            mold
            sccache
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          RUST_BACKTRACE = 1;
          RUST_LOG = "debug";
          
          shellHook = ''
            echo "CIM Domain Nix Development Shell"
            echo "================================"
            echo ""
            echo "Available commands:"
            echo "  cargo build    - Build the project"
            echo "  cargo test     - Run tests"
            echo "  cargo watch    - Watch for changes and rebuild"
            echo "  cargo nextest  - Run tests with nextest"
            echo "  nix flake check - Check the flake"
            echo ""
          '';
        };
      });
} 