{ pkgs ? import <nixpkgs> { } }:

let
  # Import our cache configuration
  cacheConfig = import ./cache-config.nix;

  # Generate a unique hash from derivation
  hashForAttr = attr: pkgs.lib.substring 0 8 (builtins.hashString "sha256" (builtins.toJSON attr));

  # Write config to a file
  writeConfig = pkgs.writeTextFile {
    name = "nix-cache-config";
    text = cacheConfig.nixConfig;
    destination = "/share/nix-cache-config.conf";
  };

  # Create a script to push a derivation to the cache
  push-to-cache = pkgs.writeScriptBin "push-to-cache" ''
    #!/usr/bin/env bash
    set -e
    
    # Ensure we have the right path to push
    if [ $# -lt 1 ]; then
      echo "Usage: push-to-cache <path-to-push>"
      echo "Example: push-to-cache ./result"
      exit 1
    fi
    
    PATH_TO_PUSH="$1"
    
    # If the path is a symbolic link, resolve it
    if [ -L "$PATH_TO_PUSH" ]; then
      REAL_PATH=$(readlink -f "$PATH_TO_PUSH")
      echo "Resolved symlink $PATH_TO_PUSH to $REAL_PATH"
      PATH_TO_PUSH="$REAL_PATH"
    fi
    
    # Validate that this is a /nix/store path
    if [[ ! "$PATH_TO_PUSH" =~ ^/nix/store/.* ]]; then
      echo "Error: Path must be a /nix/store path, but got: $PATH_TO_PUSH"
      exit 1
    fi
    
    echo "Pushing to cache: $PATH_TO_PUSH"
    echo "Cache URL: ${cacheConfig.localCache}"
    echo "Cache Key: ${cacheConfig.localCacheKey}"
    
    # Push the path and its dependencies to the cache
    echo "Pushing with runtime deps..."
    nix copy --to ${cacheConfig.localCache} "$PATH_TO_PUSH"
    
    echo "Pushing derivation closure..."
    DRV_PATH=$(nix-store -q --deriver "$PATH_TO_PUSH")
    if [ "$DRV_PATH" != "unknown-deriver" ]; then
      echo "Pushing derivation: $DRV_PATH"
      nix copy --to ${cacheConfig.localCache} "$DRV_PATH"
    else
      echo "No derivation path found for $PATH_TO_PUSH"
    fi
    
    echo "Done! Path and dependencies pushed to cache."
    
    # Verify the path is now in the cache
    echo -n "Verifying path in cache: "
    if nix path-info --store ${cacheConfig.localCache} "$PATH_TO_PUSH" &>/dev/null; then
      echo "✅ Successfully pushed to cache"
    else
      echo "❌ Failed to push to cache"
      exit 1
    fi
  '';

  # Push rustDeps to the cache specifically
  push-rust-deps = pkgs.writeScriptBin "push-rust-deps" ''
    #!/usr/bin/env bash
    set -e
    
    echo "Building rust dependencies..."
    nix build .#rustDeps --no-link
    
    RUST_DEPS=$(nix-build --no-out-link .#rustDeps)
    echo "Pushing rust dependencies to cache: $RUST_DEPS"
    
    # Use our push-to-cache script
    ${push-to-cache}/bin/push-to-cache "$RUST_DEPS"
  '';

  # Push the main package to the cache
  push-main-package = pkgs.writeScriptBin "push-main-package" ''
    #!/usr/bin/env bash
    set -e
    
    echo "Building main package..."
    nix build .#default --no-link
    
    MAIN_PKG=$(nix-build --no-out-link .#default)
    echo "Pushing main package to cache: $MAIN_PKG"
    
    # Use our push-to-cache script
    ${push-to-cache}/bin/push-to-cache "$MAIN_PKG"
  '';

  # Create a script to poll the cache periodically and check for missing dependencies
  cache-monitor = pkgs.writeScriptBin "cache-monitor" ''
    #!/usr/bin/env bash
    
    INTERVAL=${if builtins.getEnv "INTERVAL" == "" then "60" else builtins.getEnv "INTERVAL"}
    
    echo "Starting cache monitor (polling every $INTERVAL seconds)"
    echo "Press Ctrl+C to exit"
    
    while true; do
      clear
      echo "Alchemist Cache Monitor - $(date)"
      echo "=================================="
      echo "Cache URL: ${cacheConfig.localCache}"
      
      # Check cache connectivity
      if curl -s -I ${cacheConfig.localCache}/nix-cache-info >/dev/null; then
        echo "✅ Cache server is reachable"
      else
        echo "❌ Cannot connect to cache server"
      fi
      
      # Check if common packages are in the cache
      echo -e "\nChecking key packages:"
      
      PKG_PATH=$(nix-build --no-out-link .#rustDeps 2>/dev/null || echo "")
      if [ -n "$PKG_PATH" ]; then
        if nix path-info --store ${cacheConfig.localCache} "$PKG_PATH" &>/dev/null; then
          echo "✅ rustDeps is cached"
        else
          echo "❌ rustDeps is not cached"
        fi
      else
        echo "⚠️ Could not build rustDeps"
      fi
      
      # Check main package
      PKG_PATH=$(nix-build --no-out-link .#default 2>/dev/null || echo "")
      if [ -n "$PKG_PATH" ]; then
        if nix path-info --store ${cacheConfig.localCache} "$PKG_PATH" &>/dev/null; then
          echo "✅ Main package is cached"
        else
          echo "❌ Main package is not cached"
        fi
      else
        echo "⚠️ Could not build main package"
      fi
      
      sleep $INTERVAL
    done
  '';

  # Put everything together in one environment
  cache-tools = pkgs.buildEnv {
    name = "alchemist-cache-tools";
    paths = [
      push-to-cache
      push-rust-deps
      push-main-package
      cache-monitor
      writeConfig
    ];
  };

in
{
  inherit push-to-cache push-rust-deps push-main-package cache-monitor cache-tools;
}
