# CIM-ContextGraph Module Implementation Plan

## Overview

The `cim-contextgraph` module provides the fundamental graph abstractions for CIM:
- **ContextGraph**: The base graph theory abstraction representing ALL graphs
- **ConceptGraph**: A specific type that composes multiple ContextGraphs
- **Recursive Composition**: Graphs can contain graphs through the Subgraph component
- **Component-Based Architecture**: Following ECS principles, nodes and edges are values with components attached
- **DDD Integration**: Known types represent domain concepts, primitives for simple values

## Core Architecture

```
cim-contextgraph/
├── src/
│   ├── lib.rs
│   ├── context_graph.rs      # Base graph abstraction
│   ├── types.rs              # Core types and components
│   ├── concept_graph.rs      # Compositional concept graphs
│   ├── morphisms.rs          # Graph transformations
│   ├── composition.rs        # Composition operations
│   ├── invariants.rs         # Graph invariants
│   └── visualization.rs      # Support for IA visualization
```

## Phase 1: ContextGraph Foundation (Week 1) ✓ COMPLETED

### 1.1 Base ContextGraph Type
```rust
/// The fundamental graph abstraction - represents ANY graph
/// N and E can be any type including primitives (String, i32, bool, etc.)
pub struct ContextGraph<N, E> {
    pub id: ContextGraphId,
    pub nodes: HashMap<NodeId, NodeEntry<N>>,
    pub edges: HashMap<EdgeId, EdgeEntry<E>>,
    pub metadata: Metadata,
    pub invariants: Vec<Box<dyn GraphInvariant<N, E>>>,
}

/// Node entry wraps the value with components (ECS pattern)
pub struct NodeEntry<N> {
    pub id: NodeId,
    pub value: N,                    // The actual node value (primitive or known type)
    pub components: ComponentStorage, // Immutable components attached
}

/// Edge entry wraps the value with components
pub struct EdgeEntry<E> {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub value: E,                    // The actual edge value
    pub components: ComponentStorage, // Immutable components attached
}
```

### 1.2 Component System with Recursion (ECS Pattern)
```rust
/// Components provide additional behavior/data without modifying the core value
pub trait Component: Any + Send + Sync + Debug {
    fn as_any(&self) -> &dyn Any;
}

/// Common components
pub struct Label(pub String);
pub struct Metadata { ... }
pub struct GraphReference(pub ContextGraphId);

/// RECURSION: Nodes can contain entire graphs via Subgraph component
pub struct Subgraph<N, E> {
    pub graph: Box<ContextGraph<N, E>>
}

/// Component storage ensures immutability
impl ComponentStorage {
    pub fn add<T: Component>(&mut self, component: T) { ... }
    pub fn get<T: Component>(&self) -> Option<&T> { ... }
    // No mutable access to components!
}
```

### 1.3 Recursive Operations
```rust
impl<N, E> ContextGraph<N, E> {
    /// Get all nodes that contain subgraphs (for recursive traversal)
    pub fn get_subgraph_nodes(&self) -> Vec<(&NodeId, &NodeEntry<N>)> {
        self.nodes
            .iter()
            .filter(|(_, node)| node.components.has::<Subgraph<N, E>>())
            .collect()
    }

    /// Count total nodes including nodes in subgraphs (recursive)
    pub fn total_node_count(&self) -> usize {
        let mut count = self.nodes.len();

        for (_, node) in self.get_subgraph_nodes() {
            if let Some(subgraph) = node.get_component::<Subgraph<N, E>>() {
                count += subgraph.graph.total_node_count();
            }
        }

        count
    }

    /// Recursive graph visitor pattern
    pub fn visit_recursive<F>(&self, visitor: &mut F)
    where F: FnMut(&ContextGraph<N, E>, usize) // graph, depth
    {
        self.visit_recursive_impl(visitor, 0);
    }
}
```

### 1.4 DDD Integration
```rust
/// Known domain types can be node/edge values
pub enum DomainNode {
    Entity { id: EntityId, type_name: String },
    ValueObject { type_name: String, data: Value },
    Service { name: String },
    Event { event_type: String, aggregate_id: AggregateId },
}

/// Example: Creating recursive domain structures
let mut aggregate_graph = ContextGraph::<DomainNode, Relationship>::new("OrderAggregate");

// Root entity node
let order_node = aggregate_graph.add_node(DomainNode::Entity {
    id: EntityId::new(),
    type_name: "Order".to_string(),
});

// Create a subgraph for line items
let mut line_items_graph = ContextGraph::<DomainNode, Relationship>::new("LineItems");
// ... populate line items ...

// Attach the line items graph to a node
aggregate_graph.get_node_mut(order_node).unwrap()
    .add_component(Subgraph { graph: Box::new(line_items_graph) });
```

## Phase 2: ConceptGraph Implementation (Week 2)

### 2.1 ConceptGraph as Recursive ContextGraph Composition
```rust
/// A ConceptGraph composes multiple ContextGraphs recursively
pub struct ConceptGraph {
    pub id: ConceptGraphId,
    pub root: ContextGraph<ConceptNode, ConceptEdge>,
    pub subgraphs: HashMap<ContextGraphId, Box<dyn Any>>, // Type-erased subgraphs
    pub morphisms: Vec<GraphMorphism>,
    pub conceptual_space: ConceptualSpace,
}

/// Concept nodes use components for composition
pub enum ConceptNode {
    Concept { name: String },
    Reference { graph_id: ContextGraphId },
    // SuperConcept nodes can contain other ConceptGraphs!
    SuperConcept { name: String },
}

impl ConceptGraph {
    /// Add a subgraph of any type
    pub fn add_subgraph<N, E>(&mut self, graph: ContextGraph<N, E>) -> NodeId {
        let graph_id = graph.id;
        self.subgraphs.insert(graph_id, Box::new(graph));

        // Add reference node in root graph
        let node = self.root.add_node(ConceptNode::Reference { graph_id });
        self.root.get_node_mut(node).unwrap()
            .add_component(GraphReference(graph_id));
        node
    }

    /// Create a superconcept that contains other ConceptGraphs
    pub fn create_superconcept(&mut self, name: String, concepts: Vec<ConceptGraph>) -> NodeId {
        let super_node = self.root.add_node(ConceptNode::SuperConcept { name });

        // Create a subgraph to hold the concepts
        let mut super_graph = ContextGraph::<ConceptNode, ConceptEdge>::new("SuperConceptGraph");

        // Add each concept as a node in the super_graph
        for concept in concepts {
            let concept_id = concept.id;
            self.subgraphs.insert(concept_id, Box::new(concept));

            let ref_node = super_graph.add_node(ConceptNode::Reference { graph_id: concept_id });
            super_graph.get_node_mut(ref_node).unwrap()
                .add_component(GraphReference(concept_id));
        }

        // Attach the super_graph to the superconcept node
        self.root.get_node_mut(super_node).unwrap()
            .add_component(Subgraph { graph: Box::new(super_graph) });

        super_node
    }
}
```

### 2.2 Composition Operations
- Union of graphs
- Intersection of graphs
- Product graphs
- Quotient graphs

### 2.3 Conceptual Space Integration
- Quality dimensions
- Similarity metrics
- Concept regions

## Phase 3: DDD Component Graphs (Week 3)

### 3.1 DDD Components as Graphs
```rust
/// Every DDD component can be represented as a ContextGraph
/// Following ECS: data (nodes/edges) + components (behavior)

/// Entity as a graph with ID nodes and attribute edges
pub fn create_entity_graph(entity: &Entity) -> ContextGraph<String, String> {
    let mut graph = ContextGraph::new(format!("Entity:{}", entity.id));

    // Identity node
    let id_node = graph.add_node(entity.id.to_string());
    graph.get_node_mut(id_node).unwrap()
        .add_component(Label("Identity".to_string()));

    // Attribute nodes
    for (key, value) in &entity.attributes {
        let attr_node = graph.add_node(value.to_string());
        graph.add_edge(id_node, attr_node, key.clone()).unwrap();
    }

    graph
}

/// Aggregate as a graph composition
pub fn create_aggregate_graph(aggregate: &Aggregate) -> ConceptGraph {
    let mut concept = ConceptGraph::new(format!("Aggregate:{}", aggregate.id));

    // Add root entity
    let root_graph = create_entity_graph(&aggregate.root);
    let root_node = concept.add_subgraph(root_graph);

    // Add child entities
    for entity in &aggregate.entities {
        let entity_graph = create_entity_graph(entity);
        let entity_node = concept.add_subgraph(entity_graph);

        // Connect to root
        concept.root.add_edge(root_node, entity_node, ConceptEdge::Contains).unwrap();
    }

    concept
}
```

### 3.2 Workflow Graphs
- Command graphs
- Event graphs
- Policy graphs
- Process graphs

## Phase 4: Recursive Composition Patterns (Week 4)

### 4.1 Fractal Graph Structures
```rust
/// Recursive patterns that appear at multiple scales
pub struct FractalPattern {
    pub base_pattern: ContextGraph<NodePattern, EdgePattern>,
    pub recursion_depth: usize,
    pub scale_factor: f32,
}

impl FractalPattern {
    /// Generate a fractal graph by recursive substitution
    pub fn generate<N, E>(&self) -> ContextGraph<N, E>
    where
        N: From<NodePattern>,
        E: From<EdgePattern>,
    {
        self.generate_recursive(self.recursion_depth)
    }

    fn generate_recursive<N, E>(&self, depth: usize) -> ContextGraph<N, E> {
        if depth == 0 {
            // Base case: convert pattern to graph
            self.base_pattern.map_into()
        } else {
            // Recursive case: each node becomes a subgraph
            let mut graph = ContextGraph::new(format!("Fractal-{}", depth));

            for (node_id, node) in &self.base_pattern.nodes {
                let sub_pattern = self.generate_recursive(depth - 1);

                let new_node = graph.add_node(node.value.into());
                graph.get_node_mut(new_node).unwrap()
                    .add_component(Subgraph { graph: Box::new(sub_pattern) });
            }

            // Copy edges from pattern
            for edge in &self.base_pattern.edges {
                // Map edge connections...
            }

            graph
        }
    }
}
```

### 4.2 Infinite Graph Representations
```rust
/// Lazy recursive graphs that expand on demand
pub struct LazyRecursiveGraph<N, E> {
    pub generator: Box<dyn Fn(usize) -> ContextGraph<N, E>>,
    pub expansion_rule: Box<dyn Fn(&NodeId) -> bool>,
}

/// Example: Infinite binary tree
let infinite_tree = LazyRecursiveGraph {
    generator: Box::new(|depth| {
        let mut g = ContextGraph::<i32, ()>::new(format!("Level-{}", depth));
        let root = g.add_node(depth as i32);

        // Attach expansion marker
        g.get_node_mut(root).unwrap()
            .add_component(Label("Expandable".to_string()));

        root
    }),
    expansion_rule: Box::new(|node_id| {
        // Expand nodes marked as expandable
        true
    }),
};
```

## Phase 5: IA Visualization Support (Week 5)

### 5.1 Visualization Metadata
```rust
pub struct VisualizationHints {
    pub layout_algorithm: LayoutType,
    pub node_styles: HashMap<NodeId, NodeStyle>,
    pub edge_styles: HashMap<EdgeId, EdgeStyle>,
    pub interaction_modes: Vec<InteractionMode>,
}
```

### 5.2 Interactive Operations
- Graph editing commands
- Real-time composition
- Visual morphism creation
- Concept space navigation

## Key Design Principles (UPDATED)

1. **Component-Based Architecture** - Following ECS, nodes/edges are values with components
2. **Recursive by Design** - Graphs contain graphs through Subgraph components
3. **Type Flexibility** - Nodes and edges can be ANY type (primitives or domain types)
4. **Immutable Components** - Components provide read-only additional data/behavior
5. **DDD Integration** - Known domain types work seamlessly as node/edge values
6. **Fractal Composition** - Support for self-similar patterns at different scales
7. **Type-Safe Recursion** - Generic parameters ensure type safety even in recursive structures
8. **Visualization-First** - Components carry visualization hints for all levels

## Benefits of Component-Based Recursive Approach

1. **ECS Alignment** - Follows established ECS patterns from Bevy
2. **Flexible Recursion** - Any node can contain a graph via components
3. **Type-Safe Nesting** - Subgraphs maintain their own type parameters
4. **Infinite Structures** - Can represent lazy/infinite graphs
5. **Pattern Reuse** - Fractal patterns enable complex structures from simple rules
6. **Performance** - Components can be queried efficiently at any depth
7. **Simplicity** - No complex type hierarchies, just values + components + recursion

## Integration with Existing Code

1. Replace current graph implementations with ContextGraph
2. Migrate domain models to use ContextGraph representation
3. Update IA to work with ContextGraph abstractions
4. Implement ConceptGraph composition in IA

## Success Criteria

1. Can represent any graph structure as ContextGraph
2. Can compose ContextGraphs into ConceptGraphs
3. Can create superconcepts through recursive composition
4. IA can visualize and manipulate all graph types
5. All DDD components expressible as graphs

## Migration Strategy

1. Create cim-contextgraph as new crate
2. Implement ContextGraph without breaking existing code
3. Gradually migrate existing graph usage
4. Update IA to use new abstractions
5. Deprecate old graph implementations
