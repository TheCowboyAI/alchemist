# Test runner that mimics production build configuration
{ pkgs
, rust-toolchain
, nonRustDeps
}:

pkgs.writeShellScriptBin "test-runner" ''
  # Set up the environment exactly like the production build
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

  # Use the same Rust flags as production build
  export RUSTFLAGS="-C link-arg=-fuse-ld=lld -C link-arg=-Wl,-rpath,${pkgs.vulkan-loader}/lib"
  export BINDGEN_EXTRA_CLANG_ARGS="-I${pkgs.alsa-lib}/include"

  # Try to run tests with the same configuration as production
  exec ${rust-toolchain}/bin/cargo test --lib --no-default-features "$@"
''
