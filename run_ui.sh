#!/bin/bash
# Simple script to run Alchemist UI components

echo "═══════════════════════════════════════════"
echo "    🧪 ALCHEMIST UI Runner"
echo "═══════════════════════════════════════════"
echo ""
echo "Select a component to run:"
echo "1. Minimal Test UI"
echo "2. Dashboard"
echo "3. Dialog Window"
echo "4. Simple Launcher"
echo "5. Exit"
echo ""
read -p "Enter choice (1-5): " choice

case $choice in
    1)
        echo "Building and running minimal test..."
        cargo run --example minimal_ui_test
        ;;
    2)
        echo "Building and running dashboard..."
        cargo run --example nats_dashboard_demo
        ;;
    3)
        echo "Building and running dialog window..."
        cargo run --example dialog_window_demo
        ;;
    4)
        echo "Building and running simple launcher..."
        cargo run --example simple_launcher_demo
        ;;
    *)
        echo "Exiting..."
        exit 0
        ;;
esac