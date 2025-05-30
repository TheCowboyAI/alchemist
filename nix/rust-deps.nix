{ pkgs }:

with pkgs; [
  # Audio support
  alsa-lib
  alsa-utils

  # Wayland support
  wayland
  wayland-protocols
  wayland-scanner
  libxkbcommon

  # X11 support (fallback)
  xorg.libX11
  xorg.libXcursor
  xorg.libXrandr
  xorg.libXi

  # Vulkan
  vulkan-headers
  vulkan-loader
  vulkan-validation-layers

  # Graphics
  libGL
  freetype

  # System libraries
  udev
  systemd
  stdenv.cc.cc.lib
  zlib
]
