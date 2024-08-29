{
  description = "Information Alchemist";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };
  outputs = { self, nixpkgs }:
    let
      pkgs = import nixpkgs { system = "x86_64-linux"; };
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
          rustc
          rustfmt
          rustPackages.clippy
          rust-analyzer

          systemd
          wayland
          waylandpp
          libxkbcommon
          vulkan-loader
        ];

        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;

        RUST_SRC_PATH = rustPlatform.rustLibSrc;
      };
    };
}
