# Plan: Update Documentation for ContextGraph Consistency

## Current State Analysis

### What We Have (Actual Implementation)
- **ContextGraph<N, E>**: Base graph abstraction where N and E can be ANY type
- **Component System**: Nodes and edges have immutable components attached
- **Recursive Composition**: Via Subgraph component
- **CID-based DAGs**: For event sourcing and object storage
- **ConceptGraph**: Planned but not yet implemented (will compose multiple ContextGraphs)

### Documentation Inconsistencies
1. **conceptual-graph-composition-system.md**: Uses "ConceptGraph" as the base type
2. **seven-sketches-graph-implementation.md**: Uses "ConceptGraph" throughout
3. **graph-composition-architecture.md**: Uses "GraphComposition" as the main type
4. **graph-composition-refinement.md**: Also uses "GraphComposition"
5. **recursive-graph-architecture.md**: Uses "ContentGraph" terminology
6. **update-architecture-for-conceptual-graphs.md**: Mixes ConceptGraph and ContextGraph

## Update Strategy

### Phase 1: Establish Correct Terminology

**Core Concepts:**
- **ContextGraph<N, E>**: The fundamental graph abstraction (what we have)
- **ConceptGraph**: Higher-level type that composes multiple ContextGraphs (to be built)
- **Component System**: How we attach metadata/behavior to nodes/edges
- **Recursive Composition**: Via Subgraph component

### Phase 2: Document Updates

#### 1. Update conceptual-graph-composition-system.md
- Change "ConceptGraph" to "ContextGraph" for the base abstraction
- Clarify that ConceptGraph is a higher-level composition of ContextGraphs
- Update code examples to match actual implementation
- Add section on component system

#### 2. Update seven-sketches-graph-implementation.md
- Replace ConceptGraph with ContextGraph<N, E> throughout
- Show how each sketch maps to specific ContextGraph configurations
- Update examples to use component system

#### 3. Archive/Update graph-composition-*.md files
- Move to archive as they describe a different approach
- Create new graph-composition-with-contextgraph.md showing actual approach

#### 4. Update recursive-graph-architecture.md
- Replace "ContentGraph" with "ContextGraph"
- Show actual recursive composition via Subgraph component
- Update examples to match implementation

#### 5. Create new contextgraph-architecture.md
- Document the actual ContextGraph<N, E> design
- Explain component system
- Show how it enables "everything is a graph"
- Clarify relationship to future ConceptGraph

### Phase 3: Create Missing Documentation

#### 1. component-system-design.md
- Document the Component trait
- Show built-in components (Label, Metadata, GraphReference, Subgraph)
- Explain immutability constraints

#### 2. conceptgraph-future-design.md
- Clarify what ConceptGraph will be (composition of ContextGraphs)
- Show how it relates to Applied Category Theory
- Define the roadmap for implementation

## Implementation Order

1. **Immediate** (Today):
   - Create contextgraph-architecture.md documenting what we have
   - Update recursive-graph-architecture.md to use correct terminology

2. **Next** (This Week):
   - Update conceptual-graph-composition-system.md
   - Update seven-sketches-graph-implementation.md
   - Archive outdated documents

3. **Future** (Next Week):
   - Create component-system-design.md
   - Create conceptgraph-future-design.md
   - Update all code examples

## Key Messages to Clarify

1. **ContextGraph<N, E>** is our fundamental abstraction - it can represent ANY graph
2. **Components** provide the extensibility - any metadata/behavior can be attached
3. **Recursion** is achieved through the Subgraph component
4. **ConceptGraph** (future) will compose multiple ContextGraphs for higher-level modeling
5. **Everything is a graph** is achieved through ContextGraph + Components

## Success Criteria

- [ ] All documents use consistent terminology
- [ ] Clear distinction between ContextGraph (implemented) and ConceptGraph (planned)
- [ ] Examples match actual code
- [ ] No references to GraphComposition or ContentGraph (except in archived docs)
- [ ] Component system is properly documented
- [ ] Recursive composition is clearly explained
