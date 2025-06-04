{ pkgs, lib, rustPlatform, fetchFromGitHub }:

rustPlatform.buildRustPackage rec {
  pname = "bevy-patched";
  version = "0.16.1";

  src = fetchFromGitHub {
    owner = "bevyengine";
    repo = "bevy";
    rev = "v${version}";
    hash = "sha256-PLACEHOLDER"; # We'll need to get the actual hash
  };

  # Apply our patch to remove experimental occlusion culling
  patches = [ ../patches/bevy-remove-experimental-occlusion.patch ];

  cargoLock = {
    lockFile = "${src}/Cargo.lock";
  };

  # Bevy's dependencies
  buildInputs = with pkgs; [
    alsa-lib
    udev
    vulkan-loader
    libxkbcommon
    wayland
  ];

  nativeBuildInputs = with pkgs; [
    pkg-config
    rustPlatform.bindgenHook
  ];

  # Build all Bevy crates
  buildPhase = ''
    cargo build --release --workspace
  '';

  # We don't install binaries, just build the libraries
  installPhase = ''
    mkdir -p $out
    cp -r target/release/deps $out/
  '';

  meta = with lib; {
    description = "Patched Bevy without experimental occlusion culling";
    license = licenses.mit;
  };
}
