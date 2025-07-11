#!/bin/bash

# Test script for markdown and chart renderers

echo "Testing Alchemist Markdown and Chart Renderers"
echo "============================================="

# Build the project first
echo "Building alchemist-renderer..."
cd alchemist-renderer
cargo build --bin alchemist-renderer
cd ..

echo ""
echo "1. Testing Markdown Renderer Demo"
echo "---------------------------------"
cargo run -- render demo markdown

echo ""
echo "2. Testing Chart Renderer Demo"
echo "------------------------------"
cargo run -- render demo chart

echo ""
echo "3. Testing Markdown File Rendering"
echo "----------------------------------"
cargo run -- render markdown alchemist-renderer/examples/markdown_example.md

echo ""
echo "4. Testing Chart File Rendering (Line Chart)"
echo "-------------------------------------------"
cargo run -- render chart alchemist-renderer/examples/chart_example.json

echo ""
echo "5. Testing Chart File Rendering (Bar Chart)"
echo "------------------------------------------"
cargo run -- render chart alchemist-renderer/examples/bar_chart_example.json --chart-type bar

echo ""
echo "6. Testing Chart File Rendering (Pie Chart)"
echo "------------------------------------------"
cargo run -- render chart alchemist-renderer/examples/pie_chart_example.json --chart-type pie

echo ""
echo "Test complete!"