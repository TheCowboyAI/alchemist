#!/bin/bash

# Script to commit and push all submodules with changes

echo "Committing and pushing all submodules with changes..."

# Function to commit and push a submodule
commit_and_push_submodule() {
    local submodule=$1
    local message=$2
    
    echo "Processing $submodule..."
    cd "$submodule" || return
    
    # Check if there are changes
    if [[ -n $(git status --porcelain) ]]; then
        echo "Found changes in $submodule"
        
        # Add all changes
        git add -A
        
        # Commit with message
        git commit -m "$message"
        
        # Push to origin
        git push origin HEAD
        
        echo "Pushed $submodule"
    else
        echo "No changes in $submodule"
    fi
    
    cd ..
}

# Commit each submodule with changes
commit_and_push_submodule "cim-agent-alchemist" "feat: update agent implementation for enhanced CIM integration"
commit_and_push_submodule "cim-compose" "test: add test infrastructure"
commit_and_push_submodule "cim-contextgraph" "feat: add infrastructure tests and update dependencies"
commit_and_push_submodule "cim-domain" "feat: update CQRS implementation and event handling"
commit_and_push_submodule "cim-domain-agent" "feat: update aggregate and handlers for enhanced functionality"
commit_and_push_submodule "cim-domain-git" "feat: add CQRS adapter for Git domain integration"
commit_and_push_submodule "cim-domain-graph" "feat: update handlers and queries for graph domain"
commit_and_push_submodule "cim-domain-identity" "feat: update CQRS adapter for identity domain"
commit_and_push_submodule "cim-domain-location" "feat: update location command handler"
commit_and_push_submodule "cim-domain-person" "feat: update CQRS adapter and query handlers"
commit_and_push_submodule "cim-domain-policy" "feat: update policy command handler"
commit_and_push_submodule "cim-ipld" "test: add infrastructure test modules"
commit_and_push_submodule "cim-keys" "test: add infrastructure test modules"
commit_and_push_submodule "cim-subject" "feat: add correlation and message algebra implementation"
commit_and_push_submodule "cim-workflow-graph" "test: add test infrastructure"

echo "All submodules processed!"

# Now update the main repository
echo "Updating main repository..."
git add -A
git commit -m "feat: update submodules with enhanced implementations

- Advanced Nix parser with AST manipulation (cim-domain-nix)
- Enhanced CQRS implementations across domains
- Correlation and message algebra (cim-subject)
- Infrastructure test improvements
- Various domain handler updates"

echo "Done! Don't forget to push the main repository." 