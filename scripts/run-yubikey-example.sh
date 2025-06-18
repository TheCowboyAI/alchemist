#!/usr/bin/env bash
# Simple wrapper to run YubiKey example with proper library paths

set -e

# Find pcsclite library
PCSCLITE_LIB=$(nix-build '<nixpkgs>' -A pcsclite.lib --no-out-link)/lib

# Build the example
echo "Building YubiKey example..."
cargo build --manifest-path cim-keys/Cargo.toml --example yubikey_demo

# Run with proper library path
echo "Running YubiKey example..."
LD_LIBRARY_PATH="$PCSCLITE_LIB:$LD_LIBRARY_PATH" \
    ./target/x86_64-unknown-linux-gnu/debug/examples/yubikey_demo "$@" 