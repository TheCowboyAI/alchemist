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
    pkgs.pcsclite # Add pcscd daemon
    pkgs.pcsctools # PC/SC tools for testing
    pkgs.yubikey-manager # YubiKey management tools
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

    # Cryptographic and smart card support for cim-keys
    pcsclite # PC/SC smart card interface for YubiKey
    gpgme # GPG Made Easy library
    libgpg-error # GPG error handling library
    nettle.dev # Low-level cryptographic library with development headers
    gmp # GNU Multiple Precision Arithmetic Library (required by nettle)
    libclang # For bindgen support

    # Development tools
    rust-analyzer
    bacon

    # NATS for testing
    natscli
    nsc
  ] ++ nonRustDeps;

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
    
    # Set LIBCLANG_PATH for bindgen
    export LIBCLANG_PATH="$(find /nix/store -name "*clang-19*-lib" -type d | head -1)/lib"
    echo "LIBCLANG_PATH set to: $LIBCLANG_PATH"
    
    # Export the library path so cargo run can find runtime libraries
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath (nonRustDeps ++ [
      pkgs.vulkan-loader
      pkgs.pcsclite
      pkgs.gpgme
      pkgs.libgpg-error
      pkgs.nettle
      pkgs.gmp
      rust-toolchain
    ])}:${toString ../.}/target/debug/deps:${toString ../.}/target/release/deps:$LD_LIBRARY_PATH"
    
    # For YubiKey support, we need pcscd running
    # On NixOS, this is typically handled by the system service
    # For non-NixOS or testing, you can start pcscd manually:
    echo ""
    echo "YubiKey Support:"
    echo "----------------"
    if command -v systemctl &> /dev/null && systemctl is-active --quiet pcscd; then
      echo "✓ PC/SC daemon is running (system service)"
    else
      echo "⚠ PC/SC daemon not detected"
      echo "  For YubiKey support, start pcscd:"
      echo "  - On NixOS: sudo systemctl start pcscd"
      echo "  - Or run: sudo pcscd --foreground --debug"
    fi
    
    # Check for connected YubiKeys
    if command -v ykman &> /dev/null; then
      echo ""
      echo "Checking for YubiKeys..."
      ykman list 2>/dev/null || echo "No YubiKeys detected (pcscd may not be running)"
    fi
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

  # Remove the LD_LIBRARY_PATH from here since we set it in shellHook
  # LD_LIBRARY_PATH is now set in shellHook with proper interpolation

  # PKG_CONFIG_PATH for finding system libraries
  PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" [
    pkgs.alsa-lib
    pkgs.udev
    pkgs.systemd
    pkgs.vulkan-loader
    pkgs.libxkbcommon
    pkgs.openssl
    pkgs.pcsclite
    pkgs.gpgme
    pkgs.libgpg-error
    pkgs.nettle.dev
    pkgs.gmp
  ];

  # Disable experimental features that might cause issues
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-C link-arg=-fuse-ld=mold";
}
