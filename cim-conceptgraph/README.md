# CIM ConceptGraph

Semantic graph implementation composing context graphs with conceptual spaces.

## Overview

ConceptGraph provides a powerful abstraction that combines graph-theoretic structures with semantic positioning in conceptual space. It builds on top of ContextGraph to provide domain concepts as nodes positioned in multi-dimensional conceptual space, with relationships that have both structural and semantic properties.

## Key Concepts

### ConceptGraph

A graph where nodes represent concepts positioned in a multi-dimensional semantic space:

```rust
let mut graph = ConceptGraph::new("Knowledge Base");

// Add concepts with positions in conceptual space
let ai_concept = graph.add_concept(
    ConceptId::new(),
    ConceptualPoint { coordinates: vec![0.8, 0.9, 0.7] }, // High on intelligence, automation, complexity
    "Artificial Intelligence".to_string(),
);

let ml_concept = graph.add_concept(
    ConceptId::new(),
    ConceptualPoint { coordinates: vec![0.7, 0.8, 0.6] }, // Similar but slightly different
    "Machine Learning".to_string(),
);

// Connect with semantic relationship
graph.connect_concepts(
    ai_concept,
    ml_concept,
    SemanticRelationship::Hierarchy, // ML is a subset of AI
    0.9, // Strong relationship
)?;
```

### Conceptual Points

Positions in multi-dimensional semantic space:

```rust
pub struct ConceptualPoint {
    pub coordinates: Vec<f32>,
}

// Example dimensions might represent:
// [intelligence, automation, complexity, abstraction, ...]
```

### Semantic Relationships

Typed relationships between concepts:

- `Similarity`: Based on conceptual distance
- `Hierarchy`: Is-a relationships
- `Meronymy`: Part-whole relationships
- `Causality`: Cause-effect relationships
- `Custom(String)`: Domain-specific relationships

## Architecture

ConceptGraph is built on top of ContextGraph:

```
ConceptGraph
    ├── ContextGraph<ConceptNode, ConceptEdge>  // Graph structure
    ├── Regions (HashMap<String, ConvexRegion>)  // Named semantic regions
    └── Concept Map (ConceptId → NodeId)         // Fast concept lookup
```

## Features

### Semantic Search

Find concepts similar to a given concept:

```rust
// Find all concepts within distance 2.0 of the AI concept
let similar = graph.find_similar_concepts(ai_concept, 2.0)?;

for (node_id, distance) in similar {
    println!("Found similar concept at distance {}", distance);
}
```

### Convex Regions

Define named regions in conceptual space:

```rust
// Define a region for "Technical Concepts"
let tech_region = ConvexRegion {
    center: ConceptualPoint { coordinates: vec![0.7, 0.8, 0.6] },
    radius: 3.0,
};

graph.add_region("Technical".to_string(), tech_region);

// Find all concepts in the technical region
let tech_concepts = graph.concepts_in_region("Technical")?;
```

### Concept Navigation

Navigate the graph based on both structure and semantics:

```rust
// Get a concept by its ID
let concept = graph.get_concept_by_id(concept_id);

// Get all nodes and edges for visualization
let nodes = graph.nodes(); // Vec<(NodeId, ConceptNode)>
let edges = graph.edges(); // Vec<(EdgeId, ConceptEdge)>
```

## Usage Examples

### Building a Knowledge Graph

```rust
use cim_conceptgraph::{ConceptGraph, ConceptId, ConceptualPoint, SemanticRelationship};

// Create a knowledge graph for programming concepts
let mut graph = ConceptGraph::new("Programming Concepts");

// Add programming paradigms
let oop = graph.add_concept(
    ConceptId::new(),
    ConceptualPoint { coordinates: vec![0.6, 0.8, 0.7] }, // Moderate abstraction, high structure
    "Object-Oriented Programming".to_string(),
);

let functional = graph.add_concept(
    ConceptId::new(),
    ConceptualPoint { coordinates: vec![0.9, 0.4, 0.8] }, // High abstraction, low state
    "Functional Programming".to_string(),
);

let procedural = graph.add_concept(
    ConceptId::new(),
    ConceptualPoint { coordinates: vec![0.3, 0.6, 0.4] }, // Low abstraction, moderate structure
    "Procedural Programming".to_string(),
);

// Connect paradigms
graph.connect_concepts(oop, procedural, SemanticRelationship::Hierarchy, 0.7)?;
graph.connect_concepts(functional, procedural, SemanticRelationship::Hierarchy, 0.6)?;
```

### Semantic Analysis

```rust
// Find concepts similar to OOP
let similar_to_oop = graph.find_similar_concepts(oop, 3.0)?;

println!("Concepts similar to OOP:");
for (node_id, distance) in similar_to_oop {
    if let Some(node) = graph.get_node(node_id) {
        println!("- {} (distance: {})", node.value.label, distance);
    }
}
```

### Region-Based Queries

```rust
// Define regions for different programming paradigm families
graph.add_region("Imperative", ConvexRegion {
    center: ConceptualPoint { coordinates: vec![0.4, 0.7, 0.5] },
    radius: 2.5,
});

graph.add_region("Declarative", ConvexRegion {
    center: ConceptualPoint { coordinates: vec![0.8, 0.3, 0.7] },
    radius: 2.5,
});

// Find all imperative programming concepts
let imperative_concepts = graph.concepts_in_region("Imperative")?;
```

## Integration with Conceptual Spaces

ConceptGraph is designed to work with the conceptual spaces domain:

```rust
// TODO: Once cim-domain-conceptualspaces types are compatible
// use cim_domain_conceptualspaces::{
//     ConceptualSpace, QualityDimension, ConvexRegion
// };

// Future integration will allow:
// - Automatic region detection from conceptual spaces
// - Quality dimension mapping to coordinates
// - Prototype-based concept creation
// - Voronoi tessellation for concept boundaries
```

## Performance Considerations

### Spatial Indexing

For large graphs, consider implementing spatial indices:

- R-tree for efficient region queries
- KD-tree for nearest neighbor searches
- Locality-sensitive hashing for approximate similarity

### Caching

- Cache frequently computed distances
- Memoize region membership tests
- Store precomputed similarity matrices for hot concepts

## Error Handling

```rust
use cim_conceptgraph::{ConceptGraphError, Result};

match graph.connect_concepts(source, target, relationship, strength) {
    Ok(edge_id) => {
        // Connection successful
    }
    Err(ConceptGraphError::ConceptNotFound(node_id)) => {
        // Handle missing concept
    }
    Err(ConceptGraphError::InvalidRelationship) => {
        // Handle invalid semantic relationship
    }
    Err(e) => {
        // Handle other errors
    }
}
```

## Testing

```bash
# Run all tests
cargo test -p cim-conceptgraph

# Run specific test
cargo test -p cim-conceptgraph test_find_similar_concepts
```

## Future Enhancements

### Planned Features

1. **Automatic Concept Positioning**: Use machine learning to automatically position concepts based on their properties
2. **Dynamic Regions**: Regions that adapt based on the concepts they contain
3. **Concept Drift Detection**: Monitor how concepts move in space over time
4. **Multi-Space Support**: Allow concepts to exist in multiple conceptual spaces
5. **Similarity Metrics**: Pluggable distance functions beyond Euclidean

### Integration Goals

- Full integration with cim-domain-conceptualspaces
- Support for Gärdenfors' conceptual spaces theory
- Prototype theory implementation
- Natural category formation

## Contributing

1. Maintain compatibility with ContextGraph interface
2. Ensure spatial operations are efficient
3. Add tests for new semantic features
4. Document conceptual space dimensions used
5. Consider performance for large-scale graphs

## References

- Gärdenfors, P. (2000). Conceptual Spaces: The Geometry of Thought
- ContextGraph documentation
- Semantic Web standards (RDF, OWL)

## License

See the main project LICENSE file. 