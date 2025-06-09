{ lib
, pkgs
, nonRustDeps
, srcOverride ? null
}:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "ia";
  version = "0.1.0";

  # Use provided src or the parent directory
  src = if srcOverride != null then srcOverride else ../.;

  # Use the Cargo lock file
  cargoLock = {
    lockFile = ../Cargo.lock;
    outputHashes = {
      "bevy_egui-0.34.1" = "sha256-c31dDcMX4Wnktlz7/1UFf9neWJLhhnlYfpI2jipB1Dk=";
    };
  };

  # Build inputs for compilation
  buildInputs = nonRustDeps;

  nativeBuildInputs = with pkgs; [
    pkg-config
    llvmPackages.clang
    llvmPackages.bintools
    lld
    patchelf
    makeWrapper
  ];

  # Production build flags - build without dynamic linking to avoid Bevy 0.16.1 issues
  cargoBuildFlags = "";

  # Disable tests by default - we'll run them explicitly when needed
  doCheck = false;

  # Environment for build
  BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.alsa-lib}/include";
  BEVY_ASSET_ROOT = toString ../.;

  # Rust flags - use lld for faster linking
  RUSTFLAGS = "-C link-arg=-fuse-ld=lld";

  # Disable patchelf initially to preserve RPATH
  dontPatchELF = true;

    # Install phase to copy assets
  postInstall = ''
    # Copy assets if they exist
    if [ -d assets ]; then
      mkdir -p $out/share/ia
      cp -r assets $out/share/ia/
    fi
  '';

      # Post-fixup phase to manually patch RPATH and wrap binaries
  postFixup = ''
    # Add necessary libraries to RPATH
    for bin in $out/bin/*; do
      patchelf --add-rpath "${pkgs.lib.makeLibraryPath nonRustDeps}" "$bin"

      # Wrap the binary with proper library paths
      wrapProgram "$bin" \
        --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath nonRustDeps}" \
        --set VK_LAYER_PATH "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d" \
        --set BEVY_ASSET_ROOT "$out/share/ia/assets"
    done
  '';

  # Metadata
  meta = with lib; {
    description = "Information Alchemist - A graph editor and workflow manager for Domain Driven Design";
    homepage = "https://github.com/thecowboyai/alchemist";
    license = licenses.mit;
    maintainers = [ ];
    platforms = platforms.linux;
  };
}
