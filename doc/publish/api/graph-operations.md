# Graph Operations

## Graph Manipulation via NATS

CIM provides comprehensive graph operations through NATS messaging patterns. All graph manipulations are event-sourced, ensuring complete auditability and real-time collaboration across distributed systems.

## Graph Creation and Management

### Creating Graphs

#### Create New Graph
```rust
// Command: cmd.graph.create_graph
#[derive(Serialize, Deserialize)]
pub struct CreateGraphCommand {
    pub graph_type: GraphType,
    pub name: String,
    pub description: Option<String>,
    pub access_policy: AccessPolicy,
    pub initial_nodes: Vec<NodeSpec>,
    pub conceptual_space_config: Option<ConceptualSpaceConfig>,
}

// NATS Example
let command = CreateGraphCommand {
    graph_type: GraphType::ConceptualGraph,
    name: "ML Knowledge Graph".to_string(),
    description: Some("Machine learning concepts and relationships".to_string()),
    access_policy: AccessPolicy::Private,
    initial_nodes: vec![],
    conceptual_space_config: Some(ConceptualSpaceConfig::default()),
};

let response = client.request(
    "cmd.graph.create_graph",
    serde_json::to_vec(&command)?.into()
).timeout(Duration::from_secs(5)).await?;

let result: CommandResult = serde_json::from_slice(&response.payload)?;
match result {
    CommandResult::Success { event_id, .. } => {
        println!("Graph created: {}", event_id);
    }
    _ => eprintln!("Failed to create graph: {:?}", result),
}
```

### Graph Configuration

#### Update Graph Settings
```rust
// Command: cmd.graph.configure_graph
#[derive(Serialize, Deserialize)]
pub struct ConfigureGraphCommand {
    pub graph_id: GraphId,
    pub settings: GraphSettings,
    pub layout_preferences: LayoutPreferences,
    pub collaboration_rules: CollaborationRules,
}

#[derive(Serialize, Deserialize)]
pub struct GraphSettings {
    pub max_nodes: Option<u32>,
    pub max_edges: Option<u32>,
    pub auto_layout: bool,
    pub real_time_sync: bool,
    pub persistence_policy: PersistencePolicy,
}
```

## Node Operations

### Node Creation Patterns

#### Single Node Creation
```rust
// Command: cmd.graph.create_node
let create_node = CreateNodeCommand {
    graph_id: "graph-123".into(),
    node_type: NodeType::Concept {
        name: "Neural Networks".into(),
        description: "Deep learning architectures".into(),
        category: "AI/ML".into(),
    },
    position: Position3D::new(0.0, 0.0, 0.0),
    components: vec![
        ComponentData::Metadata(HashMap::from([
            ("importance".into(), Value::Number(0.9.into())),
            ("complexity".into(), Value::String("high".into())),
        ])),
        ComponentData::Visual(VisualProperties {
            color: Color::rgb(0.2, 0.6, 0.9),
            size: 1.5,
            shape: NodeShape::Sphere,
        }),
    ],
    metadata: HashMap::new(),
    parent_node: None,
};

client.publish("cmd.graph.create_node", serde_json::to_vec(&create_node)?.into()).await?;
```

#### Bulk Node Creation
```rust
// Command: cmd.graph.batch_create_nodes
let batch_command = BatchCreateNodesCommand {
    graph_id: "graph-123".into(),
    nodes: vec![
        NodeCreationSpec {
            node_type: NodeType::Concept { 
                name: "Supervised Learning".into(),
                description: "Learning with labeled data".into(),
                category: "ML".into(),
            },
            position: Position3D::new(2.0, 0.0, 0.0),
            components: vec![],
        },
        NodeCreationSpec {
            node_type: NodeType::Concept {
                name: "Unsupervised Learning".into(),
                description: "Learning without labels".into(),
                category: "ML".into(),
            },
            position: Position3D::new(-2.0, 0.0, 0.0),
            components: vec![],
        },
    ],
    auto_connect: true,
    connection_strategy: Some(ConnectionStrategy::ByCategory),
};

let response = client.request(
    "cmd.graph.batch_create_nodes",
    serde_json::to_vec(&batch_command)?.into()
).timeout(Duration::from_secs(10)).await?;
```

### Node Modification

#### Position Updates
```rust
// Command: cmd.graph.move_node
#[derive(Serialize, Deserialize)]
pub struct MoveNodeCommand {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub new_position: Position3D,
    pub animation_duration: Option<Duration>,
    pub update_conceptual_position: bool,
}

// NATS Example for smooth movement
let move_command = MoveNodeCommand {
    graph_id: "graph-123".into(),
    node_id: "node-456".into(),
    new_position: Position3D::new(5.0, 2.0, 1.0),
    animation_duration: Some(Duration::from_millis(500)),
    update_conceptual_position: true,
};

client.publish("cmd.graph.move_node", serde_json::to_vec(&move_command)?.into()).await?;

// Subscribe to movement events for real-time updates
let mut movement_subscriber = client.subscribe("event.graph.node_moved").await?;
while let Some(message) = movement_subscriber.next().await {
    let event: NodeMovedEvent = serde_json::from_slice(&message.payload)?;
    update_node_position_in_ui(event.node_id, event.new_position);
}
```

#### Component Updates
```rust
// Command: cmd.graph.update_node_components
#[derive(Serialize, Deserialize)]
pub struct UpdateNodeComponentsCommand {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub component_updates: Vec<ComponentUpdate>,
    pub merge_strategy: MergeStrategy,
}

#[derive(Serialize, Deserialize)]
pub enum ComponentUpdate {
    Add(ComponentData),
    Remove(ComponentType),
    Replace { 
        component_type: ComponentType, 
        new_data: ComponentData 
    },
    Modify { 
        component_type: ComponentType, 
        changes: Vec<PropertyChange> 
    },
}
```

## Edge Operations

### Connection Patterns

#### Direct Connection
```rust
// Command: cmd.graph.connect_nodes
let connect_command = ConnectNodesCommand {
    graph_id: "graph-123".into(),
    source_node: "node-neural-networks".into(),
    target_node: "node-deep-learning".into(),
    edge_type: EdgeType::Hierarchy {
        relationship: HierarchyType::Contains,
        strength: 0.9,
    },
    properties: HashMap::from([
        ("created_by".into(), Value::String("user-123".into())),
        ("confidence".into(), Value::Number(0.95.into())),
    ]),
    weight: Some(0.9),
};

client.publish("cmd.graph.connect_nodes", serde_json::to_vec(&connect_command)?.into()).await?;
```

#### Smart Connection Based on Similarity
```rust
// Command: cmd.graph.auto_connect_similar
#[derive(Serialize, Deserialize)]
pub struct AutoConnectSimilarCommand {
    pub graph_id: GraphId,
    pub node_ids: Vec<NodeId>,
    pub similarity_threshold: f32,
    pub max_connections_per_node: u32,
    pub connection_type: EdgeType,
}

// Connect nodes based on conceptual similarity
let auto_connect = AutoConnectSimilarCommand {
    graph_id: "graph-123".into(),
    node_ids: vec!["node-1".into(), "node-2".into(), "node-3".into()],
    similarity_threshold: 0.7,
    max_connections_per_node: 5,
    connection_type: EdgeType::Similarity { strength: 0.0 }, // Calculated automatically
};

client.publish("cmd.graph.auto_connect_similar", serde_json::to_vec(&auto_connect)?.into()).await?;
```

### Edge Management

#### Modify Edge Properties
```rust
// Command: cmd.graph.update_edge
let update_edge = UpdateEdgeCommand {
    graph_id: "graph-123".into(),
    edge_id: "edge-456".into(),
    changes: EdgeChanges {
        weight: Some(0.8),
        properties: Some(HashMap::from([
            ("last_updated".into(), Value::String(chrono::Utc::now().to_rfc3339())),
            ("update_reason".into(), Value::String("user feedback".into())),
        ])),
        edge_type: None, // Keep existing type
    },
    expected_version: Some(3),
};

let response = client.request(
    "cmd.graph.update_edge",
    serde_json::to_vec(&update_edge)?.into()
).timeout(Duration::from_secs(3)).await?;
```

#### Remove Connections
```rust
// Command: cmd.graph.disconnect_nodes
let disconnect = DisconnectNodesCommand {
    graph_id: "graph-123".into(),
    edge_id: "edge-456".into(),
    reason: "User requested removal".to_string(),
};

client.publish("cmd.graph.disconnect_nodes", serde_json::to_vec(&disconnect)?.into()).await?;
```

## Layout Operations

### Automatic Layout Algorithms

#### Force-Directed Layout
```rust
// Command: cmd.graph.apply_layout
#[derive(Serialize, Deserialize)]
pub struct ApplyLayoutCommand {
    pub graph_id: GraphId,
    pub algorithm: LayoutAlgorithm,
    pub parameters: LayoutParameters,
    pub apply_to_nodes: Option<Vec<NodeId>>,
    pub preserve_positions: Vec<NodeId>,
}

let layout_command = ApplyLayoutCommand {
    graph_id: "graph-123".into(),
    algorithm: LayoutAlgorithm::ForceDirected {
        iterations: 1000,
        cooling_factor: 0.95,
        repulsion_strength: 100.0,
        attraction_strength: 0.1,
        gravity: 0.01,
    },
    parameters: LayoutParameters {
        area_bounds: BoundingBox::new(
            Position3D::new(-50.0, -50.0, -10.0),
            Position3D::new(50.0, 50.0, 10.0),
        ),
        animation_duration: Duration::from_secs(2),
        update_frequency: Duration::from_millis(16), // 60 FPS
    },
    apply_to_nodes: None, // Apply to all nodes
    preserve_positions: vec![], // Don't preserve any positions
};

let response = client.request(
    "cmd.graph.apply_layout",
    serde_json::to_vec(&layout_command)?.into()
).timeout(Duration::from_secs(30)).await?;

// Monitor layout progress
let mut layout_subscriber = client.subscribe("event.graph.layout_progress").await?;
while let Some(message) = layout_subscriber.next().await {
    let progress: LayoutProgressEvent = serde_json::from_slice(&message.payload)?;
    println!("Layout progress: {}%", progress.completion_percentage);
    
    if progress.completed {
        println!("Layout calculation completed!");
        break;
    }
}
```

#### Hierarchical Layout
```rust
let hierarchical_layout = ApplyLayoutCommand {
    graph_id: "graph-123".into(),
    algorithm: LayoutAlgorithm::Hierarchical {
        direction: HierarchicalDirection::TopDown,
        level_separation: 5.0,
        node_separation: 3.0,
        rank_assignment: RankAssignment::LongestPath,
    },
    parameters: LayoutParameters::default(),
    apply_to_nodes: None,
    preserve_positions: vec!["root-node".into()], // Keep root at origin
};
```

#### Circular Layout
```rust
let circular_layout = ApplyLayoutCommand {
    graph_id: "graph-123".into(),
    algorithm: LayoutAlgorithm::Circular {
        radius: 10.0,
        start_angle: 0.0,
        ordering: CircularOrdering::Optimal, // Minimize edge crossings
    },
    parameters: LayoutParameters::default(),
    apply_to_nodes: None,
    preserve_positions: vec![],
};
```

## Subgraph Operations

### Subgraph Extraction

#### Extract by Node Selection
```rust
// Command: cmd.graph.extract_subgraph
#[derive(Serialize, Deserialize)]
pub struct ExtractSubgraphCommand {
    pub source_graph_id: GraphId,
    pub target_graph_id: Option<GraphId>,
    pub extraction_criteria: ExtractionCriteria,
    pub include_edges: EdgeInclusionPolicy,
    pub copy_or_move: OperationType,
}

let extract_command = ExtractSubgraphCommand {
    source_graph_id: "main-graph".into(),
    target_graph_id: Some("subgraph-ml".into()),
    extraction_criteria: ExtractionCriteria::ByNodeIds(vec![
        "node-neural-networks".into(),
        "node-deep-learning".into(),
        "node-supervised-learning".into(),
    ]),
    include_edges: EdgeInclusionPolicy::AllConnecting,
    copy_or_move: OperationType::Copy,
};

let response = client.request(
    "cmd.graph.extract_subgraph",
    serde_json::to_vec(&extract_command)?.into()
).timeout(Duration::from_secs(10)).await?;
```

#### Extract by Criteria
```rust
let extract_by_category = ExtractSubgraphCommand {
    source_graph_id: "main-graph".into(),
    target_graph_id: Some("ai-concepts".into()),
    extraction_criteria: ExtractionCriteria::ByFilter(NodeFilter::ByMetadata {
        key: "category".to_string(),
        value: Value::String("AI".to_string()),
    }),
    include_edges: EdgeInclusionPolicy::OnlyWithinSelection,
    copy_or_move: OperationType::Copy,
};
```

### Subgraph Merging

#### Merge Multiple Graphs
```rust
// Command: cmd.graph.merge_graphs
#[derive(Serialize, Deserialize)]
pub struct MergeGraphsCommand {
    pub target_graph_id: GraphId,
    pub source_graphs: Vec<GraphId>,
    pub merge_strategy: MergeStrategy,
    pub conflict_resolution: ConflictResolution,
    pub position_adjustment: PositionAdjustment,
}

let merge_command = MergeGraphsCommand {
    target_graph_id: "unified-graph".into(),
    source_graphs: vec!["graph-1".into(), "graph-2".into(), "graph-3".into()],
    merge_strategy: MergeStrategy::Union,
    conflict_resolution: ConflictResolution::KeepBoth {
        suffix_pattern: "_{source_graph}".to_string(),
    },
    position_adjustment: PositionAdjustment::AvoidOverlap {
        min_distance: 2.0,
    },
};

let response = client.request(
    "cmd.graph.merge_graphs",
    serde_json::to_vec(&merge_command)?.into()
).timeout(Duration::from_secs(15)).await?;
```

## Real-Time Collaboration

### Collaborative Editing

#### Lock Nodes for Editing
```rust
// Command: cmd.graph.lock_nodes
#[derive(Serialize, Deserialize)]
pub struct LockNodesCommand {
    pub graph_id: GraphId,
    pub node_ids: Vec<NodeId>,
    pub lock_type: LockType,
    pub timeout: Duration,
    pub exclusive: bool,
}

let lock_command = LockNodesCommand {
    graph_id: "collaborative-graph".into(),
    node_ids: vec!["node-being-edited".into()],
    lock_type: LockType::Edit,
    timeout: Duration::from_minutes(5),
    exclusive: true,
};

let response = client.request(
    "cmd.graph.lock_nodes",
    serde_json::to_vec(&lock_command)?.into()
).timeout(Duration::from_secs(3)).await?;

// Process exclusive editing
if let CommandResult::Success { .. } = response {
    // Perform edits...
    
    // Release lock when done
    let unlock_command = UnlockNodesCommand {
        graph_id: "collaborative-graph".into(),
        node_ids: vec!["node-being-edited".into()],
    };
    
    client.publish("cmd.graph.unlock_nodes", serde_json::to_vec(&unlock_command)?.into()).await?;
}
```

#### Broadcast User Presence
```rust
// Command: cmd.graph.update_user_presence
#[derive(Serialize, Deserialize)]
pub struct UpdateUserPresenceCommand {
    pub graph_id: GraphId,
    pub user_id: UserId,
    pub presence_info: PresenceInfo,
}

#[derive(Serialize, Deserialize)]
pub struct PresenceInfo {
    pub viewport: BoundingBox,
    pub selected_nodes: Vec<NodeId>,
    pub cursor_position: Option<Position3D>,
    pub activity_status: ActivityStatus,
}

// Update presence every few seconds
let presence_command = UpdateUserPresenceCommand {
    graph_id: "collaborative-graph".into(),
    user_id: "user-123".into(),
    presence_info: PresenceInfo {
        viewport: current_viewport,
        selected_nodes: get_selected_nodes(),
        cursor_position: Some(get_cursor_3d_position()),
        activity_status: ActivityStatus::Active,
    },
};

client.publish("cmd.graph.update_user_presence", serde_json::to_vec(&presence_command)?.into()).await?;

// Subscribe to other users' presence updates
let mut presence_subscriber = client.subscribe("event.graph.user_presence_updated").await?;
while let Some(message) = presence_subscriber.next().await {
    let event: UserPresenceUpdatedEvent = serde_json::from_slice(&message.payload)?;
    update_user_cursor_display(event.user_id, event.presence_info);
}
```

## Performance Optimization

### Bulk Operations

#### Batch Multiple Commands
```rust
// Command: cmd.graph.batch_operations
#[derive(Serialize, Deserialize)]
pub struct BatchOperationsCommand {
    pub graph_id: GraphId,
    pub operations: Vec<GraphOperation>,
    pub transaction_id: Option<TransactionId>,
    pub atomic: bool,
}

#[derive(Serialize, Deserialize)]
pub enum GraphOperation {
    CreateNode(CreateNodeCommand),
    UpdateNode(UpdateNodeCommand),
    DeleteNode(DeleteNodeCommand),
    ConnectNodes(ConnectNodesCommand),
    DisconnectNodes(DisconnectNodesCommand),
}

let batch_command = BatchOperationsCommand {
    graph_id: "graph-123".into(),
    operations: vec![
        GraphOperation::CreateNode(create_node_1),
        GraphOperation::CreateNode(create_node_2),
        GraphOperation::ConnectNodes(connect_command),
    ],
    transaction_id: Some("txn-456".into()),
    atomic: true, // All operations succeed or all fail
};

let response = client.request(
    "cmd.graph.batch_operations",
    serde_json::to_vec(&batch_command)?.into()
).timeout(Duration::from_secs(30)).await?;
```

### Streaming Large Graphs

#### Stream Graph Data
```rust
// For large graphs, use streaming pattern
let stream_query = StreamGraphQuery {
    graph_id: "large-graph".into(),
    chunk_size: 1000,
    include_components: false,
    compression: CompressionType::Zstd,
};

let mut subscriber = client.subscribe("stream.graph.data.large-graph").await?;

// Request stream start
client.publish(
    "query.graph.stream_data",
    serde_json::to_vec(&stream_query)?.into()
).await?;

// Process streamed chunks
while let Some(message) = subscriber.next().await {
    let chunk: GraphDataChunk = serde_json::from_slice(&message.payload)?;
    
    process_graph_chunk(chunk.nodes, chunk.edges).await?;
    
    if chunk.is_final {
        println!("Graph streaming completed");
        break;
    }
}
```

## Monitoring and Analytics

### Graph Metrics

#### Calculate Graph Statistics
```rust
// Query: query.graph.calculate_metrics
let metrics_query = CalculateMetricsQuery {
    graph_id: "graph-123".into(),
    metrics: vec![
        MetricType::NodeCount,
        MetricType::EdgeCount,
        MetricType::Density,
        MetricType::ClusteringCoefficient,
        MetricType::AveragePathLength,
        MetricType::Centrality(CentralityMeasure::Betweenness),
        MetricType::Centrality(CentralityMeasure::Eigenvector),
    ],
    time_range: Some(TimeRange::LastHour),
    group_by: Some(GroupingCriteria::NodeType),
};

let response = client.request(
    "query.graph.calculate_metrics",
    serde_json::to_vec(&metrics_query)?.into()
).timeout(Duration::from_secs(10)).await?;

let metrics: QueryResult<GraphMetrics> = serde_json::from_slice(&response.payload)?;
println!("Graph density: {:.3}", metrics.data.density);
println!("Average path length: {:.2}", metrics.data.average_path_length);
```

### Performance Monitoring

#### Subscribe to Performance Events
```rust
// Monitor graph operation performance
let mut perf_subscriber = client.subscribe("event.graph.performance.*").await?;

while let Some(message) = perf_subscriber.next().await {
    match message.subject.as_str() {
        "event.graph.performance.slow_operation" => {
            let event: SlowOperationEvent = serde_json::from_slice(&message.payload)?;
            log::warn!("Slow operation detected: {} took {}ms", 
                event.operation_type, event.duration_ms);
        }
        "event.graph.performance.memory_usage" => {
            let event: MemoryUsageEvent = serde_json::from_slice(&message.payload)?;
            if event.usage_percentage > 80.0 {
                log::error!("High memory usage: {:.1}%", event.usage_percentage);
            }
        }
        _ => {}
    }
}
```

---

**All graph operations in CIM are performed through NATS messaging, ensuring distributed consistency, real-time collaboration, and complete auditability of all graph modifications.** 