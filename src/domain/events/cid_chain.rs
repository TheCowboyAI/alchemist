//! CID chain support for domain events

use super::DomainEvent;
use cid::Cid;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// A domain event with CID chain information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainedEvent {
    pub event_cid: Cid,
    pub previous_cid: Option<Cid>,
    pub sequence: u64,
    pub timestamp: SystemTime,
    pub payload: serde_json::Value,
}

/// Event chain for managing event sequences
pub struct EventChain;

impl Default for EventChain {
    fn default() -> Self {
        Self::new()
    }
}

impl EventChain {
    pub fn new() -> Self {
        Self
    }

    pub fn add_event(&self, event: DomainEvent, previous_cid: Option<Cid>) -> Result<ChainedEvent, Box<dyn std::error::Error>> {
        let payload = serde_json::to_value(&event)?;
        let event_cid = self.calculate_cid(&event)?;

        Ok(ChainedEvent {
            event_cid,
            previous_cid,
            sequence: 0, // Would be calculated based on previous events
            timestamp: SystemTime::now(),
            payload,
        })
    }

    pub fn calculate_cid(&self, event: &DomainEvent) -> Result<Cid, Box<dyn std::error::Error>> {
        // Serialize event for hashing
        let bytes = serde_json::to_vec(event)?;

        // Create hash using BLAKE3
        let hash = blake3::hash(&bytes);
        let hash_bytes = hash.as_bytes();

        // Create multihash with BLAKE3 code (0x1e)
        let code = 0x1e; // BLAKE3-256
        let size = hash_bytes.len() as u8;

        // Build multihash: <varint code><varint size><hash>
        let mut multihash_bytes = Vec::new();
        multihash_bytes.push(code);
        multihash_bytes.push(size);
        multihash_bytes.extend_from_slice(hash_bytes);

        // Create CID v1
        let mh = multihash::Multihash::from_bytes(&multihash_bytes)?;
        let cid = Cid::new_v1(0x71, mh); // 0x71 = dag-cbor

        Ok(cid)
    }
}

impl DomainEvent {
    /// Convert this event to a chained event
    pub fn as_chained_event(&self) -> Result<ChainedEvent, Box<dyn std::error::Error>> {
        let chain = EventChain::new();
        chain.add_event(self.clone(), None)
    }
}
