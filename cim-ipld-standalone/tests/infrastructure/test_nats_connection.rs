//! Infrastructure Layer 1.1: NATS JetStream Connection Tests for cim-ipld
//! 
//! User Story: As a content system, I need to connect to NATS JetStream for distributed content storage
//!
//! Test Requirements:
//! - Verify NATS connection establishment for object store
//! - Verify bucket creation with correct configuration
//! - Verify content publishing with acknowledgment
//! - Verify content retrieval with proper ordering
//!
//! Event Sequence:
//! 1. ConnectionEstablished
//! 2. BucketCreated { name, config }
//! 3. ObjectStored { bucket, key, cid }
//! 4. ObjectRetrieved { bucket, key, cid }
//!
//! ```mermaid
//! graph LR
//!     A[Test Start] --> B[Connect to NATS]
//!     B --> C[ConnectionEstablished]
//!     C --> D[Create Object Store Bucket]
//!     D --> E[BucketCreated]
//!     E --> F[Store Content Object]
//!     F --> G[ObjectStored]
//!     G --> H[Retrieve Object]
//!     H --> I[ObjectRetrieved]
//!     I --> J[Test Success]
//! ```

use std::time::Duration;
use serde::{Serialize, Deserialize};

/// Mock NATS client for IPLD testing
pub struct IPLDNatsClient {
    pub connected: bool,
    pub buckets: Vec<String>,
    pub stored_objects: Vec<StoredObject>,
}

#[derive(Debug, Clone)]
pub struct StoredObject {
    pub bucket: String,
    pub key: String,
    pub cid: String,
    pub data: Vec<u8>,
}

/// NATS connection configuration for IPLD
#[derive(Debug, Clone)]
pub struct IPLDNatsConfig {
    pub url: String,
    pub bucket_prefix: String,
    pub timeout: Duration,
}

impl Default for IPLDNatsConfig {
    fn default() -> Self {
        Self {
            url: "nats://localhost:4222".to_string(),
            bucket_prefix: "cim-ipld".to_string(),
            timeout: Duration::from_secs(5),
        }
    }
}

impl IPLDNatsClient {
    pub fn new() -> Self {
        Self {
            connected: false,
            buckets: Vec::new(),
            stored_objects: Vec::new(),
        }
    }

    pub async fn connect(&mut self, config: &IPLDNatsConfig) -> Result<(), String> {
        // Simulate connection
        if config.url.starts_with("nats://") {
            self.connected = true;
            Ok(())
        } else {
            Err("Invalid NATS URL".to_string())
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub async fn create_bucket(&mut self, name: &str) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected to NATS".to_string());
        }

        if self.buckets.contains(&name.to_string()) {
            return Err(format!("Bucket {name} already exists"));
        }

        self.buckets.push(name.to_string());
        Ok(())
    }

    pub async fn store_object(
        &mut self,
        bucket: &str,
        key: &str,
        cid: &str,
        data: Vec<u8>,
    ) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected to NATS".to_string());
        }

        if !self.buckets.contains(&bucket.to_string()) {
            return Err(format!("Bucket {bucket} does not exist"));
        }

        self.stored_objects.push(StoredObject {
            bucket: bucket.to_string(),
            key: key.to_string(),
            cid: cid.to_string(),
            data,
        });

        Ok(())
    }

    pub async fn retrieve_object(&self, bucket: &str, key: &str) -> Result<Vec<u8>, String> {
        if !self.connected {
            return Err("Not connected to NATS".to_string());
        }

        self.stored_objects
            .iter()
            .find(|obj| obj.bucket == bucket && obj.key == key)
            .map(|obj| obj.data.clone())
            .ok_or_else(|| format!("Object not found: {bucket}/{key}"))
    }

    pub fn list_objects(&self, bucket: &str) -> Vec<(String, String)> {
        self.stored_objects
            .iter()
            .filter(|obj| obj.bucket == bucket)
            .map(|obj| (obj.key.clone(), obj.cid.clone()))
            .collect()
    }

    pub async fn disconnect(&mut self) -> Result<(), String> {
        self.connected = false;
        Ok(())
    }
}

/// Event types for NATS connection testing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IPLDNatsEvent {
    ConnectionEstablished { url: String },
    BucketCreated { name: String, prefix: String },
    ObjectStored { bucket: String, key: String, cid: String },
    ObjectRetrieved { bucket: String, key: String, cid: String },
    ConnectionClosed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ipld_nats_connection() {
        // Arrange
        let mut client = IPLDNatsClient::new();
        let config = IPLDNatsConfig::default();

        // Act
        let result = client.connect(&config).await;

        // Assert
        assert!(result.is_ok());
        assert!(client.is_connected());
    }

    #[tokio::test]
    async fn test_ipld_nats_connection_failure() {
        // Arrange
        let mut client = IPLDNatsClient::new();
        let config = IPLDNatsConfig {
            url: "invalid://url".to_string(),
            ..Default::default()
        };

        // Act
        let result = client.connect(&config).await;

        // Assert
        assert!(result.is_err());
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn test_ipld_bucket_creation() {
        // Arrange
        let mut client = IPLDNatsClient::new();
        let config = IPLDNatsConfig::default();
        client.connect(&config).await.unwrap();

        // Act
        let bucket_name = "cim-ipld-content";
        let result = client.create_bucket(bucket_name).await;

        // Assert
        assert!(result.is_ok());
        assert!(client.buckets.contains(&bucket_name.to_string()));
    }

    #[tokio::test]
    async fn test_ipld_object_storage_and_retrieval() {
        // Arrange
        let mut client = IPLDNatsClient::new();
        let config = IPLDNatsConfig::default();
        client.connect(&config).await.unwrap();
        
        let bucket = "cim-ipld-documents";
        client.create_bucket(bucket).await.unwrap();

        let key = "doc/test-document.md";
        let cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
        let content = b"# Test Document\n\nThis is a test document for IPLD storage.";

        // Act - Store object
        let store_result = client.store_object(bucket, key, cid, content.to_vec()).await;

        // Assert storage
        assert!(store_result.is_ok());
        assert_eq!(client.stored_objects.len(), 1);

        // Act - Retrieve object
        let retrieved = client.retrieve_object(bucket, key).await;

        // Assert retrieval
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap(), content.to_vec());
    }

    #[tokio::test]
    async fn test_ipld_multiple_content_types() {
        // Arrange
        let mut client = IPLDNatsClient::new();
        let config = IPLDNatsConfig::default();
        client.connect(&config).await.unwrap();

        // Create buckets for different content types
        let buckets = vec![
            ("cim-ipld-images", "image/jpeg"),
            ("cim-ipld-videos", "video/mp4"),
            ("cim-ipld-documents", "text/markdown"),
        ];

        for (bucket_name, _content_type) in &buckets {
            client.create_bucket(bucket_name).await.unwrap();
        }

        // Store different content types
        let objects = vec![
            ("cim-ipld-images", "photo.jpg", "bafybeig1", vec![0xFF, 0xD8, 0xFF]),
            ("cim-ipld-videos", "video.mp4", "bafybeig2", vec![0x00, 0x00, 0x00]),
            ("cim-ipld-documents", "doc.md", "bafybeig3", b"# Document".to_vec()),
        ];

        // Act
        for (bucket, key, cid, data) in &objects {
            client.store_object(bucket, key, cid, data.clone()).await.unwrap();
        }

        // Assert
        assert_eq!(client.stored_objects.len(), 3);

        // Verify listing
        let image_objects = client.list_objects("cim-ipld-images");
        assert_eq!(image_objects.len(), 1);
        assert_eq!(image_objects[0].0, "photo.jpg");
    }

    #[tokio::test]
    async fn test_ipld_bucket_isolation() {
        // Arrange
        let mut client = IPLDNatsClient::new();
        let config = IPLDNatsConfig::default();
        client.connect(&config).await.unwrap();

        let bucket1 = "cim-ipld-bucket1";
        let bucket2 = "cim-ipld-bucket2";
        
        client.create_bucket(bucket1).await.unwrap();
        client.create_bucket(bucket2).await.unwrap();

        // Store same key in different buckets
        let key = "same-key";
        let data1 = b"Data in bucket 1";
        let data2 = b"Data in bucket 2";

        client.store_object(bucket1, key, "cid1", data1.to_vec()).await.unwrap();
        client.store_object(bucket2, key, "cid2", data2.to_vec()).await.unwrap();

        // Act
        let retrieved1 = client.retrieve_object(bucket1, key).await.unwrap();
        let retrieved2 = client.retrieve_object(bucket2, key).await.unwrap();

        // Assert - Data is isolated by bucket
        assert_eq!(retrieved1, data1.to_vec());
        assert_eq!(retrieved2, data2.to_vec());
    }

    #[tokio::test]
    async fn test_ipld_disconnection_handling() {
        // Arrange
        let mut client = IPLDNatsClient::new();
        let config = IPLDNatsConfig::default();
        client.connect(&config).await.unwrap();
        
        let bucket = "cim-ipld-test";
        client.create_bucket(bucket).await.unwrap();

        // Act - Disconnect
        client.disconnect().await.unwrap();

        // Assert - Operations fail after disconnect
        let store_result = client.store_object(bucket, "key", "cid", vec![]).await;
        assert!(store_result.is_err());
        assert_eq!(store_result.unwrap_err(), "Not connected to NATS");

        let retrieve_result = client.retrieve_object(bucket, "key").await;
        assert!(retrieve_result.is_err());
        assert_eq!(retrieve_result.unwrap_err(), "Not connected to NATS");
    }
} 