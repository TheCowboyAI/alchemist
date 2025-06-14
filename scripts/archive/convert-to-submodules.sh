#!/usr/bin/env bash
set -euo pipefail

# Convert CIM directories to Git submodules
# This script helps convert local directories to proper git submodules

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d ".git" ]; then
    print_error "This script must be run from the alchemist repository root"
    exit 1
}

# List of modules to convert
modules=(
    "cim-component"
    "cim-infrastructure"
    "cim-compose"
    "cim-ipld"
    "cim-contextgraph"
    "cim-workflow-graph"
    "cim-ipld-graph"
    "cim-conceptgraph"
    "cim-domain-bevy"
)

# Repository descriptions
declare -A descriptions=(
    ["cim-component"]="Core component definitions for CIM"
    ["cim-infrastructure"]="Infrastructure layer for CIM"
    ["cim-compose"]="Composition utilities for CIM"
    ["cim-ipld"]="IPLD integration for CIM"
    ["cim-contextgraph"]="Context graph implementation for CIM"
    ["cim-workflow-graph"]="Workflow graph implementation for CIM"
    ["cim-ipld-graph"]="IPLD graph integration for CIM"
    ["cim-conceptgraph"]="Concept graph implementation for CIM"
    ["cim-domain-bevy"]="Bevy-based visualization components for CIM"
)

# Function to convert a directory to submodule
convert_to_submodule() {
    local dir=$1
    local desc="${descriptions[$dir]}"

    print_status "Converting $dir to submodule..."

    # Check if directory exists
    if [ ! -d "$dir" ]; then
        print_warning "$dir does not exist, skipping..."
        return
    fi

    # Check if already a submodule
    if git submodule status "$dir" 2>/dev/null | grep -q "^[+-]"; then
        print_warning "$dir is already a submodule, skipping..."
        return
    fi

    print_status "Step 1: Creating GitHub repository..."
    echo "Please create repository at: https://github.com/TheCowboyAI/$dir"
    echo "Description: $desc"
    echo "Press Enter when repository is created..."
    read -r

    print_status "Step 2: Preparing directory content..."
    (
        cd "$dir"

        # Initialize git repo if not already
        if [ ! -d ".git" ]; then
            git init
            git add .
            git commit -m "Initial commit: Extract from alchemist monorepo"
        fi

        # Set up remote
        git branch -M main
        git remote add origin "https://github.com/TheCowboyAI/$dir.git" 2>/dev/null || \
            git remote set-url origin "https://github.com/TheCowboyAI/$dir.git"

        print_status "Pushing to remote repository..."
        git push -u origin main
    )

    print_status "Step 3: Removing directory from main repository..."
    git rm -r "$dir"
    git commit -m "Remove $dir to convert to submodule"

    print_status "Step 4: Adding as submodule..."
    git submodule add "https://github.com/TheCowboyAI/$dir.git" "$dir"
    git commit -m "Add $dir as submodule"

    print_status "Successfully converted $dir to submodule!"
}

# Main execution
print_status "Starting CIM directory to submodule conversion..."
print_warning "This process will modify your git repository!"
print_warning "Make sure you have committed all changes before proceeding."
echo "Press Enter to continue or Ctrl+C to cancel..."
read -r

# Convert each directory
for dir in "${modules[@]}"; do
    echo ""
    print_status "Processing $dir..."
    convert_to_submodule "$dir"
done

print_status "All directories have been processed!"
print_status "Don't forget to:"
echo "  1. Test the build with: cargo build"
echo "  2. Update CI/CD configuration if needed"
echo "  3. Document the submodule setup in README.md"
