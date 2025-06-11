//! Local in-memory event store for development

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::SystemTime;

use super::{AggregateSnapshot, EventEnvelope, EventId, InternalEventStore, EventStoreError};
use crate::domain::events::DomainEvent;
use crate::domain::value_objects::GraphId;

/// Local in-memory event store
pub struct LocalEventStore {
    events: RwLock<Vec<EventEnvelope>>,
    sequences: RwLock<HashMap<GraphId, u64>>,
    #[allow(dead_code)]
    snapshots: RwLock<HashMap<GraphId, AggregateSnapshot>>,
}

impl LocalEventStore {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            sequences: RwLock::new(HashMap::new()),
            snapshots: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for LocalEventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InternalEventStore for LocalEventStore {
    fn append(&mut self, aggregate_id: GraphId, event: DomainEvent) -> EventEnvelope {
        let mut sequences = self.sequences.write().unwrap();
        let sequence = sequences.entry(aggregate_id).or_insert(0);
        *sequence += 1;

        let envelope = EventEnvelope {
            event_id: EventId::new(),
            aggregate_id,
            event,
            sequence: *sequence,
            timestamp: SystemTime::now(),
        };

        let mut events = self.events.write().unwrap();
        events.push(envelope.clone());

        envelope
    }

    fn append_events(&self, events: Vec<EventEnvelope>) -> Result<(), EventStoreError> {
        let mut store_events = self.events.write().unwrap();
        let mut sequences = self.sequences.write().unwrap();

        for event in events {
            // Update sequence tracking
            let seq = sequences.entry(event.aggregate_id).or_insert(0);
            *seq = (*seq).max(event.sequence);

            store_events.push(event);
        }

        Ok(())
    }

    fn get_events(&self, aggregate_id: GraphId) -> Vec<EventEnvelope> {
        let events = self.events.read().unwrap();
        events
            .iter()
            .filter(|e| e.aggregate_id == aggregate_id)
            .cloned()
            .collect()
    }

    fn get_events_after(&self, aggregate_id: GraphId, sequence: u64) -> Vec<EventEnvelope> {
        let events = self.events.read().unwrap();
        events
            .iter()
            .filter(|e| e.aggregate_id == aggregate_id && e.sequence > sequence)
            .cloned()
            .collect()
    }

    fn get_snapshot(&self, aggregate_id: GraphId) -> Option<AggregateSnapshot> {
        let snapshots = self.snapshots.read().unwrap();
        snapshots.get(&aggregate_id).cloned()
    }

    fn save_snapshot(&self, snapshot: AggregateSnapshot) -> Result<(), EventStoreError> {
        let mut snapshots = self.snapshots.write().unwrap();
        snapshots.insert(snapshot.aggregate_id, snapshot);
        Ok(())
    }
}
