#!/usr/bin/env bash

# Script to fix format string interpolations
# Converts format!("text {}", var) to format!("text {var}")

echo "Fixing format string interpolations..."

# Find all Rust files
find . -name "*.rs" -type f | grep -v "target/" | grep -v "bevy-patched/" | while read -r file; do
    # Skip if file doesn't contain format! macros
    if ! grep -q 'format!' "$file"; then
        continue
    fi
    
    echo "Processing: $file"
    
    # Create a temporary file
    temp_file=$(mktemp)
    
    # Use sed to fix simple format string patterns
    # Pattern 1: format!("text {}", var)
    sed -E 's/format!\("([^"]*) \{\}", ([a-zA-Z_][a-zA-Z0-9_]*)\)/format!("\1 {\2}")/g' "$file" > "$temp_file"
    
    # Pattern 2: format!("{}", var)
    sed -i -E 's/format!\("\{\}", ([a-zA-Z_][a-zA-Z0-9_]*)\)/format!("{\1}")/g' "$temp_file"
    
    # Pattern 3: format!("text {} text", var)
    sed -i -E 's/format!\("([^"]*) \{\} ([^"]*)", ([a-zA-Z_][a-zA-Z0-9_]*)\)/format!("\1 {\3} \2")/g' "$temp_file"
    
    # Pattern 4: Multiple variables - format!("{} {}", a, b)
    sed -i -E 's/format!\("\{\} \{\}", ([a-zA-Z_][a-zA-Z0-9_]*), ([a-zA-Z_][a-zA-Z0-9_]*)\)/format!("{\1} {\2}")/g' "$temp_file"
    
    # Pattern 5: With field access - format!("{}", obj.field)
    sed -i -E 's/format!\("\{\}", ([a-zA-Z_][a-zA-Z0-9_]*\.[a-zA-Z_][a-zA-Z0-9_]*)\)/format!("{\1}")/g' "$temp_file"
    
    # Pattern 6: println! and eprintln! macros
    sed -i -E 's/println!\("([^"]*) \{\}", ([a-zA-Z_][a-zA-Z0-9_]*)\)/println!("\1 {\2}")/g' "$temp_file"
    sed -i -E 's/eprintln!\("([^"]*) \{\}", ([a-zA-Z_][a-zA-Z0-9_]*)\)/eprintln!("\1 {\2}")/g' "$temp_file"
    
    # Only update the file if changes were made
    if ! diff -q "$file" "$temp_file" > /dev/null; then
        mv "$temp_file" "$file"
        echo "  Fixed: $file"
    else
        rm "$temp_file"
    fi
done

echo "Format string fixes complete!"
echo "Running cargo fmt to ensure proper formatting..."
cargo fmt

echo "Done! Run 'cargo clippy' to verify the fixes." 