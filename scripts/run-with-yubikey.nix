#!/usr/bin/env nix-shell
#!nix-shell -i bash -p pcsclite yubikey-manager pcsctools

# Script to run programs with YubiKey/PC/SC support
# This ensures all required libraries are available

set -e

echo "YubiKey Support Runner (Nix)"
echo "============================"

# Check if pcscd is running
if systemctl is-active --quiet pcscd 2>/dev/null; then
    echo "✓ PC/SC daemon is running (system service)"
elif pgrep -x pcscd > /dev/null; then
    echo "✓ PC/SC daemon is running"
else
    echo "⚠ PC/SC daemon is not running"
    echo ""
    echo "Starting pcscd..."
    
    # Try to start pcscd
    if command -v systemctl &> /dev/null; then
        sudo systemctl start pcscd || {
            echo "Failed to start pcscd via systemctl"
            exit 1
        }
    else
        echo "Please start pcscd manually: sudo pcscd --foreground"
        exit 1
    fi
fi

# List YubiKeys
echo ""
echo "Checking for YubiKeys..."
ykman list || echo "No YubiKeys detected"

# Run the command with proper library paths
echo ""
echo "Running: $@"
echo ""

# The nix-shell already sets up the proper environment
exec "$@" 