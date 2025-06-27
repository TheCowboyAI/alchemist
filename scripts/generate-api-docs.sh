#!/bin/bash

# Script to generate API documentation for all domains

echo "Generating API documentation for all domains..."

# Define domains that need API docs
DOMAINS=(
    "agent"
    "conceptualspaces"
    "dialog"
    "document"
    "git"
    "graph"
    "identity"
    "location"
    "nix"
    "organization"
    "person"
    "policy"
    "workflow"
)

# Function to generate API doc for a domain
generate_api_doc() {
    local domain=$1
    local domain_dir="cim-domain-${domain}"
    local api_file="${domain_dir}/doc/api.md"
    
    if [ -f "$api_file" ]; then
        echo "✓ API documentation already exists for ${domain}"
        return
    fi
    
    echo "Generating API documentation for ${domain}..."
    
    # Create doc directory if it doesn't exist
    mkdir -p "${domain_dir}/doc"
    
    # Copy template and customize
    cp doc/templates/domain-api.md "$api_file"
    
    # Replace placeholders with domain-specific values
    # Convert domain name to proper case
    local domain_proper=$(echo "$domain" | sed 's/\b\(.\)/\u\1/g')
    
    # Update the file with domain-specific information
    sed -i "s/{Domain Name}/${domain_proper}/g" "$api_file"
    sed -i "s/{domain name}/${domain}/g" "$api_file"
    sed -i "s/{Domain}/${domain_proper}/g" "$api_file"
    sed -i "s/{domain}/${domain}/g" "$api_file"
    sed -i "s/{Entity}/${domain_proper}/g" "$api_file"
    sed -i "s/{entity}/${domain}/g" "$api_file"
    sed -i "s/{name}/${domain}/g" "$api_file"
    
    echo "✓ Generated API documentation for ${domain}"
}

# Generate API docs for each domain
for domain in "${DOMAINS[@]}"; do
    generate_api_doc "$domain"
done

echo ""
echo "API documentation generation complete!"
echo ""
echo "Note: The generated documentation contains templates that should be customized with:"
echo "- Actual command and query definitions from each domain"
echo "- Specific validation rules and business logic"
echo "- Real examples from the implementation"
echo "- Domain-specific error types" 