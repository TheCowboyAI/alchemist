# Subgraph as Graph Principle

## Core Principle

A "subgraph" is not a distinct entity type - it's simply a graph that happens to be identified as part of another graph. This is a fundamental principle in our recursive graph architecture.

## What This Means

### 1. No Special Subgraph Type
```rust
// ❌ WRONG - Subgraph as a separate type
pub struct Subgraph {
    id: SubgraphId,
    parent: GraphId,
    nodes: Vec<NodeId>,
}

// ✅ CORRECT - Subgraph is just a ContentGraph
pub struct ContentGraph {
    id: GraphId,
    nodes: HashMap<NodeId, ContentNode>,
    edges: HashMap<EdgeId, ContentEdge>,
}
```

### 2. Subgraph Identification

A subgraph is identified in two ways:

1. **As a Node**: A node in a parent graph can contain a reference to another graph
```rust
NodeContent::Graph {
    graph_id: GraphId,      // This IS the subgraph
    graph_type: GraphType,
    summary: String,
}
```

2. **As a View**: A named subset of nodes/edges within a graph
```rust
// This is just a view/selection, not a separate entity
pub struct GraphView {
    name: String,
    node_ids: HashSet<NodeId>,  // Subset of parent graph's nodes
}
```

### 3. Recursive Structure

Since everything is a graph, we get natural recursion:
- A ContentGraph can contain nodes
- Some nodes can contain other ContentGraphs
- Those ContentGraphs can contain nodes that contain graphs
- And so on...

## Implementation Implications

### 1. No Subgraph Commands
```rust
// ❌ WRONG
ContentGraphCommand::CreateSubgraph { ... }
ContentGraphCommand::AddNodeToSubgraph { ... }

// ✅ CORRECT - Just use regular graph commands
ContentGraphCommand::AddContent {
    content: NodeContent::Graph { graph_id, ... }
}
```

### 2. No Subgraph Events
```rust
// ❌ WRONG
DomainEvent::SubgraphCreated { ... }

// ✅ CORRECT - It's just content being added
DomainEvent::ContentAdded {
    content: NodeContent::Graph { ... }
}
```

### 3. Graph Extraction

When you need to work with a "subgraph" as its own graph:
```rust
// Extract nodes that form a logical subgraph
pub fn extract_as_graph(&self, node_ids: &[NodeId]) -> ContentGraph {
    let mut new_graph = ContentGraph::new(GraphId::new());

    // Copy relevant nodes
    for id in node_ids {
        if let Some(node) = self.nodes.get(id) {
            new_graph.nodes.insert(*id, node.clone());
        }
    }

    // Copy induced edges
    for (edge_id, edge) in &self.edges {
        if node_ids.contains(&edge.source) && node_ids.contains(&edge.target) {
            new_graph.edges.insert(*edge_id, edge.clone());
        }
    }

    new_graph
}
```

## Benefits

1. **Simplicity**: One graph type to rule them all
2. **Uniformity**: Same operations work at every level
3. **Flexibility**: Any graph can contain any other graph
4. **Composability**: Graphs compose naturally through nesting

## Example: Document Context

```rust
// The document context IS a graph
let doc_context = ContentGraph::new(GraphId::new());

// It contains an aggregate node
let doc_aggregate_node = ContentNode {
    content: NodeContent::Graph {
        graph_id: doc_aggregate_graph_id,
        graph_type: GraphType::Aggregate {
            aggregate_type: "Document".to_string()
        },
        summary: "Document aggregate".to_string(),
    },
    ...
};

// The aggregate itself IS a graph (not a "subgraph")
let doc_aggregate = ContentGraph::new(doc_aggregate_graph_id);

// Which contains entity nodes that ARE graphs
let file_entity_node = ContentNode {
    content: NodeContent::Graph {
        graph_id: file_entity_graph_id,
        graph_type: GraphType::Entity {
            entity_type: "FileMetadata".to_string()
        },
        summary: "File metadata entity".to_string(),
    },
    ...
};
```

## Summary

"Subgraph" is a conceptual term we use to describe a graph that is referenced by or contained within another graph. It's not a distinct type - it's just a graph like any other. This recursive, uniform structure is what makes our architecture powerful and composable.
