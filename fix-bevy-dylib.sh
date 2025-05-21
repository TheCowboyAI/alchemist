#!/usr/bin/env bash
# Script to fix Bevy dynamic linking issues on NixOS

set -e

echo "==== Bevy Dynamic Linking Fix ====="
echo "Finding the result symlink..."
RESULT_PATH=$(readlink -f ./result)
echo "Result path: $RESULT_PATH"

echo "Looking for bevy_dylib shared libraries..."
BEVY_DYLIB=$(find $RESULT_PATH -name "libbevy_dylib*.so" | head -1)
if [ -z "$BEVY_DYLIB" ]; then
  echo "Error: Could not find libbevy_dylib*.so in the result path!"
  exit 1
fi

echo "Found bevy_dylib at: $BEVY_DYLIB"

echo "Looking for fixedbitset shared libraries..."
FIXED_BITSET=$(find /nix/store -name "*fixedbitset*.so*" -type f | grep -v "\.a$" | sort | head -1)
if [ -z "$FIXED_BITSET" ]; then
  echo "Error: Could not find fixedbitset shared library in /nix/store!"
  exit 1
fi

echo "Found fixedbitset at: $FIXED_BITSET"

FIXED_BITSET_NAME=$(basename "$FIXED_BITSET")
DEST_DIR="$RESULT_PATH/lib"

echo "Copying $FIXED_BITSET to $DEST_DIR/$FIXED_BITSET_NAME"
cp "$FIXED_BITSET" "$DEST_DIR/$FIXED_BITSET_NAME"
chmod +x "$DEST_DIR/$FIXED_BITSET_NAME"

# Create symlinks
echo "Creating symlinks for fixedbitset..."
ln -sf "$FIXED_BITSET_NAME" "$DEST_DIR/libfixedbitset.so" || true

# Run the program with additional LD_LIBRARY_PATH
echo "Running alchemist with LD_LIBRARY_PATH set to include fixedbitset..."
echo "You can now run your program with:"
echo "LD_LIBRARY_PATH=$DEST_DIR:$LD_LIBRARY_PATH $RESULT_PATH/bin/alchemist"

echo "===== Fix completed =====" 