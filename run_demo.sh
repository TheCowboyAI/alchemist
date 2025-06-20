#!/bin/bash

echo "🚀 Information Alchemist Demo"
echo "============================="
echo ""
echo "This demo shows the AI-powered graph editor with agent assistance."
echo ""
echo "Prerequisites:"
echo "  ✅ NATS running on localhost:4222"
echo "  ✅ Ollama running on localhost:11434 with vicuna model"
echo ""
echo "Controls:"
echo "  F1 - Toggle AI Assistant window"
echo "  F2 - Ask about event sourcing (via F1 first)"
echo "  F3 - Ask about the 8 domains (via F1 first)"
echo "  F4 - Ask about graph editing (via F1 first)"
echo "  ESC - Exit"
echo ""
echo "Building and starting the application..."
echo ""

# Build the nix package which properly wraps the binary with library paths
nix build -L .#ia

# Run the wrapped binary
exec ./result/bin/ia 