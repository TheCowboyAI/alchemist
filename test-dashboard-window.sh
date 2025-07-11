#!/usr/bin/env bash
# Test script to verify dashboard window works

echo "Testing dashboard window..."
echo "Building with dashboard-local command..."

# Run in nix develop environment
exec nix develop -c cargo run --bin ia -- dashboard-local