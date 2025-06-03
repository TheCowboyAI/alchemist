# Nix derivation for running tests with proper Vulkan support
{ pkgs
, rust-toolchain
, nonRustDeps
}:

pkgs.writeShellScriptBin "run-tests" ''
  # Set up the test environment with proper library paths
  export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath (nonRustDeps ++ [ pkgs.vulkan-loader ])}"

  # Vulkan configuration
  export VK_LAYER_PATH="${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d"
  export VULKAN_SDK="${pkgs.vulkan-headers}"
  export VK_ICD_FILENAMES="${pkgs.mesa}/share/vulkan/icd.d/radeon_icd.x86_64.json:${pkgs.mesa}/share/vulkan/icd.d/intel_icd.x86_64.json"

  # Bevy configuration
  export BEVY_HEADLESS=1
  export RUST_BACKTRACE=full
  export WINIT_UNIX_BACKEND=wayland
  export BEVY_ASSET_ROOT="$(pwd)"

  # Rust configuration
  export RUSTFLAGS="-C link-arg=-fuse-ld=lld -C link-arg=-Wl,-rpath,${pkgs.vulkan-loader}/lib"
  export BINDGEN_EXTRA_CLANG_ARGS="-I${pkgs.alsa-lib}/include"

  # Run the tests
  exec ${rust-toolchain}/bin/cargo test --lib "$@"
''
