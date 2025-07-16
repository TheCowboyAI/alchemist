//! Content-addressed chains for integrity
//!
//! This module provides chain-based content management with cryptographic
//! linking between content items, ensuring integrity and auditability.
//!
//! # Example
//!
//! ```
//! use cim_ipld::chain::ChainedContent;
//! use cim_ipld::{TypedContent, ContentType};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! struct MyContent {
//!     message: String,
//! }
//!
//! impl TypedContent for MyContent {
//!     const CODEC: u64 = 0x0129; // DAG-JSON
//!     const CONTENT_TYPE: ContentType = ContentType::Json;
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create the first item in a chain
//! let content1 = MyContent { message: "First".to_string() };
//! let chained1 = ChainedContent::new(content1, None)?;
//! 
//! // Add another item linked to the first
//! let content2 = MyContent { message: "Second".to_string() };
//! let chained2 = ChainedContent::new(content2, Some(&chained1))?;
//! 
//! // Verify chain properties
//! assert_eq!(chained1.sequence, 0);
//! assert_eq!(chained2.sequence, 1);
//! assert_eq!(chained2.previous_cid, Some(chained1.cid.clone()));
//! # Ok(())
//! # }
//! ```

use crate::{Cid, Error, Result, TypedContent};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// A content item with chain linking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainedContent<T> {
    /// The actual content
    pub content: T,

    /// Content identifier for this item
    pub cid: String, // Store as string for easier serialization

    /// CID of the previous item in the chain
    pub previous_cid: Option<String>,

    /// Sequence number in the chain
    pub sequence: u64,

    /// Timestamp when the item was chained
    pub timestamp: SystemTime,
}

impl<T: TypedContent> ChainedContent<T> {
    /// Create a new chained content item
    pub fn new(content: T, previous: Option<&ChainedContent<T>>) -> Result<Self> {
        let sequence = previous.map(|p| p.sequence + 1).unwrap_or(0);
        let previous_cid = previous.map(|p| p.cid.clone());
        let timestamp = SystemTime::now();

        // Create a temporary item to calculate CID
        let mut chained = Self {
            content,
            cid: String::new(), // Temporary
            previous_cid,
            sequence,
            timestamp,
        };

        // Calculate and set the actual CID
        chained.cid = chained.calculate_cid()?;

        Ok(chained)
    }

    /// Calculate the CID for this chained content
    fn calculate_cid(&self) -> Result<String> {
        // Create a deterministic representation for hashing
        let chain_data = ChainData {
            content: &self.content,
            previous_cid: &self.previous_cid,
            sequence: self.sequence,
            // Don't include timestamp in CID calculation for determinism
        };

        // Serialize to calculate CID
        let bytes = serde_json::to_vec(&chain_data)?;

        // Create hash using BLAKE3
        let hash = blake3::hash(&bytes);
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
            .map_err(|e| Error::MultihashError(e.to_string()))?;
        let cid = Cid::new_v1(T::CODEC, mh);

        Ok(cid.to_string())
    }

    /// Validate this item against a previous item
    pub fn validate_chain(&self, previous: Option<&ChainedContent<T>>) -> Result<()> {
        match (previous, &self.previous_cid) {
            // First item in chain
            (None, None) => {
                if self.sequence != 0 {
                    return Err(Error::SequenceValidationError {
                        expected: 0,
                        actual: self.sequence,
                    });
                }
            }
            // Continuing chain
            (Some(prev), Some(prev_cid)) => {
                // Validate CID link
                if prev.cid != *prev_cid {
                    return Err(Error::ChainValidationError {
                        expected: prev.cid.clone(),
                        actual: prev_cid.clone(),
                    });
                }
                // Validate sequence
                if self.sequence != prev.sequence + 1 {
                    return Err(Error::SequenceValidationError {
                        expected: prev.sequence + 1,
                        actual: self.sequence,
                    });
                }
            }
            // Mismatch
            _ => {
                return Err(Error::ChainValidationError {
                    expected: previous.map(|p| p.cid.clone()).unwrap_or_default(),
                    actual: self.previous_cid.clone().unwrap_or_default(),
                });
            }
        }

        // Verify our own CID
        let calculated_cid = self.calculate_cid()?;
        if calculated_cid != self.cid {
            return Err(Error::InvalidCid(format!("CID mismatch: expected {}, calculated {}", self.cid, calculated_cid)));
        }

        Ok(())
    }

    /// Parse a CID from string
    pub fn parse_cid(cid_str: &str) -> Result<Cid> {
        Cid::try_from(cid_str).map_err(|e| Error::InvalidCid(e.to_string()))
    }
}

/// Content used for CID calculation (excludes mutable fields)
#[derive(Serialize)]
struct ChainData<'a, T: Serialize> {
    content: &'a T,
    previous_cid: &'a Option<String>,
    sequence: u64,
}

/// A chain of content items with validation
#[derive(Debug, Clone)]
pub struct ContentChain<T: TypedContent> {
    items: Vec<ChainedContent<T>>,
}

impl<T: TypedContent> ContentChain<T> {
    /// Create a new empty chain
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add content to the chain
    pub fn append(&mut self, content: T) -> Result<&ChainedContent<T>> {
        let previous = self.items.last();
        let chained = ChainedContent::new(content, previous)?;

        // Validate the chain
        chained.validate_chain(previous)?;

        self.items.push(chained);
        Ok(self.items.last().unwrap())
    }

    /// Validate the entire chain
    pub fn validate(&self) -> Result<()> {
        let mut previous: Option<&ChainedContent<T>> = None;

        for item in &self.items {
            item.validate_chain(previous)?;
            previous = Some(item);
        }

        Ok(())
    }

    /// Get the chain head (latest item)
    pub fn head(&self) -> Option<&ChainedContent<T>> {
        self.items.last()
    }

    /// Get the chain length
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if chain is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get all items
    pub fn items(&self) -> &[ChainedContent<T>] {
        &self.items
    }

    /// Get items since a specific CID
    pub fn items_since(&self, cid: &str) -> Result<Vec<&ChainedContent<T>>> {
        // Find the item with the given CID
        let start_idx = self
            .items
            .iter()
            .position(|e| e.cid == cid)
            .ok_or_else(|| Error::InvalidCid(format!("CID not found in chain: {cid}")))?;

        // Return all items after that one
        Ok(self.items[start_idx + 1..].iter().collect())
    }
}

impl<T: TypedContent> Default for ContentChain<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ContentType;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestContent {
        id: String,
        data: String,
    }

    impl TypedContent for TestContent {
        const CODEC: u64 = 0x300000;
        const CONTENT_TYPE: ContentType = ContentType::Event;
    }

    #[test]
    fn test_chained_content_creation() {
        // Given
        let content = TestContent {
            id: "test-1".to_string(),
            data: "Test data".to_string(),
        };

        // When
        let chained = ChainedContent::new(content.clone(), None).unwrap();

        // Then
        assert_eq!(chained.sequence, 0);
        assert!(chained.previous_cid.is_none());
        assert!(!chained.cid.is_empty());

        // Verify CID format
        let cid = ChainedContent::<TestContent>::parse_cid(&chained.cid).unwrap();
        assert_eq!(cid.version(), cid::Version::V1);
    }

    #[test]
    fn test_content_chain_append() {
        // Given
        let mut chain = ContentChain::new();
        let content1 = TestContent {
            id: "test-1".to_string(),
            data: "Data 1".to_string(),
        };
        let content2 = TestContent {
            id: "test-2".to_string(),
            data: "Data 2".to_string(),
        };

        // When
        let chained1 = chain.append(content1).unwrap();
        let cid1 = chained1.cid.clone();

        let chained2 = chain.append(content2).unwrap();
        let sequence2 = chained2.sequence;
        let prev_cid2 = chained2.previous_cid.clone();

        // Then
        assert_eq!(chain.len(), 2);
        assert_eq!(sequence2, 1);
        assert_eq!(prev_cid2, Some(cid1));
    }

    #[test]
    fn test_chain_validation() {
        // Given
        let mut chain = ContentChain::new();

        // Add multiple items
        for i in 0..5 {
            let content = TestContent {
                id: format!("test-{i}"),
                data: format!("Data {i}"),
            };
            chain.append(content).unwrap();
        }

        // When/Then - validation should pass
        chain.validate().unwrap();

        // Verify sequence numbers
        for (i, item) in chain.items().iter().enumerate() {
            assert_eq!(item.sequence, i as u64);
        }
    }

    #[test]
    fn test_items_since() {
        // Given
        let mut chain = ContentChain::new();
        let mut cids = Vec::new();

        // Add items and track CIDs
        for i in 0..5 {
            let content = TestContent {
                id: format!("test-{i}"),
                data: format!("Data {i}"),
            };
            let chained = chain.append(content).unwrap();
            cids.push(chained.cid.clone());
        }

        // When - get items since index 2
        let items_since = chain.items_since(&cids[2]).unwrap();

        // Then
        assert_eq!(items_since.len(), 2); // Items 3 and 4
        assert_eq!(items_since[0].sequence, 3);
        assert_eq!(items_since[1].sequence, 4);
    }

    #[test]
    fn test_cid_determinism() {
        // Given - same content
        let content = TestContent {
            id: "test-determinism".to_string(),
            data: "Deterministic data".to_string(),
        };

        // When - create two chained items
        let chained1 = ChainedContent::new(content.clone(), None).unwrap();
        let chained2 = ChainedContent::new(content, None).unwrap();

        // Then - CIDs should be identical (deterministic)
        assert_eq!(chained1.cid, chained2.cid);
    }

    #[test]
    fn test_chain_tampering_detection() {
        // Given
        let mut chain = ContentChain::new();
        let content = TestContent {
            id: "test-tamper".to_string(),
            data: "Original data".to_string(),
        };
        chain.append(content).unwrap();

        // When - tamper with the item
        let mut tampered_item = chain.items[0].clone();
        tampered_item.sequence = 999; // Change sequence

        // Then - validation should fail
        let result = tampered_item.validate_chain(None);
        assert!(result.is_err());

        match result {
            Err(Error::SequenceValidationError { expected: 0, actual: 999 }) => {}, // Sequence mismatch detected
            Err(Error::InvalidCid(_)) => {}, // CID mismatch detected
            _ => panic!("Expected validation error"),
        }
    }
}
