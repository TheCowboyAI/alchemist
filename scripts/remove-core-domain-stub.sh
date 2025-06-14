#!/bin/bash
# Remove cim-core-domain stub and update all dependencies

echo "Removing cim-core-domain stub and updating dependencies..."

# Step 1: Remove cim-core-domain from workspace members
echo "Removing cim-core-domain from workspace..."
sed -i '/"cim-core-domain",/d' Cargo.toml

# Step 2: Update all Cargo.toml files to use cim-domain instead of cim-core-domain
echo "Updating dependencies in all modules..."

# Find all Cargo.toml files and update them
find . -name "Cargo.toml" -type f ! -path "./target/*" ! -path "./bevy-patched/*" -exec sed -i \
    -e 's/cim-core-domain = { path = "..\/cim-core-domain" }/cim-domain = { path = "..\/cim-domain" }/g' \
    -e 's/cim-core-domain = { path = "\.\.\/cim-core-domain" }/cim-domain = { path = "..\/cim-domain" }/g' \
    {} +

# Step 3: Remove duplicate dependencies
echo "Removing duplicate dependencies..."
MODULES=(
    "cim-domain-agent"
    "cim-domain-document"
    "cim-domain-policy"
    "cim-domain-organization"
    "cim-domain-person"
    "cim-domain-workflow"
    "cim-domain-location"
    "cim-domain-graph"
    "cim-identity-context"
    "cim-conceptual-core"
    "cim-infrastructure"
)

for module in "${MODULES[@]}"; do
    if [ -f "$module/Cargo.toml" ]; then
        echo "Cleaning $module/Cargo.toml..."
        # Remove consecutive duplicate lines
        awk '!seen[$0]++ || NF==0' "$module/Cargo.toml" > "$module/Cargo.toml.tmp"
        mv "$module/Cargo.toml.tmp" "$module/Cargo.toml"
    fi
done

# Step 4: Update imports in source files
echo "Updating imports in source files..."

# Update imports in all Rust files
find . -name "*.rs" -type f ! -path "./target/*" ! -path "./bevy-patched/*" -exec sed -i \
    -e 's/use cim_core_domain::/use cim_domain::/g' \
    -e 's/cim_core_domain::/cim_domain::/g' \
    {} +

# Step 5: Remove cim-core-domain dependency from cim-domain itself
echo "Removing cim-core-domain dependency from cim-domain..."
sed -i '/cim-core-domain = { path = "..\/cim-core-domain" }/d' cim-domain/Cargo.toml

# Step 6: Move any unique types from cim-core-domain to cim-domain if needed
echo "Checking for types that need to be moved..."

# Check if NodeId, EdgeId, etc. are already in cim-domain
if ! grep -q "pub use identifiers::{NodeId, EdgeId" cim-domain/src/lib.rs; then
    echo "Note: You may need to ensure NodeId, EdgeId, StateId, TransitionId are properly exported from cim-domain"
fi

echo "Migration complete!"
echo ""
echo "Next steps:"
echo "1. Verify that all types from cim-core-domain are available in cim-domain"
echo "2. Remove the cim-core-domain directory: rm -rf cim-core-domain"
echo "3. Run 'cargo check' to ensure everything compiles"
