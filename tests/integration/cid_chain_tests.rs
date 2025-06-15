//! CID chain integrity tests

use cim_domain::infrastructure::{ChainedEvent, EventChain};
use cim_domain::{DomainError, DomainResult};

#[tokio::test]
async fn test_cid_chain_creation() -> DomainResult<()> {
    // Arrange
    let mut chain = EventChain::new();

    // Act - Add events to chain
    let event1_data = b"First event in chain";
    let event1 = chain.add_event(event1_data)?;

    let event2_data = b"Second event in chain";
    let event2 = chain.add_event(event2_data)?;

    let event3_data = b"Third event in chain";
    let event3 = chain.add_event(event3_data)?;

    // Assert - Verify chain integrity
    assert!(
        event1.previous_cid.is_none(),
        "First event should have no previous CID"
    );
    assert_eq!(event2.previous_cid, Some(event1.event_cid.clone()));
    assert_eq!(event3.previous_cid, Some(event2.event_cid.clone()));

    // Verify chain validation
    let events = vec![event1, event2, event3];
    assert!(chain.validate_chain(&events).is_ok());

    Ok(())
}

#[tokio::test]
async fn test_cid_chain_tampering_detection() -> DomainResult<()> {
    // Arrange
    let mut chain = EventChain::new();

    // Create valid chain
    let event1 = chain.add_event(b"Event 1")?;
    let event2 = chain.add_event(b"Event 2")?;
    let mut event3 = chain.add_event(b"Event 3")?;

    // Act - Tamper with event data
    event3.data = b"Tampered event 3".to_vec();

    // Assert - Chain validation should fail
    let events = vec![event1, event2, event3];
    let validation_result = chain.validate_chain(&events);

    assert!(validation_result.is_err());
    match validation_result {
        Err(DomainError::Validation(msg)) => {
            assert!(msg.contains("CID mismatch"), "Expected CID mismatch error");
        }
        _ => panic!("Expected validation error for tampered chain"),
    }

    Ok(())
}

#[tokio::test]
async fn test_cid_chain_out_of_order_detection() -> DomainResult<()> {
    // Arrange
    let mut chain = EventChain::new();

    // Create valid chain
    let event1 = chain.add_event(b"Event 1")?;
    let event2 = chain.add_event(b"Event 2")?;
    let event3 = chain.add_event(b"Event 3")?;

    // Act - Reorder events
    let events = vec![event1, event3, event2]; // Wrong order

    // Assert - Chain validation should fail
    let validation_result = chain.validate_chain(&events);

    assert!(validation_result.is_err());
    match validation_result {
        Err(DomainError::Validation(msg)) => {
            assert!(msg.contains("Chain broken"), "Expected chain broken error");
        }
        _ => panic!("Expected validation error for out-of-order chain"),
    }

    Ok(())
}

#[tokio::test]
async fn test_cid_determinism() -> DomainResult<()> {
    // Arrange
    let data = b"Deterministic event data";
    let aggregate_id = cim_domain::GraphId::new();
    let event_type = "TestEvent";
    let timestamp = std::time::SystemTime::UNIX_EPOCH;

    // Act - Calculate CID multiple times with same inputs
    let cid1 = cim_domain::infrastructure::calculate_event_cid(
        data,
        None,
        &aggregate_id,
        event_type,
        timestamp,
    )?;

    let cid2 = cim_domain::infrastructure::calculate_event_cid(
        data,
        None,
        &aggregate_id,
        event_type,
        timestamp,
    )?;

    // Assert - CIDs should be identical
    assert_eq!(cid1, cid2, "CID calculation should be deterministic");

    Ok(())
}

#[tokio::test]
async fn test_event_chain_with_domain_events() -> DomainResult<()> {
    use cim_domain_graph::{ConceptualPoint, GraphDomainEvent, NodeType, Position3D, StepType};

    // Arrange
    let mut chain = EventChain::new();
    let graph_id = cim_domain::GraphId::new();

    // Create domain events
    let events = vec![
        DomainEvent::from(GraphDomainEvent::GraphCreated {
            graph_id,
            graph_type: cim_domain_graph::GraphType::WorkflowGraph,
            name: "Test Graph".to_string(),
            metadata: Default::default(),
        }),
        DomainEvent::from(GraphDomainEvent::NodeAdded {
            graph_id,
            node_id: cim_domain::NodeId::new(),
            node_type: NodeType::WorkflowStep {
                step_type: StepType::Start,
            },
            position: Position3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            conceptual_point: ConceptualPoint::default(),
            metadata: Default::default(),
        }),
        DomainEvent::from(GraphDomainEvent::NodeAdded {
            graph_id,
            node_id: cim_domain::NodeId::new(),
            node_type: NodeType::WorkflowStep {
                step_type: StepType::End,
            },
            position: Position3D {
                x: 10.0,
                y: 0.0,
                z: 0.0,
            },
            conceptual_point: ConceptualPoint::default(),
            metadata: Default::default(),
        }),
    ];

    // Act - Add events to chain
    let mut chained_events = Vec::new();
    for event in events {
        let serialized =
            serde_json::to_vec(&event).map_err(|e| DomainError::Serialization(e.to_string()))?;
        let chained = chain.add_event(&serialized)?;
        chained_events.push(chained);
    }

    // Assert - Verify chain integrity
    assert_eq!(chained_events.len(), 3);
    assert!(chain.validate_chain(&chained_events).is_ok());

    // Verify sequence
    assert!(chained_events[0].previous_cid.is_none());
    assert_eq!(
        chained_events[1].previous_cid,
        Some(chained_events[0].event_cid.clone())
    );
    assert_eq!(
        chained_events[2].previous_cid,
        Some(chained_events[1].event_cid.clone())
    );

    Ok(())
}

#[tokio::test]
async fn test_parallel_chain_branches() -> DomainResult<()> {
    // Test that multiple chains can branch from same event

    // Arrange
    let mut main_chain = EventChain::new();
    let mut branch_a = EventChain::new();
    let mut branch_b = EventChain::new();

    // Create common ancestor
    let common_event = main_chain.add_event(b"Common ancestor")?;

    // Act - Create two branches from common event
    branch_a.set_previous_cid(Some(common_event.event_cid.clone()));
    branch_b.set_previous_cid(Some(common_event.event_cid.clone()));

    let branch_a_event = branch_a.add_event(b"Branch A event")?;
    let branch_b_event = branch_b.add_event(b"Branch B event")?;

    // Assert - Both branches should reference common ancestor
    assert_eq!(
        branch_a_event.previous_cid,
        Some(common_event.event_cid.clone())
    );
    assert_eq!(
        branch_b_event.previous_cid,
        Some(common_event.event_cid.clone())
    );

    // But branches should have different CIDs
    assert_ne!(branch_a_event.event_cid, branch_b_event.event_cid);

    Ok(())
}
