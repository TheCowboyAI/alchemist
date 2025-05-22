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
      
      # Configure Nix to use the local cache
      flake.nixConfig = {
        extra-substituters = [
          "http://localhost:5000"
        ];
        extra-trusted-public-keys = [
          "dell-62S6063:F1R/DQVxh0R0YUBXEdVClqDsddJ5VLWVYzPrHC9mmqc="
        ];
      };
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
          
          # Configure rust-analyzer to properly resolve proc macros
          rust-analyzer-settings = pkgs.writeTextFile {
            name = "rust-analyzer-settings.json";
            text = builtins.toJSON {
              "rust-analyzer.check.command" = "clippy";
              "rust-analyzer.check.features" = "all";
              "rust-analyzer.cargo.features" = "all";
              "rust-analyzer.procMacro.enable" = true;
              "rust-analyzer.procMacro.attributes.enable" = true;
              "rust-analyzer.cargo.buildScripts.enable" = true;
              "rust-analyzer.cargo.extraEnv" = {
                "RUSTFLAGS" = "-C target-cpu=native";
              };
            };
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
            
            # Add udev and systemd for libudev dependency
            udev
            systemd
          ];

          # Custom script to fix Bevy dynamic linking
          fix-bevy-linking = pkgs.writeShellScriptBin "fix-bevy-linking" ''
            CARGO_TARGET_DIR=''${CARGO_TARGET_DIR:-./target}
            mkdir -p $CARGO_TARGET_DIR
            DYLIB_PATH=$(find $CARGO_TARGET_DIR -name "libbevy_dylib*.so" 2>/dev/null | head -n 1)
            
            if [ -n "$DYLIB_PATH" ]; then
              echo "===== Fixing dynamic linking for $DYLIB_PATH ====="
              echo "1. Removing problematic dependencies..."
              ${pkgs.patchelf}/bin/patchelf --remove-needed libwayland-client.so.0 $DYLIB_PATH 2>/dev/null || true
              ${pkgs.patchelf}/bin/patchelf --remove-needed libwayland-cursor.so.0 $DYLIB_PATH 2>/dev/null || true
              ${pkgs.patchelf}/bin/patchelf --remove-needed libwayland-egl.so.1 $DYLIB_PATH 2>/dev/null || true
              
              echo "2. Adding correct wayland dependencies..."
              ${pkgs.patchelf}/bin/patchelf --add-needed libwayland-client.so $DYLIB_PATH || true
              
              echo "3. Setting comprehensive library search path..."
              LIBRARY_PATH=""
              # Add all the nonRustDeps library paths
              for lib in ${pkgs.lib.concatStringsSep " " nonRustDeps}; do
                if [ -d "$lib/lib" ]; then
                  LIBRARY_PATH="$LIBRARY_PATH:$lib/lib"
                fi
              done
              
              # Add specific important libraries manually to ensure they're included
              LIBRARY_PATH="$LIBRARY_PATH:${pkgs.wayland}/lib"
              LIBRARY_PATH="$LIBRARY_PATH:${pkgs.libGL}/lib"
              LIBRARY_PATH="$LIBRARY_PATH:${pkgs.vulkan-loader}/lib"
              LIBRARY_PATH="$LIBRARY_PATH:${pkgs.alsa-lib}/lib"
              LIBRARY_PATH="$LIBRARY_PATH:${pkgs.systemd}/lib"
              LIBRARY_PATH="$LIBRARY_PATH:${pkgs.xorg.libX11}/lib"
              LIBRARY_PATH="$LIBRARY_PATH:${pkgs.libxkbcommon}/lib"
              LIBRARY_PATH="$LIBRARY_PATH:${pkgs.udev}/lib"
              
              # Remove leading colon if present
              LIBRARY_PATH=''${LIBRARY_PATH#:}
              
              echo "Setting RPATH to: $LIBRARY_PATH"
              ${pkgs.patchelf}/bin/patchelf --set-rpath "$LIBRARY_PATH" $DYLIB_PATH || true
              
              # Check for missing libraries with ldd
              echo "Checking for missing dependencies..."
              MISSING_DEPS=$(ldd $DYLIB_PATH 2>/dev/null | grep "not found" || true)
              if [ -n "$MISSING_DEPS" ]; then
                echo "Found missing dependencies: $MISSING_DEPS"
                
                # Look specifically for fixedbitset
                echo "Looking for fixedbitset libraries..."
                FIXEDBITSET_LIBS=$(find /nix/store -name "*fixedbitset*.so*" -type f | grep -v "\.a$" || true)
                if [ -n "$FIXEDBITSET_LIBS" ]; then
                  for LIB_PATH in $FIXEDBITSET_LIBS; do
                    LIB_NAME=$(basename "$LIB_PATH")
                    echo "Copying $LIB_PATH to $CARGO_TARGET_DIR/lib/$LIB_NAME"
                    mkdir -p "$CARGO_TARGET_DIR/lib"
                    cp "$LIB_PATH" "$CARGO_TARGET_DIR/lib/$LIB_NAME" || true
                    chmod +x "$CARGO_TARGET_DIR/lib/$LIB_NAME" || true
                    
                    # Update LD_LIBRARY_PATH to include this directory
                    echo "export LD_LIBRARY_PATH=$CARGO_TARGET_DIR/lib:\$LD_LIBRARY_PATH"
                  done
                else
                  echo "WARNING: Could not find fixedbitset libraries in /nix/store"
                fi
              fi
              
              # Print verification information
              echo ""
              echo "4. Verifying dylib dependencies:"
              ldd $DYLIB_PATH
              
              echo ""
              echo "===== Dylib successfully patched ====="
              echo "When running your program, ensure environment variables are set:"
              echo "export XDG_RUNTIME_DIR=/run/user/1000"
              echo "export WAYLAND_DISPLAY=wayland-1"
              echo "export LD_LIBRARY_PATH=$CARGO_TARGET_DIR/lib:\$LD_LIBRARY_PATH"
            else
              echo "No Bevy dylib found. Build the project first with 'cargo build'"
            fi
          '';
        in
        {
          # Add packages output to build and cache binaries
          packages = {
            # Default package builds the bevy_test binary
            default = pkgs.rustPlatform.buildRustPackage {
              pname = "alchemist";
              version = "0.1.0";
              
              # Create a minimal source directory with only what we need
              src = pkgs.runCommand "alchemist-source" {} ''
                # Copy entire project structure except examples
                mkdir -p $out
                cp -r ${./src} $out/src
                cp -r ${./assets} $out/assets 2>/dev/null || mkdir -p $out/assets
                cp -r ${./doc} $out/doc 2>/dev/null || mkdir -p $out/doc
                
                # Copy configuration files
                cp ${./Cargo.toml} $out/Cargo.toml
                cp ${./Cargo.lock} $out/Cargo.lock
                mkdir -p $out/.cargo
                # Create .cargo/config.toml with correct content
                cat > $out/.cargo/config.toml << 'EOF'
# Optimized linker configuration for Bevy on NixOS
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = [
  # Use LLD linker for faster linking
  "-Clink-arg=-fuse-ld=lld",
  
  # Export dynamic symbols for Bevy dynamic linking
  "-Clink-arg=-Wl,--export-dynamic",
  
  # Allow the linker to find necessary libraries
  "-Clink-arg=-Wl,-rpath,$ORIGIN",
  "-Clink-arg=-Wl,-rpath,$ORIGIN/../lib",
]

[profile.dev]
opt-level = 1

# Enable high optimizations for dependencies (excluding code you're developing locally)
[profile.dev.package."*"]
opt-level = 3

# Configure dynamic linking for Bevy
[build]
rustflags = ["--cfg", "edition2024_preview"]
EOF
                
                # Ensure we have a lib.rs for bevy_dylib if it doesn't exist
                # Instead of creating it, check if it exists and warn if it doesn't
                if [ ! -f $out/src/lib.rs ]; then
                  echo "WARNING: No src/lib.rs file found! This is needed for bevy_dylib."
                  echo "Creating a minimal lib.rs file..."
                  echo '//! Minimal library for bevy_dylib\npub fn dummy() -> &'"'"'static str { "Dummy function" }' > $out/src/lib.rs
                fi
              '';
              
              # Use the Cargo lock file
              cargoLock.lockFile = ./Cargo.lock;
              
              # Specify which binary to build - using the main binary
              buildAndTestSubdir = ".";
              
              # Build the main binary with release optimization
              cargoBuildFlags = [];
              cargoRelease = true;
              
              # Set environment variables needed by Bevy
              BEVY_ASSET_ROOT = toString ./.;
              
              # Configure linker flags for dynamic linking
              RUSTFLAGS = "--cfg edition2024_preview -C linker=clang -C link-arg=-fuse-ld=lld -C link-arg=-Wl,--export-dynamic -C link-arg=-Wl,--undefined-version -C link-arg=-Wl,--allow-shlib-undefined";
              
              # We need to build bevy_dylib to get the so file
              postBuild = ''
                # Build bevy_dylib which generates the libbevy_dylib-*.so file
                export CARGO_TARGET_DIR=$NIX_BUILD_TOP/target
                mkdir -p $CARGO_TARGET_DIR
                
                # Our source is at NIX_BUILD_TOP/alchemist-source
                cd $NIX_BUILD_TOP/alchemist-source
                
                # Use cargo directly with release mode
                cargo build --lib
                
                # Find the dylib and print info
                DYLIB_PATH=$(find $CARGO_TARGET_DIR -name "libbevy_dylib*.so" | head -n 1)
                if [ -z "$DYLIB_PATH" ]; then
                  echo "ERROR: Could not find libbevy_dylib*.so"
                  exit 1
                fi
                echo "Found dylib at: $DYLIB_PATH"
                
                # Create a lib directory in the output
                mkdir -p $out/lib
                cp $DYLIB_PATH $out/lib/
                chmod +x $out/lib/$(basename $DYLIB_PATH)
                
                # Print the copied dylib info
                echo "Copied dylib to $out/lib/$(basename $DYLIB_PATH)"
                ls -la $out/lib/
                
                # Search for any fixedbitset shared libraries
                echo "Looking for fixedbitset shared libraries..."
                # Try to find the fixedbitset library in dependencies
                FIXEDBITSET_LIBS=$(find $CARGO_TARGET_DIR -name "*fixedbitset*.so*" -type f 2>/dev/null || true)
                if [ -n "$FIXEDBITSET_LIBS" ]; then
                  for FIXEDBITSET_LIB in $FIXEDBITSET_LIBS; do
                    echo "Found fixedbitset library at: $FIXEDBITSET_LIB"
                    cp "$FIXEDBITSET_LIB" "$out/lib/"
                    chmod +x "$out/lib/$(basename $FIXEDBITSET_LIB)"
                  done
                else
                  echo "Warning: Could not find fixedbitset libraries in target directory"
                  # Look in nix store as fallback
                  FIXEDBITSET_NSTORE=$(find /nix/store -name "*fixedbitset*.so*" -type f | head -1 || true)
                  if [ -n "$FIXEDBITSET_NSTORE" ]; then
                    echo "Found fixedbitset in nix store: $FIXEDBITSET_NSTORE"
                    cp "$FIXEDBITSET_NSTORE" "$out/lib/"
                    chmod +x "$out/lib/$(basename $FIXEDBITSET_NSTORE)"
                  else
                    echo "WARNING: Could not find fixedbitset library"
                  fi
                fi
              '';
              
              # Fix rpath after building
              postFixup = ''
                # Find the dylib filename to use in rpath setting
                DYLIB_NAME=$(find $out/lib -name "libbevy_dylib*.so" | xargs basename)
                if [ -z "$DYLIB_NAME" ]; then
                  echo "ERROR: Could not find libbevy_dylib*.so in output directory"
                  exit 1
                fi
                
                # Find the main binary (alchemist)
                MAIN_BIN=$(find $out/bin -type f -executable | grep -v "\.bin$" | head -1)
                if [ -z "$MAIN_BIN" ]; then
                  echo "ERROR: Could not find main binary in output directory"
                  exit 1
                fi
                MAIN_BIN_NAME=$(basename "$MAIN_BIN")
                echo "Found main binary: $MAIN_BIN_NAME"
                
                # Set rpath to include the lib directory where we copied the dylib
                echo "Setting rpath for $MAIN_BIN_NAME binary to \$ORIGIN/../lib"
                patchelf --set-rpath "\$ORIGIN/../lib:${pkgs.lib.makeLibraryPath nonRustDeps}" "$MAIN_BIN"
                
                # Create a wrapper script that sets LD_LIBRARY_PATH
                mv "$MAIN_BIN" "$out/bin/$MAIN_BIN_NAME.bin"
                
                # Create symlinks for all possible hash variations to ensure the binary finds the library
                # Use patchelf to find needed libraries
                NEEDED_LIBS=$(patchelf --print-needed "$out/bin/$MAIN_BIN_NAME.bin" | grep libbevy_dylib)
                for LIB in $NEEDED_LIBS; do
                  if [[ "$LIB" == libbevy_dylib* && "$LIB" != "$DYLIB_NAME" ]]; then
                    echo "Creating symlink from $DYLIB_NAME to $LIB"
                    ln -sf $DYLIB_NAME $out/lib/$LIB
                  fi
                done
                
                # Find all Rust standard library dependencies and copy them
                RUST_STD_LIBS=$(patchelf --print-needed "$out/bin/$MAIN_BIN_NAME.bin" | grep "libstd-")
                if [ -n "$RUST_STD_LIBS" ]; then
                  echo "Found Rust standard library dependencies: $RUST_STD_LIBS"
                  
                  # Copy from the specific path we know has the library
                  for LIB in $RUST_STD_LIBS; do
                    RUST_LIB_PATH=$(find /nix/store -name "$LIB" | head -1)
                    if [ -n "$RUST_LIB_PATH" ]; then
                      echo "Copying $RUST_LIB_PATH to $out/lib/$LIB"
                      cp "$RUST_LIB_PATH" "$out/lib/$LIB"
                      chmod +x "$out/lib/$LIB"
                    else
                      echo "WARNING: Could not find $LIB in /nix/store"
                    fi
                  done
                fi
                
                # Check for fixedbitset library and add it
                echo "Checking for fixedbitset library..."
                FIXEDBITSET_PATHS=$(find /nix/store -name "*fixedbitset*.so*" -type f | grep -v "\.a$" || true)
                if [ -n "$FIXEDBITSET_PATHS" ]; then
                  for FIXEDBITSET_PATH in $FIXEDBITSET_PATHS; do
                    FIXEDBITSET_NAME=$(basename "$FIXEDBITSET_PATH")
                    echo "Copying $FIXEDBITSET_PATH to $out/lib/$FIXEDBITSET_NAME"
                    cp "$FIXEDBITSET_PATH" "$out/lib/$FIXEDBITSET_NAME"
                    chmod +x "$out/lib/$FIXEDBITSET_NAME"
                    # Create symlink with standard name
                    ln -sf "$FIXEDBITSET_NAME" "$out/lib/libfixedbitset.so"
                  done
                else
                  echo "WARNING: Could not find fixedbitset library"
                fi
                
                cat > "$out/bin/$MAIN_BIN_NAME" << EOF
#!/bin/sh
# Set LD_LIBRARY_PATH to include our dylib directory and run the real binary
SCRIPT_DIR="\$(cd "\$(dirname "\$0")" && pwd)"
export LD_LIBRARY_PATH="\$SCRIPT_DIR/../lib:\$LD_LIBRARY_PATH"
export XDG_RUNTIME_DIR="/run/user/1000"
export WAYLAND_DISPLAY="wayland-1"
export WINIT_UNIX_BACKEND="wayland"
export BEVY_WINIT_UNIX_BACKEND="wayland"
export WLR_NO_HARDWARE_CURSORS="1"
export BEVY_ASSET_ROOT="${toString ./.}"
# Print diagnostic information
echo "Running with LD_LIBRARY_PATH=\$LD_LIBRARY_PATH"
echo "Available libraries in \$SCRIPT_DIR/../lib:"
ls -l \$SCRIPT_DIR/../lib
exec "\$SCRIPT_DIR/$MAIN_BIN_NAME.bin" "\$@"
EOF
                chmod +x "$out/bin/$MAIN_BIN_NAME"
                
                echo "Created wrapper script for $MAIN_BIN_NAME with required environment variables"
                
                # Also create wrappers for other useful binaries
                for BIN in $(find $out/bin -type f -executable -name "*.bin" | grep -v "$MAIN_BIN_NAME.bin"); do
                  BIN_NAME=$(basename "$BIN" .bin)
                  cat > "$out/bin/$BIN_NAME" << EOF
#!/bin/sh
# Set LD_LIBRARY_PATH to include our dylib directory and run the real binary
SCRIPT_DIR="\$(cd "\$(dirname "\$0")" && pwd)"
export LD_LIBRARY_PATH="\$SCRIPT_DIR/../lib:\$LD_LIBRARY_PATH"
export XDG_RUNTIME_DIR="/run/user/1000"
export WAYLAND_DISPLAY="wayland-1"
export WINIT_UNIX_BACKEND="wayland"
export BEVY_WINIT_UNIX_BACKEND="wayland"
export WLR_NO_HARDWARE_CURSORS="1"
export BEVY_ASSET_ROOT="${toString ./.}"
# Print diagnostic information
echo "Running with LD_LIBRARY_PATH=\$LD_LIBRARY_PATH"
echo "Available libraries in \$SCRIPT_DIR/../lib:"
ls -l \$SCRIPT_DIR/../lib
exec "\$SCRIPT_DIR/$BIN_NAME.bin" "\$@"
EOF
                  chmod +x "$out/bin/$BIN_NAME"
                  echo "Created wrapper script for $BIN_NAME"
                done
              '';
              
              # Extra dependencies and libraries
              buildInputs = nonRustDeps;
              nativeBuildInputs = with pkgs; [
                pkg-config
                llvmPackages.clang
                llvmPackages.bintools
                patchelf
              ];
              
              # Set library paths
              LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath nonRustDeps;
            };
          };
          
          # Alchemist dev environment
          devShells.default = pkgs.mkShell {
            inputsFrom = [
              config.treefmt.build.devShell
            ];
            buildInputs = with pkgs; [
              # Rust development
              # rustcVersion
              rust-toolchain
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
              
              # Add all nonRustDeps to ensure consistency
            ] ++ nonRustDeps;
            
            nativeBuildInputs = with pkgs; [ 
              # Developer tools
              cargo-watch
              just
              
              # Patching and building tools
              patchelf
              llvmPackages.clang
              llvmPackages.bintools
              lld
              
              # Custom scripts
              fix-bevy-linking
            ];
            
            # Environment variables for proper development
            RUST_SRC_PATH = "${rust-toolchain}/lib/rustlib/src/rust/library";
            RUST_BACKTRACE = 1;
            
            # Use the same rustflags as in the build
            RUSTFLAGS = "--cfg edition2024_preview -C linker=clang -C link-arg=-fuse-ld=lld -C link-arg=-Wl,--export-dynamic -C link-arg=-Wl,--undefined-version -C link-arg=-Wl,--allow-shlib-undefined";
            
            # The dynamic linker needs to find our libraries
            NIX_LDFLAGS = "-rpath $PWD/target/debug";
            
            # Set proper environment for nix dev shell
            BEVY_ASSET_ROOT = toString ./.;
            XDG_RUNTIME_DIR = "/run/user/1000";
            WAYLAND_DISPLAY = "wayland-1";
            WINIT_UNIX_BACKEND = "wayland";
            WLR_NO_HARDWARE_CURSORS = "1";
            
            # Tell rust-analyzer about our settings
            RUST_ANALYZER_CONFIG = builtins.readFile rust-analyzer-settings;
            
            # Enable bevy dynamic linking in development
            LD_LIBRARY_PATH = pkgs.lib.concatStringsSep ":" [
              "${pkgs.lib.makeLibraryPath nonRustDeps}"
              "${pkgs.lib.makeLibraryPath [
                pkgs.vulkan-loader
                pkgs.libxkbcommon
                pkgs.wayland
                pkgs.xorg.libX11
                pkgs.xorg.libXcursor
                pkgs.xorg.libXrandr
                pkgs.xorg.libXi
                pkgs.alsa-lib
                pkgs.udev
                pkgs.systemd
                pkgs.stdenv.cc.cc.lib
                pkgs.libGL
              ]}"
              "$PWD/target/debug"
              "$PWD/target/debug/deps"
              "\"$LD_LIBRARY_PATH\""
            ];
            
            # Make sure vscode and other editors can find rust-analyzer
            shellHook = ''
              # Configure Nix to use the local cache
              export NIX_CONFIG="substituters = https://cache.nixos.org/ http://localhost:5000 https://nix-community.cachix.org https://devenv.cachix.org
              trusted-public-keys = cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY= dell-62S6063:F1R/DQVxh0R0YUBXEdVClqDsddJ5VLWVYzPrHC9mmqc= nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs= devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw="
              export RUST_LOG=info
              
              # Create .vscode settings if it doesn't exist
              mkdir -p .vscode
              if [ ! -f .vscode/settings.json ]; then
                echo '{
                  "rust-analyzer.check.command": "clippy",
                  "rust-analyzer.check.features": "all",
                  "rust-analyzer.cargo.features": "all", 
                  "rust-analyzer.procMacro.enable": true,
                  "rust-analyzer.procMacro.attributes.enable": true,
                  "rust-analyzer.cargo.buildScripts.enable": true
                }' > .vscode/settings.json
                echo "Created .vscode/settings.json with rust-analyzer settings"
              fi
              
              # Make sure target/debug exists
              mkdir -p target/debug target/debug/deps
              
              # Find and set up fixedbitset library
              echo "Setting up fixedbitset libraries..."
              FIXEDBITSET_LIBS=$(find /nix/store -name "*fixedbitset*.so*" -type f | grep -v "\.a$" || true)
              if [ -n "$FIXEDBITSET_LIBS" ]; then
                for LIB_PATH in $FIXEDBITSET_LIBS; do
                  LIB_NAME=$(basename "$LIB_PATH")
                  echo "Copying $LIB_PATH to target/debug/deps/$LIB_NAME"
                  cp "$LIB_PATH" "target/debug/deps/$LIB_NAME" || true
                  chmod +x "target/debug/deps/$LIB_NAME" || true
                  
                  # Create symlink for standard name
                  ln -sf "target/debug/deps/$LIB_NAME" "target/debug/deps/libfixedbitset.so" || true
                done
              else
                echo "WARNING: Could not find fixedbitset libraries in /nix/store"
              fi
              
              # Auto-build bevy_dylib if it doesn't exist
              if [ ! -f target/debug/libbevy_dylib.so ]; then
                echo "Building bevy_dylib..."
                cargo build --lib
                
                # Find the dylib
                DYLIB_PATH=$(find ./target -name "libbevy_dylib*.so" | head -n 1)
                if [ -n "$DYLIB_PATH" ]; then
                  echo "Found Bevy dylib at: $DYLIB_PATH"
                  
                  # Create symlink in target/debug if needed
                  if [[ "$DYLIB_PATH" != "./target/debug/libbevy_dylib.so" ]]; then
                    ln -sf "$DYLIB_PATH" "target/debug/libbevy_dylib.so" || true
                    echo "Created symlink: target/debug/libbevy_dylib.so"
                  fi
                else
                  echo "WARNING: Could not find libbevy_dylib*.so after build"
                fi
              fi
              
              # Tell the user about the configuration
              echo "Devshell activated with full Bevy development support"
              echo "Using RUSTFLAGS: $RUSTFLAGS"
              echo "Using LD_LIBRARY_PATH: $LD_LIBRARY_PATH"
              echo "Using bevy dynamic linking - build with 'cargo build' first, then run with 'cargo run'"
              echo "For faster iteration, use: RUSTFLAGS='-A warnings' cargo watch -x run"
              echo "" 
              echo "For builds with local cache, use: just build .#"
              echo "For running with local cache, use: just run .#"
              echo "All just commands: just --list"
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
