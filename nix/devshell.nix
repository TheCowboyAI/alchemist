{ pkgs
, rust-toolchain
, nonRustDeps
}:

pkgs.mkShell {
  buildInputs = nonRustDeps ++ [
    rust-toolchain
    pkgs.pkg-config
    pkgs.llvmPackages.clang
    pkgs.llvmPackages.bintools
    pkgs.lld
    # Development tools
    pkgs.just
    pkgs.git
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath nonRustDeps;
  BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.alsa-lib}/include";
  BEVY_ASSET_ROOT = toString ../.; # Point to project root

  # Rust flags for development with lld linker
  RUSTFLAGS = "--cfg edition2024_preview -C link-arg=-fuse-ld=lld";

  # Enable development features with dynamic linking
  CARGO_FEATURES_DEV = "--features dev";

  # Set Wayland backend and enable backtraces
  WINIT_UNIX_BACKEND = "wayland";
  RUST_BACKTRACE = "full";

  shellHook = ''
    echo "Information Alchemist Development Environment"
    echo "============================================"
    echo "Rust: $(rustc --version)"
    echo "Cargo: $(cargo --version)"
    echo ""
    echo "Run 'nix run' to start the application (production build)"
    echo "Run 'cargo run $CARGO_FEATURES_DEV' for development builds with dynamic linking"
    echo ""
    echo "Dynamic linking is enabled for faster compilation in development."
    echo "Production builds via 'nix build' will use static linking."
  '';
}
