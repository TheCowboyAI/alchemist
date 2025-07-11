#!/bin/bash
# Test dialog functionality

echo "Testing Alchemist Dialog UI..."

# Test 1: Create a new dialog
echo "1. Creating new dialog..."
cargo run -- dialog new --title "AI Assistant Test" --model claude-3-sonnet

# Test 2: List dialogs
echo -e "\n2. Listing dialogs..."
cargo run -- dialog list

# Test 3: Dashboard local with dialog interaction
echo -e "\n3. Testing dashboard..."
cargo run -- dashboard-local