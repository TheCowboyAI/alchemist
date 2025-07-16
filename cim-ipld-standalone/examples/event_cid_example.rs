//! Example of proper CID calculation for domain events
//!
//! This example demonstrates how to implement the `canonical_payload` method
//! to ensure that only the actual event data is used for CID calculation,
//! excluding transient metadata like timestamps, UUIDs, and correlation IDs.

use cim_ipld::{ContentType, Result, TypedContent};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

/// A domain event with both payload and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    // Metadata that changes per message
    pub event_id: String,
    pub timestamp: String,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,

    // The actual event payload
    pub event_type: String,
    pub aggregate_id: String,
    pub payload: EventPayload,
}

/// The actual event data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventPayload {
    UserCreated {
        username: String,
        email: String,
    },
    UserUpdated {
        changes: HashMap<String, String>,
    },
    OrderPlaced {
        items: Vec<OrderItem>,
        total_amount: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id: String,
    pub quantity: u32,
    pub price: f64,
}

impl TypedContent for DomainEvent {
    const CODEC: u64 = 0x71; // dag-cbor
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x1000);

    /// Extract only the stable payload for CID calculation
    fn canonical_payload(&self) -> Result<Vec<u8>> {
        // Create a canonical representation with only stable fields
        let canonical = CanonicalEvent {
            event_type: &self.event_type,
            aggregate_id: &self.aggregate_id,
            payload: &self.payload,
        };

        // Serialize the canonical form
        Ok(serde_json::to_vec(&canonical)?)
    }
}

/// Internal struct for canonical serialization
#[derive(Serialize)]
struct CanonicalEvent<'a> {
    event_type: &'a str,
    aggregate_id: &'a str,
    payload: &'a EventPayload,
}

/// Example of a content wrapper that extracts payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope<T> {
    // Message metadata
    pub message_id: String,
    pub sent_at: String,
    pub sender: String,
    pub headers: HashMap<String, String>,

    // The actual content
    pub content: T,
}

impl<T: Serialize + Clone> MessageEnvelope<T> {
    /// Extract just the content for CID calculation
    pub fn content_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(&self.content)?)
    }
}

impl<T: Serialize + DeserializeOwned + Send + Sync + Clone> TypedContent for MessageEnvelope<T> {
    const CODEC: u64 = 0x71;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x2000);

    fn canonical_payload(&self) -> Result<Vec<u8>> {
        // Only use the content, not the envelope metadata
        self.content_bytes()
    }
}

fn main() -> Result<()> {
    // Example 1: Two events with same payload but different metadata
    let event1 = DomainEvent {
        event_id: "evt_123".to_string(),
        timestamp: "2024-01-01T10:00:00Z".to_string(),
        correlation_id: Some("corr_456".to_string()),
        causation_id: None,
        event_type: "UserCreated".to_string(),
        aggregate_id: "user_789".to_string(),
        payload: EventPayload::UserCreated {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        },
    };

    let event2 = DomainEvent {
        event_id: "evt_999".to_string(),               // Different ID
        timestamp: "2024-01-02T15:30:00Z".to_string(), // Different timestamp
        correlation_id: Some("corr_888".to_string()),  // Different correlation
        causation_id: Some("cause_777".to_string()),
        event_type: "UserCreated".to_string(),
        aggregate_id: "user_789".to_string(),
        payload: EventPayload::UserCreated {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        },
    };

    // Calculate CIDs
    let cid1 = event1.calculate_cid()?;
    let cid2 = event2.calculate_cid()?;

    println!("Event 1 CID: {cid1}");
    println!("Event 2 CID: {cid2}");
    println!("CIDs are equal: {cid1 == cid2}");
    println!("✓ Same payload produces same CID despite different metadata\n");

    // Example 2: Different payloads produce different CIDs
    let event3 = DomainEvent {
        event_id: "evt_321".to_string(),
        timestamp: "2024-01-01T10:00:00Z".to_string(),
        correlation_id: None,
        causation_id: None,
        event_type: "UserCreated".to_string(),
        aggregate_id: "user_789".to_string(),
        payload: EventPayload::UserCreated {
            username: "bob".to_string(), // Different username
            email: "bob@example.com".to_string(),
        },
    };

    let cid3 = event3.calculate_cid()?;
    println!("Event 3 CID: {cid3}");
    println!("CID1 != CID3: {cid1 != cid3}");
    println!("✓ Different payload produces different CID\n");

    // Example 3: Message envelope
    let message1 = MessageEnvelope {
        message_id: "msg_001".to_string(),
        sent_at: "2024-01-01T10:00:00Z".to_string(),
        sender: "service-a".to_string(),
        headers: vec![("trace-id".to_string(), "trace_123".to_string())]
            .into_iter()
            .collect(),
        content: "Important business data".to_string(),
    };

    let message2 = MessageEnvelope {
        message_id: "msg_002".to_string(),           // Different message ID
        sent_at: "2024-01-02T15:00:00Z".to_string(), // Different time
        sender: "service-b".to_string(),             // Different sender
        headers: vec![("trace-id".to_string(), "trace_999".to_string())]
            .into_iter()
            .collect(),
        content: "Important business data".to_string(), // Same content!
    };

    let msg_cid1 = message1.calculate_cid()?;
    let msg_cid2 = message2.calculate_cid()?;

    println!("Message 1 CID: {msg_cid1}");
    println!("Message 2 CID: {msg_cid2}");
    println!("Message CIDs are equal: {msg_cid1 == msg_cid2}");
    println!("✓ Same content produces same CID despite different envelope metadata");

    Ok(())
}
