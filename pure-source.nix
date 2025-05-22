# pure-source.nix - Creates a content-addressable source input that doesn't depend on Git state
{ pkgs ? import <nixpkgs> {} }:

let
  # Function to filter sources based on file patterns that matter for builds
  filterSource = src: extraFiles:
    let
      isRelevantFile = file: type:
        let
          baseName = builtins.baseNameOf file;
          validPath =
            # Include Rust project files
            (pkgs.lib.hasSuffix ".rs" file) ||
            (pkgs.lib.hasSuffix ".toml" file) ||
            (pkgs.lib.hasSuffix "Cargo.lock" file) ||
            (pkgs.lib.hasSuffix ".cargo/config.toml" file) ||
            # Allow assets directory
            (pkgs.lib.hasInfix "/assets/" file) ||
            # Build-related files
            (pkgs.lib.hasSuffix ".nix" file) ||
            (pkgs.lib.hasSuffix "justfile" file) ||
            # Allow doc files (needed for build)
            (pkgs.lib.hasInfix "/doc/" file) ||
            # Explicitly included files
            (builtins.elem baseName extraFiles);
          
          # Skip directories that aren't needed
          isIgnoredDir = 
            (pkgs.lib.hasInfix "/.git/" file) ||
            (pkgs.lib.hasInfix "/target/" file) ||
            (pkgs.lib.hasInfix "/.direnv/" file) ||
            (pkgs.lib.hasInfix "/result" file);
          
        in
          # Ignore Git and other directories that change often
          (type == "directory" && !isIgnoredDir) ||
          # Only include specific file types
          (type == "regular" && validPath);
    in
      builtins.filterSource isRelevantFile src;

  # Create a pure source derivation with only the files needed for building
  pureSource = extraFiles: pkgs.stdenv.mkDerivation {
    name = "alchemist-pure-source";
    src = filterSource ./. extraFiles;
    
    # Don't need to build anything
    dontBuild = true;
    
    # Just copy the filtered source
    installPhase = ''
      cp -r $src $out
    '';
    
    # Don't extract version info from Git
    CARGO_GIT_DIR = "";
    GIT_DIR = "";
  };
in
{
  # Source for the main application
  appSource = pureSource [
    "README.md"
    "cache-config.nix"
    "flake.lock"
  ];
  
  # Source for just the dependencies
  depsSource = pureSource [
    "Cargo.toml"
    "Cargo.lock"
  ];
  
  # The filter function for use in other Nix expressions
  inherit filterSource;
} 