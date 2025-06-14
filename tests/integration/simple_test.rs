//! Simple integration test to verify test infrastructure
//!
//! This test verifies that our integration test setup is working correctly
//! before running more complex tests.

use cim_domain::{DomainResult, NodeId, EdgeId};

#[tokio::test]
async fn test_basic_domain_types() -> DomainResult<()> {
    // Test that we can create basic domain types
    let node_id = NodeId::new();
    let edge_id = EdgeId::new();

    assert!(!node_id.as_uuid().is_nil());
    assert!(!edge_id.as_uuid().is_nil());
    assert_ne!(node_id.as_uuid(), edge_id.as_uuid());

    Ok(())
}

#[tokio::test]
async fn test_async_runtime() -> DomainResult<()> {
    // Test that async runtime is working
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(1),
        async {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            42
        }
    ).await;

    assert_eq!(result?, 42);
    Ok(())
}

#[test]
fn test_sync_functionality() {
    // Test that sync tests also work
    let node1 = NodeId::new();
    let node2 = NodeId::new();

    assert_ne!(node1, node2);
}
