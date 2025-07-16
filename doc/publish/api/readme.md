# CIM API Reference

The Composable Information Machine provides a consistent event-driven API across all domains.

## API Documentation

### Core APIs
- **[Event API](./events.md)** - Domain event catalog and schemas
- **[Command API](./commands.md)** - Command patterns and handlers
- **[Query API](./queries.md)** - Query patterns and projections
- **[NATS Integration](./nats.md)** - Messaging patterns and subjects

### Domain-Specific APIs
- **[Graph Operations](./graph-operations.md)** - Graph manipulation APIs
- **[Workflow API](./workflow-api.md)** - Process automation APIs
- **[Conceptual Space API](./conceptual-spaces.md)** - Semantic operations

## API Patterns

### Command Pattern
Commands express intent to change state:

```rust
// Command structure
pub struct CreateNode {
    pub graph_id: GraphId,
    pub node_type: NodeType,
    pub position: Position3D,
    pub metadata: HashMap<String, Value>,
}

// Send via NATS
client.publish("cmd.graph.create_node", command).await?;
```

### Event Pattern
Events record what happened:

```rust
// Event structure
pub struct NodeCreated {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub node_type: NodeType,
    pub position: Position3D,
    pub timestamp: SystemTime,
}

// Subscribe to events
let mut sub = client.subscribe("event.graph.node_created").await?;
```

### Query Pattern
Queries retrieve current state:

```rust
// Query structure
pub struct FindNodesInRadius {
    pub center: Position3D,
    pub radius: f32,
}

// Request-reply pattern
let response = client.request(
    "query.graph.find_nodes_in_radius",
    query
).await?;
```

## NATS Subject Hierarchy

```
cmd.{domain}.{action}
event.{domain}.{aggregate}_{event}
query.{domain}.{query_type}
stream.{domain}.{stream_type}
```

### Examples
- `cmd.graph.create_node`
- `event.graph.node_created`
- `query.graph.find_nodes`
- `stream.graph.updates`

## Authentication & Authorization

All API calls require authentication:

1. **JWT Token**: Include in NATS connection
2. **Subject Permissions**: Based on user roles
3. **Domain Access**: Per-domain authorization

## Error Handling

Consistent error responses across all APIs:

```rust
#[derive(Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
    pub correlation_id: Uuid,
}
```

## Rate Limiting

- **Commands**: 100/minute per user
- **Queries**: 1000/minute per user
- **Event Subscriptions**: Unlimited
- **Stream Consumption**: Based on consumer group

## Quick Start

1. **Connect to NATS**
   ```rust
   let client = async_nats::connect("nats://localhost:4222").await?;
   ```

2. **Send a Command**
   ```rust
   client.publish("cmd.graph.create_node", command).await?;
   ```

3. **Subscribe to Events**
   ```rust
   let mut sub = client.subscribe("event.graph.>").await?;
   ```

4. **Query Data**
   ```rust
   let response = client.request("query.graph.get_node", id).await?;
   ```

For detailed examples, see the [API Examples](./examples/) directory. 