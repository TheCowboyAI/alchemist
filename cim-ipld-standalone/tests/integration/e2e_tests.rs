//! End-to-end integration tests for CIM-IPLD
//!
//! Tests complete workflows and integration with CIM components.
//!
//! ## Test Scenarios
//!
//! ```mermaid
//! graph TD
//!     A[E2E Tests] --> B[Complete Workflow]
//!     A --> C[Event Sourcing]
//!     A --> D[Multi-Domain]
//!     A --> E[Performance]
//!
//!     B --> B1[Store-Chain-Retrieve]
//!     B --> B2[Migration-Transform-Query]
//!     B --> B3[Backup-Restore]
//!
//!     C --> C1[Event Storage]
//!     C --> C2[Event Replay]
//!     C --> C3[State Reconstruction]
//!
//!     D --> D1[Cross-Domain Events]
//!     D --> D2[Domain Boundaries]
//!     D --> D3[Integration Points]
//!
//!     E --> E1[Load Testing]
//!     E --> E2[Stress Testing]
//!     E --> E3[Scalability Testing]
//! ```

use cim_ipld::*;
use cim_ipld::object_store::NatsObjectStore;
use cim_ipld::chain::{ChainedContent, ContentChain};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use futures::stream::{self, StreamExt};

// Import common test utilities from parent
#[path = "../common/mod.rs"]
mod common;
use common::*;

/// Domain event for event sourcing tests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
enum DomainEvent {
    UserCreated {
        user_id: String,
        username: String,
        email: String,
        timestamp: String,
    },
    UserUpdated {
        user_id: String,
        changes: HashMap<String, String>,
        timestamp: String,
    },
    OrderPlaced {
        order_id: String,
        user_id: String,
        items: Vec<OrderItem>,
        total: f64,
        timestamp: String,
    },
    PaymentProcessed {
        payment_id: String,
        order_id: String,
        amount: f64,
        status: String,
        timestamp: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct OrderItem {
    product_id: String,
    quantity: u32,
    price: f64,
}

/// Aggregate state rebuilt from events
#[derive(Debug, Clone, Default)]
struct UserAggregate {
    user_id: String,
    username: String,
    email: String,
    version: u64,
    last_updated: String,
}

impl UserAggregate {
    fn apply_event(&mut self, event: &DomainEvent) {
        match event {
            DomainEvent::UserCreated { user_id, username, email, timestamp } => {
                self.user_id = user_id.clone();
                self.username = username.clone();
                self.email = email.clone();
                self.version = 1;
                self.last_updated = timestamp.clone();
            }
            DomainEvent::UserUpdated { user_id, changes, timestamp } => {
                if self.user_id == *user_id {
                    if let Some(username) = changes.get("username") {
                        self.username = username.clone();
                    }
                    if let Some(email) = changes.get("email") {
                        self.email = email.clone();
                    }
                    self.version += 1;
                    self.last_updated = timestamp.clone();
                }
            }
            _ => {} // Ignore non-user events
        }
    }
}

#[tokio::test]
#[ignore] // Requires NATS server
async fn test_complete_workflow() {
    /// Test complete CIM-IPLD workflow
    ///
    /// Given: Fresh CIM-IPLD instance
    /// When: Complete workflow executed
    /// Then: All components work together

    let context = TestContext::new().await
        .expect("Test context creation should succeed");

    // Step 1: Store individual content items
    let content1 = TestContent {
        data: b"First piece of content".to_vec(),
        metadata: vec![("type".to_string(), "document".to_string())].into_iter().collect(),
    };

    let content2 = TestContent {
        data: b"Second piece of content".to_vec(),
        metadata: vec![("type".to_string(), "document".to_string())].into_iter().collect(),
    };

    let cid1 = context.storage.put(&content1).await
        .expect("First content storage should succeed");
    let cid2 = context.storage.put(&content2).await
        .expect("Second content storage should succeed");

    // Step 2: Create a chain linking the content
    let mut chain = ContentChain::<TestContent>::new();

    let chain_item1 = ChainedContent {
        content: content1.clone(),
        previous_cid: None,
        timestamp: chrono::Utc::now(),
        metadata: HashMap::new(),
    };

    let chain_cid1 = chain.append(chain_item1)
        .expect("First chain append should succeed");

    let chain_item2 = ChainedContent {
        content: content2.clone(),
        previous_cid: Some(chain_cid1),
        timestamp: chrono::Utc::now(),
        metadata: HashMap::new(),
    };

    let chain_cid2 = chain.append(chain_item2)
        .expect("Second chain append should succeed");

    // Step 3: Validate the chain
    assert!(chain.validate().is_ok(), "Chain validation should succeed");

    // Step 4: Query chain items
    let recent_items = chain.items_since(chrono::Utc::now() - chrono::Duration::hours(1));
    assert_eq!(recent_items.len(), 2, "Should have 2 recent items");

    // Step 5: Store chain metadata
    let chain_metadata = TestContent {
        data: serde_json::to_vec(&serde_json::json!({
            "chain_id": "test_chain_001",
            "head_cid": chain_cid2.to_string(),
            "length": chain.len(),
            "created_at": chrono::Utc::now().to_rfc3339(),
        })).unwrap(),
        metadata: vec![("type".to_string(), "chain_metadata".to_string())].into_iter().collect(),
    };

    let metadata_cid = context.storage.put(&chain_metadata).await
        .expect("Chain metadata storage should succeed");

    // Step 6: Retrieve and verify everything
    let retrieved1: TestContent = context.storage.get(&cid1).await
        .expect("First content retrieval should succeed");
    assert_eq!(retrieved1.data, content1.data);

    let retrieved2: TestContent = context.storage.get(&cid2).await
        .expect("Second content retrieval should succeed");
    assert_eq!(retrieved2.data, content2.data);

    let retrieved_metadata: TestContent = context.storage.get(&metadata_cid).await
        .expect("Metadata retrieval should succeed");

    let metadata_json: serde_json::Value = serde_json::from_slice(&retrieved_metadata.data)
        .expect("Metadata deserialization should succeed");

    assert_eq!(metadata_json["chain_id"], "test_chain_001");
    assert_eq!(metadata_json["length"], 2);
}

#[tokio::test]
#[ignore] // Requires NATS server
async fn test_event_sourcing_integration() {
    /// Test event sourcing with CIM-IPLD
    ///
    /// Given: Event stream
    /// When: Events stored and replayed
    /// Then: State correctly reconstructed

    let context = TestContext::new().await
        .expect("Test context creation should succeed");

    // Create event stream
    let events = vec![
        DomainEvent::UserCreated {
            user_id: "user_001".to_string(),
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        DomainEvent::OrderPlaced {
            order_id: "order_001".to_string(),
            user_id: "user_001".to_string(),
            items: vec![
                OrderItem {
                    product_id: "prod_001".to_string(),
                    quantity: 2,
                    price: 29.99,
                },
            ],
            total: 59.98,
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        DomainEvent::UserUpdated {
            user_id: "user_001".to_string(),
            changes: vec![
                ("email".to_string(), "alice.smith@example.com".to_string()),
            ].into_iter().collect(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        DomainEvent::PaymentProcessed {
            payment_id: "pay_001".to_string(),
            order_id: "order_001".to_string(),
            amount: 59.98,
            status: "completed".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ];

    // Store events in chain
    let mut event_chain = ContentChain::<DomainEvent>::new();
    let mut event_cids = Vec::new();

    for event in &events {
        let chained_event = ChainedContent {
            content: event.clone(),
            previous_cid: event_cids.last().cloned(),
            timestamp: chrono::Utc::now(),
            metadata: vec![
                ("aggregate_id".to_string(), "user_001".to_string()),
            ].into_iter().collect(),
        };

        let cid = event_chain.append(chained_event)
            .expect("Event append should succeed");
        event_cids.push(cid);
    }

    // Validate event chain
    assert!(event_chain.validate().is_ok(), "Event chain should be valid");

    // Replay events to reconstruct state
    let mut user_aggregate = UserAggregate::default();

    for item in event_chain.iter() {
        user_aggregate.apply_event(&item.content);
    }

    // Verify reconstructed state
    assert_eq!(user_aggregate.user_id, "user_001");
    assert_eq!(user_aggregate.username, "alice");
    assert_eq!(user_aggregate.email, "alice.smith@example.com");
    assert_eq!(user_aggregate.version, 2);

    // Store snapshot of current state
    let snapshot = TestContent {
        data: serde_json::to_vec(&serde_json::json!({
            "aggregate_type": "User",
            "aggregate_id": user_aggregate.user_id,
            "version": user_aggregate.version,
            "state": {
                "username": user_aggregate.username,
                "email": user_aggregate.email,
            },
            "event_chain_head": event_cids.last().unwrap().to_string(),
        })).unwrap(),
        metadata: vec![
            ("type".to_string(), "aggregate_snapshot".to_string()),
        ].into_iter().collect(),
    };

    let snapshot_cid = context.storage.put(&snapshot).await
        .expect("Snapshot storage should succeed");

    // Verify snapshot retrieval
    let retrieved_snapshot: TestContent = context.storage.get(&snapshot_cid).await
        .expect("Snapshot retrieval should succeed");

    let snapshot_data: serde_json::Value = serde_json::from_slice(&retrieved_snapshot.data)
        .expect("Snapshot deserialization should succeed");

    assert_eq!(snapshot_data["aggregate_id"], "user_001");
    assert_eq!(snapshot_data["version"], 2);
}

#[tokio::test]
#[ignore] // Requires NATS server
async fn test_multi_domain_integration() {
    /// Test cross-domain integration
    ///
    /// Given: Multiple CIM domains
    /// When: Cross-domain operations
    /// Then: Proper integration maintained

    let context = TestContext::new().await
        .expect("Test context creation should succeed");

    // Simulate different domains
    let domains = vec!["users", "orders", "inventory", "payments"];
    let mut domain_chains: HashMap<String, ContentChain<DomainEvent>> = HashMap::new();

    // Initialize chains for each domain
    for domain in &domains {
        domain_chains.insert(domain.to_string(), ContentChain::new());
    }

    // User domain events
    let user_event = DomainEvent::UserCreated {
        user_id: "user_002".to_string(),
        username: "bob".to_string(),
        email: "bob@example.com".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let user_chain = domain_chains.get_mut("users").unwrap();
    let user_cid = user_chain.append(ChainedContent {
        content: user_event.clone(),
        previous_cid: None,
        timestamp: chrono::Utc::now(),
        metadata: HashMap::new(),
    }).expect("User event append should succeed");

    // Order domain events (triggered by user action)
    let order_event = DomainEvent::OrderPlaced {
        order_id: "order_002".to_string(),
        user_id: "user_002".to_string(),
        items: vec![
            OrderItem {
                product_id: "prod_002".to_string(),
                quantity: 1,
                price: 99.99,
            },
        ],
        total: 99.99,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let order_chain = domain_chains.get_mut("orders").unwrap();
    let order_cid = order_chain.append(ChainedContent {
        content: order_event.clone(),
        previous_cid: None,
        timestamp: chrono::Utc::now(),
        metadata: vec![
            ("caused_by".to_string(), user_cid.to_string()),
        ].into_iter().collect(),
    }).expect("Order event append should succeed");

    // Payment domain events (triggered by order)
    let payment_event = DomainEvent::PaymentProcessed {
        payment_id: "pay_002".to_string(),
        order_id: "order_002".to_string(),
        amount: 99.99,
        status: "pending".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let payment_chain = domain_chains.get_mut("payments").unwrap();
    let payment_cid = payment_chain.append(ChainedContent {
        content: payment_event,
        previous_cid: None,
        timestamp: chrono::Utc::now(),
        metadata: vec![
            ("caused_by".to_string(), order_cid.to_string()),
        ].into_iter().collect(),
    }).expect("Payment event append should succeed");

    // Create cross-domain index
    let cross_domain_index = TestContent {
        data: serde_json::to_vec(&serde_json::json!({
            "transaction_id": "txn_002",
            "domains_involved": domains,
            "event_flow": [
                {
                    "domain": "users",
                    "event_cid": user_cid.to_string(),
                    "event_type": "UserCreated",
                },
                {
                    "domain": "orders",
                    "event_cid": order_cid.to_string(),
                    "event_type": "OrderPlaced",
                },
                {
                    "domain": "payments",
                    "event_cid": payment_cid.to_string(),
                    "event_type": "PaymentProcessed",
                },
            ],
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })).unwrap(),
        metadata: vec![
            ("type".to_string(), "cross_domain_transaction".to_string()),
        ].into_iter().collect(),
    };

    let index_cid = context.storage.put(&cross_domain_index).await
        .expect("Cross-domain index storage should succeed");

    // Verify all chains are valid
    for (domain, chain) in &domain_chains {
        assert!(chain.validate().is_ok(), "{} chain should be valid", domain);
    }

    // Verify cross-domain traceability
    let retrieved_index: TestContent = context.storage.get(&index_cid).await
        .expect("Index retrieval should succeed");

    let index_data: serde_json::Value = serde_json::from_slice(&retrieved_index.data)
        .expect("Index deserialization should succeed");

    assert_eq!(index_data["transaction_id"], "txn_002");
    assert_eq!(index_data["event_flow"].as_array().unwrap().len(), 3);
}

#[tokio::test]
#[ignore] // Requires NATS server and extended runtime
async fn test_high_throughput_scenario() {
    /// Test high throughput scenario
    ///
    /// Given: Multiple concurrent clients
    /// When: Continuous read/write operations
    /// Then: Performance metrics within SLA

    let context = TestContext::new().await
        .expect("Test context creation should succeed");

    let num_clients = 10;
    let operations_per_client = 100;
    let storage = Arc::new(context.storage);

    // Shared metrics
    let write_times = Arc::new(RwLock::new(Vec::new()));
    let read_times = Arc::new(RwLock::new(Vec::new()));

    // Spawn concurrent clients
    let mut handles = Vec::new();

    for client_id in 0..num_clients {
        let storage_clone = storage.clone();
        let write_times_clone = write_times.clone();
        let read_times_clone = read_times.clone();

        let handle = tokio::spawn(async move {
            let mut cids = Vec::new();

            // Write phase
            for op in 0..operations_per_client {
                let content = TestContent {
                    data: format!("Client {client_id} operation {op}").into_bytes(),
                    metadata: vec![
                        ("client_id".to_string(), client_id.to_string()),
                        ("operation".to_string(), op.to_string()),
                    ].into_iter().collect(),
                };

                let start = std::time::Instant::now();
                match storage_clone.put(&content).await {
                    Ok(cid) => {
                        let duration = start.elapsed();
                        write_times_clone.write().await.push(duration);
                        cids.push(cid);
                    }
                    Err(e) => {
                        eprintln!("Write error for client {client_id}: {e}");
                    }
                }
            }

            // Read phase
            for cid in &cids {
                let start = std::time::Instant::now();
                match storage_clone.get::<TestContent>(cid).await {
                    Ok(_) => {
                        let duration = start.elapsed();
                        read_times_clone.write().await.push(duration);
                    }
                    Err(e) => {
                        eprintln!("Read error for client {client_id}: {e}");
                    }
                }
            }

            cids.len()
        });

        handles.push(handle);
    }

    // Wait for all clients to complete
    let results: Vec<usize> = futures::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    // Calculate metrics
    let total_operations = results.iter().sum::<usize>();
    let write_durations = write_times.read().await;
    let read_durations = read_times.read().await;

    if !write_durations.is_empty() {
        let avg_write = write_durations.iter().sum::<std::time::Duration>() / write_durations.len() as u32;
        let max_write = write_durations.iter().max().unwrap();

        println!("Write Performance:");
        println!("  Total writes: {write_durations.len(}"));
        println!("  Average write time: {:?}", avg_write);
        println!("  Max write time: {:?}", max_write);

        // Verify SLA
        assert!(avg_write < std::time::Duration::from_millis(50),
            "Average write time should be under 50ms");
    }

    if !read_durations.is_empty() {
        let avg_read = read_durations.iter().sum::<std::time::Duration>() / read_durations.len() as u32;
        let max_read = read_durations.iter().max().unwrap();

        println!("Read Performance:");
        println!("  Total reads: {read_durations.len(}"));
        println!("  Average read time: {:?}", avg_read);
        println!("  Max read time: {:?}", max_read);

        // Verify SLA
        assert!(avg_read < std::time::Duration::from_millis(10),
            "Average read time should be under 10ms");
    }

    assert_eq!(total_operations, num_clients * operations_per_client,
        "All operations should complete successfully");
}

#[tokio::test]
#[ignore] // Requires NATS server
async fn test_backup_restore_workflow() {
    /// Test backup and restore functionality
    ///
    /// Given: Active CIM-IPLD instance with data
    /// When: Backup created and restore performed
    /// Then: All data recovered successfully

    let context = TestContext::new().await
        .expect("Test context creation should succeed");

    // Create test data
    let mut original_cids = HashMap::new();
    let mut chains = HashMap::new();

    // Create content and chains
    for i in 0..5 {
        let mut chain = ContentChain::<TestContent>::new();

        for j in 0..10 {
            let content = TestContent {
                data: format!("Chain {i} Item {j}").into_bytes(),
                metadata: vec![
                    ("chain_id".to_string(), i.to_string()),
                    ("item_id".to_string(), j.to_string()),
                ].into_iter().collect(),
            };

            let cid = context.storage.put(&content).await
                .expect("Content storage should succeed");

            original_cids.insert(format!("{i}_{j}"), cid);

            let chained = ChainedContent {
                content,
                previous_cid: if j > 0 {
                    Some(original_cids[&format!("{i}_{j-1}")])
                } else {
                    None
                },
                timestamp: chrono::Utc::now(),
                metadata: HashMap::new(),
            };

            chain.append(chained).expect("Chain append should succeed");
        }

        chains.insert(i, chain);
    }

    // Create backup manifest
    let backup_manifest = serde_json::json!({
        "backup_id": uuid::Uuid::new_v4().to_string(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "content_count": original_cids.len(),
        "chain_count": chains.len(),
        "cids": original_cids.iter()
            .map(|(k, v)| (k, v.to_string()))
            .collect::<HashMap<_, _>>(),
    });

    let manifest_content = TestContent {
        data: serde_json::to_vec(&backup_manifest).unwrap(),
        metadata: vec![
            ("type".to_string(), "backup_manifest".to_string()),
        ].into_iter().collect(),
    };

    let manifest_cid = context.storage.put(&manifest_content).await
        .expect("Manifest storage should succeed");

    // Simulate restore process
    // In real scenario, this would be on a different instance

    // Retrieve backup manifest
    let retrieved_manifest: TestContent = context.storage.get(&manifest_cid).await
        .expect("Manifest retrieval should succeed");

    let manifest_data: serde_json::Value = serde_json::from_slice(&retrieved_manifest.data)
        .expect("Manifest deserialization should succeed");

    // Verify all content is accessible
    let cid_map = manifest_data["cids"].as_object().unwrap();
    let mut verified_count = 0;

    for (key, cid_str) in cid_map {
        let cid = Cid::try_from(cid_str.as_str().unwrap())
            .expect("CID parsing should succeed");

        let _content: TestContent = context.storage.get(&cid).await
            .expect(&format!("Content {key} should be retrievable"));

        verified_count += 1;
    }

    assert_eq!(verified_count, original_cids.len(),
        "All backed up content should be retrievable");

    // Verify chains can be reconstructed
    for (chain_id, chain) in &chains {
        assert!(chain.validate().is_ok(),
            "Chain {} should remain valid after backup/restore", chain_id);
    }
}
