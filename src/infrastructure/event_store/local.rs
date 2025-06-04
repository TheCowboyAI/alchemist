//! Local in-memory event store for development

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::SystemTime;

use crate::domain::events::DomainEvent;
use crate::domain::value_objects::GraphId;
use super::{EventEnvelope, EventId, EventStore};

/// Local in-memory event store
pub struct LocalEventStore {
    events: RwLock<Vec<EventEnvelope>>,
    sequences: RwLock<HashMap<GraphId, u64>>,
}

impl LocalEventStore {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            sequences: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for LocalEventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl EventStore for LocalEventStore {
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
}
