# Test package that builds tests separately
{ lib
, pkgs
, nonRustDeps
, rust-toolchain
}:

pkgs.stdenv.mkDerivation rec {
  pname = "ia-tests";
  version = "0.1.0";

  # Use root directory as source
  src = lib.cleanSource ../.;

  # Build inputs
  buildInputs = nonRustDeps ++ [ rust-toolchain pkgs.vulkan-loader ];

  nativeBuildInputs = with pkgs; [
    pkg-config
    llvmPackages.clang
    llvmPackages.bintools
    lld
    patchelf
    makeWrapper
  ];

  # Build phase
  buildPhase = ''
    export HOME=$TMPDIR
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath (nonRustDeps ++ [ pkgs.vulkan-loader ])}"
    export RUSTFLAGS="-C link-arg=-fuse-ld=lld -C link-arg=-Wl,-rpath,${pkgs.vulkan-loader}/lib"
    export BINDGEN_EXTRA_CLANG_ARGS="-I${pkgs.alsa-lib}/include"
    export BEVY_HEADLESS=1
    export BEVY_ASSET_ROOT="$(pwd)"

    # Try to build tests with --no-default-features
    cargo test --lib --no-default-features --no-run || true

    # If that fails, try with default features
    cargo test --lib --no-run || true
  '';

  # Install phase
  installPhase = ''
    mkdir -p $out/bin

    # Find and copy test binaries
    find target -name "ia-*" -type f -executable | while read test_bin; do
      cp "$test_bin" "$out/bin/"
      patchelf --add-rpath ${pkgs.vulkan-loader}/lib "$out/bin/$(basename "$test_bin")"
    done

    # Create a wrapper script
    cat > $out/bin/run-tests <<EOF
    #!/bin/sh
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath (nonRustDeps ++ [ pkgs.vulkan-loader ])}"
    export BEVY_HEADLESS=1
    export RUST_BACKTRACE=full

    for test in $out/bin/ia-*; do
      echo "Running \$test"
      \$test "\$@"
    done
    EOF

    chmod +x $out/bin/run-tests
  '';

  meta = with lib; {
    description = "Test runner for Information Alchemist";
    platforms = platforms.linux;
  };
}
