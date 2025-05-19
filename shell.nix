# shell.nix - For compatibility with non-flake-enabled Nix
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
    llvmPackages.clang
    llvmPackages.lld
  ];
  
  buildInputs = with pkgs; [
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    libxkbcommon
    wayland
    vulkan-loader
    alsa-lib
    udev
    
    # Required for font rendering
    fontconfig
    freetype
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
    vulkan-loader
    libxkbcommon
    wayland
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    alsa-lib
    udev
  ]);
} 