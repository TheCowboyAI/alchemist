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
echo "  F1 - Ask 'What is CIM?'"
echo "  F2 - Ask about event sourcing"
echo "  F3 - Ask about the 8 domains"
echo "  F4 - Ask about graph editing"
echo "  H  - Show help"
echo "  ESC - Exit"
echo ""
echo "Starting the application..."
echo ""

# Run with proper environment
RUST_LOG=info,ia=debug nix run 