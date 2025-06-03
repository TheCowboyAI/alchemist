use super::events::{Cid, DomainEvent};
use super::store::EventStore;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Persistence format for the Merkle DAG
#[derive(Serialize, Deserialize)]
pub struct PersistedEventStore {
    /// All events indexed by CID
    pub events: HashMap<String, DomainEvent>,
    /// Object store contents
    pub objects: HashMap<String, Vec<u8>>,
    /// Current sequence number
    pub sequence: u64,
    /// Current heads of the DAG
    pub heads: Vec<String>,
}

/// Handles persistence of the event store to disk
pub struct EventPersistence;

impl EventPersistence {
    /// Save the entire event store to a file
    pub fn save_store(event_store: &EventStore, path: &Path) -> Result<(), String> {
        // Extract data from the event store
        let events = event_store
            .events
            .read()
            .map_err(|e| format!("Lock error: {e}"))?;

        let objects = event_store
            .object_store
            .objects
            .read()
            .map_err(|e| format!("Lock error: {e}"))?;

        let sequence = *event_store
            .sequence_counter
            .read()
            .map_err(|e| format!("Lock error: {e}"))?;

        let heads = event_store
            .heads
            .read()
            .map_err(|e| format!("Lock error: {e}"))?;

        // Convert to serializable format
        let persisted = PersistedEventStore {
            events: events
                .iter()
                .map(|(cid, event)| (cid.0.clone(), event.clone()))
                .collect(),
            objects: objects
                .iter()
                .map(|(cid, data)| (cid.0.clone(), data.clone()))
                .collect(),
            sequence,
            heads: heads.iter().map(|cid| cid.0.clone()).collect(),
        };

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&persisted)
            .map_err(|e| format!("Serialization error: {e}"))?;

        // Write to file
        fs::write(path, json).map_err(|e| format!("File write error: {e}"))?;

        Ok(())
    }

    /// Load an event store from a file
    pub fn load_store(path: &Path) -> Result<EventStore, String> {
        // Read file
        let json = fs::read_to_string(path).map_err(|e| format!("File read error: {e}"))?;

        // Deserialize
        let persisted: PersistedEventStore =
            serde_json::from_str(&json).map_err(|e| format!("Deserialization error: {e}"))?;

        // Create new event store
        let event_store = EventStore::new();

        // Populate events
        {
            let mut events = event_store
                .events
                .write()
                .map_err(|e| format!("Lock error: {e}"))?;
            for (cid_str, event) in persisted.events {
                events.insert(Cid(cid_str), event);
            }
        }

        // Populate objects
        {
            let mut objects = event_store
                .object_store
                .objects
                .write()
                .map_err(|e| format!("Lock error: {e}"))?;
            for (cid_str, data) in persisted.objects {
                objects.insert(Cid(cid_str), data);
            }
        }

        // Set sequence counter
        {
            let mut counter = event_store
                .sequence_counter
                .write()
                .map_err(|e| format!("Lock error: {e}"))?;
            *counter = persisted.sequence;
        }

        // Set heads
        {
            let mut heads = event_store
                .heads
                .write()
                .map_err(|e| format!("Lock error: {e}"))?;
            *heads = persisted.heads.into_iter().map(|s| Cid(s)).collect();
        }

        // Rebuild aggregate index
        {
            let events = event_store
                .events
                .read()
                .map_err(|e| format!("Lock error: {e}"))?;
            let mut index = event_store
                .aggregate_index
                .write()
                .map_err(|e| format!("Lock error: {e}"))?;

            for (cid, event) in events.iter() {
                index
                    .entry(event.aggregate_id)
                    .or_insert_with(Vec::new)
                    .push(cid.clone());
            }
        }

        Ok(event_store)
    }

    /// Export a subset of events as a portable DAG
    pub fn export_dag(
        event_store: &EventStore,
        start_cid: &Cid,
        max_depth: usize,
    ) -> Result<String, String> {
        let events = event_store.traverse_from(start_cid, max_depth)?;

        // Collect all required objects
        let mut required_objects = HashMap::new();
        for event in &events {
            if let Some(payload) = event_store.get_event_payload(event)? {
                let bytes = serde_json::to_vec(&payload)
                    .map_err(|e| format!("Serialization error: {e}"))?;
                required_objects.insert(event.payload_cid.0.clone(), bytes);
            }
        }

        // Create export format
        let export = serde_json::json!({
            "version": "1.0",
            "root_cid": start_cid.0,
            "events": events,
            "objects": required_objects,
        });

        serde_json::to_string_pretty(&export).map_err(|e| format!("Serialization error: {e}"))
    }
}
