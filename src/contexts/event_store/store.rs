use super::events::{Cid, DomainEvent, EventPayload};
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Object store for content-addressed storage of event payloads
#[derive(Clone, Default)]
pub struct ObjectStore {
    /// Map from CID to serialized content
    pub(super) objects: Arc<RwLock<HashMap<Cid, Vec<u8>>>>,
}

impl ObjectStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Store an object and return its CID
    pub fn put(&self, content: &[u8]) -> Result<Cid, String> {
        let cid = Cid::from_content(content);
        let mut objects = self
            .objects
            .write()
            .map_err(|e| format!("Lock error: {e}"))?;
        objects.insert(cid.clone(), content.to_vec());
        Ok(cid)
    }

    /// Retrieve an object by CID
    pub fn get(&self, cid: &Cid) -> Result<Option<Vec<u8>>, String> {
        let objects = self
            .objects
            .read()
            .map_err(|e| format!("Lock error: {e}"))?;
        Ok(objects.get(cid).cloned())
    }

    /// Store an event payload and return its CID
    pub fn put_payload(&self, payload: &EventPayload) -> Result<Cid, String> {
        let bytes = serde_json::to_vec(payload).map_err(|e| format!("Serialization error: {e}"))?;
        self.put(&bytes)
    }

    /// Retrieve an event payload by CID
    pub fn get_payload(&self, cid: &Cid) -> Result<Option<EventPayload>, String> {
        match self.get(cid)? {
            Some(bytes) => {
                let payload = serde_json::from_slice(&bytes)
                    .map_err(|e| format!("Deserialization error: {e}"))?;
                Ok(Some(payload))
            }
            None => Ok(None),
        }
    }
}

/// Local event store maintaining a Merkle DAG of events
#[derive(Resource, Clone)]
pub struct EventStore {
    /// The Merkle DAG of events indexed by their CID
    pub(super) events: Arc<RwLock<HashMap<Cid, DomainEvent>>>,
    /// Index from aggregate ID to event CIDs
    pub(super) aggregate_index: Arc<RwLock<HashMap<Uuid, Vec<Cid>>>>,
    /// Sequence counter for local ordering
    pub(super) sequence_counter: Arc<RwLock<u64>>,
    /// Object store for event payloads
    pub(super) object_store: ObjectStore,
    /// The latest event CIDs (heads of the DAG)
    pub(super) heads: Arc<RwLock<Vec<Cid>>>,
}

impl Default for EventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl EventStore {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::new())),
            aggregate_index: Arc::new(RwLock::new(HashMap::new())),
            sequence_counter: Arc::new(RwLock::new(0)),
            object_store: ObjectStore::new(),
            heads: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Store a payload and create an event for it
    pub fn append_with_payload(
        &self,
        aggregate_id: Uuid,
        event_type: String,
        payload: EventPayload,
    ) -> Result<DomainEvent, String> {
        // Store the payload in object store
        let payload_cid = self.object_store.put_payload(&payload)?;

        // Get current heads as parent CIDs
        let parent_cids = self.get_heads()?;

        // Create the event
        let mut event = DomainEvent::new(aggregate_id, event_type, payload_cid, parent_cids);

        // Set sequence number
        let mut counter = self
            .sequence_counter
            .write()
            .map_err(|e| format!("Lock error: {e}"))?;
        *counter += 1;
        event.sequence = *counter;

        // Recompute CID with sequence number
        event.compute_cid();

        let event_cid = event.cid().ok_or("Event CID not computed")?.clone();

        // Store the event
        let mut events = self
            .events
            .write()
            .map_err(|e| format!("Lock error: {e}"))?;
        events.insert(event_cid.clone(), event.clone());

        // Update aggregate index
        let mut index = self
            .aggregate_index
            .write()
            .map_err(|e| format!("Lock error: {e}"))?;
        index
            .entry(aggregate_id)
            .or_insert_with(Vec::new)
            .push(event_cid.clone());

        // Update heads
        self.update_heads(event_cid)?;

        Ok(event)
    }

    /// Get the current heads of the DAG
    pub fn get_heads(&self) -> Result<Vec<Cid>, String> {
        let heads = self.heads.read().map_err(|e| format!("Lock error: {e}"))?;
        Ok(heads.clone())
    }

    /// Update the heads after adding a new event
    fn update_heads(&self, new_cid: Cid) -> Result<(), String> {
        let mut heads = self.heads.write().map_err(|e| format!("Lock error: {e}"))?;

        // For now, just replace all heads with the new event
        // In a more sophisticated implementation, we'd maintain multiple heads
        heads.clear();
        heads.push(new_cid);

        Ok(())
    }

    /// Get all events for an aggregate
    pub fn get_events_for_aggregate(&self, aggregate_id: Uuid) -> Result<Vec<DomainEvent>, String> {
        let index = self
            .aggregate_index
            .read()
            .map_err(|e| format!("Lock error: {e}"))?;

        let cids = match index.get(&aggregate_id) {
            Some(cids) => cids.clone(),
            None => return Ok(Vec::new()),
        };

        let events = self.events.read().map_err(|e| format!("Lock error: {e}"))?;

        let mut result = Vec::new();
        for cid in cids {
            if let Some(event) = events.get(&cid) {
                result.push(event.clone());
            }
        }

        // Sort by sequence number
        result.sort_by_key(|e| e.sequence);

        Ok(result)
    }

    /// Get an event by its CID
    pub fn get_event(&self, cid: &Cid) -> Result<Option<DomainEvent>, String> {
        let events = self.events.read().map_err(|e| format!("Lock error: {e}"))?;
        Ok(events.get(cid).cloned())
    }

    /// Get the payload for an event
    pub fn get_event_payload(&self, event: &DomainEvent) -> Result<Option<EventPayload>, String> {
        self.object_store.get_payload(&event.payload_cid)
    }

    /// Traverse the DAG from a given event CID
    pub fn traverse_from(
        &self,
        start_cid: &Cid,
        max_depth: usize,
    ) -> Result<Vec<DomainEvent>, String> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![(start_cid.clone(), 0)];

        let events = self.events.read().map_err(|e| format!("Lock error: {e}"))?;

        while let Some((cid, depth)) = queue.pop() {
            if depth > max_depth || visited.contains(&cid) {
                continue;
            }

            visited.insert(cid.clone());

            if let Some(event) = events.get(&cid) {
                result.push(event.clone());

                // Add parent events to queue
                for parent_cid in &event.parent_cids {
                    queue.push((parent_cid.clone(), depth + 1));
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::event_store::events::EventPayload;
    use std::time::SystemTime;

    #[test]
    fn test_event_store_basic_operations() {
        let store = EventStore::new();
        let aggregate_id = Uuid::new_v4();

        // Create a test payload
        let payload = EventPayload {
            data: serde_json::json!({
                "test": true,
                "value": 42
            }),
            created_at: SystemTime::now(),
        };

        // Append an event
        let event = store
            .append_with_payload(aggregate_id, "TestEvent".to_string(), payload)
            .unwrap();

        // Verify the event was created correctly
        assert_eq!(event.aggregate_id, aggregate_id);
        assert_eq!(event.event_type, "TestEvent");
        assert_eq!(event.sequence, 1);
        assert!(event.event_cid.is_some());

        // Verify we can get events for the aggregate
        let events = store.get_events_for_aggregate(aggregate_id).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, event.id);

        // Add another event
        let payload2 = EventPayload {
            data: serde_json::json!({
                "test": true,
                "value": 84
            }),
            created_at: SystemTime::now(),
        };

        let event2 = store
            .append_with_payload(aggregate_id, "TestEvent2".to_string(), payload2)
            .unwrap();

        // Verify sequence increments
        assert_eq!(event2.sequence, 2);

        // Verify parent linking
        assert_eq!(event2.parent_cids.len(), 1);
        assert_eq!(event2.parent_cids[0], event.event_cid.unwrap());
    }

    #[test]
    fn test_merkle_dag_structure() {
        // Arrange
        let store = EventStore::new();
        let aggregate_id = Uuid::new_v4();

        // Act - Create chain of events
        let event1 = store
            .append_with_payload(
                aggregate_id,
                "Event1".to_string(),
                EventPayload {
                    data: serde_json::json!({"index": 1}),
                    created_at: SystemTime::now(),
                },
            )
            .unwrap();

        let event2 = store
            .append_with_payload(
                aggregate_id,
                "Event2".to_string(),
                EventPayload {
                    data: serde_json::json!({"index": 2}),
                    created_at: SystemTime::now(),
                },
            )
            .unwrap();

        let event3 = store
            .append_with_payload(
                aggregate_id,
                "Event3".to_string(),
                EventPayload {
                    data: serde_json::json!({"index": 3}),
                    created_at: SystemTime::now(),
                },
            )
            .unwrap();

        // Assert - Verify parent linking
        assert_eq!(
            event1.parent_cids.len(),
            0,
            "First event should have no parents"
        );
        assert_eq!(
            event2.parent_cids.len(),
            1,
            "Second event should have one parent"
        );
        assert_eq!(event2.parent_cids[0], event1.event_cid.unwrap());
        assert_eq!(
            event3.parent_cids.len(),
            1,
            "Third event should have one parent"
        );
        assert_eq!(event3.parent_cids[0], event2.event_cid.unwrap());

        // Verify heads
        let heads = store.get_heads().unwrap();
        assert_eq!(heads.len(), 1);
        assert_eq!(heads[0], event3.event_cid.unwrap());
    }

    #[test]
    fn test_multiple_aggregates() {
        // Arrange
        let store = EventStore::new();
        let aggregate1 = Uuid::new_v4();
        let aggregate2 = Uuid::new_v4();

        // Act - Add events for different aggregates
        for i in 0..3 {
            store
                .append_with_payload(
                    aggregate1,
                    format!("Agg1Event{i}"),
                    EventPayload {
                        data: serde_json::json!({"agg": 1, "index": i}),
                        created_at: SystemTime::now(),
                    },
                )
                .unwrap();
        }

        for i in 0..2 {
            store
                .append_with_payload(
                    aggregate2,
                    format!("Agg2Event{i}"),
                    EventPayload {
                        data: serde_json::json!({"agg": 2, "index": i}),
                        created_at: SystemTime::now(),
                    },
                )
                .unwrap();
        }

        // Assert
        let agg1_events = store.get_events_for_aggregate(aggregate1).unwrap();
        assert_eq!(agg1_events.len(), 3);

        let agg2_events = store.get_events_for_aggregate(aggregate2).unwrap();
        assert_eq!(agg2_events.len(), 2);

        // Verify events are correctly filtered
        for event in &agg1_events {
            assert_eq!(event.aggregate_id, aggregate1);
        }

        for event in &agg2_events {
            assert_eq!(event.aggregate_id, aggregate2);
        }
    }

    #[test]
    fn test_event_traversal() {
        // Arrange
        let store = EventStore::new();
        let aggregate_id = Uuid::new_v4();

        // Create a chain of 5 events
        let mut last_cid = None;
        for i in 0..5 {
            let event = store
                .append_with_payload(
                    aggregate_id,
                    format!("Event{i}"),
                    EventPayload {
                        data: serde_json::json!({"index": i}),
                        created_at: SystemTime::now(),
                    },
                )
                .unwrap();
            last_cid = event.event_cid;
        }

        // Act - Traverse from the last event
        let events = store.traverse_from(&last_cid.unwrap(), 10).unwrap();

        // Assert
        assert_eq!(events.len(), 5, "Should traverse all 5 events");

        // Verify they're in correct order when sorted
        let mut sorted = events.clone();
        sorted.sort_by_key(|e| e.sequence);
        for (i, event) in sorted.iter().enumerate() {
            assert_eq!(event.event_type, format!("Event{i}"));
        }
    }

    #[test]
    fn test_cid_determinism() {
        // Arrange
        let content = b"test content for CID";

        // Act
        let cid1 = Cid::from_content(content);
        let cid2 = Cid::from_content(content);

        // Assert
        assert_eq!(cid1, cid2, "Same content should produce same CID");

        // Different content should produce different CID
        let different = b"different content";
        let cid3 = Cid::from_content(different);
        assert_ne!(cid1, cid3);
    }

    #[test]
    fn test_object_store_operations() {
        // Arrange
        let store = EventStore::new();
        let aggregate_id = Uuid::new_v4();

        // Act
        let event = store
            .append_with_payload(
                aggregate_id,
                "TestEvent".to_string(),
                EventPayload {
                    data: serde_json::json!({"stored": true}),
                    created_at: SystemTime::now(),
                },
            )
            .unwrap();

        // Assert - Verify payload can be retrieved
        let payload = store.get_event_payload(&event).unwrap();
        assert!(payload.is_some());

        let retrieved = payload.unwrap();
        assert_eq!(retrieved.data["stored"], true);
    }

    #[test]
    fn test_event_retrieval_by_cid() {
        // Arrange
        let store = EventStore::new();
        let aggregate_id = Uuid::new_v4();

        // Act
        let event = store
            .append_with_payload(
                aggregate_id,
                "TestEvent".to_string(),
                EventPayload {
                    data: serde_json::json!({"test": "retrieval"}),
                    created_at: SystemTime::now(),
                },
            )
            .unwrap();

        let event_cid = event.event_cid.clone().unwrap();

        // Assert - Retrieve event by CID
        let retrieved = store.get_event(&event_cid).unwrap();
        assert!(retrieved.is_some());

        let retrieved_event = retrieved.unwrap();
        assert_eq!(retrieved_event.id, event.id);
        assert_eq!(retrieved_event.event_type, "TestEvent");
    }
}
