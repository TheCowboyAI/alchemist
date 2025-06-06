#!/usr/bin/env bash

# Get the Rust sysroot
RUST_SYSROOT=$(rustc --print sysroot)

# Set up library paths
export LD_LIBRARY_PATH="target/x86_64-unknown-linux-gnu/debug/deps:target/x86_64-unknown-linux-gnu/debug:$RUST_SYSROOT/lib/rustlib/x86_64-unknown-linux-gnu/lib:$LD_LIBRARY_PATH"

# Run the application
exec ./target/x86_64-unknown-linux-gnu/debug/ia "$@"
