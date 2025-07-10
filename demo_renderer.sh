#!/bin/bash
# Demo script to show Alchemist renderer capabilities

echo "🚀 Alchemist Renderer Demo"
echo "========================="
echo ""
echo "This demo shows how Alchemist can spawn both Bevy (3D) and Iced (2D) windows"
echo ""

# Build the projects first
echo "Building Alchemist..."
cargo build --release

echo "Building Alchemist Renderer..."
cd alchemist-renderer && cargo build --release && cd ..

echo ""
echo "Available demos:"
echo "1. 3D Graph Visualization (Bevy)"
echo "2. Document Viewer (Iced)"
echo "3. Text Editor (Iced)"
echo "4. Split View (Multiple Windows)"
echo ""

# Run interactive shell with render demo
echo "Starting Alchemist interactive shell..."
echo "Try these commands:"
echo "  render demo graph3d    - Launch 3D graph demo"
echo "  render demo document   - Launch document viewer demo"
echo "  render list           - List active windows"
echo "  render graph          - Create a simple 3D graph"
echo ""

./target/release/alchemist --interactive