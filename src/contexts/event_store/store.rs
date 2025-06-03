use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use super::events::{DomainEvent, EventPayload, Cid};
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
        let mut objects = self.objects.write()
            .map_err(|e| format!("Lock error: {}", e))?;
        objects.insert(cid.clone(), content.to_vec());
        Ok(cid)
    }

    /// Retrieve an object by CID
    pub fn get(&self, cid: &Cid) -> Result<Option<Vec<u8>>, String> {
        let objects = self.objects.read()
            .map_err(|e| format!("Lock error: {}", e))?;
        Ok(objects.get(cid).cloned())
    }

    /// Store an event payload and return its CID
    pub fn put_payload(&self, payload: &EventPayload) -> Result<Cid, String> {
        let bytes = serde_json::to_vec(payload)
            .map_err(|e| format!("Serialization error: {}", e))?;
        self.put(&bytes)
    }

    /// Retrieve an event payload by CID
    pub fn get_payload(&self, cid: &Cid) -> Result<Option<EventPayload>, String> {
        match self.get(cid)? {
            Some(bytes) => {
                let payload = serde_json::from_slice(&bytes)
                    .map_err(|e| format!("Deserialization error: {}", e))?;
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
        let mut event = DomainEvent::new(
            aggregate_id,
            event_type,
            payload_cid,
            parent_cids,
        );

        // Set sequence number
        let mut counter = self.sequence_counter.write()
            .map_err(|e| format!("Lock error: {}", e))?;
        *counter += 1;
        event.sequence = *counter;

        // Recompute CID with sequence number
        event.compute_cid();

        let event_cid = event.cid()
            .ok_or("Event CID not computed")?
            .clone();

        // Store the event
        let mut events = self.events.write()
            .map_err(|e| format!("Lock error: {}", e))?;
        events.insert(event_cid.clone(), event.clone());

        // Update aggregate index
        let mut index = self.aggregate_index.write()
            .map_err(|e| format!("Lock error: {}", e))?;
        index.entry(aggregate_id)
            .or_insert_with(Vec::new)
            .push(event_cid.clone());

        // Update heads
        self.update_heads(event_cid)?;

        Ok(event)
    }

    /// Get the current heads of the DAG
    pub fn get_heads(&self) -> Result<Vec<Cid>, String> {
        let heads = self.heads.read()
            .map_err(|e| format!("Lock error: {}", e))?;
        Ok(heads.clone())
    }

    /// Update the heads after adding a new event
    fn update_heads(&self, new_cid: Cid) -> Result<(), String> {
        let mut heads = self.heads.write()
            .map_err(|e| format!("Lock error: {}", e))?;

        // For now, just replace all heads with the new event
        // In a more sophisticated implementation, we'd maintain multiple heads
        heads.clear();
        heads.push(new_cid);

        Ok(())
    }

    /// Get all events for an aggregate
    pub fn get_events_for_aggregate(&self, aggregate_id: Uuid) -> Result<Vec<DomainEvent>, String> {
        let index = self.aggregate_index.read()
            .map_err(|e| format!("Lock error: {}", e))?;

        let cids = match index.get(&aggregate_id) {
            Some(cids) => cids.clone(),
            None => return Ok(Vec::new()),
        };

        let events = self.events.read()
            .map_err(|e| format!("Lock error: {}", e))?;

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
        let events = self.events.read()
            .map_err(|e| format!("Lock error: {}", e))?;
        Ok(events.get(cid).cloned())
    }

    /// Get the payload for an event
    pub fn get_event_payload(&self, event: &DomainEvent) -> Result<Option<EventPayload>, String> {
        self.object_store.get_payload(&event.payload_cid)
    }

    /// Traverse the DAG from a given event CID
    pub fn traverse_from(&self, start_cid: &Cid, max_depth: usize) -> Result<Vec<DomainEvent>, String> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![(start_cid.clone(), 0)];

        let events = self.events.read()
            .map_err(|e| format!("Lock error: {}", e))?;

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
