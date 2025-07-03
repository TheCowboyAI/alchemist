#!/usr/bin/env python3
"""
Fix the final remaining format string field access errors in Rust code.
"""

import re
import sys
from pathlib import Path

def fix_format_string_field_access(content):
    """Fix format string field access patterns."""
    
    # More patterns to catch all cases
    patterns = [
        # Match format!("...{self.field}...")
        (r'format!\s*\(\s*"([^"]*?)\{self\.(\w+)\}([^"]*?)"\s*\)', 
         lambda m: f'format!("{m.group(1)}{{}}{m.group(3)}", self.{m.group(2)})'),
        
        # Match format!("...{var.field}...")
        (r'format!\s*\(\s*"([^"]*?)\{(\w+)\.(\w+)\}([^"]*?)"\s*\)',
         lambda m: f'format!("{m.group(1)}{{}}{m.group(4)}", {m.group(2)}.{m.group(3)})'),
         
        # Match format!("...{self.method()}...")
        (r'format!\s*\(\s*"([^"]*?)\{self\.(\w+)\(\)\}([^"]*?)"\s*\)', 
         lambda m: f'format!("{m.group(1)}{{}}{m.group(3)}", self.{m.group(2)}())'),
         
        # Match format!("...{var.method()}...")
        (r'format!\s*\(\s*"([^"]*?)\{(\w+)\.(\w+)\(\)\}([^"]*?)"\s*\)',
         lambda m: f'format!("{m.group(1)}{{}}{m.group(4)}", {m.group(2)}.{m.group(3)}())'),
         
        # Match println!("...{self.field}...")
        (r'println!\s*\(\s*"([^"]*?)\{self\.(\w+)\}([^"]*?)"\s*\)',
         lambda m: f'println!("{m.group(1)}{{}}{m.group(3)}", self.{m.group(2)})'),
         
        # Match println!("...{var.field}...")
        (r'println!\s*\(\s*"([^"]*?)\{(\w+)\.(\w+)\}([^"]*?)"\s*\)',
         lambda m: f'println!("{m.group(1)}{{}}{m.group(4)}", {m.group(2)}.{m.group(3)})'),
         
        # Match write!(f, "...{self.field}...")
        (r'write!\s*\(\s*(\w+)\s*,\s*"([^"]*?)\{self\.(\w+)\}([^"]*?)"\s*\)',
         lambda m: f'write!({m.group(1)}, "{m.group(2)}{{}}{m.group(4)}", self.{m.group(3)})'),
         
        # Match write!(f, "...{var.field}...")
        (r'write!\s*\(\s*(\w+)\s*,\s*"([^"]*?)\{(\w+)\.(\w+)\}([^"]*?)"\s*\)',
         lambda m: f'write!({m.group(1)}, "{m.group(2)}{{}}{m.group(5)}", {m.group(3)}.{m.group(4)})'),
    ]
    
    modified = content
    for pattern, replacement in patterns:
        modified = re.sub(pattern, replacement, modified)
    
    return modified

def process_file(file_path):
    """Process a single file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        modified = fix_format_string_field_access(content)
        
        if modified != content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(modified)
            print(f"Fixed: {file_path}")
            return True
        return False
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    """Main function."""
    # Files with format string errors based on the build output
    files_to_fix = [
        "cim-domain-document/src/value_objects/mod.rs",
        "cim-domain-workflow/src/value_objects/workflow_step.rs",
        "cim-domain-organization/src/cross_domain/mod.rs",
        "cim-domain-person/src/projections/person_summary_projection.rs",
    ]
    
    fixed_count = 0
    for file_path in files_to_fix:
        if Path(file_path).exists():
            if process_file(file_path):
                fixed_count += 1
        else:
            print(f"File not found: {file_path}")
    
    print(f"\nFixed {fixed_count} files")

if __name__ == "__main__":
    main() 