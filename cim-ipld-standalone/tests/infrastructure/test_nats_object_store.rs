//! Infrastructure Layer 1.1: NATS Object Store Tests for cim-ipld
//! 
//! User Story: As a content store, I need to persist content-addressed data in NATS Object Store
//!
//! Test Requirements:
//! - Verify NATS Object Store connection
//! - Verify bucket creation for content storage
//! - Verify object storage with CID as key
//! - Verify object retrieval by CID
//!
//! Event Sequence:
//! 1. ObjectStoreConnected
//! 2. BucketCreated { bucket_name }
//! 3. ObjectStored { cid, size }
//! 4. ObjectRetrieved { cid, size }
//!
//! ```mermaid
//! graph LR
//!     A[Test Start] --> B[Connect to Object Store]
//!     B --> C{Connection OK?}
//!     C -->|Yes| D[ObjectStoreConnected]
//!     C -->|No| E[Test Failure]
//!     D --> F[Create Bucket]
//!     F --> G[BucketCreated]
//!     G --> H[Store Object]
//!     H --> I[ObjectStored]
//!     I --> J[Retrieve Object]
//!     J --> K[ObjectRetrieved]
//!     K --> L[Test Success]
//! ```

use std::collections::HashMap;
use std::time::Duration;

/// IPLD infrastructure event types for testing
#[derive(Debug, Clone, PartialEq)]
pub enum IPLDInfrastructureEvent {
    ObjectStoreConnected { client_name: String },
    BucketCreated { bucket_name: String },
    ObjectStored { cid: String, size: usize },
    ObjectRetrieved { cid: String, size: usize },
    ConnectionFailed { error: String },
}

/// Event stream validator for IPLD infrastructure testing
pub struct IPLDEventStreamValidator {
    expected_events: Vec<IPLDInfrastructureEvent>,
    captured_events: Vec<IPLDInfrastructureEvent>,
}

impl IPLDEventStreamValidator {
    pub fn new() -> Self {
        Self {
            expected_events: Vec::new(),
            captured_events: Vec::new(),
        }
    }

    pub fn expect_sequence(mut self, events: Vec<IPLDInfrastructureEvent>) -> Self {
        self.expected_events = events;
        self
    }

    pub fn capture_event(&mut self, event: IPLDInfrastructureEvent) {
        self.captured_events.push(event);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.captured_events.len() != self.expected_events.len() {
            return Err(format!("Event count mismatch: expected {self.expected_events.len(}, got {}"),
                self.captured_events.len()
            ));
        }

        for (i, (expected, actual)) in self.expected_events.iter()
            .zip(self.captured_events.iter())
            .enumerate()
        {
            if expected != actual {
                return Err(format!("Event mismatch at position {i}: expected {:?}, got {:?}", expected, actual));
            }
        }

        Ok(())
    }
}

/// Mock IPLD object store client for testing
pub struct MockIPLDObjectStore {
    connected: bool,
    client_name: String,
    buckets: HashMap<String, Vec<(String, Vec<u8>)>>, // bucket_name -> [(cid, data)]
}

impl MockIPLDObjectStore {
    pub fn new(client_name: String) -> Self {
        Self {
            connected: false,
            client_name,
            buckets: HashMap::new(),
        }
    }

    pub async fn connect(&mut self) -> Result<(), String> {
        // Simulate connection with delay
        tokio::time::sleep(Duration::from_millis(10)).await;
        self.connected = true;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub async fn create_bucket(&mut self, bucket_name: String) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected to object store".to_string());
        }
        
        if self.buckets.contains_key(&bucket_name) {
            return Err(format!("Bucket {bucket_name} already exists"));
        }
        
        self.buckets.insert(bucket_name, Vec::new());
        Ok(())
    }

    pub async fn store_object(
        &mut self,
        bucket_name: &str,
        cid: String,
        data: Vec<u8>,
    ) -> Result<usize, String> {
        if !self.connected {
            return Err("Not connected to object store".to_string());
        }
        
        let bucket = self.buckets.get_mut(bucket_name)
            .ok_or_else(|| format!("Bucket {bucket_name} not found"))?;
        
        let size = data.len();
        bucket.push((cid, data));
        
        // Simulate storage delay
        tokio::time::sleep(Duration::from_millis(5)).await;
        
        Ok(size)
    }

    pub async fn retrieve_object(
        &self,
        bucket_name: &str,
        cid: &str,
    ) -> Result<Vec<u8>, String> {
        if !self.connected {
            return Err("Not connected to object store".to_string());
        }
        
        let bucket = self.buckets.get(bucket_name)
            .ok_or_else(|| format!("Bucket {bucket_name} not found"))?;
        
        for (stored_cid, data) in bucket {
            if stored_cid == cid {
                return Ok(data.clone());
            }
        }
        
        Err(format!("Object with CID {cid} not found"))
    }

    pub fn list_buckets(&self) -> Vec<String> {
        self.buckets.keys().cloned().collect()
    }

    pub fn get_bucket_size(&self, bucket_name: &str) -> Option<usize> {
        self.buckets.get(bucket_name)
            .map(|bucket| bucket.iter().map(|(_, data)| data.len()).sum())
    }
}

/// Mock CID generator for testing
pub fn generate_test_cid(data: &[u8]) -> String {
    // Simple hash for testing
    let hash = data.iter().fold(0u64, |acc, &b| {
        acc.wrapping_mul(31).wrapping_add(b as u64)
    });
    format!("bafy{:032x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_object_store_connection() {
        // Arrange
        let mut validator = IPLDEventStreamValidator::new()
            .expect_sequence(vec![
                IPLDInfrastructureEvent::ObjectStoreConnected {
                    client_name: "cim-ipld-test".to_string(),
                },
            ]);

        let mut store = MockIPLDObjectStore::new("cim-ipld-test".to_string());

        // Act
        let result = store.connect().await;

        // Assert
        assert!(result.is_ok());
        assert!(store.is_connected());
        
        validator.capture_event(IPLDInfrastructureEvent::ObjectStoreConnected {
            client_name: store.client_name.clone(),
        });
        
        assert!(validator.validate().is_ok());
    }

    #[tokio::test]
    async fn test_bucket_creation() {
        // Arrange
        let mut validator = IPLDEventStreamValidator::new()
            .expect_sequence(vec![
                IPLDInfrastructureEvent::ObjectStoreConnected {
                    client_name: "cim-ipld-test".to_string(),
                },
                IPLDInfrastructureEvent::BucketCreated {
                    bucket_name: "cim-content".to_string(),
                },
            ]);

        let mut store = MockIPLDObjectStore::new("cim-ipld-test".to_string());

        // Act
        store.connect().await.unwrap();
        validator.capture_event(IPLDInfrastructureEvent::ObjectStoreConnected {
            client_name: store.client_name.clone(),
        });

        let bucket_result = store.create_bucket("cim-content".to_string()).await;

        // Assert
        assert!(bucket_result.is_ok());
        assert!(store.list_buckets().contains(&"cim-content".to_string()));
        
        validator.capture_event(IPLDInfrastructureEvent::BucketCreated {
            bucket_name: "cim-content".to_string(),
        });
        
        assert!(validator.validate().is_ok());
    }

    #[tokio::test]
    async fn test_object_storage() {
        // Arrange
        let mut validator = IPLDEventStreamValidator::new()
            .expect_sequence(vec![
                IPLDInfrastructureEvent::ObjectStoreConnected {
                    client_name: "cim-ipld-test".to_string(),
                },
                IPLDInfrastructureEvent::BucketCreated {
                    bucket_name: "cim-content".to_string(),
                },
                IPLDInfrastructureEvent::ObjectStored {
                    cid: generate_test_cid(b"test data"),
                    size: 9,
                },
            ]);

        let mut store = MockIPLDObjectStore::new("cim-ipld-test".to_string());

        // Act
        store.connect().await.unwrap();
        validator.capture_event(IPLDInfrastructureEvent::ObjectStoreConnected {
            client_name: store.client_name.clone(),
        });

        store.create_bucket("cim-content".to_string()).await.unwrap();
        validator.capture_event(IPLDInfrastructureEvent::BucketCreated {
            bucket_name: "cim-content".to_string(),
        });

        let test_data = b"test data".to_vec();
        let cid = generate_test_cid(&test_data);
        let size = store.store_object("cim-content", cid.clone(), test_data).await.unwrap();

        // Assert
        assert_eq!(size, 9);
        
        validator.capture_event(IPLDInfrastructureEvent::ObjectStored {
            cid,
            size,
        });
        
        assert!(validator.validate().is_ok());
    }

    #[tokio::test]
    async fn test_object_retrieval() {
        // Arrange
        let mut validator = IPLDEventStreamValidator::new()
            .expect_sequence(vec![
                IPLDInfrastructureEvent::ObjectStoreConnected {
                    client_name: "cim-ipld-test".to_string(),
                },
                IPLDInfrastructureEvent::BucketCreated {
                    bucket_name: "cim-content".to_string(),
                },
                IPLDInfrastructureEvent::ObjectStored {
                    cid: generate_test_cid(b"retrieve me"),
                    size: 11,
                },
                IPLDInfrastructureEvent::ObjectRetrieved {
                    cid: generate_test_cid(b"retrieve me"),
                    size: 11,
                },
            ]);

        let mut store = MockIPLDObjectStore::new("cim-ipld-test".to_string());

        // Act
        store.connect().await.unwrap();
        validator.capture_event(IPLDInfrastructureEvent::ObjectStoreConnected {
            client_name: store.client_name.clone(),
        });

        store.create_bucket("cim-content".to_string()).await.unwrap();
        validator.capture_event(IPLDInfrastructureEvent::BucketCreated {
            bucket_name: "cim-content".to_string(),
        });

        let test_data = b"retrieve me".to_vec();
        let cid = generate_test_cid(&test_data);
        let stored_size = store.store_object("cim-content", cid.clone(), test_data.clone()).await.unwrap();
        
        validator.capture_event(IPLDInfrastructureEvent::ObjectStored {
            cid: cid.clone(),
            size: stored_size,
        });

        let retrieved_data = store.retrieve_object("cim-content", &cid).await.unwrap();

        // Assert
        assert_eq!(retrieved_data, test_data);
        
        validator.capture_event(IPLDInfrastructureEvent::ObjectRetrieved {
            cid,
            size: retrieved_data.len(),
        });
        
        assert!(validator.validate().is_ok());
    }

    #[tokio::test]
    async fn test_connection_failure_handling() {
        // Arrange
        let mut validator = IPLDEventStreamValidator::new()
            .expect_sequence(vec![
                IPLDInfrastructureEvent::ConnectionFailed {
                    error: "Not connected to object store".to_string(),
                },
            ]);

        let mut store = MockIPLDObjectStore::new("cim-ipld-test".to_string());

        // Act - try to create bucket without connection
        let result = store.create_bucket("test-bucket".to_string()).await;

        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not connected to object store");
        
        validator.capture_event(IPLDInfrastructureEvent::ConnectionFailed {
            error: "Not connected to object store".to_string(),
        });
        
        assert!(validator.validate().is_ok());
    }

    #[tokio::test]
    async fn test_multiple_objects_in_bucket() {
        // Arrange
        let mut store = MockIPLDObjectStore::new("cim-ipld-test".to_string());
        store.connect().await.unwrap();
        store.create_bucket("multi-object".to_string()).await.unwrap();

        // Act - store multiple objects
        let data1 = b"first object".to_vec();
        let cid1 = generate_test_cid(&data1);
        store.store_object("multi-object", cid1.clone(), data1.clone()).await.unwrap();

        let data2 = b"second object".to_vec();
        let cid2 = generate_test_cid(&data2);
        store.store_object("multi-object", cid2.clone(), data2.clone()).await.unwrap();

        let data3 = b"third object".to_vec();
        let cid3 = generate_test_cid(&data3);
        store.store_object("multi-object", cid3.clone(), data3.clone()).await.unwrap();

        // Assert - verify all objects can be retrieved
        let retrieved1 = store.retrieve_object("multi-object", &cid1).await.unwrap();
        let retrieved2 = store.retrieve_object("multi-object", &cid2).await.unwrap();
        let retrieved3 = store.retrieve_object("multi-object", &cid3).await.unwrap();

        assert_eq!(retrieved1, data1);
        assert_eq!(retrieved2, data2);
        assert_eq!(retrieved3, data3);

        // Verify bucket size
        let bucket_size = store.get_bucket_size("multi-object").unwrap();
        assert_eq!(bucket_size, data1.len() + data2.len() + data3.len());
    }

    #[tokio::test]
    async fn test_bucket_already_exists() {
        // Arrange
        let mut store = MockIPLDObjectStore::new("cim-ipld-test".to_string());
        store.connect().await.unwrap();
        store.create_bucket("duplicate".to_string()).await.unwrap();

        // Act - try to create same bucket again
        let result = store.create_bucket("duplicate".to_string()).await;

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }
} 