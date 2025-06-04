//! Event Store Infrastructure - Internal Only

use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

mod local;

use local::LocalEventStore;

use crate::domain::events::DomainEvent;
use crate::domain::value_objects::GraphId;

/// Event ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

/// Event envelope that wraps domain events with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: EventId,
    pub aggregate_id: GraphId,
    pub event: DomainEvent,
    pub sequence: u64,
    pub timestamp: SystemTime,
}

/// Event Store trait - Internal use only
trait EventStore: Send + Sync {
    /// Append an event to the store
    fn append(&mut self, aggregate_id: GraphId, event: DomainEvent) -> EventEnvelope;

    /// Get all events for an aggregate
    fn get_events(&self, aggregate_id: GraphId) -> Vec<EventEnvelope>;

    /// Get events after a specific sequence number
    fn get_events_after(&self, aggregate_id: GraphId, sequence: u64) -> Vec<EventEnvelope>;
}

// Global event store instance - hidden from the rest of the application
static EVENT_STORE: std::sync::Mutex<Option<LocalEventStore>> = std::sync::Mutex::new(None);

/// Initialize the event store (called once at startup)
pub fn initialize_event_store() {
    let mut store = EVENT_STORE.lock().unwrap();
    *store = Some(LocalEventStore::new());
}

/// Store an event - this is the only public interface
pub fn store_event(aggregate_id: GraphId, event: DomainEvent) -> EventEnvelope {
    let mut store = EVENT_STORE.lock().unwrap();
    let store = store.as_mut().expect("Event store not initialized");
    store.append(aggregate_id, event)
}

/// Get events for an aggregate
pub fn get_aggregate_events(aggregate_id: GraphId) -> Vec<EventEnvelope> {
    let store = EVENT_STORE.lock().unwrap();
    let store = store.as_ref().expect("Event store not initialized");
    store.get_events(aggregate_id)
}
