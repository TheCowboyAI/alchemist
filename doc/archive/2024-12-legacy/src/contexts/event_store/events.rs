use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

/// Content Identifier for Merkle DAG structure
/// This is a placeholder - in production this would be a proper CID
/// like from the `cid` crate or IPFS
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Cid(pub String);

impl Cid {
    /// Create a CID from content (placeholder implementation)
    /// In production, this would compute a proper multihash
    pub fn from_content(content: &[u8]) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let hash = hasher.finish();

        // Placeholder CID format
        Cid(format!("cid:v1:dag-cbor:{hash:x}"))
    }
}

/// Metadata for tracking event origin and relationships
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventMetadata {
    /// User who initiated the action (if applicable)
    pub user_id: Option<String>,
    /// Session identifier for tracking related events
    pub session_id: Uuid,
    /// Links this event to a broader operation
    pub correlation_id: Option<Uuid>,
    /// The event that directly caused this one
    pub causation_id: Option<Uuid>,
    /// Whether this event has been synced to NATS
    pub synced_to_nats: bool,
    /// NATS sequence number if synced
    pub nats_sequence: Option<u64>,
}

/// Local domain event for event sourcing and audit trail
/// Events form a Merkle DAG where each event links to previous events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    /// Unique identifier for this event
    pub id: Uuid,
    /// When the event occurred
    pub timestamp: SystemTime,
    /// Local sequence number for ordering
    pub sequence: u64,
    /// The aggregate (graph) this event belongs to
    pub aggregate_id: Uuid,
    /// Type of event as a string for flexibility
    pub event_type: String,
    /// CID of the event payload stored in object store
    pub payload_cid: Cid,
    /// Links to parent events in the Merkle DAG
    pub parent_cids: Vec<Cid>,
    /// CID of this event (computed from all fields except itself)
    pub event_cid: Option<Cid>,
    /// Tracking and relationship metadata
    pub metadata: EventMetadata,
}

/// The actual payload data that gets stored by CID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPayload {
    /// The event-specific data
    pub data: serde_json::Value,
    /// Timestamp when payload was created
    pub created_at: SystemTime,
}

/// Bevy Event wrapper for ECS integration
/// This allows us to process domain events through Bevy's event system
#[derive(Event, Clone)]
pub struct DomainEventOccurred(pub DomainEvent);

/// Marker for events that should be synced to NATS
#[derive(Event, Clone)]
pub struct SyncToNats(pub DomainEvent);

impl DomainEvent {
    /// Create a new domain event with a payload CID
    pub fn new(
        aggregate_id: Uuid,
        event_type: String,
        payload_cid: Cid,
        parent_cids: Vec<Cid>,
    ) -> Self {
        let mut event = Self {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            sequence: 0, // Will be set by EventStore
            aggregate_id,
            event_type,
            payload_cid,
            parent_cids,
            event_cid: None, // Will be computed
            metadata: EventMetadata {
                session_id: Uuid::new_v4(), // Should be injected from context
                synced_to_nats: false,
                ..Default::default()
            },
        };

        // Compute the CID for this event
        event.compute_cid();
        event
    }

    /// Compute and set the CID for this event
    pub fn compute_cid(&mut self) {
        // Serialize everything except the event_cid field
        let content = serde_json::json!({
            "id": self.id,
            "timestamp": self.timestamp,
            "sequence": self.sequence,
            "aggregate_id": self.aggregate_id,
            "event_type": &self.event_type,
            "payload_cid": &self.payload_cid,
            "parent_cids": &self.parent_cids,
            "metadata": &self.metadata,
        });

        let bytes = serde_json::to_vec(&content).unwrap();
        self.event_cid = Some(Cid::from_content(&bytes));
    }

    /// Check if this event has been synced to NATS
    pub fn is_synced(&self) -> bool {
        self.metadata.synced_to_nats
    }

    /// Mark this event as synced to NATS
    pub fn mark_synced(&mut self, nats_sequence: u64) {
        self.metadata.synced_to_nats = true;
        self.metadata.nats_sequence = Some(nats_sequence);
    }

    /// Get the CID of this event
    pub fn cid(&self) -> Option<&Cid> {
        self.event_cid.as_ref()
    }
}
