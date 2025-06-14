# Convert CIM Directories to Git Submodules Plan

## Overview
Convert all `/cim-*` directories into proper git submodules connected to remote repositories under the TheCowboyAI organization.

## Current Status

### Already Submodules:
- `cim-ipld` → https://github.com/TheCowboyAI/cim-ipld.git
- `cim-contextgraph` → git@github.com:TheCowboyAI/cim-contextgraph.git

### Need Conversion:
1. `cim-component` - Core component definitions
2. `cim-compose` - Composition utilities
3. `cim-conceptual-core` - Conceptual space core functionality
4. `cim-core-domain` - Core domain models
5. `cim-domain` - Domain implementation
6. `cim-identity-context` - Identity bounded context
7. `cim-infrastructure` - Infrastructure layer
8. `cim-subject` - Subject management
9. `cim-viz-bevy` - Bevy visualization components

## Process for Each Directory

### Step 1: Create GitHub Repository
```bash
# Using GitHub CLI (gh) or web interface
gh repo create TheCowboyAI/cim-<name> --public --description "<description>"
```

### Step 2: Prepare Directory Content
```bash
# Save current directory content
cd cim-<name>
git init
git add .
git commit -m "Initial commit: Extract from alchemist monorepo"
git branch -M main
git remote add origin https://github.com/TheCowboyAI/cim-<name>.git
git push -u origin main
cd ..
```

### Step 3: Remove from Main Repository
```bash
# Remove directory but keep in git history
git rm -r cim-<name>
git commit -m "Remove cim-<name> to convert to submodule"
```

### Step 4: Add as Submodule
```bash
# Add back as submodule
git submodule add https://github.com/TheCowboyAI/cim-<name>.git cim-<name>
git commit -m "Add cim-<name> as submodule"
```

## Repository Descriptions

1. **cim-component**: Core component definitions for the Composable Information Machine
2. **cim-compose**: Composition utilities and helpers for CIM
3. **cim-conceptual-core**: Conceptual space core functionality and category theory implementations
4. **cim-core-domain**: Core domain models and abstractions
5. **cim-domain**: Domain implementation with aggregates, events, and commands
6. **cim-identity-context**: Identity bounded context for person and organization management
7. **cim-infrastructure**: Infrastructure layer with NATS integration and persistence
8. **cim-subject**: Subject management and routing utilities
9. **cim-viz-bevy**: Bevy-based visualization components for CIM

## Execution Order

1. Start with leaf dependencies (no dependencies on other cim-* modules)
2. Progress to modules with dependencies
3. Update Cargo.toml references after conversion

## Post-Conversion Tasks

1. Update `.gitmodules` with all new submodules
2. Update main `Cargo.toml` workspace members
3. Test build with all submodules
4. Update CI/CD configuration if needed
5. Document submodule initialization process in README

## Commands Summary

```bash
# Initialize all submodules after clone
git submodule update --init --recursive

# Update all submodules to latest
git submodule update --remote --merge
```
