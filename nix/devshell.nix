# Development shell for Information Alchemist
{ pkgs
, rust-toolchain
, nonRustDeps
}:

pkgs.mkShell {
  packages = [
    rust-toolchain
    pkgs.cargo-watch
    pkgs.cargo-nextest
  ];
  buildInputs = with pkgs; [
    # Build tools
    pkg-config
    clang
    lld
    mold

    # Graphics libraries
    vulkan-loader
    vulkan-headers
    vulkan-validation-layers
    libxkbcommon
    wayland
    libGL

    # System libraries
    udev
    alsa-lib
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr

    # SSL/TLS support
    openssl

    # Development tools
    rust-analyzer
    bacon

    # NATS for testing
    natscli
    nsc
  ];

  nativeBuildInputs = with pkgs; [
    llvmPackages.clang
    llvmPackages.bintools
    lld
    mold # Fast linker for Bevy development
  ];

  # Environment for proper Bevy development with experimental feature support
  shellHook = ''
    echo "Information Alchemist Development Environment"
    echo "============================================"
    echo "Rust: $(rustc --version)"
    echo "Cargo: $(cargo --version)"
    echo "Vulkan Layer Path: ${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d"
    echo ""
    echo "To run tests: cargo test --lib"
    echo "To build: nix build"
    echo "To run: nix run"
  '';

  # Vulkan configuration
  VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
  VULKAN_SDK = "${pkgs.vulkan-headers}";
  VK_ICD_FILENAMES = "${pkgs.mesa}/share/vulkan/icd.d/radeon_icd.x86_64.json:${pkgs.mesa}/share/vulkan/icd.d/intel_icd.x86_64.json";

  # Bevy configuration
  BEVY_HEADLESS = "1";
  RUST_BACKTRACE = "full";
  WINIT_UNIX_BACKEND = "wayland";
  BEVY_ASSET_ROOT = toString ../.;

  # Rust configuration - use mold for faster linking
  RUSTFLAGS = "-C link-arg=-fuse-ld=mold -C link-arg=-Wl,-rpath,${pkgs.vulkan-loader}/lib -Zshare-generics=y";
  BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.alsa-lib}/include";

  # Library paths - include Rust standard library path
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath
    (nonRustDeps ++ [
      pkgs.vulkan-loader
      rust-toolchain
    ]) + ":${rust-toolchain}/lib:${toString ../.}/target/debug/deps:${toString ../.}/target/release/deps";

  # PKG_CONFIG_PATH for finding system libraries
  PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" [
    pkgs.alsa-lib
    pkgs.udev
    pkgs.systemd
    pkgs.vulkan-loader
    pkgs.libxkbcommon
    pkgs.openssl
  ];

  # Disable experimental features that might cause issues
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-C link-arg=-fuse-ld=mold";
}
