# pure-inputs.nix
# This file provides a pure source input for the Alchemist project
# It filters out Git-related files and only includes what's needed for the build

{ pkgs ? import <nixpkgs> {} }:

let
  # Function to create a filtered source
  filterSource = src: patterns:
    let
      # Function to check if a path should be included
      checkPath = path: type:
        let
          relPath = builtins.substring (builtins.stringLength (toString src) + 1) 
                                      (builtins.stringLength path) 
                                      (toString path);
          # Check if path matches any of the include patterns
          matches = pattern: 
            builtins.match pattern relPath != null;
        in
          builtins.any matches patterns;
    in
      builtins.filterSource checkPath src;
      
  # Create a filtered source with only the files we need
  # This excludes .git, target dirs, and other non-essential files
  filteredSource = filterSource ./. [
    "^src/.*\\.rs$"              # Rust source files
    "^Cargo\\.toml$"             # Cargo manifests
    "^Cargo\\.lock$"
    "^cache-config\\.nix$"       # Cache configuration
    "^cache-tools\\.nix$"
    "^cache-management\\.nix$"
    "^analyze-cache-miss\\.nix$"
    "^flake\\.nix$"              # Flake files
    "^flake\\.lock$"
    "^pure-inputs\\.nix$"        # This file
    "^justfile$"                 # Build commands
    "^.*\\.md$"                  # Documentation
    "^assets/.*"                 # Assets directory
  ];
  
in {
  # The filtered source that can be used for reproducible builds
  source = filteredSource;
  
  # A derivation containing just the filtered source
  # This can be used as an input to other derivations
  sourceDrv = pkgs.runCommand "alchemist-filtered-source" {} ''
    cp -r ${filteredSource} $out
  '';
} 