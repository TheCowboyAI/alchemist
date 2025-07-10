#!/usr/bin/env bash
# Run ia with environment variables loaded

# Load .env file if it exists
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Run ia with all arguments passed through
./target/x86_64-unknown-linux-gnu/debug/ia "$@"