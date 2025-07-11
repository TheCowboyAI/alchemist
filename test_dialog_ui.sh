#!/bin/bash
# Test dialog UI functionality

echo "🎨 Testing Alchemist Dialog UI"
echo "=============================="

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust."
    exit 1
fi

# Build the project first
echo -e "\n1. Building Alchemist..."
if cargo build --release; then
    echo "✅ Build successful!"
else
    echo "❌ Build failed!"
    exit 1
fi

# Test dialog commands
echo -e "\n2. Testing Dialog Commands..."

# List dialogs (should be empty initially)
echo -e "\n📋 Listing dialogs:"
./target/release/ia dialog list --count 5

# Create a new dialog
echo -e "\n🆕 Creating new dialog:"
./target/release/ia dialog new --title "AI Assistant Test" --model claude-3-sonnet

echo -e "\n✅ Dialog UI test complete!"
echo "The dialog window should have opened if everything is working correctly."
echo ""
echo "You can also test:"
echo "  - Dashboard: ./target/release/ia dashboard-local"
echo "  - Interactive shell: ./target/release/ia --interactive"