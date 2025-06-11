# Why ContextGraph Instead of PetGraph?

## The Need for Domain-Specific Abstraction

While PetGraph is an excellent graph library, ContextGraph provides essential domain-specific features that PetGraph doesn't:

### 1. Component System

```rust
// ContextGraph: Nodes and edges have components
pub struct NodeEntry<N> {
    pub id: NodeId,
    pub value: N,
    pub components: ComponentStorage,  // This is the key difference
}

// PetGraph: Just the value
petgraph::graph::Node<N> {
    weight: N,  // No component system
}
```

The component system is crucial because:
- **Separation of Concerns**: Node/edge values remain pure data
- **Extensibility**: Add metadata without changing core types
- **Immutability**: Components are immutable once attached
- **Type Safety**: Each component type is tracked separately

### 2. Graph-Level Invariants

```rust
// ContextGraph supports invariants
pub struct ContextGraph<N, E> {
    pub invariants: Vec<Box<dyn GraphInvariant<N, E>>>,
}

// Can enforce domain rules
graph.add_invariant(Box::new(Acyclic));
graph.add_invariant(Box::new(Connected));
graph.check_invariants()?;  // Validated on every modification
```

PetGraph doesn't have built-in invariant support - you'd have to wrap it anyway.

### 3. Recursive Graph Support

```rust
// ContextGraph: First-class support for graphs containing graphs
pub struct Subgraph<N, E> {
    pub graph: Box<ContextGraph<N, E>>,
}

// A node can contain an entire graph via component
node.components.add(Subgraph { graph: Box::new(inner_graph) })?;
```

This recursive structure is fundamental to our "everything is a graph" philosophy.

### 4. Domain-Aligned API

```rust
// ContextGraph API is designed for our domain
graph.add_node(value)  // Returns NodeId
graph.get_node_mut(id)?.components.add(Label("Hello"))?;
graph.query_nodes_with_component::<ConceptualSpace>();

// vs PetGraph's more generic API
graph.add_node(value)  // Returns NodeIndex
graph[node_idx]  // Just the weight, no components
```

### 5. Identity Management

```rust
// ContextGraph uses UUIDs for global identity
pub struct NodeId(Uuid);  // Globally unique
pub struct ContextGraphId(Uuid);  // Graphs have identity too

// PetGraph uses indices
NodeIndex(usize);  // Only unique within graph
```

This matters for:
- Distributed systems (CIDs, references across graphs)
- Persistence (stable IDs across sessions)
- Cross-graph references (GraphReference component)

## Could We Wrap PetGraph?

Yes, we could (and we did consider it):

```rust
// Option we considered:
pub struct ContextGraph<N, E> {
    graph: PetGraph<NodeEntry<N>, EdgeEntry<E>>,
    invariants: Vec<Box<dyn GraphInvariant>>,
}
```

But this would:
1. Add an extra layer of indirection
2. Expose PetGraph's index-based API alongside our ID-based API
3. Make it harder to optimize for our specific use cases
4. Complicate the component system implementation

## Update: We Actually Built Both!

We have two implementations:

1. **context_graph.rs**: Our custom implementation (currently used)
2. **context_graph_v2.rs**: Wraps PetGraph while keeping our API

The v2 implementation proves we can have the best of both worlds:

```rust
// From context_graph_v2.rs - this actually exists!
pub struct ContextGraph<N, E> {
    pub id: ContextGraphId,

    // The actual PetGraph - we get all its algorithms!
    pub graph: Graph<NodeEntry<N>, EdgeEntry<E>>,

    // Additional mappings for our ID system
    node_id_map: HashMap<NodeId, NodeIndex>,
    edge_id_map: HashMap<EdgeId, EdgeIndex>,

    pub metadata: Metadata,
    pub invariants: Vec<Box<dyn GraphInvariant<N, E>>>,
}
```

With v2, we get PetGraph algorithms directly:
```rust
// Shortest path using Dijkstra
graph.shortest_path(start, end)

// Cycle detection
graph.is_cyclic()

// Strongly connected components
graph.strongly_connected_components()

// Topological sort
graph.topological_sort()

// And many more from PetGraph!
```

Benefits:
- Get PetGraph's optimized algorithms
- Proven implementation
- Better performance for large graphs

Costs:
- ID translation overhead
- More complex implementation
- Dependency on external crate

## Why We Currently Use the Custom Implementation

Even though v2 exists and works, we use the custom implementation because:

1. **Simplicity**: Fewer dependencies, easier to understand
2. **Control**: We can optimize specifically for our use cases
3. **No Translation Overhead**: Direct access without ID mapping
4. **Sufficient for Now**: We haven't needed complex graph algorithms yet

## When to Switch to v2?

Consider switching when:
- Need advanced algorithms (shortest path, minimum spanning tree, etc.)
- Performance becomes critical for large graphs
- Want proven correctness of graph operations
- Need specific PetGraph features

## Conclusion

ContextGraph is our fundamental abstraction because it provides:

1. **Components**: Essential for our architecture
2. **Invariants**: Domain rule enforcement
3. **Recursion**: Graphs containing graphs
4. **Identity**: Stable UUIDs for distributed systems
5. **Domain API**: Tailored to our needs

Whether we use our custom implementation or wrap PetGraph is an **implementation detail**. The key insight is that **ContextGraph is our abstraction boundary**, not PetGraph.

Having both implementations gives us flexibility:
- Start simple with the custom version
- Switch to v2 when we need PetGraph's power
- Keep the same API either way

This is good engineering: the right abstraction with multiple implementation strategies.
