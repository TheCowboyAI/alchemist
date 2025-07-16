//! Infrastructure Layer 1.2: CID Chain Tests for cim-ipld
//! 
//! User Story: As an event store, I need to maintain CID chains for cryptographic integrity
//!
//! Test Requirements:
//! - Verify CID calculation from content
//! - Verify CID chain linking
//! - Verify chain integrity validation
//! - Verify broken chain detection
//!
//! Event Sequence:
//! 1. CIDCalculated { content_hash, cid }
//! 2. ChainLinkCreated { current_cid, previous_cid }
//! 3. ChainValidated { start_cid, end_cid, length }
//! 4. ChainBroken { position, expected_cid, actual_cid }
//!
//! ```mermaid
//! graph LR
//!     A[Test Start] --> B[Calculate First CID]
//!     B --> C[CIDCalculated]
//!     C --> D[Create Chain Link]
//!     D --> E[ChainLinkCreated]
//!     E --> F[Add More Links]
//!     F --> G[Validate Chain]
//!     G --> H{Chain Valid?}
//!     H -->|Yes| I[ChainValidated]
//!     H -->|No| J[ChainBroken]
//!     I --> K[Test Success]
//!     J --> L[Test Failure]
//! ```

use std::collections::HashMap;

/// Mock CID type for IPLD testing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IPLDCid(String);

impl IPLDCid {
    pub fn from_content(content: &[u8], previous: Option<&IPLDCid>) -> Self {
        // IPLD-specific CID calculation
        let mut hash_input = Vec::new();
        
        // Add previous CID if it exists (for chaining)
        if let Some(prev) = previous {
            hash_input.extend_from_slice(prev.0.as_bytes());
        }
        
        // Add content
        hash_input.extend_from_slice(content);
        
        // Simple hash for testing
        let hash = hash_input.iter().fold(0u64, |acc, &b| {
            acc.wrapping_mul(31).wrapping_add(b as u64)
        });
        
        IPLDCid(format!("bafk{:064x}", hash))
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

/// IPLD chain event for testing
#[derive(Debug, Clone)]
pub struct ChainedContent {
    pub content_id: String,
    pub content: Vec<u8>,
    pub cid: IPLDCid,
    pub previous_cid: Option<IPLDCid>,
    pub sequence: u64,
}

/// CID chain validator
pub struct CIDChainValidator {
    chain: Vec<ChainedContent>,
}

impl CIDChainValidator {
    pub fn new() -> Self {
        Self {
            chain: Vec::new(),
        }
    }

    pub fn add_content(
        &mut self,
        content_id: String,
        content: Vec<u8>,
        previous_cid: Option<IPLDCid>,
    ) -> IPLDCid {
        let sequence = self.chain.len() as u64 + 1;
        let cid = IPLDCid::from_content(&content, previous_cid.as_ref());
        
        let chained_content = ChainedContent {
            content_id,
            content,
            cid: cid.clone(),
            previous_cid,
            sequence,
        };
        
        self.chain.push(chained_content);
        cid
    }

    pub fn validate_chain(&self) -> Result<(IPLDCid, IPLDCid, usize), String> {
        if self.chain.is_empty() {
            return Err("Chain is empty".to_string());
        }

        // Check first link has no previous
        if self.chain[0].previous_cid.is_some() {
            return Err("First link should not have previous CID".to_string());
        }

        // Validate each link
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            // Check previous CID matches
            if current.previous_cid.as_ref() != Some(&previous.cid) {
                return Err(format!("Chain broken at position {i}: expected previous CID {:?}, got {:?}", previous.cid, current.previous_cid));
            }

            // Recalculate CID to verify
            let expected_cid = IPLDCid::from_content(
                &current.content,
                Some(&previous.cid),
            );

            if current.cid != expected_cid {
                return Err(format!("Invalid CID at position {i}: expected {:?}, got {:?}", expected_cid, current.cid));
            }

            // Check sequence
            if current.sequence != previous.sequence + 1 {
                return Err(format!("Invalid sequence at position {i}: expected {previous.sequence + 1}, got {current.sequence}"));
            }
        }

        let start_cid = self.chain.first().unwrap().cid.clone();
        let end_cid = self.chain.last().unwrap().cid.clone();
        
        Ok((start_cid, end_cid, self.chain.len()))
    }

    pub fn get_chain_length(&self) -> usize {
        self.chain.len()
    }

    pub fn get_content_by_cid(&self, cid: &IPLDCid) -> Option<&ChainedContent> {
        self.chain.iter().find(|c| &c.cid == cid)
    }

    pub fn break_chain_at(&mut self, position: usize) -> Result<(), String> {
        if position >= self.chain.len() {
            return Err("Position out of bounds".to_string());
        }

        // Modify the CID at position to break the chain
        self.chain[position].cid = IPLDCid("broken_cid_12345".to_string());
        Ok(())
    }
}

/// IPLD content store with CID indexing
pub struct IPLDContentStore {
    content: HashMap<IPLDCid, Vec<u8>>,
    metadata: HashMap<IPLDCid, HashMap<String, String>>,
}

impl IPLDContentStore {
    pub fn new() -> Self {
        Self {
            content: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn store_with_cid(&mut self, cid: IPLDCid, content: Vec<u8>) -> Result<(), String> {
        if self.content.contains_key(&cid) {
            return Err(format!("Content with CID {:?} already exists", cid));
        }
        
        self.content.insert(cid, content);
        Ok(())
    }

    pub fn retrieve_by_cid(&self, cid: &IPLDCid) -> Option<&Vec<u8>> {
        self.content.get(cid)
    }

    pub fn add_metadata(&mut self, cid: &IPLDCid, key: String, value: String) {
        self.metadata
            .entry(cid.clone())
            .or_insert_with(HashMap::new)
            .insert(key, value);
    }

    pub fn get_metadata(&self, cid: &IPLDCid) -> Option<&HashMap<String, String>> {
        self.metadata.get(cid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cid_calculation() {
        // Arrange
        let content = b"test content for CID";
        
        // Act
        let cid = IPLDCid::from_content(content, None);
        
        // Assert
        assert!(cid.to_string().starts_with("bafk"));
        assert_eq!(cid.to_string().len(), 68); // "bafk" + 64 hex chars
        
        // Verify same content produces same CID
        let cid2 = IPLDCid::from_content(content, None);
        assert_eq!(cid, cid2);
    }

    #[test]
    fn test_cid_chain_creation() {
        // Arrange
        let mut validator = CIDChainValidator::new();
        
        // Act
        let cid1 = validator.add_content(
            "content_1".to_string(),
            b"first content".to_vec(),
            None,
        );
        
        let cid2 = validator.add_content(
            "content_2".to_string(),
            b"second content".to_vec(),
            Some(cid1.clone()),
        );
        
        let cid3 = validator.add_content(
            "content_3".to_string(),
            b"third content".to_vec(),
            Some(cid2.clone()),
        );
        
        // Assert
        assert_eq!(validator.get_chain_length(), 3);
        
        // Verify chain validation
        let result = validator.validate_chain();
        assert!(result.is_ok());
        
        let (start_cid, end_cid, length) = result.unwrap();
        assert_eq!(start_cid, cid1);
        assert_eq!(end_cid, cid3);
        assert_eq!(length, 3);
    }

    #[test]
    fn test_broken_chain_detection() {
        // Arrange
        let mut validator = CIDChainValidator::new();
        
        // Create a valid chain
        let cid1 = validator.add_content(
            "content_1".to_string(),
            b"first".to_vec(),
            None,
        );
        
        let _cid2 = validator.add_content(
            "content_2".to_string(),
            b"second".to_vec(),
            Some(cid1),
        );
        
        let _cid3 = validator.add_content(
            "content_3".to_string(),
            b"third".to_vec(),
            Some(IPLDCid("wrong_cid".to_string())), // Wrong previous CID
        );
        
        // Act
        let result = validator.validate_chain();
        
        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chain broken at position 2"));
    }

    #[test]
    fn test_content_store_with_cid() {
        // Arrange
        let mut store = IPLDContentStore::new();
        let content = b"store this content".to_vec();
        let cid = IPLDCid::from_content(&content, None);
        
        // Act
        store.store_with_cid(cid.clone(), content.clone()).unwrap();
        
        // Assert
        let retrieved = store.retrieve_by_cid(&cid);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &content);
    }

    #[test]
    fn test_duplicate_cid_rejection() {
        // Arrange
        let mut store = IPLDContentStore::new();
        let content = b"duplicate content".to_vec();
        let cid = IPLDCid::from_content(&content, None);
        
        // Act
        store.store_with_cid(cid.clone(), content.clone()).unwrap();
        let result = store.store_with_cid(cid, content);
        
        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_metadata_storage() {
        // Arrange
        let mut store = IPLDContentStore::new();
        let content = b"content with metadata".to_vec();
        let cid = IPLDCid::from_content(&content, None);
        
        // Act
        store.store_with_cid(cid.clone(), content).unwrap();
        store.add_metadata(&cid, "content-type".to_string(), "text/plain".to_string());
        store.add_metadata(&cid, "author".to_string(), "test-user".to_string());
        
        // Assert
        let metadata = store.get_metadata(&cid).unwrap();
        assert_eq!(metadata.get("content-type").unwrap(), "text/plain");
        assert_eq!(metadata.get("author").unwrap(), "test-user");
    }

    #[test]
    fn test_chain_integrity_after_modification() {
        // Arrange
        let mut validator = CIDChainValidator::new();
        
        // Create chain
        validator.add_content("c1".to_string(), b"content1".to_vec(), None);
        validator.add_content("c2".to_string(), b"content2".to_vec(), 
            Some(validator.chain[0].cid.clone()));
        validator.add_content("c3".to_string(), b"content3".to_vec(),
            Some(validator.chain[1].cid.clone()));
        
        // Verify valid chain
        assert!(validator.validate_chain().is_ok());
        
        // Act - break the chain
        validator.break_chain_at(1).unwrap();
        
        // Assert
        let result = validator.validate_chain();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid CID at position 1"));
    }

    #[test]
    fn test_content_retrieval_by_cid() {
        // Arrange
        let mut validator = CIDChainValidator::new();
        
        let cid1 = validator.add_content(
            "doc1".to_string(),
            b"document one".to_vec(),
            None,
        );
        
        let cid2 = validator.add_content(
            "doc2".to_string(),
            b"document two".to_vec(),
            Some(cid1.clone()),
        );
        
        // Act
        let content1 = validator.get_content_by_cid(&cid1);
        let content2 = validator.get_content_by_cid(&cid2);
        let missing = validator.get_content_by_cid(&IPLDCid("missing".to_string()));
        
        // Assert
        assert!(content1.is_some());
        assert_eq!(content1.unwrap().content_id, "doc1");
        
        assert!(content2.is_some());
        assert_eq!(content2.unwrap().content_id, "doc2");
        
        assert!(missing.is_none());
    }
} 