{ lib
, pkgs
, nonRustDeps
}:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "ia";
  version = "0.1.0";

  # Use root directory as source, including bevy-patched
  src = lib.cleanSourceWith {
    src = ../.;
    filter = path: type:
      # Include everything except .git and target directories
      !(lib.hasSuffix "/.git" path) &&
      !(lib.hasSuffix "/target" path) &&
      !(lib.hasInfix "/target/" path);
  };

  # Use the Cargo lock file
  cargoLock.lockFile = ../Cargo.lock;

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

  # Production build flags - disable dynamic linking for Nix builds
  cargoBuildFlags = "--no-default-features";

  # Environment for build
  BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.alsa-lib}/include";
  BEVY_ASSET_ROOT = toString ../.;

  # Rust flags - use lld for faster linking
  RUSTFLAGS = "-C link-arg=-fuse-ld=lld";

  # Disable patchelf initially to preserve RPATH
  dontPatchELF = true;

  # Post-fixup phase to manually patch RPATH
  postFixup = ''
    # Add vulkan-loader to RPATH to fix experimental occlusion culling linking issues
    patchelf --add-rpath ${pkgs.vulkan-loader}/lib $out/bin/*
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
