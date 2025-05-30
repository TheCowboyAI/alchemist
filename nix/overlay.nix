# Overlay to add Information Alchemist (ia) to nixpkgs
final: prev: {
  ia = final.callPackage ./package.nix {
    nonRustDeps = import ./rust-deps.nix { pkgs = final; };
  };
}
