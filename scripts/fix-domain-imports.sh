#!/bin/bash
# Fix import issues in domain submodules

echo "Fixing imports in domain submodules..."

# List of domain submodules
DOMAINS=(
    "cim-domain-agent"
    "cim-domain-document"
    "cim-domain-policy"
    "cim-domain-organization"
    "cim-domain-person"
    "cim-domain-workflow"
    "cim-domain-location"
    "cim-domain-graph"
)

# Fix imports in each domain
for domain in "${DOMAINS[@]}"; do
    echo "Fixing imports in $domain..."

    # Replace cim_core_domain imports with cim_domain
    find "$domain/src" -name "*.rs" -type f -exec sed -i \
        -e 's/use cim_core_domain::{/use cim_domain::{/g' \
        -e 's/use cim_core_domain::/use cim_domain::/g' \
        -e 's/cim_core_domain::/cim_domain::/g' \
        {} +

    # Fix specific import patterns
    find "$domain/src" -name "*.rs" -type f -exec sed -i \
        -e 's/use cim_domain::component::/use cim_domain::/g' \
        -e 's/use cim_domain::entity::/use cim_domain::/g' \
        -e 's/use cim_domain::errors::/use cim_domain::/g' \
        -e 's/use cim_domain::command::/use cim_domain::/g' \
        -e 's/use cim_domain::event::/use cim_domain::/g' \
        -e 's/use cim_domain::query::/use cim_domain::/g' \
        -e 's/use cim_domain::repository::/use cim_domain::/g' \
        -e 's/use cim_domain::identifiers::/use cim_domain::/g' \
        -e 's/use cim_domain::subject::/use cim_subject::/g' \
        {} +

    # Fix crate:: imports in cim-domain-organization
    if [ "$domain" = "cim-domain-organization" ]; then
        find "$domain/src" -name "*.rs" -type f -exec sed -i \
            -e 's/use crate::entity::/use cim_domain::/g' \
            -e 's/use crate::errors::/use cim_domain::/g' \
            -e 's/use crate::component::/use cim_domain::/g' \
            {} +
    fi
done

echo "Import fixes complete!"
