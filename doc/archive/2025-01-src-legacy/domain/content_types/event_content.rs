//! Event content type for CIM-IPLD with chaining support

use crate::domain::events::DomainEvent;
use cim_ipld::{ContentType, TypedContent};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

/// Content wrapper for domain events with CID chaining
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContent {
    /// The domain event
    pub event: DomainEvent,
    /// Aggregate ID this event belongs to
    pub aggregate_id: Uuid,
    /// Event sequence number within the aggregate
    pub sequence: u64,
    /// Timestamp when the event occurred
    pub timestamp: SystemTime,
    /// Optional correlation ID for tracking related events
    pub correlation_id: Option<Uuid>,
    /// Optional causation ID (the command that caused this event)
    pub causation_id: Option<Uuid>,
}

impl TypedContent for EventContent {
    const CODEC: u64 = 0x300105;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x300105);
}

impl EventContent {
    /// Create a new event content
    pub fn new(event: DomainEvent, aggregate_id: Uuid, sequence: u64) -> Self {
        Self {
            event,
            aggregate_id,
            sequence,
            timestamp: SystemTime::now(),
            correlation_id: None,
            causation_id: None,
        }
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Set causation ID
    pub fn with_causation_id(mut self, causation_id: Uuid) -> Self {
        self.causation_id = Some(causation_id);
        self
    }

    /// Get the event type
    pub fn event_type(&self) -> &'static str {
        self.event.event_type()
    }

    /// Create a genesis event (first in chain)
    pub fn genesis(event: DomainEvent, aggregate_id: Uuid) -> Self {
        Self::new(event, aggregate_id, 0)
    }
}

/// Event chain metadata for tracking event sequences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventChainMetadata {
    /// Aggregate ID this chain belongs to
    pub aggregate_id: Uuid,
    /// CID of the first event in the chain (stored as string for serialization)
    pub genesis_cid: String,
    /// CID of the latest event in the chain (stored as string for serialization)
    pub head_cid: String,
    /// Total number of events in the chain
    pub event_count: u64,
    /// Timestamp of the first event
    pub created_at: SystemTime,
    /// Timestamp of the latest event
    pub updated_at: SystemTime,
}

impl TypedContent for EventChainMetadata {
    const CODEC: u64 = 0x300106;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x300106);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::events::GraphEvent;
    use crate::domain::value_objects::{GraphId, GraphMetadata};
    use cim_ipld::ContentChain;

    #[test]
    fn test_event_content_creation() {
        let aggregate_id = Uuid::new_v4();
        let graph_event = DomainEvent::Graph(GraphEvent::GraphCreated {
            id: GraphId::from(aggregate_id),
            metadata: GraphMetadata::new("Test Graph".to_string()),
        });

        let event_content = EventContent::new(graph_event, aggregate_id, 1);

        assert_eq!(event_content.aggregate_id, aggregate_id);
        assert_eq!(event_content.sequence, 1);
    }

    #[test]
    fn test_event_content_chaining() {
        let aggregate_id = Uuid::new_v4();
        let graph_event = DomainEvent::Graph(GraphEvent::GraphCreated {
            id: GraphId::from(aggregate_id),
            metadata: GraphMetadata::new("Test Graph".to_string()),
        });

        // Create a chain of events
        let mut chain = ContentChain::<EventContent>::new();

        let event1 = EventContent::genesis(graph_event.clone(), aggregate_id);
        let chained1 = chain.append(event1).unwrap();
        let cid1 = chained1.cid.clone();

        let event2 = EventContent::new(graph_event, aggregate_id, 1);
        let chained2 = chain.append(event2).unwrap();

        assert_eq!(chained2.previous_cid, Some(cid1));
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn test_event_content_cid() {
        let aggregate_id = Uuid::new_v4();
        let graph_event = DomainEvent::Graph(GraphEvent::GraphCreated {
            id: GraphId::from(aggregate_id),
            metadata: GraphMetadata::new("Test Graph".to_string()),
        });

        let event_content = EventContent::new(graph_event, aggregate_id, 1);
        let cid = event_content.calculate_cid().unwrap();

        assert!(!cid.to_string().is_empty());
    }
}
