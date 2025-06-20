# Plan: Update Event-Sourced Architecture for Conceptual Graph Compliance

## Overview

This plan outlines the steps to update the event-sourced-graph-architecture.md document to fully comply with the conceptual-graph-composition-system.md vision where "everything is a graph" and composition is based on Applied Category Theory.

## Phase 1: Core Graph Transformation (Immediate)

### 1.1 Update Domain Model Section

Replace the current Graph aggregate with ConceptGraph:

```rust
// OLD
pub struct Graph {
    pub id: GraphId,
    pub metadata: GraphMetadata,
    pub nodes: HashMap<NodeId, Node>,
    pub edges: HashMap<EdgeId, Edge>,
    // ...
}

// NEW
pub struct ConceptGraph {
    pub id: ConceptId,
    pub name: String,
    pub category: CategoryType, // From ACT
    pub quality_dimensions: Vec<QualityDimension>,
    pub structure: Graph<ConceptNode, ConceptEdge>,
    pub morphisms: Vec<GraphMorphism>,
}
```

### 1.2 Transform All Domain Objects to Graphs

Update the document to show:
- Commands as CommandGraph
- Events as EventGraph
- Policies as PolicyGraph
- Aggregates as AggregateGraph
- Entities as EntityGraph

### 1.3 Update Node and Edge Definitions

```rust
// Nodes in a concept graph
pub enum ConceptNode {
    Atom {
        id: NodeId,
        concept_type: ConceptType,
        properties: HashMap<String, Value>,
    },
    Composite {
        id: NodeId,
        subgraph: Box<ConceptGraph>,
    },
    Function {
        id: NodeId,
        input_type: ConceptType,
        output_type: ConceptType,
        implementation: FunctionImpl,
    },
}
```

## Phase 2: Add Category Theory Foundation

### 2.1 New Section: Applied Category Theory Structures

Add after the Domain Model section:

```markdown
## Applied Category Theory Foundation

Based on "Seven Sketches in Compositionality", the system implements:

### Orders (Posets)
- Hierarchical relationships between concepts
- Partial ordering for concept organization

### Database Schemas as Categories
- Graph schemas define the category structure
- Objects are concept types, morphisms are relationships

### Monoidal Categories
- Parallel composition of processes
- Tensor products for combining graphs

### Profunctors
- Relationships between different bounded contexts
- Cross-domain mappings

### Enriched Categories
- Graphs with additional structure (metrics, costs)
- Distance and similarity measures

### Toposes
- Logic and computation within graph structures
- Boolean operations on subgraphs

### Operads
- Compositional patterns for building complex systems
- Reusable graph templates
```

### 2.2 Add Graph Morphism Types

```rust
pub enum GraphMorphism {
    Homomorphism { /* structure-preserving map */ },
    Embedding { /* subgraph injection */ },
    Quotient { /* graph collapsing */ },
    Product { /* graph combination */ },
    Coproduct { /* disjoint union */ },
}
```

## Phase 3: Update Event Sourcing

### 3.1 Events as Graphs

Replace DomainEvent with EventGraph:

```rust
pub struct EventGraph {
    pub event_type: ConceptGraph,
    pub payload: DataGraph,
    pub metadata: MetadataGraph,
    pub morphisms: Vec<EventMorphism>, // How this event relates to others
}
```

### 3.2 Event Morphisms

Add event relationships through morphisms:
- Causation morphisms
- Correlation morphisms
- Transformation morphisms

## Phase 4: Enhance Conceptual Spaces

### 4.1 Conceptual Space as Graph Repository

```rust
pub struct ConceptualSpace {
    // Existing fields...

    // New: Graph repository functionality
    pub concept_graphs: HashMap<ConceptId, ConceptGraph>,
    pub morphism_index: MorphismIndex,
    pub composition_cache: CompositionCache,
}
```

### 4.2 Quality Dimensions

Update to match the new design:

```rust
pub struct QualityDimension {
    pub name: String,
    pub dimension_type: DimensionType,
    pub range: Range<f64>,
    pub metric: DistanceMetric,
}
```

## Phase 5: Add Composition Operations

### 5.1 New Section: Graph Composition

```markdown
## Graph Composition Operations

### Composition Methods
- Product: Combining two graphs
- Coproduct: Disjoint union
- Pushout: Gluing graphs together
- Pullback: Finding common structure

### Morphism Application
- Functors for graph transformation
- Natural transformations between functors
- Adjunctions for optimal mappings
```

### 5.2 Composition Examples

Add examples showing how to compose domain concepts:

```rust
// Compose User and Role graphs
let user_role = user_graph.product(role_graph, ProductType::Cartesian);

// Embed permission graph
let authorized_user = user_role.embed(permission_graph);

// Apply security functor
let secure_user = authorized_user.apply_functor(SecurityFunctor);
```

## Phase 6: Update NATS Integration

### 6.1 New Subjects for Graph Operations

```rust
// Graph composition commands
pub const GRAPH_COMPOSE: &str = "graph.commands.compose";
pub const GRAPH_EMBED: &str = "graph.commands.embed";
pub const MORPHISM_APPLY: &str = "graph.commands.apply_morphism";

// Category theory operations
pub const FUNCTOR_MAP: &str = "category.commands.functor_map";
pub const NATURAL_TRANSFORM: &str = "category.commands.natural_transform";
```

### 6.2 Morphism Events

```rust
// Morphism-related events
pub const MORPHISM_CREATED: &str = "graph.events.morphism_created";
pub const COMPOSITION_COMPLETED: &str = "graph.events.composition_completed";
```

## Phase 7: Update Architecture Diagram

Replace the current architecture diagram with one that shows:
1. Conceptual Graph Layer (top)
2. Category Theory Operations
3. Graph Composition Engine
4. Event Graphs flowing through NATS
5. Morphism-based transformations

## Phase 8: Add Migration Section

### 8.1 Migration from Current Architecture

```markdown
## Migration Path

### Phase 1: Parallel Implementation
- Build ConceptGraph alongside existing Graph
- Create adapters between representations

### Phase 2: Gradual Transformation
- Convert one aggregate at a time
- Maintain backward compatibility

### Phase 3: Full Migration
- Replace all traditional DDD with graph-based DDD
- Remove adapter layers
```

## Implementation Order

1. **Week 1**: Update core domain model sections
2. **Week 1**: Add category theory foundation
3. **Week 2**: Transform events and commands
4. **Week 2**: Update conceptual spaces
5. **Week 3**: Add composition operations
6. **Week 3**: Update NATS subjects
7. **Week 4**: Create new architecture diagrams
8. **Week 4**: Add migration guide

## Success Criteria

- [ ] All domain objects represented as graphs
- [ ] Category theory structures documented
- [ ] Morphism types defined
- [ ] Composition operations specified
- [ ] Event graphs with morphisms
- [ ] Updated NATS subjects
- [ ] Migration path clear
- [ ] Examples demonstrate graph composition

## Next Steps

1. Create updated version of event-sourced-graph-architecture.md
2. Review with team for feedback
3. Create proof-of-concept implementation
4. Update existing code to match new architecture
