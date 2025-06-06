#!/usr/bin/env bash
# Build script for Information Alchemist Hyper-V Development VM

set -e

echo "Building Information Alchemist Development VM for Hyper-V..."
echo "This will create a .vhdx file that can be imported into Hyper-V"
echo ""

# Check if we're in the right directory
if [ ! -f "hyperv-flake.nix" ]; then
    echo "Error: This script must be run from the nix/ directory"
    exit 1
fi

# Build the VM
echo "Starting build process..."
echo "This may take 20-30 minutes depending on your system..."
echo ""

nix build -L .#hyperv -f hyperv-flake.nix

if [ $? -eq 0 ]; then
    echo ""
    echo "Build completed successfully!"
    echo ""
    echo "The VM image is located at: ./result/hyperv.vhdx"
    echo ""
    echo "To use this VM:"
    echo "1. Copy the .vhdx file to your Windows machine"
    echo "2. Open Hyper-V Manager"
    echo "3. Create a new Virtual Machine"
    echo "4. Choose 'Use an existing virtual hard disk' and select the .vhdx file"
    echo "5. Configure VM settings:"
    echo "   - Memory: At least 8GB recommended"
    echo "   - Processors: At least 4 vCPUs recommended"
    echo "   - Enable nested virtualization if needed"
    echo "6. Start the VM and log in as 'developer' (password: 'developer')"
    echo ""
    echo "Once logged in:"
    echo "- Run 'setup-ia-dev' to clone the repo and set up the environment"
    echo "- Run 'install-cursor' to install Cursor IDE"
    echo ""
else
    echo "Build failed!"
    exit 1
fi
