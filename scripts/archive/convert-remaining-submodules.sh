#!/usr/bin/env bash
set -euo pipefail

# Script to convert remaining cim-* directories to submodules
# This automates the process for the remaining directories

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# List of remaining modules to convert
modules=(
    "cim-domain"
    "cim-domain-person"
    "cim-domain-organization"
    "cim-domain-agent"
    "cim-domain-workflow"
    "cim-domain-location"
    "cim-domain-graph"
    "cim-subject"
    "cim-domain-bevy"
    "cim-domain-conceptualspaces"
    "cim-domain-identity"
)

# Function to convert a directory to submodule
convert_directory() {
    local dir=$1

    print_status "Converting $dir to submodule..."

    # Step 1: Initialize git in the directory
    cd "$dir"
    git init
    git add .
    git commit -m "Initial commit: Extract $dir from alchemist monorepo"
    git branch -M main

    # Step 2: Add remote and push
    git remote add origin "https://github.com/TheCowboyAI/$dir.git"
    git push -u origin main

    # Step 3: Go back to main repo
    cd ..

    # Step 4: Remove directory from main repo
    git rm -r "$dir"
    git commit -m "Remove $dir to convert to submodule"

    # Step 5: Add as submodule
    git submodule add "https://github.com/TheCowboyAI/$dir.git" "$dir"
    git commit -m "Add $dir as submodule"

    print_status "Successfully converted $dir!"
}

# Main execution
print_status "Starting batch conversion of remaining cim-* directories..."

for dir in "${modules[@]}"; do
    if [ -d "$dir" ]; then
        print_status "Processing $dir..."
        convert_directory "$dir"
        echo ""
    else
        print_warning "$dir not found, skipping..."
    fi
done

print_status "All directories have been converted!"
print_status "Summary of converted submodules:"
git submodule status | grep cim-
