#!/usr/bin/env bash

# nix-watch.sh - Watch for changes and rebuild/run with Nix
# 
# This script uses a pure Nix approach to building and running in NixOS,
# avoiding direct Cargo usage which has library path issues.

set -e

echo "🔎 Starting Nix-only watch mode for Alchemist"
echo "👀 Watching source files for changes..."
echo "🚀 Will rebuild and run using Nix when changes are detected"
echo ""

# Track the last modification time
last_mod_time=0

# Function to get the latest modification time of project files
get_latest_mod_time() {
    find src -name "*.rs" -o -name "Cargo.toml" -o -name "Cargo.lock" -o -name "*.nix" -type f -exec stat -c %Y {} \; | sort -nr | head -n1
}

# Function to build and run the application using only Nix commands
build_and_run() {
    echo "🔨 Building with pure Nix approach (avoiding Cargo)..."
    
    # Use --no-link to avoid creating result symlinks each time
    if nix build --no-warn-dirty --no-link -L; then
        echo "✅ Build successful, running with nix run..."
        nix run --no-warn-dirty
    else
        echo "❌ Build failed"
        echo "🔍 This may be due to library path issues in NixOS"
        echo "   We're using pure Nix commands to handle this"
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
        echo "🔄 Changes detected at $(date '+%H:%M:%S'), rebuilding..."
        last_mod_time=$current_mod_time
        build_and_run
    fi
done 