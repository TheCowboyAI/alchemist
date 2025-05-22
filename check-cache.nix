{ pkgs ? import <nixpkgs> {} }:

let
  # We'll create a dummy derivation that depends on something commonly used
  dummy = pkgs.runCommand "dummy" {} ''
    touch $out
  '';
in {
  inherit dummy;
} 