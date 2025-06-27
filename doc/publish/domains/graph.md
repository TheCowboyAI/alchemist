# Graph Domain

## Overview

The Graph Domain is the foundational module of CIM, providing visual representation and manipulation of all information as interconnected graphs. Everything in CIM is ultimately a graph - workflows, knowledge structures, relationships, and system architectures.

## Key Concepts

### Node
- **Definition**: A vertex in the graph representing an entity, concept, or process step
- **Properties**: ID, type, position, content, metadata
- **Relationships**: Connected to other nodes via edges

### Edge
- **Definition**: A directed or undirected connection between nodes
- **Properties**: Source, target, relationship type, weight, metadata
- **Types**: Dependency, similarity, sequence, hierarchy, association

### Graph
- **Definition**: A collection of nodes and edges forming a coherent structure
- **Properties**: ID, name, layout algorithm, visualization settings
- **Operations**: Create, traverse, analyze, transform, visualize

### Layout
- **Definition**: Spatial arrangement of nodes for optimal visualization
- **Algorithms**: Force-directed, hierarchical, circular, grid
- **Constraints**: Minimize edge crossings, maintain readability

## Domain Events

### Commands
- `cmd.graph.create_graph` - Create a new graph
- `cmd.graph.add_node` - Add a node to a graph
- `cmd.graph.connect_nodes` - Create an edge between nodes
- `cmd.graph.update_layout` - Recalculate graph layout
- `cmd.graph.delete_node` - Remove a node and its connections

### Events
- `event.graph.graph_created` - New graph initialized
- `event.graph.node_added` - Node added to graph
- `event.graph.edge_connected` - Nodes connected
- `event.graph.layout_updated` - Graph layout recalculated
- `event.graph.node_deleted` - Node removed from graph

### Queries
- `query.graph.find_nodes` - Search nodes by criteria
- `query.graph.find_path` - Find path between nodes
- `query.graph.get_neighbors` - Get adjacent nodes
- `query.graph.calculate_centrality` - Compute node importance

## API Reference

### GraphAggregate
```rust
pub struct GraphAggregate {
    pub id: GraphId,
    pub nodes: HashMap<NodeId, Node>,
    pub edges: HashMap<EdgeId, Edge>,
    pub metadata: GraphMetadata,
}
```

### Key Methods
- `create_graph()` - Initialize new graph
- `add_node()` - Add node to graph
- `connect_nodes()` - Create edge between nodes
- `find_shortest_path()` - Dijkstra's algorithm
- `apply_layout()` - Position nodes spatially

## Integration Examples

### Creating a Workflow Graph
```rust
// Create a new workflow graph
let cmd = CreateGraph {
    name: "Order Processing".to_string(),
    graph_type: GraphType::Workflow,
};

// Add process nodes
let start = AddNode {
    graph_id,
    node_type: NodeType::Start,
    content: "Receive Order".to_string(),
};

let validate = AddNode {
    graph_id,
    node_type: NodeType::Process,
    content: "Validate Payment".to_string(),
};

// Connect with sequence edge
let connect = ConnectNodes {
    graph_id,
    source: start_id,
    target: validate_id,
    relationship: EdgeRelationship::Sequence,
};
```

### Analyzing Graph Structure
```rust
// Find critical path in workflow
let query = FindCriticalPath {
    graph_id,
    start_node: start_id,
    end_node: complete_id,
};

// Calculate node centrality
let centrality = CalculateCentrality {
    graph_id,
    algorithm: CentralityAlgorithm::Betweenness,
};
```

## Visual Representation

The Graph domain integrates with Bevy ECS for real-time 3D visualization:

- **Nodes**: Rendered as 3D shapes with type-specific colors
- **Edges**: Drawn as lines or curves with relationship indicators
- **Layouts**: Automatic positioning for clarity
- **Interactions**: Click, drag, zoom, rotate in 3D space

## Use Cases

### Business Process Modeling
- Visual workflow design
- Process optimization
- Bottleneck identification
- Execution tracking

### Knowledge Graphs
- Concept relationships
- Semantic networks
- Information navigation
- Pattern discovery

### System Architecture
- Component dependencies
- Data flow visualization
- Service mesh representation
- Impact analysis

## Performance Characteristics

- **Node Capacity**: 10,000+ nodes per graph
- **Edge Capacity**: 50,000+ edges per graph
- **Layout Speed**: <100ms for 1,000 nodes
- **Query Performance**: O(log n) for most operations

## Best Practices

1. **Node Granularity**: Keep nodes focused on single concepts
2. **Edge Semantics**: Use meaningful relationship types
3. **Layout Selection**: Choose algorithms based on graph structure
4. **Event Granularity**: Batch operations when possible
5. **Query Optimization**: Use indices for large graphs

## Related Domains

- **Workflow Domain**: Uses graphs for process representation
- **ConceptualSpaces**: Graphs with semantic dimensions
- **Identity Domain**: Relationship graphs between people/orgs
- **Git Domain**: Commit graphs and dependencies 