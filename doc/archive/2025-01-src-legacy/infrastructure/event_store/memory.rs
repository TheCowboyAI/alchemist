//! In-memory event store for testing and development

use crate::domain::events::DomainEvent;
use crate::infrastructure::event_store::{EventStore, EventStoreError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// In-memory event store implementation
#[derive(Clone)]
pub struct InMemoryEventStore {
    events: Arc<RwLock<HashMap<Uuid, Vec<DomainEvent>>>>,
}

impl InMemoryEventStore {
    /// Create a new in-memory event store
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store an event with an aggregate ID
    pub async fn store_with_aggregate(
        &self,
        event: DomainEvent,
        aggregate_id: Uuid,
    ) -> Result<(), EventStoreError> {
        let mut events = self.events.write().await;
        events
            .entry(aggregate_id)
            .or_insert_with(Vec::new)
            .push(event);
        Ok(())
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn store(&self, event: DomainEvent) -> Result<(), EventStoreError> {
        // For the trait implementation, we'll use a default aggregate ID
        // In real usage, you should use store_with_aggregate
        let aggregate_id = Uuid::new_v4();
        self.store_with_aggregate(event, aggregate_id).await
    }

    async fn load_events(&self, aggregate_id: Uuid) -> Result<Vec<DomainEvent>, EventStoreError> {
        let events = self.events.read().await;
        Ok(events.get(&aggregate_id).cloned().unwrap_or_default())
    }

    async fn load_all_events(&self) -> Result<Vec<DomainEvent>, EventStoreError> {
        let events = self.events.read().await;
        Ok(events.values().flatten().cloned().collect())
    }

    async fn append_events(
        &self,
        aggregate_id: String,
        events: Vec<DomainEvent>,
    ) -> Result<(), EventStoreError> {
        let aggregate_uuid = Uuid::parse_str(&aggregate_id)
            .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;

        let mut store = self.events.write().await;
        let entry = store.entry(aggregate_uuid).or_insert_with(Vec::new);
        entry.extend(events);
        Ok(())
    }

    async fn get_events(&self, aggregate_id: String) -> Result<Vec<DomainEvent>, EventStoreError> {
        let aggregate_uuid = Uuid::parse_str(&aggregate_id)
            .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;

        self.load_events(aggregate_uuid).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::events::GraphEvent;
    use crate::domain::value_objects::{GraphId, GraphMetadata};

    #[tokio::test]
    async fn test_store_and_load_events() {
        let store = InMemoryEventStore::new();
        let aggregate_id = Uuid::new_v4();

        // Create a test event
        let graph_event = GraphEvent::GraphCreated {
            id: GraphId::new(),
            metadata: GraphMetadata::default(),
        };
        let event = DomainEvent::Graph(graph_event);

        // Store the event
        store
            .store_with_aggregate(event.clone(), aggregate_id)
            .await
            .unwrap();

        // Load events
        let loaded = store.load_events(aggregate_id).await.unwrap();
        assert_eq!(loaded.len(), 1);
        // Check that the event is the same type
        match (&loaded[0], &event) {
            (DomainEvent::Graph(loaded_event), DomainEvent::Graph(original_event)) => {
                // For testing purposes, we can check specific fields
                match (loaded_event, original_event) {
                    (
                        GraphEvent::GraphCreated { id: id1, .. },
                        GraphEvent::GraphCreated { id: id2, .. },
                    ) => {
                        assert_eq!(id1, id2);
                    }
                    _ => panic!("Event types don't match"),
                }
            }
            _ => panic!("Event types don't match"),
        }
    }
}
