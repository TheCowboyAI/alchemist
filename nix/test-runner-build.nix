# Test runner using buildRustPackage with minimal features
{ lib
, pkgs
, nonRustDeps
}:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "ia-test-runner";
  version = "0.1.0";

  src = lib.cleanSource ../.;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  buildInputs = nonRustDeps ++ (with pkgs; [
    vulkan-loader
    vulkan-validation-layers
  ]);

  nativeBuildInputs = with pkgs; [
    pkg-config
    llvmPackages.clang
    llvmPackages.bintools
    lld
    makeWrapper
  ];

  # Override cargo test command to use minimal features
  checkPhase = ''
    runHook preCheck

    export BEVY_HEADLESS=1
    export RUST_BACKTRACE=full
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath (nonRustDeps ++ [ pkgs.vulkan-loader ])}"
    export VK_LAYER_PATH="${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d"
    export RUSTFLAGS="-C link-arg=-fuse-ld=lld"

    # Run tests with minimal features
    cargo test --lib \
      --no-default-features \
      --features "bevy/bevy_asset,bevy/bevy_core_pipeline,bevy/bevy_render,bevy/bevy_window,bevy/bevy_winit,bevy/multi_threaded" \
      -- --nocapture

    runHook postCheck
  '';

  # Create a wrapper script for running tests
  postInstall = ''
    mkdir -p $out/bin

    makeWrapper ${pkgs.cargo}/bin/cargo $out/bin/ia-test-runner \
      --set BEVY_HEADLESS 1 \
      --set RUST_BACKTRACE full \
      --set LD_LIBRARY_PATH "${pkgs.lib.makeLibraryPath (nonRustDeps ++ [ pkgs.vulkan-loader ])}" \
      --set VK_LAYER_PATH "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d" \
      --add-flags "test" \
      --add-flags "--lib" \
      --add-flags "--no-default-features" \
      --add-flags "--features" \
      --add-flags "bevy/bevy_asset,bevy/bevy_core_pipeline,bevy/bevy_render,bevy/bevy_window,bevy/bevy_winit,bevy/multi_threaded"
  '';

  # Skip the normal build phase since we only want tests
  buildPhase = ''
    echo "Skipping build phase - only running tests"
  '';

  # We want to run tests, not build the package
  doCheck = true;
  dontBuild = true;

  meta = with lib; {
    description = "Test runner for Information Alchemist";
    platforms = platforms.linux;
  };
}
