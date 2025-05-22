#!/usr/bin/env bash

# nix-pure-watch.sh - Watch for changes and rebuild/run with Nix using content-addressable approach
# 
# This script uses a content-addressable approach to build and run the project
# which is independent of Git state, providing better caching and reproducibility.

set -e

echo "🔎 Starting Nix pure content-addressable watch mode for Alchemist"
echo "👀 Watching source files for changes..."
echo "🚀 Will rebuild and run using content-addressable approach that avoids Git-state"
echo ""

# Track the last modification time
last_mod_time=0

# Function to get the latest modification time of project files
get_latest_mod_time() {
    find src -name "*.rs" -o -name "Cargo.toml" -o -name "Cargo.lock" -o -name "*.nix" -type f -exec stat -c %Y {} \; | sort -nr | head -n1
}

# Function to build and run the application using the pure content approach
build_and_run() {
    echo "🔨 Building with content-addressable approach ($(date '+%H:%M:%S'))..."
    
    # Check if pure-source.nix exists, create it if missing
    if [ ! -f pure-source.nix ]; then
        echo "Error: pure-source.nix not found. Please create it first."
        return 1
    fi
    
    # Build with content filtering
    if nix build --no-warn-dirty --no-link -L .#; then
        echo "✅ Build successful, running..."
        just run-pure
    else
        echo "❌ Build failed"
    fi
}

# Initial build and run
build_and_run
last_mod_time=$(get_latest_mod_time)

echo "Watching for changes (Press Ctrl+C to stop)..."

# Watch for changes
while true; do
    sleep 2
    current_mod_time=$(get_latest_mod_time)
    
    if [ "$current_mod_time" != "$last_mod_time" ]; then
        echo "🔄 Changes detected, rebuilding..."
        last_mod_time=$current_mod_time
        build_and_run
    fi
done 