//! Event Store Infrastructure

use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use thiserror::Error;
use uuid::Uuid;

mod local;
pub mod distributed;
pub mod distributed_impl;
pub mod memory;

use local::LocalEventStore;

// Re-export the new implementation as the primary one
pub use distributed_impl::DistributedEventStore;
pub use distributed::{DistributedEventStoreConfig, EventStoreStats};
pub use memory::InMemoryEventStore;

use crate::domain::events::DomainEvent;
use crate::domain::value_objects::GraphId;

use async_trait::async_trait;

/// Event Store errors
#[derive(Error, Debug)]
pub enum EventStoreError {
    #[error("Event store error: {0}")]
    StoreError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Aggregate not found: {0}")]
    AggregateNotFound(GraphId),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Not found")]
    NotFound,
}

/// Aggregate snapshot for faster rebuilding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateSnapshot {
    pub aggregate_id: GraphId,
    pub version: u64,
    pub data: Vec<u8>,
    pub timestamp: SystemTime,
}

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
trait InternalEventStore: Send + Sync {
    /// Append an event to the store
    fn append(&mut self, aggregate_id: GraphId, event: DomainEvent) -> EventEnvelope;

    /// Append events to the store
    #[allow(dead_code)]
    fn append_events(&self, events: Vec<EventEnvelope>) -> Result<(), EventStoreError>;

    /// Get all events for an aggregate
    fn get_events(&self, aggregate_id: GraphId) -> Vec<EventEnvelope>;

    /// Get events after a specific sequence number
    #[allow(dead_code)]
    fn get_events_after(&self, aggregate_id: GraphId, sequence: u64) -> Vec<EventEnvelope>;

    /// Get a snapshot if available
    #[allow(dead_code)]
    fn get_snapshot(&self, aggregate_id: GraphId) -> Option<AggregateSnapshot>;

    /// Save a snapshot
    #[allow(dead_code)]
    fn save_snapshot(&self, snapshot: AggregateSnapshot) -> Result<(), EventStoreError>;
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

/// Trait for async event stores
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Append events to the store for a specific aggregate
    async fn append_events(&self, aggregate_id: String, events: Vec<DomainEvent>) -> Result<(), EventStoreError>;

    /// Get all events for an aggregate
    async fn get_events(&self, aggregate_id: String) -> Result<Vec<DomainEvent>, EventStoreError>;

    /// Store an event (legacy method)
    async fn store(&self, event: DomainEvent) -> Result<(), EventStoreError>;

    /// Load events for an aggregate (legacy method)
    async fn load_events(&self, aggregate_id: Uuid) -> Result<Vec<DomainEvent>, EventStoreError>;

    /// Load all events (legacy method)
    async fn load_all_events(&self) -> Result<Vec<DomainEvent>, EventStoreError>;
}
