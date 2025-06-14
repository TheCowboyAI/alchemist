#!/bin/bash
# Fix dependencies in domain submodules

echo "Fixing dependencies in domain submodules..."

# List of domain submodules
DOMAINS=(
    "cim-domain-agent"
    "cim-domain-document"
    "cim-domain-policy"
    "cim-domain-organization"
    "cim-domain-person"
    "cim-domain-workflow"
    "cim-domain-location"
)

# Fix dependencies in each domain
for domain in "${DOMAINS[@]}"; do
    echo "Fixing dependencies in $domain/Cargo.toml..."

    # Replace cim-core-domain with cim-domain
    sed -i 's/cim-core-domain = { path = "..\/cim-core-domain" }/cim-domain = { path = "..\/cim-domain" }/' "$domain/Cargo.toml"

    # Add cim-subject if not present (for domains that use Subject)
    if ! grep -q "cim-subject" "$domain/Cargo.toml"; then
        # Add cim-subject after cim-domain
        sed -i '/cim-domain = { path = "..\/cim-domain" }/a cim-subject = { path = "../cim-subject" }' "$domain/Cargo.toml"
    fi
done

# Special handling for cim-domain-graph which already has correct dependencies
echo "Skipping cim-domain-graph as it already has correct dependencies"

echo "Dependency fixes complete!"
