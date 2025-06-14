#!/bin/bash

# Test all submodules individually
modules=(
    "cim-component"
    "cim-infrastructure"
    "cim-contextgraph"
    "cim-conceptgraph"
    "cim-domain-person"
    "cim-domain-agent"
    "cim-domain-location"
    "cim-subject"
    "cim-domain-workflow"
    "cim-workflow-graph"
    "cim-domain-policy"
    "cim-domain-document"
    "cim-domain"
    "cim-domain-organization"
    "cim-domain-graph"
    "cim-ipld"
    "cim-ipld-graph"
    "cim-compose"
    "cim-domain-bevy"
    "cim-domain-conceptualspaces"
    "cim-domain-identity"
)

echo "Testing all modules individually (library tests only)..."
echo "=================================================="

passed=0
failed=0

for module in "${modules[@]}"; do
    echo -n "Testing $module... "
    if cd "$module" && cargo test --lib &>/dev/null; then
        echo "✅ PASSED"
        ((passed++))
    else
        echo "❌ FAILED"
        ((failed++))
    fi
    cd ..
done

echo "=================================================="
echo "Summary: $passed passed, $failed failed out of ${#modules[@]} modules" 