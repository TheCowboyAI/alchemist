# shell.nix - For compatibility with non-flake-enabled Nix
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
    llvmPackages.clang
    llvmPackages.bintools
    lld
    
    # Add patchelf for patching the library paths
    patchelf
    
    # Add cargo tools for development
    cargo-watch
  ];
  
  buildInputs = with pkgs; [
    # Graphics and window management
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    libxkbcommon
    wayland
    wayland-protocols
    
    # Graphics APIs
    vulkan-loader
    vulkan-headers
    vulkan-validation-layers
    libGL
    
    # Audio and input
    alsa-lib
    udev
    systemd
    
    # Required for font rendering
    fontconfig
    freetype
  ];

  # More comprehensive library path construction
  LD_LIBRARY_PATH = pkgs.lib.concatStringsSep ":" [
    "${pkgs.lib.makeLibraryPath (with pkgs; [
      vulkan-loader
      libxkbcommon
      wayland
      xorg.libX11
      xorg.libXcursor
      xorg.libXrandr
      xorg.libXi
      alsa-lib
      udev
      systemd
      stdenv.cc.cc.lib
      libGL
    ])}"
    "$LD_LIBRARY_PATH"
  ];
  
  # Set Vulkan validation layer path
  VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";

  # Configure cargo to use lld and set environment variables for Wayland
  shellHook = ''
    # Set up cargo configuration
    mkdir -p .cargo
    cat > .cargo/config.toml << EOF
[build]
rustflags = [
  # Use LLD linker for faster linking
  "-C", "link-arg=-fuse-ld=lld",
  
  # Export dynamic symbols for Bevy dynamic linking
  "-C", "link-arg=-Wl,--export-dynamic",
  
  # Add origin-based rpath for finding libraries relative to binary
  "-C", "link-arg=-Wl,-rpath,\$ORIGIN",
  "-C", "link-arg=-Wl,-rpath,\$ORIGIN/../lib",
  
  # Add library search paths
  "-C", "link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath (with pkgs; [
    vulkan-loader libxkbcommon wayland xorg.libX11 
    xorg.libXcursor xorg.libXrandr xorg.libXi alsa-lib udev
    systemd libGL stdenv.cc.cc.lib
  ])}"
]

[target.x86_64-unknown-linux-gnu]
linker = "clang"
EOF

    # Set the LD_RUN_PATH, which is used by the linker to set the rpath
    export LD_RUN_PATH="${pkgs.lib.makeLibraryPath (with pkgs; [
      vulkan-loader libxkbcommon wayland xorg.libX11 
      xorg.libXcursor xorg.libXrandr xorg.libXi alsa-lib udev
      systemd libGL stdenv.cc.cc.lib
    ])}"
    
    # Set XDG runtime directory and Wayland variables
    export XDG_RUNTIME_DIR=/run/user/1000
    export WAYLAND_DISPLAY=wayland-0
    
    # Force Wayland as backend
    export WINIT_UNIX_BACKEND=wayland
    export BEVY_WINIT_UNIX_BACKEND=wayland
    export WLR_NO_HARDWARE_CURSORS=1
    
    # Create cache directory for Bevy
    mkdir -p $HOME/.cache/bevy
    
    # Helper function to check for libraries
    check_lib() {
      ldconfig -p | grep -q "$1" && echo "✓ Found $1" || echo "✗ Missing $1"
    }
    
    # Print environment information
    echo "===== Environment Setup ====="
    echo "LD_LIBRARY_PATH set to: $LD_LIBRARY_PATH"
    echo "LD_RUN_PATH set to: $LD_RUN_PATH"
    echo "XDG_RUNTIME_DIR set to: $XDG_RUNTIME_DIR"
    echo "WAYLAND_DISPLAY set to: $WAYLAND_DISPLAY"
    echo ""
    echo "===== Checking for critical libraries ====="
    check_lib "libwayland-client.so"
    check_lib "libvulkan.so"
    check_lib "libGL.so"
    check_lib "libudev.so"
    
    echo ""
    echo "Using lld linker for faster builds"
  '';
} 