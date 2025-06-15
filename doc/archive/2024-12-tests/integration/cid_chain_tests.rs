//! CID chain integrity tests
//!
//! These tests verify the cryptographic integrity of event chains
//! using content-addressed identifiers.

use super::fixtures::*;
use ia::domain::events::{
    DomainEvent, GraphEvent, NodeEvent,
    cid_chain::{ChainedEvent, EventChain},
};
use ia::domain::value_objects::{GraphId, GraphMetadata, NodeId, Position3D};
use ia::infrastructure::event_store::{DistributedEventStore, EventStore};
use std::time::SystemTime;

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_cid_chain_creation_and_validation() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_test_nats().await?;
    let event_store = Arc::new(DistributedEventStore::new(jetstream).await?);

    // Create a sequence of events
    let graph_id = GraphId::new();
    let events = vec![
        DomainEvent::GraphCreated {
            id: graph_id,
            metadata: GraphMetadata {
                name: "cid-test-graph".to_string(),
                ..Default::default()
            },
            timestamp: SystemTime::now(),
        },
        DomainEvent::NodeAdded {
            graph_id,
            node_id: NodeId::new(),
            content: "node-1".to_string(),
            position: Position3D {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            metadata: Default::default(),
            timestamp: SystemTime::now(),
        },
        DomainEvent::NodeAdded {
            graph_id,
            node_id: NodeId::new(),
            content: "node-2".to_string(),
            position: Position3D {
                x: 2.0,
                y: 0.0,
                z: 0.0,
            },
            metadata: Default::default(),
            timestamp: SystemTime::now(),
        },
    ];

    // Store events
    event_store
        .append_events(graph_id.to_string(), events.clone())
        .await?;

    // Retrieve and verify chain
    let stored_events = event_store.get_events(graph_id.to_string()).await?;
    assert_eq!(stored_events.len(), 3);

    // Convert to chained events
    let chain_events: Vec<ChainedEvent> = stored_events
        .iter()
        .map(|e| e.as_chained_event())
        .collect::<Result<Vec<_>, _>>()?;

    // Verify chain integrity
    TestAssertions::assert_cid_chain_valid(&chain_events)?;

    // Verify CIDs are deterministic
    let event_chain = EventChain::new();
    let mut rebuilt_chain = Vec::new();

    for (i, event) in events.iter().enumerate() {
        let previous_cid = if i > 0 {
            Some(rebuilt_chain[i - 1].event_cid.clone())
        } else {
            None
        };

        let chained = event_chain.add_event(event.clone(), previous_cid)?;
        rebuilt_chain.push(chained);
    }

    // CIDs should match
    for (original, rebuilt) in chain_events.iter().zip(rebuilt_chain.iter()) {
        assert_eq!(original.event_cid, rebuilt.event_cid);
    }

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_cid_chain_tampering_detection() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_test_nats().await?;

    // Create a valid chain
    let event_chain = EventChain::new();
    let graph_id = GraphId::new();

    let event1 = DomainEvent::GraphCreated {
        id: graph_id,
        metadata: GraphMetadata::default(),
        timestamp: SystemTime::now(),
    };

    let event2 = DomainEvent::NodeAdded {
        graph_id,
        node_id: NodeId::new(),
        content: "original".to_string(),
        position: Position3D::default(),
        metadata: Default::default(),
        timestamp: SystemTime::now(),
    };

    let chained1 = event_chain.add_event(event1, None)?;
    let chained2 = event_chain.add_event(event2.clone(), Some(chained1.event_cid.clone()))?;

    // Create a tampered event with same data but wrong previous CID
    let tampered_event = DomainEvent::NodeAdded {
        graph_id,
        node_id: NodeId::new(),
        content: "tampered".to_string(),
        position: Position3D::default(),
        metadata: Default::default(),
        timestamp: SystemTime::now(),
    };

    // Try to create chain with wrong previous CID
    let wrong_cid =
        cid::Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap();
    let tampered_chained = ChainedEvent {
        event_cid: event_chain.calculate_cid(&tampered_event)?,
        previous_cid: Some(wrong_cid),
        sequence: 2,
        timestamp: SystemTime::now(),
        payload: serde_json::to_value(&tampered_event)?,
    };

    // Validation should fail
    let chain = vec![chained1, tampered_chained];
    let result = TestAssertions::assert_cid_chain_valid(&chain);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("CID chain broken"));

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_parallel_event_chains() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_test_nats().await?;
    let event_store = Arc::new(DistributedEventStore::new(jetstream).await?);

    // Create multiple independent chains (different aggregates)
    let graph_ids: Vec<GraphId> = (0..3).map(|_| GraphId::new()).collect();

    // Create events for each graph in parallel
    let mut handles = Vec::new();

    for (i, graph_id) in graph_ids.iter().enumerate() {
        let event_store_clone = event_store.clone();
        let graph_id_clone = *graph_id;

        let handle = tokio::spawn(async move {
            let events = vec![
                DomainEvent::GraphCreated {
                    id: graph_id_clone,
                    metadata: GraphMetadata {
                        name: format!("parallel-graph-{}", i),
                        ..Default::default()
                    },
                    timestamp: SystemTime::now(),
                },
                DomainEvent::NodeAdded {
                    graph_id: graph_id_clone,
                    node_id: NodeId::new(),
                    content: format!("parallel-node-{}", i),
                    position: Position3D::default(),
                    metadata: Default::default(),
                    timestamp: SystemTime::now(),
                },
            ];

            event_store_clone
                .append_events(graph_id_clone.to_string(), events)
                .await
        });

        handles.push(handle);
    }

    // Wait for all chains to be created
    let results: Vec<_> = futures::future::join_all(handles).await;
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }

    // Verify each chain independently
    for graph_id in &graph_ids {
        let events = event_store.get_events(graph_id.to_string()).await?;
        assert_eq!(events.len(), 2);

        let chain_events: Vec<ChainedEvent> = events
            .iter()
            .map(|e| e.as_chained_event())
            .collect::<Result<Vec<_>, _>>()?;

        TestAssertions::assert_cid_chain_valid(&chain_events)?;
    }

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_event_replay_with_cid_verification() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_test_nats().await?;
    let event_store = Arc::new(DistributedEventStore::new(jetstream).await?);

    // Create a long chain of events
    let graph_id = GraphId::new();
    let mut events = vec![DomainEvent::GraphCreated {
        id: graph_id,
        metadata: GraphMetadata {
            name: "replay-test".to_string(),
            ..Default::default()
        },
        timestamp: SystemTime::now(),
    }];

    // Add many nodes
    for i in 0..50 {
        events.push(DomainEvent::NodeAdded {
            graph_id,
            node_id: NodeId::new(),
            content: format!("node-{}", i),
            position: Position3D {
                x: i as f32,
                y: 0.0,
                z: 0.0,
            },
            metadata: Default::default(),
            timestamp: SystemTime::now(),
        });
    }

    // Store all events
    event_store
        .append_events(graph_id.to_string(), events)
        .await?;

    // Replay from different points
    let all_events = event_store.get_events(graph_id.to_string()).await?;

    // Verify we can replay from any point and maintain chain integrity
    for start_index in [0, 10, 25, 40].iter() {
        let replay_slice = &all_events[*start_index..];

        if replay_slice.len() > 1 {
            let chain_events: Vec<ChainedEvent> = replay_slice
                .iter()
                .map(|e| e.as_chained_event())
                .collect::<Result<Vec<_>, _>>()?;

            // Verify sub-chain integrity
            for (i, event) in chain_events.iter().enumerate() {
                if i > 0 {
                    // Previous CID should match the previous event in the slice
                    assert_eq!(
                        event.previous_cid.as_ref(),
                        Some(&chain_events[i - 1].event_cid)
                    );
                }
            }
        }
    }

    Ok(())
}
