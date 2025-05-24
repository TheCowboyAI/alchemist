#!/usr/bin/env bash

echo "Starting Alchemist Graph Viewer..."
echo ""
echo "Features available:"
echo "  - 3D/2D viewing modes (Tab to switch)"
echo "  - Graph pattern generation"
echo "  - File loading from assets/models/"
echo "  - Fixed 2D zoom for better visibility"
echo ""

# Run using nix if available, otherwise try cargo
if command -v nix &> /dev/null; then
    echo "Running with nix..."
    nix run
else
    echo "Running with cargo..."
    cargo run
fi 