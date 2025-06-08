#!/usr/bin/env bash
# Demo script for markdown import with NATS replay

set -e

echo "=== Markdown Import with NATS Replay Demo ==="
echo

# Check if NATS is running
if ! nc -z localhost 4222 2>/dev/null; then
    echo "Starting NATS server..."
    nats-server -js > /tmp/nats.log 2>&1 &
    NATS_PID=$!
    sleep 2
    echo "NATS server started (PID: $NATS_PID)"
else
    echo "NATS server already running"
fi

echo
echo "Building the demo..."
cargo build --example markdown_import_nats_demo --release

echo
echo "Running the demo..."
echo
echo "Controls:"
echo "  M - Import markdown file with Mermaid diagrams"
echo "  R - Replay events from NATS"
echo "  C - Clear current graph"
echo "  ESC - Exit"
echo
echo "Try pressing 'M' to import a markdown file, then 'R' to replay from NATS!"
echo

cargo run --example markdown_import_nats_demo --release

# Cleanup
if [ ! -z "$NATS_PID" ]; then
    echo
    echo "Stopping NATS server..."
    kill $NATS_PID 2>/dev/null || true
fi
