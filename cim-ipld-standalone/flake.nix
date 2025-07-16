{
  description = "CIM-IPLD: Content-addressed storage for the Composable Information Machine";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          cargo-nextest
          cargo-watch
        ];

        buildInputs = with pkgs; [
          openssl
        ] ++ lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "cim-ipld";
          version = "0.3.0";
          src = ./.;
          
          cargoLock.lockFile = ./Cargo.lock;
          
          inherit nativeBuildInputs buildInputs;
          
          doCheck = true;
        };

        devShells.default = pkgs.mkShell {
          inherit buildInputs;
          nativeBuildInputs = nativeBuildInputs ++ (with pkgs; [
            rust-analyzer
            cargo-edit
            cargo-audit
            cargo-outdated
          ]);

          RUST_BACKTRACE = 1;
          RUST_LOG = "debug";
        };

        apps.test = flake-utils.lib.mkApp {
          drv = pkgs.writeShellScriptBin "test-cim-ipld" ''
            cargo test --all-features
          '';
        };

        apps.bench = flake-utils.lib.mkApp {
          drv = pkgs.writeShellScriptBin "bench-cim-ipld" ''
            cargo bench
          '';
        };
      });
}