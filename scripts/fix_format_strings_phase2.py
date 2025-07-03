#!/usr/bin/env python3
"""
Fix format string field access errors in Rust code.
Converts {self.field} to {} with self.field as an argument.
"""

import re
import sys
from pathlib import Path

def fix_format_string_field_access(content):
    """Fix format string field access patterns."""
    # Pattern to match format strings with field access
    # This will match format!("...{expr.field}...") or similar
    pattern = r'format!\s*\(\s*"([^"]*?)\{([^}]+\.[^}]+)\}([^"]*?)"\s*\)'
    
    def replacer(match):
        prefix = match.group(1)
        field_expr = match.group(2)
        suffix = match.group(3)
        
        # Count existing {} placeholders before this one
        placeholder_count = prefix.count('{}')
        
        # Replace the field access with a placeholder
        new_format = f'format!("{prefix}{{}}{suffix}", {field_expr})'
        
        return new_format
    
    # Apply the fix
    fixed = re.sub(pattern, replacer, content)
    
    # Also handle cases with multiple arguments
    # Pattern: format!("...{expr.field}...", other_args)
    pattern2 = r'format!\s*\(\s*"([^"]*?)\{([^}]+\.[^}]+)\}([^"]*?)"\s*,\s*([^)]+)\)'
    
    def replacer2(match):
        prefix = match.group(1)
        field_expr = match.group(2)
        suffix = match.group(3)
        other_args = match.group(4)
        
        # Replace the field access with a placeholder and add to args
        new_format = f'format!("{prefix}{{}}{suffix}", {field_expr}, {other_args})'
        
        return new_format
    
    fixed = re.sub(pattern2, replacer2, fixed)
    
    # Handle write! macro as well
    pattern3 = r'write!\s*\(\s*([^,]+),\s*"([^"]*?)\{([^}]+\.[^}]+)\}([^"]*?)"\s*\)'
    
    def replacer3(match):
        writer = match.group(1)
        prefix = match.group(2)
        field_expr = match.group(3)
        suffix = match.group(4)
        
        new_format = f'write!({writer}, "{prefix}{{}}{suffix}", {field_expr})'
        
        return new_format
    
    fixed = re.sub(pattern3, replacer3, fixed)
    
    return fixed

def process_file(file_path):
    """Process a single file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        fixed_content = fix_format_string_field_access(content)
        
        if content != fixed_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(fixed_content)
            print(f"Fixed: {file_path}")
            return True
        return False
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    """Main function."""
    if len(sys.argv) > 1:
        # Process specific files
        files = sys.argv[1:]
    else:
        # Process all Rust files in cim-domain
        files = list(Path('cim-domain/src').rglob('*.rs'))
    
    fixed_count = 0
    for file_path in files:
        if process_file(file_path):
            fixed_count += 1
    
    print(f"\nFixed {fixed_count} files")

if __name__ == '__main__':
    main() 