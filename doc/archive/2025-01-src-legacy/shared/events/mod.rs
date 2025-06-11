//! Shared event types and infrastructure
//!
//! This module defines the base event infrastructure used across all contexts.
//! Each context will define its own specific events, but they all share this
//! common foundation for event sourcing and NATS integration.

use crate::shared::types::{Timestamp, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Correlation ID for tracking related events across contexts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(Uuid);

impl CorrelationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Causation ID for tracking event causality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CausationId(Uuid);

impl CausationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_event_id(event_id: EventId) -> Self {
        Self(event_id.0)
    }
}

/// Base metadata for all domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: EventId,
    pub timestamp: Timestamp,
    pub correlation_id: Option<CorrelationId>,
    pub causation_id: Option<CausationId>,
    pub actor: Option<String>,
    pub version: u32,
}

impl EventMetadata {
    pub fn new() -> Self {
        Self {
            event_id: EventId::new(),
            timestamp: chrono::Utc::now(),
            correlation_id: None,
            causation_id: None,
            actor: None,
            version: 1,
        }
    }

    pub fn with_correlation(mut self, correlation_id: CorrelationId) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn with_causation(mut self, causation_id: CausationId) -> Self {
        self.causation_id = Some(causation_id);
        self
    }

    pub fn with_actor(mut self, actor: String) -> Self {
        self.actor = Some(actor);
        self
    }
}

/// Trait that all domain events must implement
pub trait DomainEvent: Send + Sync {
    /// Get the aggregate ID this event belongs to
    fn aggregate_id(&self) -> String;

    /// Get the event type name for routing
    fn event_type(&self) -> &'static str;

    /// Get the event metadata
    fn metadata(&self) -> &EventMetadata;

    /// Get the NATS subject for this event
    fn subject(&self) -> String {
        format!("events.{}.{}", self.event_type(), self.aggregate_id())
    }

    /// Serialize the event to JSON
    fn to_json(&self) -> Result<serde_json::Value>;

    /// Clone the event into a box
    fn clone_box(&self) -> Box<dyn DomainEvent>;
}

/// Event envelope for serialization over NATS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    pub metadata: EventMetadata,
    pub payload: T,
}

impl<T: DomainEvent> EventEnvelope<T> {
    pub fn new(event: T) -> Self {
        Self {
            metadata: event.metadata().clone(),
            payload: event,
        }
    }
}

/// Event store interface (to be implemented by infrastructure)
#[async_trait::async_trait]
pub trait EventStore: Send + Sync {
    /// Append events to the store
    async fn append_events(&self, events: Vec<Box<dyn DomainEvent>>) -> Result<()>;

    /// Load events for an aggregate
    async fn load_events(&self, aggregate_id: &str, from_version: u64) -> Result<Vec<serde_json::Value>>;

    /// Subscribe to events of a specific type
    async fn subscribe(&self, event_type: &str) -> Result<Box<dyn EventSubscription>>;
}

/// Event subscription handle
#[async_trait::async_trait]
pub trait EventSubscription: Send + Sync {
    /// Get the next event from the subscription
    async fn next(&mut self) -> Result<Option<serde_json::Value>>;

    /// Close the subscription
    async fn close(self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        aggregate_id: String,
        metadata: EventMetadata,
        data: String,
    }

            impl DomainEvent for TestEvent {
            fn aggregate_id(&self) -> String {
                self.aggregate_id.clone()
            }

            fn event_type(&self) -> &'static str {
                "test_event"
            }

            fn metadata(&self) -> &EventMetadata {
                &self.metadata
            }

            fn to_json(&self) -> Result<serde_json::Value> {
                serde_json::to_value(self).map_err(|e| Error::Serialization(e))
            }

            fn clone_box(&self) -> Box<dyn DomainEvent> {
                Box::new(self.clone())
            }
        }

    #[test]
    fn test_event_metadata_creation() {
        let metadata = EventMetadata::new()
            .with_actor("test_user".to_string());

        assert!(metadata.actor.is_some());
        assert_eq!(metadata.actor.unwrap(), "test_user");
        assert_eq!(metadata.version, 1);
    }

    #[test]
    fn test_event_subject_generation() {
        let event = TestEvent {
            aggregate_id: "test_123".to_string(),
            metadata: EventMetadata::new(),
            data: "test data".to_string(),
        };

        assert_eq!(event.subject(), "events.test_event.test_123");
    }

    #[test]
    fn test_event_envelope() {
        let event = TestEvent {
            aggregate_id: "test_123".to_string(),
            metadata: EventMetadata::new(),
            data: "test data".to_string(),
        };

        let envelope = EventEnvelope::new(event.clone());
        assert_eq!(envelope.metadata.event_id, event.metadata.event_id);
    }
}
