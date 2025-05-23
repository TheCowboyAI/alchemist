# This file provides a content-addressable approach to caching Rust dependencies
# It works by calculating hashes of only the files that actually affect dependency builds 
# (Cargo.toml, Cargo.lock) rather than using Git revision info

{ lib, stdenv, nix, coreutils }:

let
  # Creates a hash of just the files that matter for Rust dependencies
  # This hash can be used to uniquely identify the build inputs regardless of Git state
  hashDeps = src: extraFiles:
    let
      # Create a list of all files to consider in the hash
      # Only include files that actually affect dependency resolution, not the app code
      filesToHash = builtins.concatStringsSep " " ([
        "${src}/Cargo.toml"
        "${src}/Cargo.lock"
      ] ++ (map (file: "${src}/${file}") extraFiles));
    in
    ''
      # Compute hash of dependency files
      echo "Computing hash of Rust dependencies input files..."
      hash=$(${coreutils}/bin/cat ${filesToHash} | ${nix}/bin/nix-hash --type sha256 --flat --base32 -)
      echo "Rust dependencies hash: $hash"
      echo $hash
    '';

  # Function to extract dependency hash from source
  getDepsHash = src: extraFiles:
    let
      script = hashDeps src extraFiles;
    in
    builtins.unsafeDiscardStringContext (builtins.readFile (
      runCommand "deps-hash.txt" { } ''
        ${script} > $out
      ''
    ));

  # Command wrapper
  runCommand = name: env: script:
    stdenv.mkDerivation (env // {
      inherit name;
      passAsFile = [ "buildCommand" ];
      buildCommand = script;
    });

in
{
  inherit hashDeps getDepsHash;
}
