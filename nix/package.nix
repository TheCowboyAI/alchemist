{ lib
, pkgs
, nonRustDeps
}:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "ia";
  version = "0.1.0";

  # Use root directory as source
  src = lib.cleanSource ../.;

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

  # Rust flags - use lld linker as per rust.mdc
  RUSTFLAGS = "--cfg edition2024_preview -C link-arg=-fuse-ld=lld";

  # Required for Wayland surface creation
  LD_LIBRARY_PATH = lib.makeLibraryPath nonRustDeps;

  # Set runtime directory for Wayland
  XDG_RUNTIME_DIR = "/tmp";

  # Skip tests (may require graphics)
  doCheck = false;

  postInstall = ''
    # Wrap the binary to set the correct library path for runtime
    wrapProgram $out/bin/${pname} \
      --set LD_LIBRARY_PATH "${lib.makeLibraryPath nonRustDeps}" \
      --set RUST_BACKTRACE full \
      --set WINIT_UNIX_BACKEND wayland
  '';

  meta = with lib; {
    description = "Information Alchemist - A 3D-capable graph editor and visualization system";
    longDescription = ''
      Information Alchemist is a next-generation 3D-capable graph editor and
      visualization system built as part of the Composable Information Machine
      (CIM) ecosystem. It combines advanced graph theory, Entity-Component-System
      (ECS) architecture, Domain-Driven Design (DDD), and Category Theory to
      provide a powerful tool for understanding and manipulating complex
      information relationships.
    '';
    homepage = "https://github.com/thecowboyai/alchemist";
    license = with licenses; [ mit asl20 ];
    maintainers = with maintainers; [ ];
    platforms = platforms.linux;
    mainProgram = pname;
  };
}
