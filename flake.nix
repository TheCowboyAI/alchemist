{
  description = "Information Alchemist";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };
  outputs = { self, nixpkgs }:
    let
      pkgs = import nixpkgs { system = "x86_64-linux"; config.allowUnfree = true;};
    in
    {
      devShells."x86_64-linux".default = with pkgs; mkShell rec {
        packages = [
          libiconv
          gcc
          alsa-lib
          alsa-utils
          udev
          pkg-config
          cargo
          cargo-watch
          rustc
          rustfmt
          rustPackages.clippy
          rust-analyzer

          systemd
          wayland
          waylandpp
          libxkbcommon
          glfw
          freetype
          vulkan-headers
          vulkan-loader
          vulkan-validation-layers
          vulkan-tools # vulkaninfo
          vulkan-tools-lunarg # vkconfig
          shaderc # GLSL to SPIRV compiler - glslc
          renderdoc # Graphics debugger
          tracy
          google-chrome 
        ];

        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;
        VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
        RUST_SRC_PATH = rustPlatform.rustLibSrc;
      };
    };
}
