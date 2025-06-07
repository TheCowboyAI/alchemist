#!/bin/bash

echo "Testing Graph Import Functionality"
echo "=================================="
echo ""
echo "Available import shortcuts:"
echo "  Ctrl+I - Import from file (examples/data/sample_graph.json)"
echo "  Ctrl+M - Import Mermaid diagram (examples/data/workflow.mermaid)"
echo "  Ctrl+D - Import DOT graph (examples/data/network.dot)"
echo "  Ctrl+Shift+I - Import from clipboard (sample content)"
echo ""
echo "Starting the application..."
echo ""

# Run the application
./target/x86_64-unknown-linux-gnu/debug/ia
