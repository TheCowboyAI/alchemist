#!/usr/bin/env bash
# Coverage script for Information Alchemist

set -e

echo "Running test coverage analysis..."
echo "================================"

# Set environment variables
export BEVY_HEADLESS=1
export RUST_BACKTRACE=1

# Clean previous coverage data
cargo llvm-cov clean --workspace

# Run tests with coverage
echo "Running tests with coverage..."
cargo llvm-cov --lib --html --output-dir coverage

# Generate summary
echo ""
echo "Coverage Summary:"
echo "================="
cargo llvm-cov --lib --summary-only

# Open coverage report if available
if command -v xdg-open &> /dev/null; then
    echo ""
    echo "Opening coverage report in browser..."
    xdg-open coverage/html/index.html
elif command -v open &> /dev/null; then
    echo ""
    echo "Opening coverage report in browser..."
    open coverage/html/index.html
else
    echo ""
    echo "Coverage report generated at: coverage/html/index.html"
fi
