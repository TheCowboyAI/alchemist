#!/usr/bin/env bash
# Script to run programs with YubiKey/PC/SC support

set -e

echo "YubiKey Support Runner"
echo "====================="

# Check if pcscd is available
if ! command -v pcscd &> /dev/null; then
    echo "Error: pcscd not found. Please install pcsclite."
    exit 1
fi

# Check if pcscd is running
if systemctl is-active --quiet pcscd 2>/dev/null; then
    echo "✓ PC/SC daemon is running (system service)"
elif pgrep -x pcscd > /dev/null; then
    echo "✓ PC/SC daemon is running"
else
    echo "⚠ PC/SC daemon is not running"
    echo ""
    echo "To start it:"
    echo "  - On NixOS with systemd: sudo systemctl start pcscd"
    echo "  - Or manually: sudo pcscd --foreground --debug"
    echo ""
    echo "Attempting to start pcscd (may require sudo)..."
    
    # Try to start pcscd
    if command -v systemctl &> /dev/null; then
        sudo systemctl start pcscd || {
            echo "Failed to start pcscd via systemctl"
            echo "Trying direct start..."
            sudo pcscd --foreground --debug &
            PCSCD_PID=$!
            sleep 2
        }
    else
        sudo pcscd --foreground --debug &
        PCSCD_PID=$!
        sleep 2
    fi
fi

# Check for YubiKeys
echo ""
echo "Checking for YubiKeys..."
if command -v ykman &> /dev/null; then
    ykman list || echo "No YubiKeys detected"
else
    echo "ykman not found, cannot list YubiKeys"
fi

echo ""
echo "Running: $@"
echo ""

# Run the provided command
exec "$@" 