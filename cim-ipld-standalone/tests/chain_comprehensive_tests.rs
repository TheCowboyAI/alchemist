//! Comprehensive tests for content chain functionality

use cim_ipld::chain::{ChainedContent, ContentChain};
use cim_ipld::{ContentType, Error, TypedContent};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Test content types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct EventData {
    event_id: String,
    event_type: String,
    payload: serde_json::Value,
}

impl TypedContent for EventData {
    const CODEC: u64 = 0x300001;
    const CONTENT_TYPE: ContentType = ContentType::Event;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct BlockData {
    height: u64,
    transactions: Vec<String>,
    miner: String,
}

impl TypedContent for BlockData {
    const CODEC: u64 = 0x300002;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x300002);
}

// ============================================================================
// Test: Chain Fork Detection and Resolution
// ============================================================================

#[test]
fn test_chain_fork_detection() {
    // Given: A chain with multiple blocks
    let mut main_chain = ContentChain::new();

    let genesis = BlockData {
        height: 0,
        transactions: vec!["genesis_tx".to_string()],
        miner: "system".to_string(),
    };

    let block1 = BlockData {
        height: 1,
        transactions: vec!["tx1".to_string(), "tx2".to_string()],
        miner: "miner1".to_string(),
    };

    // Create main chain
    let genesis_item = main_chain.append(genesis.clone()).unwrap();
    let genesis_cid = genesis_item.cid.clone();
    let block1_item = main_chain.append(block1).unwrap();
    let block1_cid = block1_item.cid.clone();

    // Create a forked chain from genesis
    let mut fork_chain = ContentChain::new();

    // Manually create genesis with same CID
    let genesis_fork = ChainedContent::new(genesis, None).unwrap();
    assert_eq!(genesis_fork.cid, genesis_cid); // Same genesis

    // Different block at height 1
    let fork_block1 = BlockData {
        height: 1,
        transactions: vec!["fork_tx1".to_string()],
        miner: "miner2".to_string(),
    };

    // When: Creating fork
    fork_chain.append(genesis_fork.content).unwrap();
    let fork_item = fork_chain.append(fork_block1).unwrap();

    // Then: Fork is detected
    assert_ne!(fork_item.cid, block1_cid);
    assert_eq!(fork_item.sequence, block1_item.sequence);
    assert_eq!(fork_chain.len(), main_chain.len());
}

// ============================================================================
// Test: Chain Reorganization
// ============================================================================

#[test]
fn test_chain_reorganization() {
    // Given: Two competing chains
    let mut chain_a = ContentChain::new();
    let mut chain_b = ContentChain::new();

    // Common genesis
    let genesis = BlockData {
        height: 0,
        transactions: vec!["genesis".to_string()],
        miner: "system".to_string(),
    };

    chain_a.append(genesis.clone()).unwrap();
    chain_b.append(genesis).unwrap();

    // Chain A: 3 blocks
    for i in 1..=3 {
        chain_a
            .append(BlockData {
                height: i,
                transactions: vec![format!("a_tx{i}")],
                miner: "miner_a".to_string(),
            })
            .unwrap();
    }

    // Chain B: 4 blocks (longer chain wins)
    for i in 1..=4 {
        chain_b
            .append(BlockData {
                height: i,
                transactions: vec![format!("b_tx{i}")],
                miner: "miner_b".to_string(),
            })
            .unwrap();
    }

    // When: Comparing chains
    let chain_a_len = chain_a.len();
    let chain_b_len = chain_b.len();

    // Then: Chain B is longer
    assert!(chain_b_len > chain_a_len);

    // Verify both chains are valid
    chain_a.validate().unwrap();
    chain_b.validate().unwrap();
}

// ============================================================================
// Test: Chain Timestamp Validation
// ============================================================================

#[test]
fn test_chain_timestamp_ordering() {
    // Given: Events that should be time-ordered
    let mut event_chain = ContentChain::new();

    let mut timestamps = Vec::new();

    // When: Adding events
    for i in 0..5 {
        let event = EventData {
            event_id: format!("event-{i}"),
            event_type: "test".to_string(),
            payload: serde_json::json!({ "index": i }),
        };

        let item = event_chain.append(event).unwrap();
        timestamps.push(item.timestamp);

        // Small delay to ensure different timestamps
        std::thread::sleep(Duration::from_millis(10));
    }

    // Then: Timestamps are monotonically increasing
    for i in 1..timestamps.len() {
        assert!(timestamps[i] > timestamps[i - 1]);
    }
}

// ============================================================================
// Test: Chain Integrity with Large Data
// ============================================================================

#[test]
fn test_chain_with_large_payloads() {
    // Given: Large event payloads
    let mut chain = ContentChain::new();

    // Create large payload (1MB)
    let large_data: Vec<String> = (0..10000).map(|i| format!("data_item_{i}")).collect();

    let event = EventData {
        event_id: "large-event".to_string(),
        event_type: "bulk_data".to_string(),
        payload: serde_json::json!({ "items": large_data }),
    };

    // When: Adding to chain
    let item = chain.append(event.clone()).unwrap();
    let item_cid = item.cid.clone(); // Store CID before moving chain

    // Then: Chain remains valid
    chain.validate().unwrap();

    // And: CID is deterministic
    let recalculated = ChainedContent::new(event, None).unwrap();
    assert_eq!(item_cid, recalculated.cid);
}

// ============================================================================
// Test: Chain Recovery from Partial Data
// ============================================================================

#[test]
fn test_chain_recovery() {
    // Given: A chain with some items
    let mut original_chain = ContentChain::new();
    let mut cids = Vec::new();

    for i in 0..10 {
        let event = EventData {
            event_id: format!("event-{i}"),
            event_type: "recovery_test".to_string(),
            payload: serde_json::json!({ "value": i }),
        };

        let item = original_chain.append(event).unwrap();
        cids.push(item.cid.clone());
    }

    // When: Getting items since midpoint
    let midpoint_cid = &cids[5];
    let items_since = original_chain.items_since(midpoint_cid).unwrap();

    // Then: Correct number of items returned
    assert_eq!(items_since.len(), 4); // Items 6, 7, 8, 9

    // And: Items are in correct order
    for (i, item) in items_since.iter().enumerate() {
        assert_eq!(item.sequence, 6 + i as u64);
    }
}

// ============================================================================
// Test: Chain Validation with Missing Links
// ============================================================================

#[test]
fn test_chain_missing_link_detection() {
    // Given: Items for a chain
    let event1 = EventData {
        event_id: "event-1".to_string(),
        event_type: "test".to_string(),
        payload: serde_json::json!({}),
    };

    let event2 = EventData {
        event_id: "event-2".to_string(),
        event_type: "test".to_string(),
        payload: serde_json::json!({}),
    };

    // Create first item
    let item1 = ChainedContent::new(event1, None).unwrap();

    // Create second item with wrong previous CID
    let mut item2 = ChainedContent::new(event2, Some(&item1)).unwrap();
    item2.previous_cid = Some("wrong-cid".to_string());

    // When: Validating chain link
    let result = item2.validate_chain(Some(&item1));

    // Then: Validation fails
    assert!(result.is_err());
    match result {
        Err(Error::ChainValidationError { expected, actual }) => {
            assert_eq!(expected, item1.cid);
            assert_eq!(actual, "wrong-cid");
        }
        _ => panic!("Expected ChainValidationError"),
    }
}

// ============================================================================
// Test: Chain Serialization and Deserialization
// ============================================================================

#[test]
fn test_chain_serialization() {
    // Given: A chained content item
    let event = EventData {
        event_id: "serialize-test".to_string(),
        event_type: "test".to_string(),
        payload: serde_json::json!({ "data": "test" }),
    };

    let chained = ChainedContent::new(event, None).unwrap();

    // When: Serializing to JSON
    let json = serde_json::to_string(&chained).unwrap();

    // And: Deserializing back
    let deserialized: ChainedContent<EventData> = serde_json::from_str(&json).unwrap();

    // Then: Content is preserved
    assert_eq!(chained.content, deserialized.content);
    assert_eq!(chained.cid, deserialized.cid);
    assert_eq!(chained.sequence, deserialized.sequence);

    // And: Chain validation still works
    deserialized.validate_chain(None).unwrap();
}

// ============================================================================
// Test: Chain Performance with Many Items
// ============================================================================

#[test]
fn test_chain_performance() {
    use std::time::Instant;

    // Given: A chain that will hold many items
    let mut chain = ContentChain::new();
    let item_count = 1000;

    // When: Adding many items
    let start = Instant::now();

    for i in 0..item_count {
        let event = EventData {
            event_id: format!("perf-{i}"),
            event_type: "performance".to_string(),
            payload: serde_json::json!({ "index": i }),
        };

        chain.append(event).unwrap();
    }

    let append_duration = start.elapsed();

    // Then: Performance is reasonable
    println!("Appended {item_count} items in {:?}", append_duration);
    assert!(append_duration.as_secs() < 5); // Should be much faster

    // When: Validating entire chain
    let validate_start = Instant::now();
    chain.validate().unwrap();
    let validate_duration = validate_start.elapsed();

    println!("Validated {item_count} items in {:?}", validate_duration);
    assert!(validate_duration.as_secs() < 2);
}

// ============================================================================
// Test: Chain with Different Content Types
// ============================================================================

#[test]
fn test_heterogeneous_chain_types() {
    // Test that we can have separate chains for different content types
    let mut event_chain = ContentChain::<EventData>::new();
    let mut block_chain = ContentChain::<BlockData>::new();

    // Add events
    let event = EventData {
        event_id: "evt-1".to_string(),
        event_type: "user_action".to_string(),
        payload: serde_json::json!({ "action": "login" }),
    };
    event_chain.append(event).unwrap();

    // Add blocks
    let block = BlockData {
        height: 1,
        transactions: vec!["tx1".to_string()],
        miner: "miner1".to_string(),
    };
    block_chain.append(block).unwrap();

    // Verify both chains are independent
    assert_eq!(event_chain.len(), 1);
    assert_eq!(block_chain.len(), 1);

    // Different content types have different codecs
    assert_ne!(EventData::CODEC, BlockData::CODEC);
}

// ============================================================================
// Test: Chain Pruning and Archival
// ============================================================================

#[test]
fn test_chain_pruning_simulation() {
    // Given: A chain with many items
    let mut chain = ContentChain::new();
    let total_items = 100;

    for i in 0..total_items {
        let event = EventData {
            event_id: format!("prune-{i}"),
            event_type: "prunable".to_string(),
            payload: serde_json::json!({ "index": i }),
        };
        chain.append(event).unwrap();
    }

    // When: Simulating pruning (keeping only recent items)
    let keep_recent = 20;
    let prune_point = chain.items()[total_items - keep_recent - 1].cid.clone();

    // Get items since prune point
    let recent_items = chain.items_since(&prune_point).unwrap();

    // Then: Correct number of items kept
    assert_eq!(recent_items.len(), keep_recent);

    // And: Items maintain chain integrity
    let mut prev: Option<&ChainedContent<EventData>> = None;
    for item in recent_items {
        if let Some(p) = prev {
            assert_eq!(item.sequence, p.sequence + 1);
        }
        prev = Some(item);
    }
}
