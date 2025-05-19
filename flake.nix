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
      perSystem = { config, self', pkgs, lib, system, ... }:
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
          nonRustDeps = with pkgs; [
            # Audio support
            alsa-lib
            alsa-utils
            
            # Wayland support
            wayland
            wayland-protocols
            wayland-scanner
            libxkbcommon
            
            # Vulkan
            vulkan-headers
            vulkan-loader
            vulkan-validation-layers
            
            # Graphics and debugging
            freetype
            glfw
            tracy
            renderdoc
            
            # For shader compilation
            shaderc
            
            # Additional linker-related dependencies
            stdenv.cc.cc.lib
            zlib
            xorg.libXrandr
            xorg.libXinerama
            xorg.libXxf86vm
            xorg.libXi
            xorg.libXcursor
            libGL
          ];

          # Custom script to fix Bevy dynamic linking
          fix-bevy-linking = pkgs.writeShellScriptBin "fix-bevy-linking" ''
            CARGO_TARGET_DIR=''${CARGO_TARGET_DIR:-./target}
            mkdir -p $CARGO_TARGET_DIR
            DYLIB_PATH=$(find $CARGO_TARGET_DIR -name "libbevy_dylib*.so" 2>/dev/null | head -n 1)
            
            if [ -n "$DYLIB_PATH" ]; then
              echo "Fixing dynamic linking for $DYLIB_PATH"
              ${pkgs.patchelf}/bin/patchelf --remove-needed libwayland-client.so.0 $DYLIB_PATH || true
              ${pkgs.patchelf}/bin/patchelf --add-needed libwayland-client.so $DYLIB_PATH || true
              ${pkgs.patchelf}/bin/patchelf --set-rpath "${pkgs.lib.makeLibraryPath nonRustDeps}" $DYLIB_PATH || true
              echo "Dylib successfully patched"
            else
              echo "No Bevy dylib found. Build the project first with 'cargo build'"
            fi
          '';
        in
        {
          # Alchemist dev environment
          devShells.default = pkgs.mkShell {
            inputsFrom = [
              config.treefmt.build.devShell
            ];
            buildInputs = with pkgs; [
              # Rust development
              # rustcVersion
              pkg-config
              clippy
              rustfmt
              
              # System dependencies
              wayland
              wayland-protocols
              wayland-scanner
              libGL
              libxkbcommon
              xorg.libX11
              xorg.libXcursor
              xorg.libXi
              xorg.libXrandr
              udev
              alsa-lib
              vulkan-loader
              vulkan-tools
              vulkan-headers
              vulkan-validation-layers
              
              # Use our specific Rust toolchain
              rust-toolchain
              
              # Build tools
              cmake
              ninja
              clang
              lld
              llvmPackages.bintools # Extra linker tools
              patchelf
              
              # Basic utilities
              libiconv
              gcc
              glibc # Provides ldd
              binutils # Provides additional linker tools
              
              # Development tools
              cargo-watch
              
              # Custom tools
              fix-bevy-linking
            ];
            
            # Library paths
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (nonRustDeps ++ [
              pkgs.stdenv.cc.cc.lib
              pkgs.libGL
            ]);
            VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
            
            # Special environment variables to help Cargo know we support edition2024
            RUSTC_BOOTSTRAP = "1";
            
            # Environment variables for Bevy and graphics
            BEVY_ASSET_ROOT = toString ./.;
            RUST_BACKTRACE = "1";
            RUST_LOG = "info";
            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
            
            # Rust flags for Bevy dynamic linking
            RUSTFLAGS = "--cfg edition2024_preview -C link-arg=-fuse-ld=lld -C link-arg=-Wl,--export-dynamic";
            
            shellHook = ''
              # Force Wayland support (even when using X11)
              export WINIT_UNIX_BACKEND=wayland
              export BEVY_WINIT_UNIX_BACKEND=wayland
              
              # Disable hardware cursor for Wayland compatibility
              export WLR_NO_HARDWARE_CURSORS=1
              
              # Wayland and XDG runtime directories
              export WAYLAND_DISPLAY=/run/user/1000/wayland-1
              export XDG_RUNTIME_DIR=/run/user/1000
              
              # Setup library paths
              export PKG_CONFIG_PATH=$PKG_CONFIG_PATH:${pkgs.wayland.dev}/lib/pkgconfig:${pkgs.libxkbcommon.dev}/lib/pkgconfig
              
              # Set linker path
              export LD_RUN_PATH=${pkgs.lib.makeLibraryPath nonRustDeps}
              
              # Ensure we can find Wayland libraries
              export LD_LIBRARY_PATH=${pkgs.wayland}/lib:$LD_LIBRARY_PATH
              
              # Show the rust version information on shell start
              echo "Rust toolchain information:"
              rustc --version
              cargo --version
              rustfmt --version
              echo "Using nightly toolchain with edition2024 support"
              
              # Instructions for fixing dynamic linking
              echo ""
              echo "After building with 'cargo build', you can fix dynamic linking with:"
              echo "fix-bevy-linking"
              echo ""
            '';
          };

          # Add auto-formatters
          treefmt.config = {
            projectRootFile = "flake.nix";
            programs = {
              nixpkgs-fmt.enable = true;
              rustfmt.enable = true;
            };
          };
        };
    };
}
