#!/usr/bin/env python3

import re
import os
import sys
from pathlib import Path

def fix_format_strings(content):
    """Fix format string interpolations in Rust code."""
    
    # Pattern 1: format!("{}.{}.{}.{}", a, b, c, d) -> format!("{a}.{b}.{c}.{d}")
    pattern1 = re.compile(
        r'format!\("([^"]*?)\{\}([^"]*?)"\s*,\s*([^,)]+?)(?:\s*,\s*([^,)]+?))*\)'
    )
    
    # Pattern 2: More general - capture all {} placeholders and corresponding variables
    def replace_format(match):
        format_str = match.group(1)
        remaining = match.group(2)
        
        # Extract all variables from the remaining part
        vars_part = remaining.strip()
        if vars_part.startswith(','):
            vars_part = vars_part[1:].strip()
        
        # Split by commas, handling nested parentheses
        variables = []
        current_var = ""
        paren_depth = 0
        
        for char in vars_part:
            if char == '(' or char == '<':
                paren_depth += 1
            elif char == ')' or char == '>':
                paren_depth -= 1
                if paren_depth < 0:
                    break
            elif char == ',' and paren_depth == 0:
                if current_var.strip():
                    variables.append(current_var.strip())
                current_var = ""
                continue
            current_var += char
        
        if current_var.strip() and paren_depth >= 0:
            variables.append(current_var.strip())
        
        # Count {} in format string
        placeholder_count = format_str.count('{}')
        
        if placeholder_count == 0 or len(variables) == 0:
            return match.group(0)
        
        # Replace {} with {var} for each variable
        result = format_str
        for i, var in enumerate(variables[:placeholder_count]):
            # Clean up the variable name
            var = var.strip()
            if var.endswith(')') and '(' not in var:
                var = var[:-1]
            
            # Replace the i-th {} with {var}
            result = result.replace('{}', f'{{{var}}}', 1)
        
        # Reconstruct the format! call
        remaining_vars = variables[placeholder_count:]
        if remaining_vars:
            return f'format!("{result}", {", ".join(remaining_vars)})'
        else:
            return f'format!("{result}")'
    
    # Apply the general pattern
    content = re.sub(
        r'format!\s*\(\s*"([^"]*?)"\s*((?:,\s*[^,)]+)+)\s*\)',
        replace_format,
        content,
        flags=re.MULTILINE | re.DOTALL
    )
    
    # Also fix println! and eprintln!
    content = re.sub(
        r'println!\s*\(\s*"([^"]*?)"\s*((?:,\s*[^,)]+)+)\s*\)',
        lambda m: replace_format(m).replace('format!', 'println!'),
        content,
        flags=re.MULTILINE | re.DOTALL
    )
    
    content = re.sub(
        r'eprintln!\s*\(\s*"([^"]*?)"\s*((?:,\s*[^,)]+)+)\s*\)',
        lambda m: replace_format(m).replace('format!', 'eprintln!'),
        content,
        flags=re.MULTILINE | re.DOTALL
    )
    
    # Fix write! and writeln! macros too
    content = re.sub(
        r'write!\s*\(\s*([^,]+)\s*,\s*"([^"]*?)"\s*((?:,\s*[^,)]+)+)\s*\)',
        lambda m: f'write!({m.group(1)}, "{fix_placeholders(m.group(2), extract_vars(m.group(3)))}")',
        content,
        flags=re.MULTILINE | re.DOTALL
    )
    
    return content

def fix_placeholders(format_str, variables):
    """Replace {} with {var} for each variable."""
    result = format_str
    for var in variables:
        result = result.replace('{}', f'{{{var}}}', 1)
    return result

def extract_vars(vars_str):
    """Extract variables from a comma-separated string."""
    vars_str = vars_str.strip()
    if vars_str.startswith(','):
        vars_str = vars_str[1:].strip()
    
    variables = []
    current_var = ""
    paren_depth = 0
    
    for char in vars_str:
        if char == '(' or char == '<':
            paren_depth += 1
        elif char == ')' or char == '>':
            paren_depth -= 1
            if paren_depth < 0:
                break
        elif char == ',' and paren_depth == 0:
            if current_var.strip():
                variables.append(current_var.strip())
            current_var = ""
            continue
        current_var += char
    
    if current_var.strip() and paren_depth >= 0:
        variables.append(current_var.strip())
    
    return variables

def process_file(filepath):
    """Process a single Rust file."""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original = content
        fixed = fix_format_strings(content)
        
        if fixed != original:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(fixed)
            return True
        return False
    except Exception as e:
        print(f"Error processing {filepath}: {e}")
        return False

def main():
    """Main function to process all Rust files."""
    fixed_count = 0
    total_count = 0
    
    # Find all Rust files
    for root, dirs, files in os.walk('.'):
        # Skip target and bevy-patched directories
        if 'target' in root or 'bevy-patched' in root:
            continue
        
        for file in files:
            if file.endswith('.rs'):
                filepath = os.path.join(root, file)
                total_count += 1
                
                if process_file(filepath):
                    print(f"Fixed: {filepath}")
                    fixed_count += 1
    
    print(f"\nProcessed {total_count} files, fixed {fixed_count} files")
    print("Run 'cargo fmt' to ensure proper formatting")

if __name__ == "__main__":
    main() 