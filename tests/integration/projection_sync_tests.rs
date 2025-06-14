//! Integration tests for projection synchronization
//!
//! These tests verify that projections stay synchronized with domain events:
//! 1. Events are published to NATS
//! 2. Multiple projections subscribe to events
//! 3. Projections update consistently
//! 4. Recovery from failures works correctly
//!
//! ```mermaid
//! graph LR
//!     A[Domain Event] --> B[NATS JetStream]
//!     B --> C[Projection A]
//!     B --> D[Projection B]
//!     B --> E[Projection C]
//!     C --> F[Read Model A]
//!     D --> G[Read Model B]
//!     E --> H[Read Model C]
//! ```

use crate::fixtures::{TestNatsServer, TestEventStore, create_test_graph, assertions::*};
use cim_domain::{DomainResult, GraphId, NodeId, DomainEvent};
use cim_domain_graph::{
    GraphAggregate, GraphDomainEvent, NodeType, Position3D,
    GraphSummaryProjection, NodeListProjection, EdgeListProjection,
    Projection, ProjectionSync,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test that multiple projections stay in sync
#[tokio::test]
async fn test_multiple_projections_sync() -> DomainResult<()> {
    // Arrange
    let nats = TestNatsServer::start().await?;
    let event_store = TestEventStore::with_nats(&nats).await?;

    // Create multiple projections
    let summary_projection = Arc::new(RwLock::new(GraphSummaryProjection::new()));
    let node_list_projection = Arc::new(RwLock::new(NodeListProjection::new()));
    let edge_list_projection = Arc::new(RwLock::new(EdgeListProjection::new()));

    // Subscribe projections to events
    let sync = ProjectionSync::new(event_store.clone());
    sync.subscribe_projection("summary", summary_projection.clone()).await?;
    sync.subscribe_projection("node_list", node_list_projection.clone()).await?;
    sync.subscribe_projection("edge_list", edge_list_projection.clone()).await?;

    // Act - Generate events
    let graph_id = GraphId::new();
    let node1 = NodeId::new();
    let node2 = NodeId::new();

    let events = vec![
        DomainEvent::Graph(GraphDomainEvent::NodeAdded {
            graph_id,
            node_id: node1,
            node_type: NodeType::Concept,
            position: Position3D::default(),
            conceptual_point: Default::default(),
            metadata: Default::default(),
        }),
        DomainEvent::Graph(GraphDomainEvent::NodeAdded {
            graph_id,
            node_id: node2,
            node_type: NodeType::Concept,
            position: Position3D::default(),
            conceptual_point: Default::default(),
            metadata: Default::default(),
        }),
        DomainEvent::Graph(GraphDomainEvent::EdgeConnected {
            graph_id,
            edge_id: cim_domain::EdgeId::new(),
            source: node1,
            target: node2,
            relationship: cim_domain_graph::EdgeRelationship::default(),
        }),
    ];

    // Publish events
    for event in events {
        event_store.append(event).await?;
    }

    // Wait for projections to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Assert - All projections should be in sync
    let summary = summary_projection.read().await;
    let graph_summary = summary.get_summary(&graph_id)?;
    assert_eq!(graph_summary.node_count, 2);
    assert_eq!(graph_summary.edge_count, 1);

    let node_list = node_list_projection.read().await;
    let nodes = node_list.get_nodes(&graph_id)?;
    assert_eq!(nodes.len(), 2);

    let edge_list = edge_list_projection.read().await;
    let edges = edge_list.get_edges(&graph_id)?;
    assert_eq!(edges.len(), 1);

    // Cleanup
    nats.cleanup().await?;

    Ok(())
}

/// Test projection recovery after failure
#[tokio::test]
async fn test_projection_recovery_after_failure() -> DomainResult<()> {
    // Arrange
    let nats = TestNatsServer::start().await?;
    let event_store = TestEventStore::with_nats(&nats).await?;
    let graph_id = GraphId::new();

    // Publish some events before projection starts
    let early_events = vec![
        create_node_added_event(graph_id, NodeId::new()),
        create_node_added_event(graph_id, NodeId::new()),
    ];

    for event in early_events {
        event_store.append(event).await?;
    }

    // Act - Start projection after events already published
    let projection = Arc::new(RwLock::new(NodeListProjection::new()));
    let sync = ProjectionSync::new(event_store.clone());

    // Subscribe with replay from beginning
    sync.subscribe_projection_with_replay("node_list", projection.clone(), 0).await?;

    // Wait for replay to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Publish more events
    let new_event = create_node_added_event(graph_id, NodeId::new());
    event_store.append(new_event).await?;

    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Assert - Should have all events including replayed ones
    let proj = projection.read().await;
    let nodes = proj.get_nodes(&graph_id)?;
    assert_eq!(nodes.len(), 3, "Should have 2 replayed + 1 new node");

    // Cleanup
    nats.cleanup().await?;

    Ok(())
}

/// Test projection checkpoint and resume
#[tokio::test]
async fn test_projection_checkpoint_resume() -> DomainResult<()> {
    // Arrange
    let nats = TestNatsServer::start().await?;
    let event_store = TestEventStore::with_nats(&nats).await?;
    let graph_id = GraphId::new();

    // First projection processes some events
    let projection1 = Arc::new(RwLock::new(NodeListProjection::new()));
    let sync1 = ProjectionSync::new(event_store.clone());
    sync1.subscribe_projection("node_list", projection1.clone()).await?;

    // Publish events
    for i in 0..5 {
        let event = create_node_added_event(graph_id, NodeId::new());
        event_store.append(event).await?;
    }

    // Wait and checkpoint
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let checkpoint = sync1.checkpoint("node_list").await?;

    // Stop first projection
    sync1.stop().await?;

    // Publish more events while projection is down
    for i in 0..3 {
        let event = create_node_added_event(graph_id, NodeId::new());
        event_store.append(event).await?;
    }

    // Act - Start new projection from checkpoint
    let projection2 = Arc::new(RwLock::new(NodeListProjection::new()));
    let sync2 = ProjectionSync::new(event_store.clone());
    sync2.subscribe_projection_from_checkpoint("node_list", projection2.clone(), checkpoint).await?;

    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Assert - Should only have events after checkpoint
    let proj = projection2.read().await;
    let nodes = proj.get_nodes(&graph_id)?;
    assert_eq!(nodes.len(), 3, "Should only have 3 events after checkpoint");

    // Cleanup
    nats.cleanup().await?;

    Ok(())
}

/// Test concurrent event processing doesn't corrupt projections
#[tokio::test]
async fn test_concurrent_event_processing() -> DomainResult<()> {
    // Arrange
    let nats = TestNatsServer::start().await?;
    let event_store = TestEventStore::with_nats(&nats).await?;
    let projection = Arc::new(RwLock::new(GraphSummaryProjection::new()));

    let sync = ProjectionSync::new(event_store.clone());
    sync.subscribe_projection("summary", projection.clone()).await?;

    // Act - Publish many events concurrently
    let handles: Vec<_> = (0..10).map(|i| {
        let store = event_store.clone();
        let graph_id = GraphId::new();

        tokio::spawn(async move {
            // Each task creates a graph with nodes and edges
            let node1 = NodeId::new();
            let node2 = NodeId::new();

            let events = vec![
                DomainEvent::Graph(GraphDomainEvent::NodeAdded {
                    graph_id,
                    node_id: node1,
                    node_type: NodeType::Concept,
                    position: Position3D::default(),
                    conceptual_point: Default::default(),
                    metadata: Default::default(),
                }),
                DomainEvent::Graph(GraphDomainEvent::NodeAdded {
                    graph_id,
                    node_id: node2,
                    node_type: NodeType::Concept,
                    position: Position3D::default(),
                    conceptual_point: Default::default(),
                    metadata: Default::default(),
                }),
                DomainEvent::Graph(GraphDomainEvent::EdgeConnected {
                    graph_id,
                    edge_id: cim_domain::EdgeId::new(),
                    source: node1,
                    target: node2,
                    relationship: cim_domain_graph::EdgeRelationship::default(),
                }),
            ];

            for event in events {
                store.append(event).await?;
            }

            Ok::<GraphId, cim_domain::DomainError>(graph_id)
        })
    }).collect();

    // Wait for all tasks
    let mut graph_ids = Vec::new();
    for handle in handles {
        graph_ids.push(handle.await??);
    }

    // Wait for projection processing
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Assert - Each graph should have correct counts
    let proj = projection.read().await;
    for graph_id in graph_ids {
        let summary = proj.get_summary(&graph_id)?;
        assert_eq!(summary.node_count, 2, "Each graph should have 2 nodes");
        assert_eq!(summary.edge_count, 1, "Each graph should have 1 edge");
    }

    // Cleanup
    nats.cleanup().await?;

    Ok(())
}

/// Test projection lag monitoring
#[tokio::test]
async fn test_projection_lag_monitoring() -> DomainResult<()> {
    // Arrange
    let nats = TestNatsServer::start().await?;
    let event_store = TestEventStore::with_nats(&nats).await?;

    // Create slow projection that simulates processing delay
    let projection = Arc::new(RwLock::new(SlowProjection::new(50))); // 50ms delay per event
    let sync = ProjectionSync::new(event_store.clone());
    sync.subscribe_projection("slow", projection.clone()).await?;

    // Act - Publish events rapidly
    let graph_id = GraphId::new();
    for i in 0..10 {
        let event = create_node_added_event(graph_id, NodeId::new());
        event_store.append(event).await?;
    }

    // Check lag immediately
    let lag1 = sync.get_projection_lag("slow").await?;
    assert!(lag1 > 0, "Should have lag immediately after publishing");

    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Check lag after processing
    let lag2 = sync.get_projection_lag("slow").await?;
    assert_eq!(lag2, 0, "Should have no lag after processing completes");

    // Cleanup
    nats.cleanup().await?;

    Ok(())
}

/// Test projection error handling
#[tokio::test]
async fn test_projection_error_handling() -> DomainResult<()> {
    // Arrange
    let nats = TestNatsServer::start().await?;
    let event_store = TestEventStore::with_nats(&nats).await?;

    // Create projection that fails on certain events
    let projection = Arc::new(RwLock::new(FailingProjection::new()));
    let sync = ProjectionSync::new(event_store.clone());
    sync.subscribe_projection_with_error_handler(
        "failing",
        projection.clone(),
        |error, event| {
            // Log error and continue
            eprintln!("Projection error: {} for event: {:?}", error, event);
            Ok(())
        }
    ).await?;

    // Act - Publish mix of good and bad events
    let graph_id = GraphId::new();
    let events = vec![
        create_node_added_event(graph_id, NodeId::new()), // Good
        create_failing_event(graph_id),                    // Bad
        create_node_added_event(graph_id, NodeId::new()), // Good
    ];

    for event in events {
        event_store.append(event).await?;
    }

    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Assert - Should process good events despite failures
    let proj = projection.read().await;
    assert_eq!(proj.successful_count(), 2, "Should process 2 good events");
    assert_eq!(proj.failed_count(), 1, "Should have 1 failed event");

    // Cleanup
    nats.cleanup().await?;

    Ok(())
}

// Helper types and functions

struct SlowProjection {
    delay_ms: u64,
    processed: Vec<DomainEvent>,
}

impl SlowProjection {
    fn new(delay_ms: u64) -> Self {
        Self {
            delay_ms,
            processed: Vec::new(),
        }
    }
}

#[async_trait::async_trait]
impl Projection for SlowProjection {
    async fn apply_event(&mut self, event: &DomainEvent) -> DomainResult<()> {
        tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
        self.processed.push(event.clone());
        Ok(())
    }
}

struct FailingProjection {
    successful: Vec<DomainEvent>,
    failed: Vec<DomainEvent>,
}

impl FailingProjection {
    fn new() -> Self {
        Self {
            successful: Vec::new(),
            failed: Vec::new(),
        }
    }

    fn successful_count(&self) -> usize {
        self.successful.len()
    }

    fn failed_count(&self) -> usize {
        self.failed.len()
    }
}

#[async_trait::async_trait]
impl Projection for FailingProjection {
    async fn apply_event(&mut self, event: &DomainEvent) -> DomainResult<()> {
        // Fail on events with specific metadata
        if let DomainEvent::Graph(GraphDomainEvent::NodeAdded { metadata, .. }) = event {
            if metadata.contains_key("fail") {
                self.failed.push(event.clone());
                return Err(cim_domain::DomainError::ValidationError(
                    "Simulated projection failure".to_string()
                ));
            }
        }

        self.successful.push(event.clone());
        Ok(())
    }
}

fn create_node_added_event(graph_id: GraphId, node_id: NodeId) -> DomainEvent {
    DomainEvent::Graph(GraphDomainEvent::NodeAdded {
        graph_id,
        node_id,
        node_type: NodeType::Concept,
        position: Position3D::default(),
        conceptual_point: Default::default(),
        metadata: Default::default(),
    })
}

fn create_failing_event(graph_id: GraphId) -> DomainEvent {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("fail".to_string(), "true".to_string());

    DomainEvent::Graph(GraphDomainEvent::NodeAdded {
        graph_id,
        node_id: NodeId::new(),
        node_type: NodeType::Concept,
        position: Position3D::default(),
        conceptual_point: Default::default(),
        metadata,
    })
}
