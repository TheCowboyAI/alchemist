# Graph Domain API Documentation

## Overview

The Graph Domain provides comprehensive graph visualization and manipulation capabilities within CIM. It supports workflow graphs, conceptual graphs, event flow graphs, and development graphs with full CQRS/Event Sourcing implementation.

**Status**: ✅ Production Ready  
**Tests**: 100 passing  
**Version**: 1.0.0

## Table of Contents

1. [Commands](#commands)
2. [Events](#events)
3. [Queries](#queries)
4. [Value Objects](#value-objects)
5. [Integration Examples](#integration-examples)
6. [Error Handling](#error-handling)

## Commands

### CreateGraph

Creates a new graph instance.

```rust
pub struct CreateGraph {
    pub graph_id: GraphId,
    pub name: String,
    pub graph_type: GraphType,
    pub metadata: HashMap<String, Value>,
}
```

**Example:**
```json
{
  "type": "CreateGraph",
  "payload": {
    "graph_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Order Processing Workflow",
    "graph_type": "WorkflowGraph",
    "metadata": {
      "department": "Operations",
      "version": "1.0"
    }
  }
}
```

**Response:** `CommandAccepted` or `CommandRejected` with validation errors

### AddNode

Adds a node to an existing graph.

```rust
pub struct AddNode {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub node_type: NodeType,
    pub position: Position3D,
    pub metadata: HashMap<String, Value>,
}
```

**Example:**
```json
{
  "type": "AddNode",
  "payload": {
    "graph_id": "550e8400-e29b-41d4-a716-446655440000",
    "node_id": "node-001",
    "node_type": {
      "WorkflowStep": {
        "step_type": "Approval"
      }
    },
    "position": {
      "x": 100.0,
      "y": 50.0,
      "z": 0.0
    },
    "metadata": {
      "label": "Manager Approval",
      "timeout": "24h"
    }
  }
}
```

### ConnectNodes

Creates an edge between two nodes.

```rust
pub struct ConnectNodes {
    pub graph_id: GraphId,
    pub source_id: NodeId,
    pub target_id: NodeId,
    pub edge_type: EdgeType,
    pub metadata: HashMap<String, Value>,
}
```

**Example:**
```json
{
  "type": "ConnectNodes",
  "payload": {
    "graph_id": "550e8400-e29b-41d4-a716-446655440000",
    "source_id": "node-001",
    "target_id": "node-002",
    "edge_type": "Sequence",
    "metadata": {
      "condition": "approved == true"
    }
  }
}
```

### UpdateNodePosition

Updates the visual position of a node.

```rust
pub struct UpdateNodePosition {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub new_position: Position3D,
}
```

### RemoveNode

Removes a node and all its connections.

```rust
pub struct RemoveNode {
    pub graph_id: GraphId,
    pub node_id: NodeId,
}
```

### ExecuteWorkflow

Executes a workflow graph.

```rust
pub struct ExecuteWorkflow {
    pub graph_id: GraphId,
    pub start_node: NodeId,
    pub context: HashMap<String, Value>,
}
```

## Events

### GraphCreated

Emitted when a new graph is created.

```rust
pub struct GraphCreated {
    pub graph_id: GraphId,
    pub name: String,
    pub graph_type: GraphType,
    pub metadata: HashMap<String, Value>,
    pub created_at: SystemTime,
    pub created_by: UserId,
}
```

### NodeAdded

Emitted when a node is added to a graph.

```rust
pub struct NodeAdded {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub node_type: NodeType,
    pub position: Position3D,
    pub conceptual_point: ConceptualPoint,
    pub metadata: HashMap<String, Value>,
    pub added_at: SystemTime,
}
```

### NodesConnected

Emitted when nodes are connected.

```rust
pub struct NodesConnected {
    pub graph_id: GraphId,
    pub edge_id: EdgeId,
    pub source_id: NodeId,
    pub target_id: NodeId,
    pub edge_type: EdgeType,
    pub semantic_distance: f32,
    pub connected_at: SystemTime,
}
```

### NodePositionUpdated

Emitted when a node's position changes.

```rust
pub struct NodePositionUpdated {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub old_position: Position3D,
    pub new_position: Position3D,
    pub updated_at: SystemTime,
}
```

### NodeRemoved

Emitted when a node is removed.

```rust
pub struct NodeRemoved {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub removed_edges: Vec<EdgeId>,
    pub removed_at: SystemTime,
}
```

### WorkflowExecuted

Emitted when a workflow completes execution.

```rust
pub struct WorkflowExecuted {
    pub graph_id: GraphId,
    pub execution_id: ExecutionId,
    pub path: Vec<NodeId>,
    pub results: HashMap<NodeId, Value>,
    pub duration: Duration,
    pub status: WorkflowStatus,
}
```

## Queries

### GetGraph

Retrieves a complete graph with all nodes and edges.

```rust
pub struct GetGraph {
    pub graph_id: GraphId,
    pub include_metadata: bool,
}
```

**Response:**
```rust
pub struct GraphView {
    pub graph_id: GraphId,
    pub name: String,
    pub graph_type: GraphType,
    pub nodes: Vec<NodeView>,
    pub edges: Vec<EdgeView>,
    pub metadata: HashMap<String, Value>,
    pub stats: GraphStats,
}
```

### FindNodesByType

Finds all nodes of a specific type.

```rust
pub struct FindNodesByType {
    pub graph_id: GraphId,
    pub node_type: NodeType,
    pub limit: Option<usize>,
}
```

**Response:** `Vec<NodeView>`

### FindConnectedNodes

Finds all nodes connected to a given node.

```rust
pub struct FindConnectedNodes {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub direction: ConnectionDirection,
    pub max_depth: Option<u32>,
}
```

### CalculateShortestPath

Calculates the shortest path between two nodes.

```rust
pub struct CalculateShortestPath {
    pub graph_id: GraphId,
    pub source_id: NodeId,
    pub target_id: NodeId,
    pub edge_weight: EdgeWeightType,
}
```

**Response:**
```rust
pub struct PathResult {
    pub path: Vec<NodeId>,
    pub total_weight: f32,
    pub edges_traversed: Vec<EdgeId>,
}
```

### FindNodesInRegion

Finds nodes within a spatial region.

```rust
pub struct FindNodesInRegion {
    pub graph_id: GraphId,
    pub center: Position3D,
    pub radius: f32,
}
```

### GetGraphMetrics

Retrieves graph statistics and metrics.

```rust
pub struct GetGraphMetrics {
    pub graph_id: GraphId,
}
```

**Response:**
```rust
pub struct GraphMetrics {
    pub node_count: usize,
    pub edge_count: usize,
    pub average_degree: f32,
    pub clustering_coefficient: f32,
    pub connected_components: usize,
    pub diameter: Option<u32>,
}
```

## Value Objects

### GraphType

```rust
pub enum GraphType {
    WorkflowGraph,
    ConceptualGraph,
    EventFlowGraph,
    DevelopmentGraph,
}
```

### NodeType

```rust
pub enum NodeType {
    // Workflow Nodes
    WorkflowStep { step_type: StepType },
    Decision { criteria: DecisionCriteria },
    Integration { system: String },
    
    // Conceptual Nodes
    Concept { embedding: ConceptEmbedding },
    Category { region: ConvexRegion },
    
    // Event Nodes
    Event { event_type: String },
    Aggregate { aggregate_type: String },
    
    // Development Nodes
    Feature { status: FeatureStatus },
    Task { priority: Priority },
    Milestone { target_date: DateTime<Utc> },
}
```

### EdgeType

```rust
pub enum EdgeType {
    // Workflow Edges
    Sequence,
    Conditional { condition: String },
    Parallel,
    
    // Conceptual Edges
    Similarity { strength: f32 },
    Hierarchy { level: u32 },
    Association { relation_type: String },
    
    // Event Edges
    Triggers,
    Produces,
    ConsumesFrom,
    
    // Development Edges
    DependsOn,
    Blocks,
    Implements,
}
```

### Position3D

```rust
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
```

## Integration Examples

### Creating a Workflow Graph

```rust
// 1. Create the graph
let create_graph = CreateGraph {
    graph_id: GraphId::new(),
    name: "Order Fulfillment".to_string(),
    graph_type: GraphType::WorkflowGraph,
    metadata: HashMap::new(),
};

// 2. Add workflow steps
let add_start = AddNode {
    graph_id: graph_id.clone(),
    node_id: NodeId::new(),
    node_type: NodeType::WorkflowStep {
        step_type: StepType::Start,
    },
    position: Position3D::new(0.0, 0.0, 0.0),
    metadata: HashMap::new(),
};

// 3. Connect nodes
let connect = ConnectNodes {
    graph_id: graph_id.clone(),
    source_id: start_node,
    target_id: process_node,
    edge_type: EdgeType::Sequence,
    metadata: HashMap::new(),
};
```

### Cross-Domain Integration

```rust
// React to Document events
match event {
    DocumentCreated { document_id, .. } => {
        // Create a node for the document
        let node_cmd = AddNode {
            graph_id: workflow_graph_id,
            node_id: NodeId::from(document_id),
            node_type: NodeType::WorkflowStep {
                step_type: StepType::DocumentReview,
            },
            position: calculate_position(),
            metadata: doc_metadata,
        };
        
        command_bus.send(node_cmd).await?;
    }
}
```

### Semantic Graph Navigation

```rust
// Find similar concepts
let similar = FindSimilarNodes {
    graph_id: concept_graph_id,
    reference_node: node_id,
    similarity_threshold: 0.8,
    max_results: 10,
};

let results = query_handler.handle(similar).await?;
```

## Error Handling

### Common Errors

```rust
pub enum GraphError {
    GraphNotFound(GraphId),
    NodeNotFound(NodeId),
    DuplicateNode(NodeId),
    InvalidConnection {
        source: NodeId,
        target: NodeId,
        reason: String,
    },
    CyclicDependency(Vec<NodeId>),
    WorkflowExecutionFailed {
        node: NodeId,
        error: String,
    },
}
```

### Error Responses

```json
{
  "type": "https://cim.dev/errors/graph-not-found",
  "title": "Graph Not Found",
  "status": 404,
  "detail": "Graph with ID '550e8400-e29b-41d4-a716-446655440000' does not exist",
  "instance": "/api/graph/550e8400-e29b-41d4-a716-446655440000"
}
```

## Performance Considerations

- **Spatial Indexing**: Nodes are indexed using R-tree for efficient spatial queries
- **Graph Algorithms**: Optimized implementations for shortest path, clustering
- **Event Batching**: Multiple graph operations can be batched for efficiency
- **Caching**: Frequently accessed graphs are cached in memory

## WebSocket Subscriptions

Subscribe to real-time graph updates:

```javascript
const ws = new WebSocket('ws://localhost:8080/api/graph/subscribe');

ws.send(JSON.stringify({
  type: 'SubscribeToGraph',
  graph_id: '550e8400-e29b-41d4-a716-446655440000'
}));

ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  // Handle NodeAdded, NodesConnected, etc.
};
```

## Rate Limits

- **Commands**: 100 requests/minute per user
- **Queries**: 1000 requests/minute per user
- **Subscriptions**: 10 concurrent per user
- **Batch Operations**: 100 items per batch

## SDK Examples

### Rust

```rust
use cim_graph_client::{GraphClient, CreateGraph};

let client = GraphClient::new("https://api.cim.dev");
let graph = client.create_graph(CreateGraph {
    name: "My Graph".to_string(),
    graph_type: GraphType::WorkflowGraph,
    ..Default::default()
}).await?;
```

### TypeScript

```typescript
import { GraphClient, GraphType } from '@cim/graph-client';

const client = new GraphClient({
  baseUrl: 'https://api.cim.dev',
  apiKey: process.env.CIM_API_KEY
});

const graph = await client.createGraph({
  name: 'My Graph',
  graphType: GraphType.WorkflowGraph
});
``` 