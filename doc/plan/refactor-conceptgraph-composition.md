# Refactor ConceptGraph as Composition Module

## Overview
`ConceptGraph` is currently in `cim-domain-graph` but should be a composition module that combines:
- `ContextGraph` (from `cim-contextgraph`) - Pure graph abstractions
- `ConceptualSpaces` (from `cim-domain-conceptualspaces`) - Semantic/conceptual domain

This follows the established pattern where domain-specific graph implementations are separate composition modules.

## Current Issues
1. `ConceptGraph` is in the wrong location (`cim-domain-graph/src/aggregate/concept_graph.rs`)
2. It should be composing `ContextGraph` with `ConceptualSpaces`, not implementing its own graph
3. `cim-domain-graph` should only contain pure graph domain logic (if needed at all)

## Refactoring Plan

### Phase 1: Create New Composition Module
1. **Create `cim-conceptgraph` module**:
   ```bash
   mkdir -p cim-conceptgraph/src
   mkdir -p cim-conceptgraph/examples
   ```

2. **Module structure**:
   ```
   cim-conceptgraph/
   ├── Cargo.toml
   ├── src/
   │   └── lib.rs  # ConceptGraph implementation
   └── examples/
       └── concept_graph_example.rs
   ```

### Phase 2: Move and Refactor ConceptGraph
1. **Move `concept_graph.rs`** from `cim-domain-graph` to `cim-conceptgraph`
2. **Refactor to use composition**:
   - Import `cim_contextgraph::ContextGraph`
   - Import `cim_domain_conceptualspaces` types
   - Refactor `ConceptGraph` to compose these two domains

### Phase 3: Update Dependencies
1. **Create `Cargo.toml` for `cim-conceptgraph`**:
   ```toml
   [dependencies]
   cim-contextgraph = { path = "../cim-contextgraph" }
   cim-domain-conceptualspaces = { path = "../cim-domain-conceptualspaces" }
   cim-domain = { path = "../cim-domain" }
   ```

2. **Update workspace** in root `Cargo.toml`

### Phase 4: Clean Up cim-domain-graph
1. Remove `concept_graph.rs` from `cim-domain-graph`
2. Update exports in `cim-domain-graph/src/lib.rs`
3. Fix any dependent code

## Expected Structure After Refactoring

```
cim-contextgraph/          # Pure graph abstractions
├── context_graph.rs
├── types.rs
└── ...

cim-domain-conceptualspaces/  # Conceptual space domain
├── aggregate/
├── commands/
├── events/
└── ...

cim-conceptgraph/         # Composition: Graph + Conceptual
├── src/
│   └── lib.rs           # ConceptGraph composing the above
└── examples/

cim-workflow-graph/       # Composition: Graph + Workflow
└── ...

cim-ipld-graph/          # Composition: Graph + IPLD
└── ...
```

## Benefits
1. **Consistency**: Follows the established pattern of composition modules
2. **Modularity**: Clear separation between pure abstractions and compositions
3. **Reusability**: Other modules can compose graphs with their domains
4. **Clarity**: Makes it obvious that ConceptGraph = ContextGraph + ConceptualSpaces

## Implementation Steps
1. Create the new module structure
2. Move and refactor the code
3. Update all imports and dependencies
4. Run tests to ensure nothing breaks
5. Update documentation
