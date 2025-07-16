//! Infrastructure Layer 1.2: Event Store Tests for cim-ipld
//! 
//! User Story: As a content management system, I need to persist content with CID chains for integrity
//!
//! Test Requirements:
//! - Verify content persistence with CID calculation
//! - Verify CID chain integrity for content versions
//! - Verify content retrieval from IPLD store
//! - Verify codec operations and content type handling
//!
//! Event Sequence:
//! 1. IPLDStoreInitialized
//! 2. ContentPersisted { content_id, cid, previous_cid }
//! 3. CIDChainValidated { start_cid, end_cid, length }
//! 4. ContentRetrieved { cid, content_type }
//!
//! ```mermaid
//! graph LR
//!     A[Test Start] --> B[Initialize IPLD Store]
//!     B --> C[IPLDStoreInitialized]
//!     C --> D[Persist Content]
//!     D --> E[Calculate CID]
//!     E --> F[ContentPersisted]
//!     F --> G[Validate Chain]
//!     G --> H[CIDChainValidated]
//!     H --> I[Retrieve Content]
//!     I --> J[ContentRetrieved]
//!     J --> K[Test Success]
//! ```

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use cid::Cid;
use blake3;

/// IPLD event for testing content operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPLDEventData {
    pub event_id: String,
    pub content_id: String,
    pub event_type: String,
    pub sequence: u64,
    pub content_type: String,
    pub payload: Vec<u8>,
}

/// IPLD event store for content management testing
pub struct IPLDEventStore {
    events: Vec<(IPLDEventData, Cid, Option<Cid>)>,
    content_store: HashMap<Cid, Vec<u8>>,
    content_chains: HashMap<String, Vec<usize>>, // content_id -> event indices
    codec_registry: HashMap<String, String>, // content_type -> codec
}

impl IPLDEventStore {
    pub fn new() -> Self {
        let mut codec_registry = HashMap::new();
        // Register default codecs
        codec_registry.insert("application/json".to_string(), "dag-json".to_string());
        codec_registry.insert("application/cbor".to_string(), "dag-cbor".to_string());
        codec_registry.insert("text/plain".to_string(), "raw".to_string());
        codec_registry.insert("application/pdf".to_string(), "raw".to_string());
        codec_registry.insert("image/jpeg".to_string(), "raw".to_string());
        codec_registry.insert("image/png".to_string(), "raw".to_string());
        codec_registry.insert("video/mp4".to_string(), "raw".to_string());
        codec_registry.insert("audio/wav".to_string(), "raw".to_string());
        
        Self {
            events: Vec::new(),
            content_store: HashMap::new(),
            content_chains: HashMap::new(),
            codec_registry,
        }
    }

    pub fn persist_content(
        &mut self,
        event: IPLDEventData,
        previous_cid: Option<Cid>,
    ) -> Result<Cid, String> {
        // Serialize event for CID calculation
        let event_bytes = serde_json::to_vec(&event)
            .map_err(|e| format!("Serialization error: {e}"))?;
        
        // Calculate CID using blake3
        let hash = blake3::hash(&event_bytes);
        let hash_bytes = hash.as_bytes();
        
        // Create multihash manually with BLAKE3 code (0x1e)
        let code = 0x1e; // BLAKE3-256
        let size = hash_bytes.len() as u8;
        
        // Build multihash: <varint code><varint size><hash>
        let mut multihash_bytes = Vec::new();
        multihash_bytes.push(code);
        multihash_bytes.push(size);
        multihash_bytes.extend_from_slice(hash_bytes);
        
        // Create CID v1
        let mh = multihash::Multihash::from_bytes(&multihash_bytes)
            .map_err(|e| format!("Multihash error: {e}"))?;
        let cid = Cid::new_v1(0x71, mh); // 0x71 = dag-cbor
        
        // Store content
        self.content_store.insert(cid, event.payload.clone());
        
        let event_index = self.events.len();
        self.events.push((event.clone(), cid, previous_cid));
        
        // Update content chain index
        self.content_chains
            .entry(event.content_id.clone())
            .or_insert_with(Vec::new)
            .push(event_index);
        
        Ok(cid)
    }

    pub fn validate_chain(&self, content_id: &str) -> Result<(Cid, Cid, usize), String> {
        let indices = self.content_chains.get(content_id)
            .ok_or_else(|| format!("No events for content {content_id}"))?;
        
        if indices.is_empty() {
            return Err("No events in content chain".to_string());
        }

        // Validate chain for this content
        for i in 1..indices.len() {
            let (_, _, prev_cid) = &self.events[indices[i]];
            let (_, expected_prev_cid, _) = &self.events[indices[i - 1]];
            
            if prev_cid.as_ref() != Some(expected_prev_cid) {
                return Err(format!("Chain broken at position {i} for content {content_id}"));
            }
        }

        let start_cid = self.events[indices[0]].1;
        let end_cid = self.events[indices[indices.len() - 1]].1;
        
        Ok((start_cid, end_cid, indices.len()))
    }

    pub fn retrieve_content(&self, cid: &Cid) -> Option<Vec<u8>> {
        self.content_store.get(cid).cloned()
    }

    pub fn get_codec_for_type(&self, content_type: &str) -> Option<&str> {
        self.codec_registry.get(content_type).map(|s| s.as_str())
    }

    pub fn register_codec(&mut self, content_type: String, codec: String) {
        self.codec_registry.insert(content_type, codec);
    }

    pub fn get_latest_cid(&self, content_id: &str) -> Option<Cid> {
        self.content_chains.get(content_id)
            .and_then(|indices| indices.last())
            .map(|&i| self.events[i].1)
    }

    pub fn list_content_versions(&self, content_id: &str) -> Vec<(Cid, String)> {
        self.content_chains.get(content_id)
            .map(|indices| {
                indices.iter()
                    .map(|&i| {
                        let (event, cid, _) = &self.events[i];
                        (*cid, event.event_type.clone())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Event types for IPLD store testing
#[derive(Debug, Clone, PartialEq)]
pub enum IPLDStoreEvent {
    IPLDStoreInitialized,
    ContentPersisted { 
        content_id: String, 
        cid: Cid, 
        previous_cid: Option<Cid> 
    },
    CIDChainValidated { 
        start_cid: Cid, 
        end_cid: Cid, 
        length: usize 
    },
    ContentRetrieved { 
        cid: Cid, 
        content_type: String 
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipld_store_initialization() {
        // Arrange & Act
        let store = IPLDEventStore::new();
        
        // Assert
        assert_eq!(store.events.len(), 0);
        assert_eq!(store.content_store.len(), 0);
        assert_eq!(store.content_chains.len(), 0);
        assert!(store.codec_registry.len() > 0); // Has default codecs
        
        // Verify default codecs
        assert_eq!(store.get_codec_for_type("application/json"), Some("dag-json"));
        assert_eq!(store.get_codec_for_type("text/plain"), Some("raw"));
    }

    #[test]
    fn test_content_persistence_with_cid() {
        // Arrange
        let mut store = IPLDEventStore::new();
        let event = IPLDEventData {
            event_id: "ipld_evt_1".to_string(),
            content_id: "doc_1".to_string(),
            event_type: "DocumentCreated".to_string(),
            sequence: 1,
            content_type: "text/plain".to_string(),
            payload: b"Hello, IPLD!".to_vec(),
        };

        // Act
        let cid = store.persist_content(event.clone(), None).unwrap();

        // Assert
        assert_eq!(store.events.len(), 1);
        assert_eq!(store.content_chains.get("doc_1").unwrap().len(), 1);
        
        // Verify content retrieval
        let retrieved = store.retrieve_content(&cid);
        assert_eq!(retrieved, Some(b"Hello, IPLD!".to_vec()));
        
        let (stored_event, stored_cid, prev_cid) = &store.events[0];
        assert_eq!(stored_event.event_id, "ipld_evt_1");
        assert_eq!(stored_cid, &cid);
        assert_eq!(prev_cid, &None);
    }

    #[test]
    fn test_content_version_chain() {
        // Arrange
        let mut store = IPLDEventStore::new();
        let content_id = "doc_1";
        
        // Create a chain of content versions
        let versions = vec![
            ("DocumentCreated", b"Version 1".to_vec()),
            ("DocumentUpdated", b"Version 2".to_vec()),
            ("DocumentFinalized", b"Version 3 - Final".to_vec()),
        ];

        // Act
        let mut previous_cid = None;
        let mut cids = Vec::new();
        
        for (i, (event_type, content)) in versions.iter().enumerate() {
            let event = IPLDEventData {
                event_id: format!("evt_{i + 1}"),
                content_id: content_id.to_string(),
                event_type: event_type.to_string(),
                sequence: (i + 1) as u64,
                content_type: "text/plain".to_string(),
                payload: content.clone(),
            };
            
            let cid = store.persist_content(event, previous_cid).unwrap();
            cids.push(cid);
            previous_cid = Some(cid);
        }

        // Validate chain
        let (start_cid, end_cid, length) = store.validate_chain(content_id).unwrap();

        // Assert
        assert_eq!(start_cid, cids[0]);
        assert_eq!(end_cid, cids[2]);
        assert_eq!(length, 3);
        
        // Verify version history
        let version_list = store.list_content_versions(content_id);
        assert_eq!(version_list.len(), 3);
        assert_eq!(version_list[0].1, "DocumentCreated");
        assert_eq!(version_list[1].1, "DocumentUpdated");
        assert_eq!(version_list[2].1, "DocumentFinalized");
    }

    #[test]
    fn test_codec_registration_and_lookup() {
        // Arrange
        let mut store = IPLDEventStore::new();
        
        // Act - Register custom codec
        store.register_codec(
            "application/x-cim-workflow".to_string(),
            "cim-workflow-codec".to_string()
        );
        
        // Assert
        assert_eq!(
            store.get_codec_for_type("application/x-cim-workflow"), 
            Some("cim-workflow-codec")
        );
        
        // Verify default codecs still work
        assert_eq!(store.get_codec_for_type("application/json"), Some("dag-json"));
        assert_eq!(store.get_codec_for_type("unknown/type"), None);
    }

    #[test]
    fn test_multimedia_content_handling() {
        // Arrange
        let mut store = IPLDEventStore::new();
        
        // Test different content types
        let content_types = vec![
            ("image_1", "image/jpeg", vec![0xFF, 0xD8, 0xFF, 0xE0]), // JPEG header
            ("video_1", "video/mp4", vec![0x00, 0x00, 0x00, 0x20]),  // MP4 header
            ("audio_1", "audio/wav", vec![0x52, 0x49, 0x46, 0x46]),  // WAV header
        ];
        
        // Act & Assert
        for (content_id, content_type, payload) in content_types {
            let event = IPLDEventData {
                event_id: format!("{content_id}_created"),
                content_id: content_id.to_string(),
                event_type: "MediaUploaded".to_string(),
                sequence: 1,
                content_type: content_type.to_string(),
                payload: payload.clone(),
            };
            
            let cid = store.persist_content(event.clone(), None).unwrap();
            let retrieved = store.retrieve_content(&cid).unwrap();
            
            // Verify content matches
            assert_eq!(&retrieved[0..4], &payload[0..4]);
            
            // Verify codec selection
            assert_eq!(store.get_codec_for_type(content_type), Some("raw"));
        }
    }

    #[test]
    fn test_broken_chain_detection() {
        // Arrange
        let mut store = IPLDEventStore::new();
        let content_id = "doc_1";
        
        // Create first event properly
        let event1 = IPLDEventData {
            event_id: "evt_1".to_string(),
            content_id: content_id.to_string(),
            event_type: "Created".to_string(),
            sequence: 1,
            content_type: "text/plain".to_string(),
            payload: b"Content 1".to_vec(),
        };
        
        let _cid1 = store.persist_content(event1, None).unwrap();
        
        // Create second event with wrong chain
        let event2 = IPLDEventData {
            event_id: "evt_2".to_string(),
            content_id: content_id.to_string(),
            event_type: "Updated".to_string(),
            sequence: 2,
            content_type: "text/plain".to_string(),
            payload: b"Content 2".to_vec(),
        };
        
        // Create a fake CID for wrong chain
        let fake_bytes = b"fake_previous_content";
        let fake_hash = blake3::hash(fake_bytes);
        let fake_hash_bytes = fake_hash.as_bytes();
        
        // Create multihash manually
        let code = 0x1e; // BLAKE3-256
        let size = fake_hash_bytes.len() as u8;
        
        let mut fake_multihash_bytes = Vec::new();
        fake_multihash_bytes.push(code);
        fake_multihash_bytes.push(size);
        fake_multihash_bytes.extend_from_slice(fake_hash_bytes);
        
        let fake_mh = multihash::Multihash::from_bytes(&fake_multihash_bytes).unwrap();
        let wrong_cid = Cid::new_v1(0x71, fake_mh);
        
        // Calculate correct CID for event2
        let event_bytes = serde_json::to_vec(&event2).unwrap();
        let hash = blake3::hash(&event_bytes);
        let hash_bytes = hash.as_bytes();
        
        let mut multihash_bytes = Vec::new();
        multihash_bytes.push(code);
        multihash_bytes.push(size);
        multihash_bytes.extend_from_slice(hash_bytes);
        
        let mh = multihash::Multihash::from_bytes(&multihash_bytes).unwrap();
        let cid2 = Cid::new_v1(0x71, mh);
        
        store.events.push((event2, cid2, Some(wrong_cid)));
        store.content_chains.get_mut(content_id).unwrap().push(1);

        // Act
        let result = store.validate_chain(content_id);

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chain broken at position 1"));
    }
} 