#!/usr/bin/env bash

# Script to rename all uppercase filenames to lowercase
# This will preserve the directory structure and only lowercase the filename

set -e

echo "Starting to rename uppercase files to lowercase..."

# Function to convert filename to lowercase
rename_file() {
    local file="$1"
    local dir=$(dirname "$file")
    local basename=$(basename "$file")
    local lowercase=$(echo "$basename" | tr '[:upper:]' '[:lower:]')
    
    # Replace underscores with hyphens in the lowercase version
    lowercase=$(echo "$lowercase" | tr '_' '-')
    
    if [ "$basename" != "$lowercase" ]; then
        local newpath="$dir/$lowercase"
        if [ -e "$newpath" ]; then
            echo "Warning: $newpath already exists, skipping $file"
        else
            echo "Renaming: $file -> $newpath"
            git mv "$file" "$newpath" 2>/dev/null || mv "$file" "$newpath"
        fi
    fi
}

# Find and rename all files with uppercase letters (excluding build directories)
find . -type f -name "*[A-Z]*" | grep -v -E "(target/|\.git/|node_modules/|\.cargo/)" | while read -r file; do
    # Only process if the basename has uppercase letters
    basename=$(basename "$file")
    if echo "$basename" | grep -q '[A-Z]'; then
        rename_file "$file"
    fi
done

echo "Renaming complete!"
echo "Don't forget to commit these changes."