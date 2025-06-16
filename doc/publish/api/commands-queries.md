# Commands & Queries

## CQRS Operations over NATS

CIM implements Command Query Responsibility Segregation (CQRS) using NATS messaging patterns. Commands modify state and trigger business processes, while queries retrieve data from optimized read models.

## Command Pattern

All commands follow the pattern `cmd.{domain}.{action}` and use NATS request-reply for synchronous validation and acknowledgment.

### Command Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub command_id: CommandId,
    pub command_type: String,
    pub aggregate_id: AggregateId,
    pub actor: ActorId,
    pub timestamp: SystemTime,
    pub correlation_id: Option<CorrelationId>,
    pub payload: serde_json::Value,
    pub metadata: CommandMetadata,
}
```

### Command Result

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandResult {
    Success {
        event_id: EventId,
        aggregate_version: u64,
        generated_events: Vec<EventId>,
    },
    ValidationError {
        errors: Vec<ValidationError>,
        error_code: String,
    },
    BusinessError {
        message: String,
        error_code: String,
        retry_allowed: bool,
    },
    ConcurrencyError {
        expected_version: u64,
        actual_version: u64,
    },
}
```

## Graph Commands

### Node Management Commands

#### `cmd.graph.create_node`
Creates a new node in a graph.

```rust
#[derive(Serialize, Deserialize)]
pub struct CreateNodeCommand {
    pub graph_id: GraphId,
    pub node_type: NodeType,
    pub position: Position3D,
    pub components: Vec<ComponentData>,
    pub metadata: HashMap<String, Value>,
    pub parent_node: Option<NodeId>,
}
```

**NATS Example:**
```rust
let command = CreateNodeCommand {
    graph_id: "graph-123".into(),
    node_type: NodeType::Concept {
        name: "Machine Learning".into(),
        description: "AI pattern recognition".into(),
    },
    position: Position3D::new(1.0, 2.0, 3.0),
    components: vec![],
    metadata: HashMap::new(),
    parent_node: None,
};

let response = client.request(
    "cmd.graph.create_node",
    serde_json::to_vec(&command)?.into()
).timeout(Duration::from_secs(5)).await?;

let result: CommandResult = serde_json::from_slice(&response.payload)?;
match result {
    CommandResult::Success { event_id, .. } => {
        println!("Node created with event: {}", event_id);
    }
    CommandResult::ValidationError { errors, .. } => {
        eprintln!("Validation failed: {:?}", errors);
    }
    _ => eprintln!("Command failed: {:?}", result),
}
```

#### `cmd.graph.update_node`
Updates properties of an existing node.

```rust
#[derive(Serialize, Deserialize)]
pub struct UpdateNodeCommand {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub changes: NodeChanges,
    pub expected_version: Option<u64>,
    pub reason: String,
}

#[derive(Serialize, Deserialize)]
pub struct NodeChanges {
    pub position: Option<Position3D>,
    pub metadata: Option<HashMap<String, Value>>,
    pub component_updates: Vec<ComponentUpdate>,
}
```

#### `cmd.graph.delete_node`
Removes a node and its connections.

```rust
#[derive(Serialize, Deserialize)]
pub struct DeleteNodeCommand {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub cascade_edges: bool,
    pub reason: String,
}
```

### Edge Management Commands

#### `cmd.graph.connect_nodes`
Creates an edge between two nodes.

```rust
#[derive(Serialize, Deserialize)]
pub struct ConnectNodesCommand {
    pub graph_id: GraphId,
    pub source_node: NodeId,
    pub target_node: NodeId,
    pub edge_type: EdgeType,
    pub properties: HashMap<String, Value>,
    pub weight: Option<f32>,
}
```

#### `cmd.graph.update_edge`
Modifies edge properties.

```rust
#[derive(Serialize, Deserialize)]
pub struct UpdateEdgeCommand {
    pub graph_id: GraphId,
    pub edge_id: EdgeId,
    pub changes: EdgeChanges,
    pub expected_version: Option<u64>,
}
```

#### `cmd.graph.disconnect_nodes`
Removes an edge between nodes.

```rust
#[derive(Serialize, Deserialize)]
pub struct DisconnectNodesCommand {
    pub graph_id: GraphId,
    pub edge_id: EdgeId,
    pub reason: String,
}
```

### Batch Commands

#### `cmd.graph.batch_create_nodes`
Creates multiple nodes in a single transaction.

```rust
#[derive(Serialize, Deserialize)]
pub struct BatchCreateNodesCommand {
    pub graph_id: GraphId,
    pub nodes: Vec<NodeCreationSpec>,
    pub auto_connect: bool,
    pub connection_strategy: Option<ConnectionStrategy>,
}

// NATS Example for batch operations
let batch_command = BatchCreateNodesCommand {
    graph_id: "graph-123".into(),
    nodes: vec![
        NodeCreationSpec {
            node_type: NodeType::Concept { name: "Node 1".into() },
            position: Position3D::new(0.0, 0.0, 0.0),
            components: vec![],
        },
        NodeCreationSpec {
            node_type: NodeType::Concept { name: "Node 2".into() },
            position: Position3D::new(1.0, 0.0, 0.0),
            components: vec![],
        },
    ],
    auto_connect: true,
    connection_strategy: Some(ConnectionStrategy::FullyConnected),
};

let response = client.request(
    "cmd.graph.batch_create_nodes",
    serde_json::to_vec(&batch_command)?.into()
).timeout(Duration::from_secs(10)).await?;
```

## Workflow Commands

### Process Management

#### `cmd.workflow.start_process`
Initiates workflow execution.

```rust
#[derive(Serialize, Deserialize)]
pub struct StartProcessCommand {
    pub workflow_id: WorkflowId,
    pub input_data: HashMap<String, Value>,
    pub priority: Priority,
    pub context: WorkflowContext,
    pub timeout: Option<Duration>,
}
```

#### `cmd.workflow.advance_step`
Moves workflow to next step.

```rust
#[derive(Serialize, Deserialize)]
pub struct AdvanceStepCommand {
    pub process_instance_id: ProcessInstanceId,
    pub current_step: NodeId,
    pub step_output: HashMap<String, Value>,
    pub next_step_override: Option<NodeId>,
}
```

#### `cmd.workflow.abort_process`
Terminates workflow execution.

```rust
#[derive(Serialize, Deserialize)]
pub struct AbortProcessCommand {
    pub process_instance_id: ProcessInstanceId,
    pub reason: String,
    pub cleanup_strategy: CleanupStrategy,
}
```

## Agent Commands

### Agent Management

#### `cmd.agent.register`
Registers a new AI agent.

```rust
#[derive(Serialize, Deserialize)]
pub struct RegisterAgentCommand {
    pub agent_type: AgentType,
    pub capabilities: Vec<Capability>,
    pub communication_patterns: Vec<CommunicationPattern>,
    pub resource_requirements: ResourceRequirements,
}
```

#### `cmd.agent.assign_task`
Assigns work to an agent.

```rust
#[derive(Serialize, Deserialize)]
pub struct AssignTaskCommand {
    pub agent_id: AgentId,
    pub task_type: TaskType,
    pub task_parameters: HashMap<String, Value>,
    pub priority: Priority,
    pub deadline: Option<SystemTime>,
}
```

## Query Pattern

Queries use the pattern `query.{domain}.{query_type}` and implement request-reply for immediate responses.

### Query Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub query_id: QueryId,
    pub query_type: String,
    pub actor: ActorId,
    pub timestamp: SystemTime,
    pub parameters: QueryParameters,
    pub pagination: Option<Pagination>,
}
```

### Query Result

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult<T> {
    pub data: T,
    pub metadata: QueryMetadata,
    pub pagination: Option<PaginationResult>,
    pub cache_info: CacheInfo,
}
```

## Graph Queries

### Node Queries

#### `query.graph.find_nodes`
Searches for nodes by criteria.

```rust
#[derive(Serialize, Deserialize)]
pub struct FindNodesQuery {
    pub graph_id: GraphId,
    pub filters: Vec<NodeFilter>,
    pub sort_by: Vec<SortCriterion>,
    pub pagination: Pagination,
}

#[derive(Serialize, Deserialize)]
pub enum NodeFilter {
    ByType(NodeType),
    ByPosition(BoundingBox),
    ByMetadata { key: String, value: Value },
    ByComponent(ComponentType),
}
```

**NATS Example:**
```rust
let query = FindNodesQuery {
    graph_id: "graph-123".into(),
    filters: vec![
        NodeFilter::ByType(NodeType::Concept),
        NodeFilter::ByPosition(BoundingBox::new(
            Position3D::new(0.0, 0.0, 0.0),
            Position3D::new(10.0, 10.0, 10.0),
        )),
    ],
    sort_by: vec![SortCriterion::ByDistance(Position3D::new(5.0, 5.0, 5.0))],
    pagination: Pagination::new(0, 50),
};

let response = client.request(
    "query.graph.find_nodes",
    serde_json::to_vec(&query)?.into()
).timeout(Duration::from_secs(3)).await?;

let result: QueryResult<Vec<NodeView>> = serde_json::from_slice(&response.payload)?;
for node in result.data {
    println!("Found node: {} at {:?}", node.id, node.position);
}
```

#### `query.graph.get_node`
Retrieves a specific node by ID.

```rust
#[derive(Serialize, Deserialize)]
pub struct GetNodeQuery {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub include_components: bool,
    pub include_edges: bool,
}
```

#### `query.graph.get_neighbors`
Gets nodes connected to a specific node.

```rust
#[derive(Serialize, Deserialize)]
pub struct GetNeighborsQuery {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub direction: EdgeDirection,
    pub edge_types: Option<Vec<EdgeType>>,
    pub max_depth: u32,
}

#[derive(Serialize, Deserialize)]
pub enum EdgeDirection {
    Incoming,
    Outgoing,
    Both,
}
```

### Graph Structure Queries

#### `query.graph.get_subgraph`
Extracts a subgraph around specific nodes.

```rust
#[derive(Serialize, Deserialize)]
pub struct GetSubgraphQuery {
    pub graph_id: GraphId,
    pub center_nodes: Vec<NodeId>,
    pub radius: u32,
    pub include_metadata: bool,
    pub layout_algorithm: Option<LayoutAlgorithm>,
}
```

#### `query.graph.calculate_path`
Finds paths between nodes.

```rust
#[derive(Serialize, Deserialize)]
pub struct CalculatePathQuery {
    pub graph_id: GraphId,
    pub start_node: NodeId,
    pub end_node: NodeId,
    pub algorithm: PathfindingAlgorithm,
    pub constraints: Vec<PathConstraint>,
}

#[derive(Serialize, Deserialize)]
pub enum PathfindingAlgorithm {
    Dijkstra,
    AStar { heuristic: HeuristicFunction },
    BidirectionalBFS,
}
```

## Conceptual Space Queries

### Similarity Queries

#### `query.conceptual.find_similar`
Finds entities similar to a given entity.

```rust
#[derive(Serialize, Deserialize)]
pub struct FindSimilarQuery {
    pub entity_id: EntityId,
    pub similarity_threshold: f32,
    pub max_results: u32,
    pub similarity_type: SimilarityType,
}

// NATS Example
let query = FindSimilarQuery {
    entity_id: "entity-123".into(),
    similarity_threshold: 0.8,
    max_results: 10,
    similarity_type: SimilarityType::Cosine,
};

let response = client.request(
    "query.conceptual.find_similar",
    serde_json::to_vec(&query)?.into()
).await?;

let result: QueryResult<Vec<SimilarityMatch>> = serde_json::from_slice(&response.payload)?;
```

#### `query.conceptual.cluster_entities`
Groups entities into clusters.

```rust
#[derive(Serialize, Deserialize)]
pub struct ClusterEntitiesQuery {
    pub entity_ids: Vec<EntityId>,
    pub clustering_algorithm: ClusteringAlgorithm,
    pub num_clusters: Option<u32>,
    pub min_cluster_size: u32,
}
```

### Category Queries

#### `query.conceptual.get_categories`
Retrieves categories in a conceptual space.

```rust
#[derive(Serialize, Deserialize)]
pub struct GetCategoriesQuery {
    pub space_id: ConceptualSpaceId,
    pub filters: Vec<CategoryFilter>,
    pub include_members: bool,
}
```

## Workflow Queries

### Process Queries

#### `query.workflow.get_process_status`
Gets current status of workflow processes.

```rust
#[derive(Serialize, Deserialize)]
pub struct GetProcessStatusQuery {
    pub process_instance_ids: Vec<ProcessInstanceId>,
    pub include_history: bool,
    pub include_metrics: bool,
}
```

#### `query.workflow.list_active_processes`
Lists currently running workflows.

```rust
#[derive(Serialize, Deserialize)]
pub struct ListActiveProcessesQuery {
    pub workflow_id: Option<WorkflowId>,
    pub actor: Option<ActorId>,
    pub started_after: Option<SystemTime>,
    pub pagination: Pagination,
}
```

## Advanced Query Patterns

### Aggregation Queries

#### `query.graph.aggregate_metrics`
Calculates graph metrics and statistics.

```rust
#[derive(Serialize, Deserialize)]
pub struct AggregateMetricsQuery {
    pub graph_id: GraphId,
    pub metrics: Vec<MetricType>,
    pub time_range: Option<TimeRange>,
    pub group_by: Option<GroupingCriteria>,
}

#[derive(Serialize, Deserialize)]
pub enum MetricType {
    NodeCount,
    EdgeCount,
    Density,
    ClusteringCoefficient,
    AveragePathLength,
    Centrality(CentralityMeasure),
}
```

### Real-time Queries with Subscriptions

#### Continuous Query Pattern
For real-time updates, combine queries with event subscriptions:

```rust
// Initial query for current state
let current_state = client.request(
    "query.graph.find_nodes",
    query_payload
).await?;

// Subscribe to updates
let mut updates = client.subscribe("event.graph.node_*").await?;

// Process initial results
process_query_results(current_state).await?;

// Handle real-time updates
while let Some(update) = updates.next().await {
    let event: DomainEvent = serde_json::from_slice(&update.payload)?;
    update_local_state(event).await?;
}
```

## Error Handling

### Command Error Patterns

```rust
async fn handle_command_result(result: CommandResult) -> Result<(), ApplicationError> {
    match result {
        CommandResult::Success { event_id, .. } => {
            log::info!("Command succeeded: {}", event_id);
            Ok(())
        }
        CommandResult::ValidationError { errors, .. } => {
            for error in errors {
                log::warn!("Validation error: {}", error.message);
            }
            Err(ApplicationError::ValidationFailed)
        }
        CommandResult::BusinessError { message, retry_allowed, .. } => {
            log::error!("Business rule violation: {}", message);
            if retry_allowed {
                // Implement retry logic
                Ok(())
            } else {
                Err(ApplicationError::BusinessRuleViolation(message))
            }
        }
        CommandResult::ConcurrencyError { expected_version, actual_version } => {
            log::warn!("Concurrency conflict: expected {}, got {}", expected_version, actual_version);
            // Implement optimistic concurrency resolution
            Err(ApplicationError::ConcurrencyConflict)
        }
    }
}
```

### Query Timeout and Retry

```rust
async fn robust_query<T>(
    client: &Client,
    subject: &str,
    payload: Vec<u8>,
    max_retries: u32,
) -> Result<T, QueryError>
where
    T: for<'de> Deserialize<'de>,
{
    let mut attempts = 0;
    
    while attempts < max_retries {
        match client.request(subject, payload.clone().into())
            .timeout(Duration::from_secs(5))
            .await
        {
            Ok(response) => {
                return serde_json::from_slice(&response.payload)
                    .map_err(QueryError::DeserializationFailed);
            }
            Err(_) if attempts < max_retries - 1 => {
                attempts += 1;
                tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
            }
            Err(e) => return Err(QueryError::RequestFailed(e.to_string())),
        }
    }
    
    Err(QueryError::MaxRetriesExceeded)
}
```

## Authentication and Authorization

### Command Authorization

```rust
#[derive(Serialize, Deserialize)]
pub struct AuthorizedCommand {
    pub actor: ActorId,
    pub permissions: Vec<Permission>,
    pub command: Command,
    pub signature: Option<String>,
}

// Commands include actor identification
let authorized_command = AuthorizedCommand {
    actor: "user-123".into(),
    permissions: vec![Permission::GraphWrite("graph-123".into())],
    command: create_node_command,
    signature: None,
};
```

### Query Access Control

```rust
// Queries respect actor permissions automatically
let query_with_auth = FindNodesQuery {
    graph_id: "graph-123".into(),
    // Only returns nodes accessible to the actor
    filters: vec![NodeFilter::ByAccessLevel(AccessLevel::ReadOnly)],
    // ... other parameters
};
```

---

**All commands and queries in CIM use NATS messaging for distributed, scalable, and fault-tolerant operations. This ensures consistent behavior across all system interactions.** 