# Phase 1.1: Event Sourcing Implementation Guide

## Overview

This guide provides detailed, step-by-step instructions for implementing event sourcing with audit trails. This is the foundation for many other features.

## Pre-Implementation Checklist

- [ ] Understand current event system in `src/contexts/graph_management/events.rs`
- [ ] Review Bevy event patterns in `/samples`
- [ ] Check existing domain types in `src/contexts/graph_management/domain.rs`

## Step 1: Create Event Store Module Structure

### 1.1 Create the module directory
```bash
mkdir -p src/contexts/event_store
```

### 1.2 Create module files
```rust
// src/contexts/event_store/mod.rs
pub mod store;
pub mod events;
pub mod replay;
pub mod persistence;

pub use store::EventStore;
pub use events::{DomainEvent, EventMetadata};
pub use replay::EventReplayer;
```

### 1.3 Update main contexts module
```rust
// src/contexts/mod.rs
pub mod event_store; // Add this line
```

## Step 2: Define Domain Events

### 2.1 Create comprehensive event types
```rust
// src/contexts/event_store/events.rs
use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub user_id: Option<String>,
    pub session_id: Uuid,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub id: Uuid,
    pub timestamp: SystemTime,
    pub sequence: u64,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub metadata: EventMetadata,
}

// Bevy Event wrapper for ECS integration
#[derive(Event, Clone)]
pub struct DomainEventOccurred(pub DomainEvent);
```

### 2.2 Create specific graph events
```rust
// src/contexts/graph_management/events.rs (update existing file)
use super::domain::*;
use crate::contexts::event_store::DomainEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GraphEventPayload {
    NodeAdded {
        node_id: Uuid,
        label: String,
        position: [f32; 3],
        properties: serde_json::Value,
    },
    NodeRemoved {
        node_id: Uuid,
    },
    NodeUpdated {
        node_id: Uuid,
        changes: NodeChanges,
    },
    EdgeAdded {
        edge_id: Uuid,
        source_id: Uuid,
        target_id: Uuid,
        properties: serde_json::Value,
    },
    EdgeRemoved {
        edge_id: Uuid,
    },
    GraphCreated {
        graph_id: Uuid,
        name: String,
    },
    GraphLoaded {
        graph_id: Uuid,
        source: String,
    },
}

impl GraphEventPayload {
    pub fn to_domain_event(self, graph_id: Uuid) -> DomainEvent {
        DomainEvent {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            sequence: 0, // Will be set by EventStore
            aggregate_id: graph_id,
            event_type: self.event_type_name(),
            payload: serde_json::to_value(self).unwrap(),
            metadata: EventMetadata::default(),
        }
    }

    fn event_type_name(&self) -> String {
        match self {
            Self::NodeAdded { .. } => "NodeAdded",
            Self::NodeRemoved { .. } => "NodeRemoved",
            Self::NodeUpdated { .. } => "NodeUpdated",
            Self::EdgeAdded { .. } => "EdgeAdded",
            Self::EdgeRemoved { .. } => "EdgeRemoved",
            Self::GraphCreated { .. } => "GraphCreated",
            Self::GraphLoaded { .. } => "GraphLoaded",
        }.to_string()
    }
}
```

## Step 3: Implement Event Store

### 3.1 Create the event store
```rust
// src/contexts/event_store/store.rs
use bevy::prelude::*;
use std::sync::{Arc, RwLock};
use super::events::DomainEvent;

#[derive(Resource, Clone)]
pub struct EventStore {
    events: Arc<RwLock<Vec<DomainEvent>>>,
    sequence_counter: Arc<RwLock<u64>>,
}

impl EventStore {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            sequence_counter: Arc::new(RwLock::new(0)),
        }
    }

    pub fn append(&self, mut event: DomainEvent) -> Result<(), String> {
        let mut counter = self.sequence_counter.write()
            .map_err(|e| format!("Lock error: {}", e))?;
        *counter += 1;
        event.sequence = *counter;

        let mut events = self.events.write()
            .map_err(|e| format!("Lock error: {}", e))?;
        events.push(event.clone());

        Ok(())
    }

    pub fn get_events(&self, after_sequence: Option<u64>) -> Vec<DomainEvent> {
        let events = self.events.read().unwrap();
        match after_sequence {
            Some(seq) => events.iter()
                .filter(|e| e.sequence > seq)
                .cloned()
                .collect(),
            None => events.clone(),
        }
    }

    pub fn get_events_for_aggregate(&self, aggregate_id: Uuid) -> Vec<DomainEvent> {
        let events = self.events.read().unwrap();
        events.iter()
            .filter(|e| e.aggregate_id == aggregate_id)
            .cloned()
            .collect()
    }
}
```

### 3.2 Create event store plugin
```rust
// src/contexts/event_store/plugin.rs
use bevy::prelude::*;
use super::store::EventStore;
use super::events::DomainEventOccurred;

pub struct EventStorePlugin;

impl Plugin for EventStorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EventStore::new())
           .add_event::<DomainEventOccurred>()
           .add_systems(Update, persist_domain_events);
    }
}

fn persist_domain_events(
    mut events: EventReader<DomainEventOccurred>,
    event_store: Res<EventStore>,
) {
    for event in events.read() {
        if let Err(e) = event_store.append(event.0.clone()) {
            error!("Failed to persist event: {}", e);
        }
    }
}
```

## Step 4: Implement Event Replay

### 4.1 Create replay system
```rust
// src/contexts/event_store/replay.rs
use bevy::prelude::*;
use super::events::DomainEvent;
use crate::contexts::graph_management::{domain::*, events::GraphEventPayload};

pub struct EventReplayer;

impl EventReplayer {
    pub fn replay_to_commands(
        events: Vec<DomainEvent>,
        commands: &mut Commands,
    ) -> Result<(), String> {
        for event in events {
            Self::apply_event(event, commands)?;
        }
        Ok(())
    }

    fn apply_event(
        event: DomainEvent,
        commands: &mut Commands,
    ) -> Result<(), String> {
        let payload: GraphEventPayload = serde_json::from_value(event.payload)
            .map_err(|e| format!("Failed to deserialize event: {}", e))?;

        match payload {
            GraphEventPayload::NodeAdded { node_id, label, position, properties } => {
                let node = Node {
                    identity: NodeIdentity(node_id),
                    graph: GraphIdentity(event.aggregate_id),
                    content: NodeContent {
                        label,
                        category: "default".to_string(),
                        properties: serde_json::from_value(properties).unwrap_or_default(),
                    },
                    position: SpatialPosition::at_3d(position[0], position[1], position[2]),
                };

                commands.spawn(NodeBundle::from(node));
            },
            GraphEventPayload::NodeRemoved { node_id } => {
                // Would need to query for entity with this node_id and despawn it
                // This requires a more complex implementation with entity tracking
            },
            // Handle other event types...
            _ => {}
        }

        Ok(())
    }
}
```

## Step 5: Integrate with Existing Systems

### 5.1 Update graph management to emit events
```rust
// src/contexts/graph_management/services.rs (update existing)
use crate::contexts::event_store::{DomainEvent, DomainEventOccurred};

pub fn create_node_with_event(
    position: Vec3,
    label: String,
    graph_id: GraphIdentity,
    mut commands: Commands,
    mut domain_events: EventWriter<DomainEventOccurred>,
) {
    let node_id = NodeIdentity::new();

    // Create the node
    let node = Node {
        identity: node_id,
        graph: graph_id,
        content: NodeContent {
            label: label.clone(),
            category: "default".to_string(),
            properties: Default::default(),
        },
        position: SpatialPosition::from(position),
    };

    commands.spawn(NodeBundle::from(node));

    // Emit domain event
    let event_payload = GraphEventPayload::NodeAdded {
        node_id: node_id.0,
        label,
        position: [position.x, position.y, position.z],
        properties: serde_json::Value::Object(Default::default()),
    };

    let domain_event = event_payload.to_domain_event(graph_id.0);
    domain_events.send(DomainEventOccurred(domain_event));
}
```

### 5.2 Add event store to main app
```rust
// src/main.rs (update existing)
use contexts::event_store::plugin::EventStorePlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(EventStorePlugin) // Add this
        .add_plugins((
            contexts::graph_management::plugin::GraphManagementPlugin,
            contexts::visualization::plugin::VisualizationPlugin,
            contexts::selection::plugin::SelectionPlugin,
        ))
        .add_systems(Startup, setup_test_graph);

    app.run();
}
```

## Step 6: Update Tests

### 6.1 Update failing tests to pass
```rust
// src/testing/feature_tests.rs (update)
#[cfg(test)]
mod event_driven_architecture_tests {
    use super::*;
    use crate::contexts::event_store::EventStore;

    #[test]
    fn test_event_audit_trail() {
        // Now this should pass!
        let event_store = EventStore::new();

        // Create a test event
        let event = DomainEvent {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            sequence: 0,
            aggregate_id: Uuid::new_v4(),
            event_type: "TestEvent".to_string(),
            payload: serde_json::json!({"test": true}),
            metadata: EventMetadata::default(),
        };

        // Store it
        event_store.append(event.clone()).unwrap();

        // Verify it's in the audit trail
        let events = event_store.get_events(None);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, event.id);
    }
}
```

## Step 7: Add Persistence (Optional but recommended)

### 7.1 Create persistence layer
```rust
// src/contexts/event_store/persistence.rs
use super::events::DomainEvent;
use std::fs;
use std::path::Path;

pub struct EventPersistence;

impl EventPersistence {
    pub fn save_to_file(events: &[DomainEvent], path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(events)
            .map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(path, json)
            .map_err(|e| format!("File write error: {}", e))?;
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Vec<DomainEvent>, String> {
        let json = fs::read_to_string(path)
            .map_err(|e| format!("File read error: {}", e))?;
        serde_json::from_str(&json)
            .map_err(|e| format!("Deserialization error: {}", e))
    }
}
```

## Verification Steps

1. Run the specific test that should now pass:
   ```bash
   BEVY_HEADLESS=1 cargo test test_event_audit_trail
   ```

2. Check that events are being generated:
   - Add debug logging to `persist_domain_events`
   - Run the app and perform actions
   - Verify events appear in logs

3. Test replay functionality:
   - Save some events
   - Clear the world
   - Replay events
   - Verify state is restored

## Next Steps

Once this is working:
1. Update the other event-related tests
2. Add UI to view event history
3. Implement event filtering and querying
4. Add event versioning support
5. Move to Phase 1.2 (File I/O improvements)
