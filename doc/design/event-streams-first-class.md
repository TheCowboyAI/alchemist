# Event Streams as First-Class Objects

## Overview

Event streams in CIM should be treated as first-class domain objects that can be:
- Queried and filtered from the event store
- Saved as named streams for reuse
- Visualized as ContextGraphs
- Composed and transformed
- Analyzed for patterns and insights

## Core Concepts

### EventStream as Domain Object

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStream {
    pub id: EventStreamId,
    pub name: String,
    pub description: String,
    pub query: EventQuery,
    pub events: Vec<StoredEvent>,
    pub metadata: EventStreamMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStreamMetadata {
    pub event_count: usize,
    pub time_range: TimeRange,
    pub aggregate_types: HashSet<String>,
    pub correlation_ids: HashSet<CorrelationId>,
    pub causation_chains: Vec<CausationChain>,
    pub cid_root: Option<Cid>,
}
```

### Event Queries

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventQuery {
    /// Get all events for a specific correlation ID
    ByCorrelationId {
        correlation_id: CorrelationId,
        order: CausationOrder,
    },

    /// Get events within a time range
    ByTimeRange {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        aggregates: Option<Vec<AggregateId>>,
    },

    /// Get events by aggregate type and criteria
    ByAggregateType {
        aggregate_type: String,
        criteria: HashMap<String, Value>,
    },

    /// Complex query with multiple filters
    Complex {
        filters: Vec<EventFilter>,
        ordering: EventOrdering,
        limit: Option<usize>,
    },

    /// Get events that form a specific workflow execution
    WorkflowExecution {
        workflow_id: WorkflowId,
        instance_id: WorkflowInstanceId,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CausationOrder {
    /// Order by causation chain (cause before effect)
    Causal,
    /// Order by timestamp
    Temporal,
    /// Order by aggregate and sequence
    AggregateSequence,
}
```

### EventStream Operations

```rust
pub trait EventStreamOperations {
    /// Create a new event stream from a query
    async fn create_stream(
        &self,
        name: String,
        query: EventQuery,
    ) -> Result<EventStream, EventStreamError>;

    /// Transform an event stream
    async fn transform_stream(
        &self,
        stream: &EventStream,
        transformation: StreamTransformation,
    ) -> Result<EventStream, EventStreamError>;

    /// Compose multiple streams
    async fn compose_streams(
        &self,
        streams: Vec<EventStream>,
        composition: StreamComposition,
    ) -> Result<EventStream, EventStreamError>;

    /// Convert to ContextGraph for visualization
    async fn to_context_graph(
        &self,
        stream: &EventStream,
        layout: GraphLayout,
    ) -> Result<ContextGraph, ConversionError>;
}

#[derive(Debug, Clone)]
pub enum StreamTransformation {
    /// Filter events
    Filter(EventFilter),

    /// Map events to different representation
    Map(Box<dyn Fn(&StoredEvent) -> Option<StoredEvent>>),

    /// Group events by criteria
    GroupBy(GroupingCriteria),

    /// Window events by time or count
    Window(WindowSpec),
}

#[derive(Debug, Clone)]
pub enum StreamComposition {
    /// Union of all events
    Union,

    /// Intersection of events
    Intersection,

    /// Events in first stream but not in others
    Difference,

    /// Merge with conflict resolution
    Merge(ConflictResolution),
}
```

## Integration with ContextGraph

### EventStream to ContextGraph Conversion

```rust
impl EventStream {
    pub fn to_context_graph(&self) -> ContextGraphBuilder {
        ContextGraphBuilder::new()
            .with_name(format!("EventStream: {}", self.name))
            .with_nodes(self.create_nodes())
            .with_edges(self.create_edges())
            .with_layout(self.determine_layout())
    }

    fn create_nodes(&self) -> Vec<GraphNode> {
        let mut nodes = Vec::new();

        // Create nodes for aggregates
        for event in &self.events {
            nodes.push(GraphNode {
                id: NodeId::from(event.aggregate_id.clone()),
                node_type: NodeType::Aggregate {
                    aggregate_type: event.aggregate_type.clone(),
                },
                position: self.calculate_position(&event),
                metadata: self.extract_metadata(&event),
            });
        }

        // Create nodes for events
        for event in &self.events {
            nodes.push(GraphNode {
                id: NodeId::from(event.event_id.clone()),
                node_type: NodeType::Event {
                    event_type: event.event_type.clone(),
                },
                position: self.calculate_event_position(&event),
                metadata: event.metadata.clone(),
            });
        }

        nodes
    }

    fn create_edges(&self) -> Vec<GraphEdge> {
        let mut edges = Vec::new();

        // Causation edges
        for event in &self.events {
            if let Some(causation_id) = &event.causation_id {
                edges.push(GraphEdge {
                    id: EdgeId::new(),
                    source: NodeId::from(causation_id.clone()),
                    target: NodeId::from(event.event_id.clone()),
                    edge_type: EdgeType::Causation,
                    metadata: hashmap!{
                        "relationship" => "caused_by".to_string(),
                    },
                });
            }
        }

        // Correlation edges
        let correlation_groups = self.group_by_correlation();
        for (correlation_id, events) in correlation_groups {
            for window in events.windows(2) {
                edges.push(GraphEdge {
                    id: EdgeId::new(),
                    source: NodeId::from(window[0].event_id.clone()),
                    target: NodeId::from(window[1].event_id.clone()),
                    edge_type: EdgeType::Correlation,
                    metadata: hashmap!{
                        "correlation_id" => correlation_id.to_string(),
                    },
                });
            }
        }

        edges
    }
}
```

### Visualization Layouts

```rust
#[derive(Debug, Clone)]
pub enum GraphLayout {
    /// Timeline layout - events positioned by time
    Timeline {
        time_scale: TimeScale,
        aggregate_lanes: bool,
    },

    /// Causation tree - hierarchical by causation
    CausationTree {
        root_events: Vec<EventId>,
        branch_spacing: f32,
    },

    /// Aggregate-centric - events grouped by aggregate
    AggregateCentric {
        aggregate_spacing: f32,
        event_orbit_radius: f32,
    },

    /// Force-directed - based on relationships
    ForceDirected {
        causation_strength: f32,
        correlation_strength: f32,
        temporal_strength: f32,
    },

    /// Workflow - showing process flow
    WorkflowLayout {
        step_spacing: f32,
        parallel_offset: f32,
    },
}
```

## Use Cases

### 1. Debugging Distributed Transactions

```rust
// Get all events related to a failed order
let order_stream = event_store.create_stream(
    "Failed Order Investigation".to_string(),
    EventQuery::ByCorrelationId {
        correlation_id: order_correlation_id,
        order: CausationOrder::Causal,
    },
).await?;

// Visualize as causation tree
let graph = order_stream.to_context_graph()
    .with_layout(GraphLayout::CausationTree {
        root_events: vec![order_created_event_id],
        branch_spacing: 100.0,
    })
    .build();
```

### 2. Workflow Analysis

```rust
// Get all events for a workflow execution
let workflow_stream = event_store.create_stream(
    "Approval Workflow Analysis".to_string(),
    EventQuery::WorkflowExecution {
        workflow_id: approval_workflow_id,
        instance_id: instance_id,
    },
).await?;

// Transform to show only state transitions
let transition_stream = event_store.transform_stream(
    &workflow_stream,
    StreamTransformation::Filter(EventFilter::EventType("StateTransition")),
).await?;
```

### 3. Performance Analysis

```rust
// Get events within a time window
let perf_stream = event_store.create_stream(
    "Performance Spike Analysis".to_string(),
    EventQuery::ByTimeRange {
        start: spike_start,
        end: spike_end,
        aggregates: None,
    },
).await?;

// Group by aggregate type to find hotspots
let grouped_stream = event_store.transform_stream(
    &perf_stream,
    StreamTransformation::GroupBy(GroupingCriteria::AggregateType),
).await?;
```

### 4. Audit Trail

```rust
// Get all events that modified a specific entity
let audit_stream = event_store.create_stream(
    "Customer Data Audit".to_string(),
    EventQuery::Complex {
        filters: vec![
            EventFilter::AggregateId(customer_id),
            EventFilter::EventTypes(vec!["DataModified", "PermissionChanged"]),
        ],
        ordering: EventOrdering::Temporal,
        limit: None,
    },
).await?;
```

## Implementation Plan

### Phase 1: Core EventStream Model
- Define EventStream domain model
- Implement EventQuery types
- Create basic stream creation from queries

### Phase 2: Stream Operations
- Implement transformation operations
- Add composition capabilities
- Create stream persistence

### Phase 3: ContextGraph Integration
- Implement to_context_graph conversion
- Create layout algorithms
- Add interactive visualization

### Phase 4: Advanced Features
- Add stream versioning
- Implement stream sharing/collaboration
- Create stream templates for common queries

## Benefits

1. **Debugging**: Easily trace through complex event chains
2. **Analysis**: Identify patterns and bottlenecks
3. **Compliance**: Create audit trails and compliance reports
4. **Visualization**: See event flows as graphs
5. **Reusability**: Save and share important event queries
6. **Composition**: Build complex analyses from simple streams
