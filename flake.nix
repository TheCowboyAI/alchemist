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
          nonRustDeps = with pkgs; [
            # Audio support
            alsa-lib
            alsa-utils

            # Wayland support
            wayland
            wayland-protocols
            wayland-scanner
            libxkbcommon

            # X11 support (fallback)
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi

            # Vulkan
            vulkan-headers
            vulkan-loader
            vulkan-validation-layers

            # Graphics
            libGL
            freetype

            # System libraries
            udev
            systemd
            stdenv.cc.cc.lib
            zlib
          ];
        in
        {
          packages = {
            # Main application package - simple build
            default = pkgs.rustPlatform.buildRustPackage {
              pname = "alchemist";
              version = "0.1.0";

              # Use current directory as source
              src = pkgs.lib.cleanSourceWith {
                src = ./.;
                filter = path: type:
                  let
                    baseName = builtins.baseNameOf path;
                    relativePath = pkgs.lib.removePrefix (toString ./.) (toString path);
                  in
                  # Include Rust project files
                  (pkgs.lib.hasSuffix ".rs" path) ||
                  (pkgs.lib.hasSuffix ".toml" path) ||
                  (pkgs.lib.hasSuffix "Cargo.lock" path) ||
                  # Include assets if they exist
                  (pkgs.lib.hasInfix "/assets/" path) ||
                  # Include directories but exclude problematic ones
                  (type == "directory" &&
                  !(baseName == ".git") &&
                  !(baseName == "target") &&
                  !(baseName == ".direnv") &&
                  !(baseName == "result") &&
                  !(baseName == ".cache")
                  );
              };

              # Use the Cargo lock file
              cargoLock.lockFile = ./Cargo.lock;

              # Build inputs for compilation
              buildInputs = nonRustDeps;
              nativeBuildInputs = with pkgs; [
                pkg-config
                llvmPackages.clang
                llvmPackages.bintools
                lld
                patchelf
              ];

              # Environment for build - configure dynamic linking properly
              LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath nonRustDeps;
              BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.alsa-lib}/include";
              BEVY_ASSET_ROOT = toString ./.;

              # Rust flags for dynamic linking
              RUSTFLAGS = "--cfg edition2024_preview -C linker=clang -C link-arg=-fuse-ld=lld";

              # Skip tests (may require graphics)
              doCheck = false;

              # Simple postInstall - set up library paths properly
              postInstall = ''
                # Set up the binary to find both system libraries and its own dylibs
                if [ -f "$out/bin/alchemist" ]; then
                  patchelf --set-rpath "$out/lib:${pkgs.lib.makeLibraryPath nonRustDeps}" "$out/bin/alchemist"
                fi
                
                # Create symlinks for Bevy dylib with the hash the binary expects
                if [ -d "$out/lib" ] && [ -f "$out/bin/alchemist" ]; then
                  # Find what bevy dylib the binary actually needs
                  NEEDED_BEVY=$(patchelf --print-needed "$out/bin/alchemist" | grep "libbevy_dylib-" || true)
                  if [ -n "$NEEDED_BEVY" ] && [ -f "$out/lib/libbevy_dylib.so" ]; then
                    echo "Creating symlink: $NEEDED_BEVY -> libbevy_dylib.so"
                    ln -sf libbevy_dylib.so "$out/lib/$NEEDED_BEVY"
                  fi
                  
                  # Copy required Rust standard library
                  NEEDED_STD=$(patchelf --print-needed "$out/bin/alchemist" | grep "libstd-" || true)
                  if [ -n "$NEEDED_STD" ]; then
                    STD_LIB_PATH=$(find /nix/store -name "$NEEDED_STD" -type f 2>/dev/null | head -1)
                    if [ -n "$STD_LIB_PATH" ]; then
                      echo "Copying Rust std lib: $NEEDED_STD"
                      cp "$STD_LIB_PATH" "$out/lib/"
                      chmod 755 "$out/lib/$NEEDED_STD"
                    fi
                  fi
                fi
              '';
            };
          };

          # Development shell
          devShells.default = pkgs.mkShell {
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
            BEVY_ASSET_ROOT = toString ./.;

            # Rust flags for development
            RUSTFLAGS = "--cfg edition2024_preview -C linker=clang -C link-arg=-fuse-ld=lld";

            shellHook = ''
              echo "Entering Alchemist development environment"
              echo "Available commands:"
              echo "  nix build      - Build the project"
              echo "  nix run        - Run the project"
              echo "  cargo build    - Build with cargo (development)"
              echo "  cargo run      - Run with cargo (development)"
              echo "  cargo test     - Run tests"
              echo "  just           - See available just commands"
              echo ""
              echo "Library path: $LD_LIBRARY_PATH"
            '';
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
