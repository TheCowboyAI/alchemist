#!/usr/bin/env bash
set -euo pipefail

# Script to extract domain modules from cim-domain into separate submodules

# Color codes for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d ".git" ]; then
    print_error "This script must be run from the alchemist repository root"
    exit 1
fi

# List of domains to extract
DOMAINS=(
    "person:Person/People domain"
    "organization:Organization domain"
    "agent:Agent domain"
    "policy:Policy domain"
    "document:Document domain"
    "workflow:Workflow domain"
)

# Function to create a new domain submodule
create_domain_submodule() {
    local domain_name=$1
    local description=$2
    local repo_name="cim-domain-${domain_name}"

    print_status "Creating ${repo_name}..."

    # Create temporary directory for the new module
    local temp_dir=$(mktemp -d)
    cd "$temp_dir"

    # Initialize git repository
    git init

    # Create basic structure
    mkdir -p src

    # Create Cargo.toml
    cat > Cargo.toml << EOF
[package]
name = "${repo_name}"
version = "0.1.0"
edition = "2021"
authors = ["The Cowboy AI"]
description = "${description}"
license = "MIT OR Apache-2.0"
repository = "https://github.com/TheCowboyAI/${repo_name}"

[dependencies]
# Core dependencies
cim-domain = { path = "../cim-domain" }
cim-core-domain = { path = "../cim-core-domain" }
cim-infrastructure = { path = "../cim-infrastructure" }

# Async runtime
tokio = { version = "1.42", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Logging
tracing = "0.1"

# UUID generation
uuid = { version = "1.11", features = ["v4", "serde"] }

# Time handling
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tokio-test = "0.4"
pretty_assertions = "1.4"
EOF

    # Create lib.rs
    cat > src/lib.rs << EOF
//! ${description}
//!
//! This crate provides the ${domain_name} domain implementation for CIM.

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;

// Re-export main types
pub use aggregate::*;
pub use commands::*;
pub use events::*;
EOF

    # Create README.md
    cat > README.md << EOF
# ${repo_name}

${description}

This is a bounded context within the CIM (Composable Information Machine) system.

## Overview

This crate implements the ${domain_name} domain, providing:
- Domain aggregates and entities
- Command and event definitions
- Command and query handlers
- Projections and read models

## Usage

\`\`\`rust
use ${repo_name//-/_}::{${domain_name^}Aggregate, ${domain_name^}Command};

// Example usage
let aggregate = ${domain_name^}Aggregate::new();
\`\`\`

## License

MIT OR Apache-2.0
EOF

    # Create .gitignore
    cat > .gitignore << EOF
/target
Cargo.lock
*.swp
.DS_Store
EOF

    # Initial commit
    git add .
    git commit -m "Initial commit: ${repo_name} domain module"

    cd - > /dev/null

    print_status "Created ${repo_name} in ${temp_dir}"
    echo "Next steps:"
    echo "1. cd ${temp_dir}"
    echo "2. gh repo create TheCowboyAI/${repo_name} --public"
    echo "3. git remote add origin https://github.com/TheCowboyAI/${repo_name}.git"
    echo "4. git push -u origin main"
    echo "5. cd back to alchemist root"
    echo "6. git submodule add https://github.com/TheCowboyAI/${repo_name}.git ${repo_name}"
    echo ""
}

# Function to extract domain code
extract_domain_code() {
    local domain_name=$1
    local repo_path=$2

    print_status "Extracting ${domain_name} domain code..."

    # This is where you would:
    # 1. Copy the domain module file
    # 2. Extract related commands
    # 3. Extract related events
    # 4. Extract related handlers
    # 5. Update imports

    print_warning "Manual extraction required for ${domain_name} domain code"
}

# Main menu
echo "Domain Module Extraction Tool"
echo "============================="
echo ""
echo "This tool helps extract domain modules from cim-domain into separate submodules."
echo ""
echo "Available domains to extract:"
for i in "${!DOMAINS[@]}"; do
    IFS=':' read -r name desc <<< "${DOMAINS[$i]}"
    echo "$((i+1)). ${name} - ${desc}"
done
echo ""
echo "0. Exit"
echo ""

read -p "Select domain to extract (0-${#DOMAINS[@]}): " choice

if [ "$choice" -eq 0 ]; then
    print_status "Exiting..."
    exit 0
fi

if [ "$choice" -lt 1 ] || [ "$choice" -gt "${#DOMAINS[@]}" ]; then
    print_error "Invalid choice"
    exit 1
fi

# Get selected domain
IFS=':' read -r domain_name domain_desc <<< "${DOMAINS[$((choice-1))]}"

print_status "Selected: ${domain_name} - ${domain_desc}"
read -p "Continue? (y/n): " confirm

if [ "$confirm" != "y" ]; then
    print_status "Cancelled"
    exit 0
fi

# Create the domain submodule
create_domain_submodule "$domain_name" "$domain_desc"
