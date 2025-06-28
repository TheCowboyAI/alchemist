# Unified Graph Vision

## Core Philosophy

In CIM, **everything is a graph**, and we achieve this through a single, flexible abstraction:

### ContextGraph<N, E>
- The **only** graph type we need
- Nodes (N) and Edges (E) can be **any type** (primitives, structs, etc.)
- **Components** provide extensibility without modifying core values
- **Recursion** through the Subgraph component

## The Power of Components

Components transform simple graphs into rich domain models:

```rust
// Start with a simple graph
let mut graph = ContextGraph::<String, f64>::new("Network");

// Add components to give it meaning
graph.get_node_mut(node_id)?
    .components.add(Label("Router".to_string()))?
    .components.add(Metadata {
        properties: json!({ "ip": "192.168.1.1" })
    })?;

// Add conceptual space features
// This makes it a "ConceptGraph" without changing its type
graph.metadata.properties.insert("conceptual_space", json!({
    "dimensions": ["reliability", "throughput"],
    "category": "Network"
}));
```

## ConceptGraph: A DDD Aggregate Pattern

A **ConceptGraph** is a DDD Aggregate that:
- Has a **root ContextGraph** as its aggregate root
- **Composes multiple ContextGraphs** as aggregate members
- **Maintains invariants** across all composed graphs
- Uses **conceptual components** for positioning in conceptual space

```rust
// ConceptGraph as DDD Aggregate
pub struct ConceptGraph {
    // Root ContextGraph (aggregate root)
    pub root: ContextGraph<String, String>,

    // Composed graphs (aggregate members)
    pub members: HashMap<ContextGraphId, Box<dyn Any>>,

    // Conceptual components on the root
    // (ConceptualSpace, Morphisms attached to root.components)
}

// Example: User Management Concept
let mut user_concept = ConceptGraph::new("UserManagement");

// Add User entity graph
let user_entity = ContextGraph::<String, String>::new("User");
user_concept.add_member(user_entity)?;

// Add Role entity graph
let role_entity = ContextGraph::<String, String>::new("Role");
user_concept.add_member(role_entity)?;

// Add Permission value object graph
let permission_vo = ContextGraph::<String, ()>::new("Permission");
user_concept.add_member(permission_vo)?;

// The root maintains consistency across all members
// and has ConceptualSpace component for positioning
```

## Everything as Graphs

### 1. Value Objects
```rust
// Money as a simple graph
let money = ContextGraph::<f64, ()>::new("Money");
let amount = money.add_node(100.0);
money.get_node_mut(amount)?
    .components.add(Label("USD".to_string()))?;
```

### 2. Entities
```rust
// User entity with identity
let user = ContextGraph::<String, String>::new("User");
let id = user.add_node("user-123".to_string());
user.get_node_mut(id)?
    .components.add(Label("Identity".to_string()))?;
```

### 3. Aggregates
```rust
// Order aggregate with subgraphs
let order = ContextGraph::<String, String>::new("Order");
let items_node = order.add_node("items".to_string());

// Line items as a subgraph
let items = ContextGraph::<String, f64>::new("LineItems");
// ... add items ...

order.get_node_mut(items_node)?
    .components.add(Subgraph { graph: Box::new(items) })?;
```

### 4. Workflows
```rust
// Workflow as a graph with conceptual features
let workflow = ContextGraph::<String, String>::new("OrderProcessing");
// Add nodes for steps
// Add edges for flow
// Add ConceptualSpace component to position in concept space
```

## Benefits of This Unified Approach

1. **Simplicity**: One graph type to rule them all
2. **Flexibility**: Any graph can gain any features through components
3. **Composability**: Graphs compose naturally through Subgraph components
4. **Type Safety**: Rust's type system ensures correctness
5. **Progressive Enhancement**: Start simple, add features as needed
6. **No Inheritance**: No complex type hierarchies to manage

## Implementation Roadmap

### Phase 1: Core Components (Done)
- ✅ ContextGraph<N, E> implementation
- ✅ Component system
- ✅ Built-in components (Label, Metadata, Subgraph)
- ✅ Recursive composition

### Phase 2: Conceptual Components (Next)
- ⏳ ConceptualSpace component
- ⏳ Morphisms component
- ⏳ Quality dimensions
- ⏳ Applied Category Theory components

### Phase 3: Domain Patterns
- ⏳ Helper functions for common patterns
- ⏳ Builder patterns for domain concepts
- ⏳ Validation and invariants

### Phase 4: Visualization
- ⏳ Render any ContextGraph
- ⏳ Special rendering for conceptual components
- ⏳ Interactive graph manipulation

## Key Insight

By using components instead of inheritance, we achieve maximum flexibility:
- A graph can be simple (just nodes and edges)
- Or complex (with conceptual space, morphisms, subgraphs)
- Or anything in between
- All without changing the core type

This is the power of composition over inheritance, applied to graph theory.
