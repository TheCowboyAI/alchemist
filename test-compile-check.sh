#!/bin/bash
# Script to check test compilation

echo "Checking basic_integration_test..."
cargo test --test basic_integration_test --no-run 2>&1 | grep -E "error\[E[0-9]+\]" | head -10

echo -e "\nChecking simple_passing_test..."
cargo test --test simple_passing_test --no-run 2>&1 | grep -E "error\[E[0-9]+\]" | head -10

echo -e "\nChecking graph_integration_test (without bevy feature)..."
cargo test --test graph_integration_test --no-run 2>&1 | grep -E "error\[E[0-9]+\]" | head -10

echo -e "\nChecking graph_integration_test (with bevy feature)..."
cargo test --test graph_integration_test --no-run --features bevy 2>&1 | grep -E "error\[E[0-9]+\]" | head -10

echo -e "\nChecking ai_model_tests..."
cargo test --test ai_model_tests --no-run 2>&1 | grep -E "error\[E[0-9]+\]" | head -10