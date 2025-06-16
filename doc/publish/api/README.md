# CIM API Documentation

## NATS-First Communication Architecture

**CIM (Composable Information Machine) is designed from the ground up as a NATS-native system.** All communication with CIM occurs through NATS messaging patterns - there are no traditional REST endpoints or GraphQL schemas. This approach provides:

- **Event-Driven Architecture**: All state changes flow through immutable events
- **Distributed by Design**: Natural scaling across multiple nodes
- **Real-Time Collaboration**: Instant updates across all connected clients
- **Fault Tolerance**: Built-in retries, dead letter queues, and circuit breakers
- **Security**: NATS authentication, authorization, and TLS encryption

## API Structure

### 🎯 **Communication Patterns**

#### **Commands** → NATS Subjects: `cmd.{domain}.{action}`
Send commands to modify state and trigger business processes.

#### **Events** → NATS Subjects: `event.{domain}.{event_type}`
Subscribe to domain events to react to state changes.

#### **Queries** → NATS Subjects: `query.{domain}.{query_type}`
Request-reply pattern for data retrieval and projections.

#### **Streams** → NATS JetStream: `stream.{domain}`
Persistent event streams with replay capabilities.

## API Documentation

### 📋 [Domain Events](domain-events.md)
**Core Event Catalog for NATS Subscriptions**
- Graph domain events (nodes, edges, layouts)
- Conceptual space events (embeddings, categories)
- Workflow events (steps, decisions, flows)
- Agent events (registration, capabilities, interactions)
- Complete event schemas with examples

### ⚡ [Commands & Queries](commands-queries.md)
**CQRS Operations over NATS**
- Command patterns and validation
- Query request-reply patterns
- Batch operations and transactions
- Error handling and retries
- Authentication and authorization

### 🔄 [Graph Operations](graph-operations.md)
**Graph Manipulation via NATS**
- Node creation, updates, deletion
- Edge management and relationships
- Layout algorithms and positioning
- Subgraph operations and composition
- Real-time collaboration patterns

### 🧠 [Conceptual Spaces](conceptual-spaces.md)
**Semantic Operations and AI Integration**
- Embedding calculations and storage
- Similarity searches and clustering
- Category formation and boundaries
- Knowledge graph navigation
- AI agent interaction patterns

### 🌊 [Event Streaming](event-streaming.md)
**NATS JetStream Integration**
- Stream configuration and policies
- Event replay and time travel
- Snapshot creation and restoration
- Performance optimization
- Monitoring and observability

## Quick Start

### 1. Connect to NATS
```rust
use async_nats::Client;

let client = async_nats::connect("nats://localhost:4222").await?;
```

### 2. Send a Command
```rust
// Create a new graph node
let command = CreateNodeCommand {
    graph_id: "graph-123".into(),
    node_type: NodeType::Concept { 
        name: "Machine Learning".into(),
        description: "AI technique for pattern recognition".into()
    },
    position: Position3D::new(0.0, 0.0, 0.0),
};

client.publish(
    "cmd.graph.create_node",
    serde_json::to_vec(&command)?.into()
).await?;
```

### 3. Subscribe to Events
```rust
let mut subscriber = client.subscribe("event.graph.>").await?;

while let Some(message) = subscriber.next().await {
    let event: GraphEvent = serde_json::from_slice(&message.payload)?;
    println!("Received event: {:?}", event);
}
```

### 4. Query Data
```rust
// Request-reply pattern for queries
let response = client.request(
    "query.graph.nodes_by_type",
    serde_json::to_vec(&NodeTypeQuery { 
        node_type: NodeType::Concept 
    })?.into()
).await?;

let nodes: Vec<Node> = serde_json::from_slice(&response.payload)?;
```

## NATS Configuration

### Connection Options
```rust
let options = async_nats::ConnectOptions::new()
    .credentials_file("path/to/creds")  // Authentication
    .tls_required(true)                 // Secure connection
    .max_reconnects(5)                  // Fault tolerance
    .reconnect_delay_callback(|attempts| {
        std::time::Duration::from_millis(attempts * 100)
    });

let client = options.connect("nats://nats.example.com:4222").await?;
```

### JetStream Setup
```rust
let jetstream = async_nats::jetstream::new(client);

// Create persistent streams
jetstream.create_stream(async_nats::jetstream::stream::Config {
    name: "CIM_EVENTS".to_string(),
    subjects: vec!["event.>".to_string()],
    retention: async_nats::jetstream::stream::RetentionPolicy::Limits,
    storage: async_nats::jetstream::stream::StorageType::File,
    max_age: std::time::Duration::from_secs(86400 * 365), // 1 year
    ..Default::default()
}).await?;
```

## Subject Naming Conventions

### Command Subjects
```
cmd.graph.create_node          # Create new graph node
cmd.graph.update_node          # Update existing node
cmd.graph.delete_node          # Remove node
cmd.graph.connect_nodes        # Create edge between nodes
cmd.workflow.start_process     # Begin workflow execution
cmd.agent.register             # Register new AI agent
```

### Event Subjects
```
event.graph.node_created       # Node was created
event.graph.node_updated       # Node was modified
event.graph.edge_added         # New edge established
event.workflow.step_completed  # Workflow step finished
event.agent.capability_changed # Agent updated capabilities
```

### Query Subjects
```
query.graph.find_nodes         # Search for nodes
query.graph.get_neighbors      # Get connected nodes
query.workflow.get_status      # Check workflow state
query.agent.list_capabilities  # Get agent features
```

## Error Handling

### Command Validation Errors
```rust
// Commands publish validation results to response subjects
let response = client.request(
    "cmd.graph.create_node",
    command_payload
).timeout(Duration::from_secs(5)).await?;

match serde_json::from_slice::<CommandResult>(&response.payload)? {
    CommandResult::Success { event_id } => {
        println!("Command succeeded: {}", event_id);
    }
    CommandResult::ValidationError { errors } => {
        eprintln!("Validation failed: {:?}", errors);
    }
    CommandResult::BusinessError { message } => {
        eprintln!("Business rule violation: {}", message);
    }
}
```

### Event Stream Error Handling
```rust
// Dead letter queue for failed event processing
let mut dlq_subscriber = client.subscribe("event.dlq.>").await?;

while let Some(failed_message) = dlq_subscriber.next().await {
    let error_info: ProcessingError = serde_json::from_slice(&failed_message.payload)?;
    log::error!("Event processing failed: {:?}", error_info);
    
    // Implement retry logic or manual intervention
}
```

## Security and Authentication

### Credentials-Based Authentication
```rust
// Use NATS credentials file for secure access
let client = async_nats::ConnectOptions::new()
    .credentials_file("/path/to/cim.creds")
    .connect("nats://secure.cim.example.com:4222").await?;
```

### Subject-Level Permissions
```
# Example NATS authorization
authorization: {
  users: [
    {
      user: "graph_admin"
      password: "secure_password"
      permissions: {
        publish: ["cmd.graph.>", "cmd.workflow.>"]
        subscribe: ["event.>", "query.>"]
      }
    }
    {
      user: "readonly_user"
      password: "readonly_password"
      permissions: {
        subscribe: ["event.>", "query.>"]
      }
    }
  ]
}
```

## Performance Considerations

### Batch Operations
```rust
// Use NATS request-many for bulk operations
let batch_command = BatchCreateNodesCommand {
    graph_id: "graph-123".into(),
    nodes: vec![/* ... nodes ... */],
};

client.publish("cmd.graph.batch_create_nodes", payload).await?;
```

### Stream Processing
```rust
// Use JetStream consumers for high-throughput processing
let consumer = jetstream.create_consumer_on_stream(
    async_nats::jetstream::consumer::pull::Config {
        name: Some("graph_processor".to_string()),
        filter_subjects: vec!["event.graph.>".to_string()],
        max_ack_pending: 1000,
        ..Default::default()
    },
    "CIM_EVENTS"
).await?;

// Process events in batches
let mut messages = consumer.batch().max_messages(100).messages().await?;
while let Some(message) = messages.next().await {
    // Process event
    message.ack().await?;
}
```

## Monitoring and Observability

### Built-in Metrics
CIM publishes operational metrics to NATS subjects:

```
metrics.graph.node_count       # Current number of nodes
metrics.graph.edge_count       # Current number of edges  
metrics.workflow.active_count  # Running workflows
metrics.agent.connected_count  # Active AI agents
metrics.system.memory_usage    # System resource usage
```

### Health Checks
```rust
// System health via NATS request-reply
let health = client.request("query.system.health", "".into()).await?;
let status: SystemHealth = serde_json::from_slice(&health.payload)?;

println!("System status: {:?}", status);
```

## Next Steps

1. **[Review Domain Events](domain-events.md)** - Understand the event catalog
2. **[Explore Commands & Queries](commands-queries.md)** - Learn CQRS patterns
3. **[Try Graph Operations](graph-operations.md)** - Start building graphs
4. **[Integrate Conceptual Spaces](conceptual-spaces.md)** - Add semantic intelligence
5. **[Set up Event Streaming](event-streaming.md)** - Configure persistent streams

---

**Remember: CIM is NATS-native. Every interaction uses NATS messaging patterns for maximum scalability, reliability, and real-time collaboration.** 