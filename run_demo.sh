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
echo "Starting the application..."
echo ""

# Run in nix develop environment which sets all the paths correctly
exec nix develop -c cargo run --bin ia 