# Graph Aggregate Implementation Plan

## Immediate Actions Required

### Step 1: Create GraphRepository (TODAY)
```rust
// src/graph/repository.rs
use daggy::Dag;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct GraphRepository {
    /// Primary storage using Daggy
    graphs: HashMap<GraphId, Dag<NodeData, EdgeData>>,
    /// Node ID to Daggy index mapping
    node_indices: HashMap<(GraphId, NodeId), daggy::NodeIndex>,
    /// Edge ID to Daggy index mapping
    edge_indices: HashMap<(GraphId, EdgeId), daggy::EdgeIndex>,
}

impl GraphRepository {
    pub fn create_graph(&mut self, id: GraphId) -> &mut Dag<NodeData, EdgeData> {
        self.graphs.entry(id).or_insert_with(Dag::new)
    }

    pub fn add_node(&mut self, graph_id: GraphId, data: NodeData) -> daggy::NodeIndex {
        let dag = self.graphs.get_mut(&graph_id).unwrap();
        let index = dag.add_node(data.clone());
        self.node_indices.insert((graph_id, data.id), index);
        index
    }
}
```

### Step 2: Enhance Events with Correlation
```rust
// src/graph/events.rs
use std::time::SystemTime;

/// Enhanced base event
#[derive(Debug, Clone)]
pub struct EventMetadata {
    pub event_id: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub timestamp: SystemTime,
    pub sequence: u64,
    pub aggregate_version: u64,
}

/// Macro to enhance existing events
macro_rules! enhanced_event {
    ($name:ident { $($field:ident: $type:ty),* }) => {
        #[derive(Debug, Clone, Event)]
        pub struct $name {
            pub metadata: EventMetadata,
            $(pub $field: $type,)*
        }
    };
}

// Apply to all events
enhanced_event!(GraphCreatedEvent {
    graph_id: GraphId,
    graph_metadata: GraphMetadata
});
```

### Step 3: Add Event Store
```rust
// src/graph/event_store.rs
#[derive(Resource, Default)]
pub struct GraphEventStore {
    /// All events by graph
    events: HashMap<GraphId, Vec<Box<dyn Any + Send + Sync>>>,
    /// Global sequence counter
    sequence: Arc<AtomicU64>,
}

impl GraphEventStore {
    pub fn append<E: Event + 'static>(&mut self, graph_id: GraphId, event: E) -> u64 {
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        self.events.entry(graph_id)
            .or_default()
            .push(Box::new(event));
        seq
    }

    pub fn replay(&self, graph_id: GraphId) -> impl Iterator<Item = &dyn Any> {
        self.events.get(&graph_id)
            .map(|events| events.iter().map(|e| e.as_ref()))
            .into_iter()
            .flatten()
    }
}
```

### Step 4: Create Sync System
```rust
// src/graph/systems.rs
/// Syncs Daggy changes to ECS entities
pub fn sync_daggy_to_ecs(
    mut commands: Commands,
    repository: Res<GraphRepository>,
    mut node_query: Query<(&mut Transform, &GraphNodeRef)>,
    mut edge_query: Query<&GraphEdgeRef>,
) {
    // For each graph in repository
    for (graph_id, dag) in repository.graphs.iter() {
        // Sync nodes
        for node_index in dag.node_indices() {
            let node_data = dag.node_weight(node_index).unwrap();

            // Find or create ECS entity
            if let Some((mut transform, _)) = node_query.iter_mut()
                .find(|(_, ref node_ref)| node_ref.node_index == node_index)
            {
                // Update position
                transform.translation = node_data.position_3d;
            } else {
                // Spawn new entity
                commands.spawn(/* ... */);
            }
        }
    }
}
```

### Step 5: Serialization Traits
```rust
// src/graph/serialization/mod.rs
pub trait GraphFormat {
    type Error: std::error::Error;

    fn export(&self, dag: &Dag<NodeData, EdgeData>) -> Result<String, Self::Error>;
    fn import(&self, content: &str) -> Result<Dag<NodeData, EdgeData>, Self::Error>;
}

// src/graph/serialization/json.rs
#[derive(Serialize, Deserialize)]
struct JsonGraph {
    nodes: Vec<JsonNode>,
    edges: Vec<JsonEdge>,
}

impl GraphFormat for JsonSerializer {
    // ... implementation
}
```

### Step 6: 2D/3D View System
```rust
// src/graph/view.rs
#[derive(Component)]
pub struct GraphView {
    pub mode: ViewMode,
    pub camera_2d: Entity,
    pub camera_3d: Entity,
}

pub fn toggle_view_mode(
    mut view_query: Query<&mut GraphView>,
    mut camera_query: Query<&mut Camera>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        for mut view in view_query.iter_mut() {
            // Switch mode
            view.mode = match view.mode {
                ViewMode::TwoD => ViewMode::ThreeD,
                ViewMode::ThreeD => ViewMode::TwoD,
            };

            // Toggle cameras
            // ... camera switching logic
        }
    }
}
```

## Migration Strategy

### Phase 1: Add Alongside (Don't Break Current Code)
1. Create new `repository.rs`, `event_store.rs` modules
2. Add enhanced events alongside existing ones
3. Create sync system but don't activate yet

### Phase 2: Gradual Migration
1. Update systems to use repository
2. Replace direct component updates with events
3. Add serialization one format at a time

### Phase 3: Remove Old Code
1. Delete old component definitions
2. Remove deprecated event types
3. Clean up unused systems

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_repository() {
        let mut repo = GraphRepository::default();
        let graph_id = GraphId::new();

        // Create graph
        repo.create_graph(graph_id);

        // Add nodes
        let node1 = repo.add_node(graph_id, NodeData::default());
        let node2 = repo.add_node(graph_id, NodeData::default());

        // Add edge
        let edge = repo.add_edge(graph_id, node1, node2, EdgeData::default());

        assert_eq!(repo.node_count(graph_id), 2);
        assert_eq!(repo.edge_count(graph_id), 1);
    }

    #[test]
    fn test_event_replay() {
        let mut store = GraphEventStore::default();
        let graph_id = GraphId::new();

        // Record events
        store.append(graph_id, GraphCreatedEvent { /* ... */ });
        store.append(graph_id, NodeAddedEvent { /* ... */ });

        // Replay
        let events: Vec<_> = store.replay(graph_id).collect();
        assert_eq!(events.len(), 2);
    }
}
```

## Timeline

- **Today**: Start GraphRepository implementation
- **Tomorrow**: Event correlation system
- **Day 3**: Basic sync system
- **Day 4**: JSON serialization
- **Day 5**: 2D view support
- **Week 2**: Complete migration
