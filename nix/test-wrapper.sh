#!/usr/bin/env bash
# Test wrapper script for running Bevy tests with proper Vulkan support

# Exit on error
set -e

# Get the directory of this script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

# Source the Nix environment
if [ -f "$PROJECT_ROOT/.envrc" ]; then
    source "$PROJECT_ROOT/.envrc"
fi

# Ensure we're in the project directory
cd "$PROJECT_ROOT"

# Run cargo test with all the necessary environment variables
exec cargo test --lib "$@"
