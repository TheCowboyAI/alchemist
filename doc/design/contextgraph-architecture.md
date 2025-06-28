# ContextGraph Architecture

## Overview

**ContextGraph<N, E>** is the fundamental graph abstraction in CIM where:
- **N** can be ANY node type (including primitives like String, i32, bool)
- **E** can be ANY edge type (including primitives like f64, (), String)
- Both nodes and edges can have **immutable components** attached
- Graphs can be **recursive** through the Subgraph component

This design enables "everything is a graph" - from simple value objects to complex domain aggregates.

## Core Design

### The ContextGraph Type

```rust
pub struct ContextGraph<N, E> {
    pub id: ContextGraphId,
    pub nodes: HashMap<NodeId, NodeEntry<N>>,
    pub edges: HashMap<EdgeId, EdgeEntry<E>>,
    pub metadata: Metadata,
    pub invariants: Vec<Box<dyn GraphInvariant<N, E>>>,
}
```

### Node and Edge Entries

Nodes and edges are stored as entries that contain the typed value plus components:

```rust
pub struct NodeEntry<N> {
    pub id: NodeId,
    pub value: N,                    // The actual node value (any type)
    pub components: ComponentStorage, // Immutable components
}

pub struct EdgeEntry<E> {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub value: E,                    // The actual edge value (any type)
    pub components: ComponentStorage, // Immutable components
}
```

## Component System

Components provide extensibility without modifying the core node/edge values:

```rust
pub trait Component: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn Component>;
    fn type_name(&self) -> &'static str;
}
```

### Built-in Components

1. **Label**: Simple string naming
   ```rust
   #[derive(Debug, Clone)]
   pub struct Label(pub String);
   ```

2. **Metadata**: Rich metadata with properties
   ```rust
   pub struct Metadata {
       pub description: Option<String>,
       pub tags: Vec<String>,
       pub properties: serde_json::Map<String, serde_json::Value>,
   }
   ```

3. **GraphReference**: Reference to another graph
   ```rust
   pub struct GraphReference(pub ContextGraphId);
   ```

4. **Subgraph**: Enables recursion - a node containing an entire graph
   ```rust
   pub struct Subgraph<N, E> {
       pub graph: Box<ContextGraph<N, E>>,
   }
   ```

### Component Immutability

Components are immutable once attached:
- Can only be added once per type
- No mutable access after attachment
- Can be removed (returns ownership)
- Ensures graph integrity

## Examples

### 1. Simple Primitive Graph

```rust
// Graph with string nodes and float edge weights
let mut graph = ContextGraph::<String, f64>::new("WordNet");

let hello = graph.add_node("Hello".to_string());
let world = graph.add_node("World".to_string());

// Edge with weight
let edge = graph.add_edge(hello, world, 0.95)?;

// Add components
graph.get_node_mut(hello)?
    .components.add(Label("Greeting".to_string()))?;

graph.get_edge_mut(edge)?
    .components.add(Label("Strong Connection".to_string()))?;
```

### 2. Integer Graph with Metadata

```rust
// Graph representing a computation
let mut graph = ContextGraph::<i32, i32>::new("Computation");

let n1 = graph.add_node(10);
let n2 = graph.add_node(20);
let n3 = graph.add_node(30);

graph.add_edge(n1, n2, 1)?; // Edge weight represents operation
graph.add_edge(n2, n3, 2)?;

// Add metadata to nodes
graph.get_node_mut(n1)?
    .components.add(Metadata {
        description: Some("Input value".to_string()),
        tags: vec!["input".to_string()],
        properties: serde_json::json!({
            "source": "user",
            "validated": true
        }).as_object().unwrap().clone(),
    })?;
```

### 3. Recursive Graph Structure

```rust
// Outer graph with string nodes
let mut outer = ContextGraph::<String, String>::new("System");

// Inner graph with different types
let mut inner = ContextGraph::<i32, i32>::new("Subsystem");
let i1 = inner.add_node(100);
let i2 = inner.add_node(200);
inner.add_edge(i1, i2, 50)?;

// Add the inner graph as a subgraph component
let container = outer.add_node("Container".to_string());
outer.get_node_mut(container)?
    .components.add(Subgraph { graph: Box::new(inner) })?;

// Regular nodes can coexist
let regular = outer.add_node("Regular".to_string());
outer.add_edge(container, regular, "contains".to_string())?;
```

### 4. Domain Modeling Example

```rust
// Model an Invoice as a graph
let mut invoice = ContextGraph::<String, String>::new("Invoice");

// Add nodes for invoice components
let header = invoice.add_node("Header".to_string());
let buyer = invoice.add_node("Buyer".to_string());
let seller = invoice.add_node("Seller".to_string());
let items = invoice.add_node("LineItems".to_string());

// Add relationships
invoice.add_edge(header, buyer, "has_buyer".to_string())?;
invoice.add_edge(header, seller, "has_seller".to_string())?;
invoice.add_edge(header, items, "contains".to_string())?;

// Buyer is itself a graph (recursive)
let mut buyer_graph = ContextGraph::<String, String>::new("Party");
let name = buyer_graph.add_node("Acme Corp".to_string());
let tax_id = buyer_graph.add_node("123-45-6789".to_string());
buyer_graph.add_edge(name, tax_id, "identified_by".to_string())?;

// Attach buyer graph as subgraph
invoice.get_node_mut(buyer)?
    .components.add(Subgraph { graph: Box::new(buyer_graph) })?;
```

## Graph Operations

### Querying

```rust
// Find nodes with specific components
let labeled_nodes = graph.query_nodes_with_component::<Label>();

// Find edges with metadata
let metadata_edges = graph.query_edges_with_component::<Metadata>();

// Find all subgraph nodes (for recursion)
let subgraphs = graph.get_subgraph_nodes();

// Count total nodes including recursive subgraphs
let total_nodes = graph.total_node_count();
```

### Path Finding

```rust
// Find all paths between two nodes
let paths = graph.find_paths(start_node, end_node);
```

### Invariants

```rust
// Add invariants that must be maintained
graph.add_invariant(Box::new(Acyclic));
graph.add_invariant(Box::new(Connected));

// Check invariants after modifications
graph.check_invariants()?;
```

## Relationship to ConceptGraph

**ConceptGraph** is a **DDD Aggregate pattern** built on ContextGraph:

```rust
// ConceptGraph = DDD Aggregate of ContextGraphs

pub struct ConceptGraph {
    /// Root ContextGraph (aggregate root)
    pub root: ContextGraph<String, String>,

    /// Member ContextGraphs (aggregate members)
    pub members: HashMap<ContextGraphId, Box<dyn Any>>,

    /// Aggregate invariants
    pub invariants: Vec<ConceptGraphInvariant>,
}

// The root has conceptual components attached:
// - ConceptualSpace: positioning in quality dimensions
// - Morphisms: relationships to other concepts
// - GraphReferences: links to member graphs

impl ConceptGraph {
    /// All operations go through the aggregate root
    pub fn add_member<N, E>(&mut self, graph: ContextGraph<N, E>) -> Result<()> {
        // Validate aggregate invariants
        // Add member and update root
    }
}
```

This follows DDD principles:
- **Aggregate Root**: Only the root is referenced externally
- **Consistency Boundary**: Invariants maintained across all members
- **Transactional Boundary**: All changes atomic within aggregate
- **Composition**: Members are composed, not inherited

## Benefits

1. **Universal**: Can represent any graph with any node/edge types
2. **Extensible**: Components add metadata without changing core types
3. **Recursive**: Graphs can contain graphs naturally
4. **Type-Safe**: Rust's type system ensures correctness
5. **Performant**: Direct storage, no boxing of values
6. **Composable**: Foundation for higher-level abstractions

## Design Decisions

1. **Why Generic Types?**: Maximum flexibility - graphs of strings, integers, custom types
2. **Why Components?**: Separation of concerns - data vs metadata
3. **Why Immutable Components?**: Maintains invariants and simplifies reasoning
4. **Why NodeEntry/EdgeEntry?**: Clean separation of identity, value, and components
5. **Why Recursive via Component?**: Keeps core type simple while enabling complexity

## Usage Guidelines

1. **Choose Appropriate Types**: Use the simplest N and E types that work
2. **Use Components for Metadata**: Don't embed metadata in node/edge values
3. **Leverage Recursion**: Use Subgraph component for hierarchical structures
4. **Maintain Invariants**: Add invariants to ensure graph properties
5. **Query Efficiently**: Use component queries to find specific patterns

This architecture provides the foundation for representing "everything as a graph" in CIM.
