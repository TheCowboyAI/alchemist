#!/bin/bash

echo "🚀 Information Alchemist (Development Mode)"
echo "=========================================="
echo ""
echo "Running in development mode with cargo..."
echo ""
echo "Controls:"
echo "  F1 - Toggle AI Assistant window"
echo "  ESC - Exit"
echo ""
echo "Starting application..."
echo ""

# Run in nix develop environment which now properly sets library paths
exec nix develop -c cargo run --bin ia 