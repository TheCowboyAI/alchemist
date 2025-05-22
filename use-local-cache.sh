#!/usr/bin/env bash

# Script to build using the local Nix cache

# The local cache URL
LOCAL_CACHE="http://localhost:5000"
# The local cache public key
LOCAL_CACHE_KEY="dell-62S6063:F1R/DQVxh0R0YUBXEdVClqDsddJ5VLWVYzPrHC9mmqc="
# The standard Nix cache key
NIX_CACHE_KEY="cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
# Nix community cache key
NIX_COMMUNITY_KEY="nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
# Devenv cachix key
DEVENV_KEY="devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw="

# Get all arguments passed to this script
ARGS="$@"

# If no arguments provided, use default target
if [ -z "$ARGS" ]; then
  ARGS=".#"
fi

# Print info
echo "Building with local cache at $LOCAL_CACHE"
echo "Target: $ARGS"
echo ""

# Run Nix build with all caches configured
nix build \
  --option substituters "https://cache.nixos.org/ $LOCAL_CACHE https://nix-community.cachix.org https://devenv.cachix.org" \
  --option trusted-public-keys "$NIX_CACHE_KEY $LOCAL_CACHE_KEY $NIX_COMMUNITY_KEY $DEVENV_KEY" \
  -L \
  $ARGS 