#!/usr/bin/env bash
set -euo pipefail

# Batch script to extract all domains from cim-domain

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "cim-domain" ]; then
    print_error "This script must be run from the alchemist repository root"
    exit 1
fi

# Define domains to extract
declare -A DOMAINS=(
    ["person"]="Person/People domain"
    ["organization"]="Organization domain"
    ["agent"]="Agent domain"
    ["policy"]="Policy domain"
    ["document"]="Document domain"
    ["workflow"]="Workflow domain"
)

# Function to extract a domain
extract_domain() {
    local domain_name=$1
    local description=$2
    local repo_name="cim-domain-${domain_name}"

    print_status "Extracting ${domain_name} domain..."

    # Create temporary directory
    local temp_dir=$(mktemp -d)

    # Clone the repository
    cd "$temp_dir"
    git clone "https://github.com/TheCowboyAI/${repo_name}.git" || {
        print_error "Failed to clone ${repo_name}"
        return 1
    }

    cd "${repo_name}"

    # Create directory structure
    mkdir -p src/{aggregate,commands,events,handlers,projections,queries,value_objects}

    # Go back to original directory
    cd "$OLDPWD"

    # Copy domain file if it exists
    if [ -f "cim-domain/src/${domain_name}.rs" ]; then
        cp "cim-domain/src/${domain_name}.rs" "$temp_dir/${repo_name}/src/aggregate/mod.rs"
    elif [ -d "cim-domain/src/${domain_name}" ]; then
        cp -r "cim-domain/src/${domain_name}"/* "$temp_dir/${repo_name}/src/"
    fi

    # Create basic lib.rs
    cat > "$temp_dir/${repo_name}/src/lib.rs" << EOF
//! ${description} for the Composable Information Machine
//!
//! This crate provides the ${domain_name} domain implementation.

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod value_objects;

// Re-export main types will be added after extraction
EOF

    # Extract related code from various files
    print_status "Extracting ${domain_name} related code from domain files..."

    # This is where manual extraction would be needed for:
    # - Commands from commands.rs
    # - Events from domain_events.rs
    # - Handlers from command_handlers.rs and query_handlers.rs

    # For now, create placeholder files
    echo "//! ${domain_name^} commands" > "$temp_dir/${repo_name}/src/commands/mod.rs"
    echo "//! ${domain_name^} events" > "$temp_dir/${repo_name}/src/events/mod.rs"
    echo "//! ${domain_name^} handlers" > "$temp_dir/${repo_name}/src/handlers/mod.rs"
    echo "//! ${domain_name^} projections" > "$temp_dir/${repo_name}/src/projections/mod.rs"
    echo "//! ${domain_name^} queries" > "$temp_dir/${repo_name}/src/queries/mod.rs"
    echo "//! ${domain_name^} value objects" > "$temp_dir/${repo_name}/src/value_objects/mod.rs"

    # Commit and push
    cd "$temp_dir/${repo_name}"
    git add .
    git commit -m "feat: Initial extraction of ${domain_name} domain from cim-domain" \
        -m "- Add basic structure and aggregate" \
        -m "- Placeholder modules for commands, events, handlers" \
        -m "- Manual extraction and refinement needed"

    git push origin main || {
        print_warning "Failed to push ${repo_name}, may need manual intervention"
    }

    print_status "${domain_name^} domain extracted to: $temp_dir/${repo_name}"

    # Return the temp directory for reference
    echo "$temp_dir/${repo_name}"
}

# Main execution
print_status "Starting batch domain extraction..."
echo ""

# Track extracted domains
declare -a EXTRACTED_PATHS=()

# Extract each domain
for domain in "${!DOMAINS[@]}"; do
    print_status "Processing ${domain} domain..."
    path=$(extract_domain "$domain" "${DOMAINS[$domain]}")
    if [ $? -eq 0 ]; then
        EXTRACTED_PATHS+=("$path")
        print_status "${domain^} domain extracted successfully"
    else
        print_error "Failed to extract ${domain} domain"
    fi
    echo ""
done

# Summary
print_status "Extraction complete!"
echo ""
echo "Extracted domains:"
for path in "${EXTRACTED_PATHS[@]}"; do
    echo "  - $path"
done

echo ""
echo "Next steps:"
echo "1. Review and refine the extracted code in each repository"
echo "2. Manually extract related commands, events, and handlers"
echo "3. Update imports and dependencies"
echo "4. Add tests for each domain"
echo "5. Add each as a submodule to the main project:"
echo ""
for domain in "${!DOMAINS[@]}"; do
    echo "   git submodule add https://github.com/TheCowboyAI/cim-domain-${domain}.git cim-domain-${domain}"
done

echo ""
echo "6. Update cim-domain to remove extracted code"
echo "7. Update main Cargo.toml to reference new submodules"
