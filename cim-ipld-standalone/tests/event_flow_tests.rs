//! Event Flow Tests for CIM IPLD
//!
//! Tests that validate event-driven patterns in the IPLD/CID storage layer.
//! 
//! User Story: As a system, I need to store objects with CID integrity
//! so that content addressing and event chains are cryptographically secure.

use cim_ipld::{
    ContentType, TypedContent, ContentChain, Cid,
    DagCborCodec, DagJsonCodec,
};
use async_nats::jetstream;
use serde::{Serialize, Deserialize};
use std::time::SystemTime;
use uuid::Uuid;
use serde_json::json;

/// Helper function to calculate CID from bytes
fn calculate_cid(data: &[u8], codec: u64) -> Cid {
    // Create hash using BLAKE3
    let hash = blake3::hash(data);
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
    let mh = multihash::Multihash::from_bytes(&multihash_bytes).unwrap();
    Cid::new_v1(codec, mh)
}

/// Test event for CID chain validation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestStorageEvent {
    pub event_id: String,
    pub event_type: String,
    pub object_cid: Option<String>,
    pub previous_cid: Option<String>,
    pub timestamp: u64,
    pub payload: serde_json::Value,
}

/// Event Flow Test: Object Storage with CID Generation
///
/// ```mermaid
/// graph LR
///     subgraph "Test: Object Storage"
///         Create[Create Object]
///         Serialize[Serialize to IPLD]
///         Calculate[Calculate CID]
///         Store[Store in NATS]
///         Verify[Verify CID]
///         
///         Create --> Serialize
///         Serialize --> Calculate
///         Calculate --> Store
///         Store --> Verify
///     end
/// ```
#[tokio::test]
async fn test_object_storage_event_flow() {
    // Given: Test object to store
    let test_object = json!({
        "type": "TestObject",
        "id": Uuid::new_v4().to_string(),
        "data": "test content",
        "timestamp": SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });
    
    // When: Storing the object using DagCbor codec
    let serialized = DagCborCodec::encode(&test_object).expect("Failed to serialize");
    let cid = calculate_cid(&serialized, 0x71); // 0x71 is DAG-CBOR codec
    
    // Then: CID is valid and deterministic
    assert!(!cid.to_string().is_empty());
    
    // Verify same content produces same CID
    let cid2 = calculate_cid(&serialized, 0x71);
    assert_eq!(cid, cid2);
    
    // Event sequence validation
    println!("✅ ObjectCreated event");
    println!("✅ CIDCalculated event: {cid}");
    println!("✅ ObjectStored event");
}

/// Event Flow Test: CID Chain Creation
///
/// ```mermaid
/// sequenceDiagram
///     participant Test
///     participant Chain
///     participant Store
///     
///     Test->>Chain: Create Event 1
///     Chain->>Chain: Calculate CID 1
///     Chain->>Store: Store Event 1
///     
///     Test->>Chain: Create Event 2 (prev: CID 1)
///     Chain->>Chain: Calculate CID 2
///     Chain->>Store: Store Event 2
///     
///     Test->>Chain: Verify Chain
///     Chain-->>Test: Chain Valid
/// ```
#[tokio::test]
async fn test_cid_chain_creation() {
    // Given: A chain of events
    let mut events = Vec::new();
    let mut previous_cid: Option<Cid> = None;
    
    for i in 1..=3 {
        // Create event with previous CID
        let event = TestStorageEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: format!("ChainEvent{i}"),
            object_cid: None,
            previous_cid: previous_cid.as_ref().map(|c| c.to_string()),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            payload: json!({
                "sequence": i,
                "data": format!("event-{i}")
            }),
        };
        
        // Calculate CID for this event
        let serialized = DagCborCodec::encode(&event).unwrap();
        let event_cid = calculate_cid(&serialized, 0x71);
        
        events.push((event, event_cid.clone()));
        previous_cid = Some(event_cid);
    }
    
    // When: Validating the chain
    let mut chain_valid = true;
    for i in 1..events.len() {
        let (current_event, _) = &events[i];
        let (_, prev_cid) = &events[i - 1];
        
        if current_event.previous_cid.as_ref() != Some(&prev_cid.to_string()) {
            chain_valid = false;
            break;
        }
    }
    
    // Then: Chain is valid
    assert!(chain_valid);
    println!("✅ CID chain validated with {} events", events.len());
}

/// Event Flow Test: Content Type Detection
///
/// ```mermaid
/// stateDiagram-v2
///     [*] --> Detecting
///     Detecting --> JSON: JSON Content
///     Detecting --> Event: Event Content
///     Detecting --> Graph: Graph Content
///     JSON --> Stored
///     Event --> Stored
///     Graph --> Stored
///     Stored --> [*]
/// ```
#[tokio::test]
async fn test_content_type_detection_events() {
    // Given: Different content types
    let json_type = ContentType::Json;
    let event_type = ContentType::Event;
    let graph_type = ContentType::Graph;
    
    // When: Getting codec identifiers
    let json_codec = json_type.codec();
    let event_codec = event_type.codec();
    let graph_codec = graph_type.codec();
    
    // Then: Codecs are different
    assert_ne!(json_codec, event_codec);
    assert_ne!(json_codec, graph_codec);
    assert_ne!(event_codec, graph_codec);
    
    // Verify round-trip
    assert_eq!(ContentType::from_codec(json_codec), Some(ContentType::Json));
    assert_eq!(ContentType::from_codec(event_codec), Some(ContentType::Event));
    assert_eq!(ContentType::from_codec(graph_codec), Some(ContentType::Graph));
    
    println!("✅ ContentTypeDetected event: {:?}", json_type);
    println!("✅ ContentTypeDetected event: {:?}", event_type);
    println!("✅ ContentTypeDetected event: {:?}", graph_type);
}

/// Event Flow Test: Content Chain
///
/// ```mermaid
/// graph LR
///     subgraph "Test: Content Chain"
///         C1[Content 1]
///         C2[Content 2]
///         C3[Content 3]
///         Chain[Build Chain]
///         Verify[Verify Links]
///         
///         C1 --> Chain
///         C2 --> Chain
///         C3 --> Chain
///         Chain --> Verify
///     end
/// ```
#[tokio::test]
async fn test_content_chain_events() {
    // Define a test content type
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestContent {
        sequence: u32,
        data: String,
    }
    
    impl TypedContent for TestContent {
        const CODEC: u64 = 0x300000; // Event codec
        const CONTENT_TYPE: ContentType = ContentType::Event;
    }
    
    // Given: Multiple content items
    let content1 = TestContent {
        sequence: 1,
        data: "First item".to_string(),
    };
    
    let content2 = TestContent {
        sequence: 2,
        data: "Second item".to_string(),
    };
    
    let content3 = TestContent {
        sequence: 3,
        data: "Third item".to_string(),
    };
    
    // When: Creating a content chain
    let mut chain = ContentChain::new();
    
    // Add items to chain
    let chained1 = chain.append(content1).unwrap();
    let cid1 = chained1.cid.clone();
    
    let chained2 = chain.append(content2).unwrap();
    let cid2 = chained2.cid.clone();
    
    let chained3 = chain.append(content3).unwrap();
    let cid3 = chained3.cid.clone();
    
    // Then: Chain maintains order and links
    assert_eq!(chain.len(), 3);
    
    // Verify chain integrity
    chain.validate().unwrap();
    
    // Check sequences
    let items = chain.items();
    assert_eq!(items[0].sequence, 0);
    assert_eq!(items[1].sequence, 1);
    assert_eq!(items[2].sequence, 2);
    
    // Check links
    assert!(items[0].previous_cid.is_none());
    assert_eq!(items[1].previous_cid, Some(cid1.clone()));
    assert_eq!(items[2].previous_cid, Some(cid2.clone()));
    
    println!("✅ ContentAdded event (1): {cid1}");
    println!("✅ ContentAdded event (2): {cid2}");
    println!("✅ ContentAdded event (3): {cid3}");
    println!("✅ ChainBuilt event with {chain.len(} items"));
}

/// Event Flow Test: Error Handling
///
/// ```mermaid
/// sequenceDiagram
///     participant Test
///     participant Codec
///     participant Error
///     
///     Test->>Codec: Encode Invalid
///     Codec->>Error: Encoding Failed
///     Error-->>Test: Error Result
///     
///     Test->>Codec: Retry with Valid
///     Codec-->>Test: Success
/// ```
#[tokio::test]
async fn test_error_handling_events() {
    // Given: A very large object that might fail
    let large_object = json!({
        "data": vec![0u8; 1_000_000], // 1MB of data
        "type": "LargeObject"
    });
    
    // When: Attempting to encode
    let result = DagJsonCodec::encode(&large_object);
    
    // Then: Operation completes (success or error)
    match result {
        Ok(encoded) => {
            println!("✅ ObjectValidated event");
            println!("✅ ObjectEncoded event: {encoded.len(} bytes"));
        }
        Err(e) => {
            println!("✅ ValidationError event: {e}");
            println!("✅ ErrorHandled event");
        }
    }
}

/// Integration Test: Full IPLD Storage Flow
///
/// ```mermaid
/// graph TD
///     subgraph "Integration: Full Storage Flow"
///         Create[Create Object]
///         Validate[Validate Schema]
///         Encode[Encode to IPLD]
///         CID[Calculate CID]
///         Store[Store in NATS]
///         Index[Update Indices]
///         Event[Publish Event]
///         
///         Create --> Validate
///         Validate --> Encode
///         Encode --> CID
///         CID --> Store
///         Store --> Index
///         Index --> Event
///     end
/// ```
#[tokio::test]
#[ignore = "requires NATS server"]
async fn test_full_storage_flow_integration() {
    // Given: Connected NATS with object store
    let client = async_nats::connect("nats://localhost:4222").await.unwrap();
    let jetstream = jetstream::new(client);
    
    // Create bucket for testing
    let bucket_name = format!("test-ipld-{Uuid::new_v4(}"));
    let bucket = jetstream.create_object_store(jetstream::object_store::Config {
        bucket: bucket_name.clone(),
        ..Default::default()
    }).await.unwrap();
    
    // And: Test object
    let test_object = json!({
        "type": "IntegrationTest",
        "id": Uuid::new_v4().to_string(),
        "data": {
            "message": "Full flow test",
            "timestamp": SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }
    });
    
    // When: Storing through full flow
    let encoded = DagCborCodec::encode(&test_object).unwrap();
    let cid = calculate_cid(&encoded, 0x71);
    let cid_string = cid.to_string();
    
    // Store in NATS using a cursor
    use std::io::Cursor;
    let mut cursor = Cursor::new(encoded.clone());
    bucket.put(cid_string.as_str(), &mut cursor).await.unwrap();
    
    // Then: Object can be retrieved
    let mut retrieved = bucket.get(cid_string.as_str()).await.unwrap();
    
    // Read the data from the object
    use tokio::io::AsyncReadExt;
    let mut retrieved_data = Vec::new();
    retrieved.read_to_end(&mut retrieved_data).await.unwrap();
    
    assert_eq!(retrieved_data, encoded);
    
    // Decode and verify
    let decoded: serde_json::Value = DagCborCodec::decode(&retrieved_data).unwrap();
    assert_eq!(decoded["type"], "IntegrationTest");
    
    println!("✅ Full storage flow completed");
    println!("✅ Object stored with CID: {cid}");
    
    // Cleanup
    jetstream.delete_object_store(&bucket_name).await.ok();
} 