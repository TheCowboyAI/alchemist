#!/bin/bash

# Script to commit and push all submodule changes

echo "Committing and pushing all submodule changes..."

# Array of submodules that have changes (excluding the ones we already handled)
submodules=(
    "cim-agent-alchemist"
    "cim-bridge"
    "cim-component"
    "cim-compose"
    "cim-conceptgraph"
    "cim-contextgraph"
    "cim-domain-agent"
    "cim-domain-bevy"
    "cim-domain-conceptualspaces"
    "cim-domain-dialog"
    "cim-domain-document"
    "cim-domain-git"
    "cim-domain-graph"
    "cim-domain-identity"
    "cim-domain-location"
    "cim-domain-nix"
    "cim-domain-organization"
    "cim-domain-policy"
    "cim-domain-workflow"
    "cim-infrastructure"
    "cim-ipld"
    "cim-ipld-graph"
    "cim-keys"
    "cim-security"
    "cim-subject"
    "cim-workflow-graph"
)

for submodule in "${submodules[@]}"; do
    echo "Processing $submodule..."
    cd "$submodule" || continue
    
    # Check if there are changes
    if [[ -n $(git status --porcelain) ]]; then
        echo "  Committing changes in $submodule..."
        git add -A
        git commit -m "Fix format string warnings and update examples

- Fixed all format string warnings in examples and tests
- Updated code to use proper format! macro syntax
- Part of system-wide cleanup for production readiness"
        
        # Check if we're on a branch
        branch=$(git rev-parse --abbrev-ref HEAD)
        if [[ "$branch" == "HEAD" ]]; then
            echo "  HEAD is detached, checking out main..."
            git checkout main
            git merge HEAD@{1}
        fi
        
        echo "  Pushing to origin..."
        git push origin main
    else
        echo "  No changes in $submodule"
    fi
    
    cd ..
done

echo "All submodules processed!" 