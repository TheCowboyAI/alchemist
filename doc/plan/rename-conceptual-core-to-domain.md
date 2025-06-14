# Plan: Rename cim-conceptual-core to cim-domain-conceptualspaces

## Overview
Rename `cim-conceptual-core` to `cim-domain-conceptualspaces` to follow the established naming pattern for domain modules and properly represent it as a domain within CIM.

## Current State
- Module name: `cim-conceptual-core`
- Location: Git submodule at root level
- Contains: Conceptual space implementation, dimensions, morphisms, projections

## Target State
- Module name: `cim-domain-conceptualspaces`
- Location: Same (git submodule)
- Structure: Follows DDD pattern like other domain modules

## Implementation Steps

### Step 1: Rename the Git Repository
1. Rename the GitHub repository from `cim-conceptual-core` to `cim-domain-conceptualspaces`
2. Update git submodule URL in `.gitmodules`
3. Update local submodule reference

### Step 2: Update Package Name
1. Update `Cargo.toml`:
   - Change `name = "cim-conceptual-core"` to `name = "cim-domain-conceptualspaces"`
   - Update description to reflect domain status
   - Add proper keywords and categories

### Step 3: Restructure to Follow DDD Pattern
Create proper DDD structure:
```
cim-domain-conceptualspaces/
├── src/
│   ├── lib.rs
│   ├── aggregate/
│   │   ├── mod.rs
│   │   └── conceptual_space.rs  # ConceptualSpace aggregate
│   ├── value_objects/
│   │   ├── mod.rs
│   │   ├── concept.rs           # Concept value object
│   │   ├── quality_dimension.rs # QualityDimension value object
│   │   ├── dimension_weight.rs  # DimensionWeight value object
│   │   └── convex_region.rs     # ConvexRegion value object
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── create_space.rs
│   │   ├── add_concept.rs
│   │   └── update_weights.rs
│   ├── events/
│   │   ├── mod.rs
│   │   ├── space_created.rs
│   │   ├── concept_added.rs
│   │   └── weights_updated.rs
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── command_handler.rs
│   │   └── event_handler.rs
│   └── queries/
│       ├── mod.rs
│       └── find_similar.rs
```

### Step 4: Move Existing Code
1. Move `space.rs` content → `aggregate/conceptual_space.rs`
2. Move dimension types → `value_objects/quality_dimension.rs`
3. Move concept types → `value_objects/concept.rs`
4. Create proper commands and events
5. Keep category theory module as-is (it's domain logic)

### Step 5: Update Dependencies
Update all references in other modules:
1. `cim-domain-graph/Cargo.toml`
2. `cim-compose/Cargo.toml` (if it uses conceptual spaces)
3. Main `Cargo.toml` workspace members
4. Any other modules that depend on conceptual core

### Step 6: Update Imports
Change all imports from:
```rust
use cim_conceptual_core::*;
```
To:
```rust
use cim_domain_conceptualspaces::*;
```

### Step 7: Add Domain-Specific Features
1. Create `ConceptualSpaceAggregate` as the main aggregate root
2. Define proper domain events for conceptual space changes
3. Implement command handlers for space operations
4. Add query handlers for similarity searches

### Step 8: Update Documentation
1. Update module documentation to reflect domain status
2. Add examples showing domain usage
3. Update README with domain description

## Benefits
1. **Consistency**: Follows established naming pattern
2. **Clarity**: Clear that this is a domain within CIM
3. **Structure**: Proper DDD structure like other domains
4. **Integration**: Better integration with cim-compose

## Testing
1. Ensure all tests pass after renaming
2. Add domain-specific tests
3. Verify integration with other domains
4. Test command/event flow
