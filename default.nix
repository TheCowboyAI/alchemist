# Simple wrapper around flake.nix for cache-friendly builds
(import
  (
    fetchTarball {
      url = "https://github.com/edolstra/flake-compat/archive/master.tar.gz";
      sha256 = "0m6nmi4jb34rykn3vcsiqy53kdvi1q12g1skit9ii3wnjpyxh7mj";
    }
  )
  {
    src = ./.;
  }).defaultNix.legacyPackages.${builtins.currentSystem}
