let
  rust-overlay = builtins.fetchTarball {
    url = https://github.com/oxalica/rust-overlay/archive/master.tar.gz;
  };
  pkgs = import <nixpkgs> { overlays = [ (import rust-overlay) ]; };
in
  builtins.attrNames pkgs.rust-bin.nightly 