//! Integration tests for query handlers
//!
//! These tests verify that queries correctly retrieve data from projections:
//! 1. Events update projections
//! 2. Queries read from projections
//! 3. Results match expected state
//! 4. Performance is acceptable
//!
//! ```mermaid
//! graph LR
//!     A[Domain Events] --> B[Projections]
//!     C[Query] --> D[Query Handler]
//!     D --> B
//!     D --> E[Query Result]
//! ```

use crate::fixtures::{TestEventStore, create_test_graph, assertions::*};
use cim_domain::{DomainResult, GraphId, NodeId, DomainEvent};
use cim_domain_graph::{
    GraphAggregate, GraphDomainEvent, NodeType, StepType, Position3D,
    GraphSummaryProjection, NodeListProjection, EdgeListProjection,
    Projection, GraphQuery, QueryHandler,
};
use std::collections::HashMap;

/// Test finding nodes by type
#[tokio::test]
async fn test_find_nodes_by_type_query() -> DomainResult<()> {
    // Arrange - Create projection with different node types
    let mut projection = NodeListProjection::new();
    let graph_id = GraphId::new();

    // Add nodes of different types
    let events = vec![
        create_node_added_event(graph_id, NodeType::Concept),
        create_node_added_event(graph_id, NodeType::WorkflowStep {
            step_type: StepType::Process
        }),
        create_node_added_event(graph_id, NodeType::Concept),
        create_node_added_event(graph_id, NodeType::Decision {
            criteria: Default::default()
        }),
    ];

    for event in events {
        projection.apply_event(&event).await?;
    }

    // Act - Query for concept nodes
    let query = GraphQuery::FindNodesByType {
        graph_id,
        node_type: NodeType::Concept,
    };

    let result = QueryHandler::handle_query(query, &projection)?;

    // Assert
    match result {
        cim_domain_graph::QueryResult::NodeList(nodes) => {
            assert_eq!(nodes.len(), 2, "Should find 2 concept nodes");
            assert!(nodes.iter().all(|n| matches!(n.node_type, NodeType::Concept)));
        }
        _ => panic!("Expected NodeList result"),
    }

    Ok(())
}

/// Test graph summary query
#[tokio::test]
async fn test_graph_summary_query() -> DomainResult<()> {
    // Arrange
    let mut projection = GraphSummaryProjection::new();
    let graph_id = GraphId::new();

    // Add nodes and edges
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
        projection.apply_event(&event).await?;
    }

    // Act
    let query = GraphQuery::GetGraphSummary { graph_id };
    let result = QueryHandler::handle_query(query, &projection)?;

    // Assert
    match result {
        cim_domain_graph::QueryResult::GraphSummary(summary) => {
            assert_eq!(summary.node_count, 2);
            assert_eq!(summary.edge_count, 1);
            assert_eq!(summary.graph_id, graph_id);
        }
        _ => panic!("Expected GraphSummary result"),
    }

    Ok(())
}

/// Test finding connected nodes (graph traversal)
#[tokio::test]
async fn test_find_connected_nodes_query() -> DomainResult<()> {
    // Arrange - Create a simple graph: A -> B -> C
    let mut node_projection = NodeListProjection::new();
    let mut edge_projection = EdgeListProjection::new();
    let graph_id = GraphId::new();

    let node_a = NodeId::new();
    let node_b = NodeId::new();
    let node_c = NodeId::new();

    // Create nodes
    for (node_id, label) in [(node_a, "A"), (node_b, "B"), (node_c, "C")] {
        let event = DomainEvent::Graph(GraphDomainEvent::NodeAdded {
            graph_id,
            node_id,
            node_type: NodeType::Concept,
            position: Position3D::default(),
            conceptual_point: Default::default(),
            metadata: HashMap::from([("label".to_string(), label.to_string())]),
        });
        node_projection.apply_event(&event).await?;
    }

    // Create edges: A -> B -> C
    let edges = vec![
        (node_a, node_b),
        (node_b, node_c),
    ];

    for (source, target) in edges {
        let event = DomainEvent::Graph(GraphDomainEvent::EdgeConnected {
            graph_id,
            edge_id: cim_domain::EdgeId::new(),
            source,
            target,
            relationship: cim_domain_graph::EdgeRelationship::default(),
        });
        edge_projection.apply_event(&event).await?;
    }

    // Act - Find nodes connected to A with max depth 2
    let query = GraphQuery::FindConnectedNodes {
        graph_id,
        start_node: node_a,
        max_depth: 2,
        direction: cim_domain_graph::TraversalDirection::Outgoing,
    };

    let result = QueryHandler::handle_traversal_query(query, &node_projection, &edge_projection)?;

    // Assert
    match result {
        cim_domain_graph::QueryResult::NodeList(nodes) => {
            assert_eq!(nodes.len(), 2, "Should find B and C from A");
            // Should contain B and C but not A (start node)
            let node_ids: Vec<_> = nodes.iter().map(|n| n.node_id).collect();
            assert!(node_ids.contains(&node_b));
            assert!(node_ids.contains(&node_c));
        }
        _ => panic!("Expected NodeList result"),
    }

    Ok(())
}

/// Test query with pagination
#[tokio::test]
async fn test_paginated_node_query() -> DomainResult<()> {
    // Arrange - Create many nodes
    let mut projection = NodeListProjection::new();
    let graph_id = GraphId::new();

    // Add 25 nodes
    for i in 0..25 {
        let event = DomainEvent::Graph(GraphDomainEvent::NodeAdded {
            graph_id,
            node_id: NodeId::new(),
            node_type: NodeType::Concept,
            position: Position3D::default(),
            conceptual_point: Default::default(),
            metadata: HashMap::from([("index".to_string(), i.to_string())]),
        });
        projection.apply_event(&event).await?;
    }

    // Act - Query with pagination
    let query = GraphQuery::ListNodes {
        graph_id,
        offset: 10,
        limit: 5,
    };

    let result = QueryHandler::handle_query(query, &projection)?;

    // Assert
    match result {
        cim_domain_graph::QueryResult::NodeList(nodes) => {
            assert_eq!(nodes.len(), 5, "Should return 5 nodes as per limit");
        }
        _ => panic!("Expected NodeList result"),
    }

    Ok(())
}

/// Test query performance with large dataset
#[tokio::test]
async fn test_query_performance_large_dataset() -> DomainResult<()> {
    // Arrange - Create large projection
    let mut projection = NodeListProjection::new();
    let graph_id = GraphId::new();

    // Add 10,000 nodes
    for i in 0..10_000 {
        let event = DomainEvent::Graph(GraphDomainEvent::NodeAdded {
            graph_id,
            node_id: NodeId::new(),
            node_type: if i % 2 == 0 { NodeType::Concept } else { NodeType::Data },
            position: Position3D::default(),
            conceptual_point: Default::default(),
            metadata: Default::default(),
        });
        projection.apply_event(&event).await?;
    }

    // Act - Time the query
    let start = std::time::Instant::now();

    let query = GraphQuery::FindNodesByType {
        graph_id,
        node_type: NodeType::Concept,
    };

    let result = QueryHandler::handle_query(query, &projection)?;
    let duration = start.elapsed();

    // Assert
    match result {
        cim_domain_graph::QueryResult::NodeList(nodes) => {
            assert_eq!(nodes.len(), 5_000, "Should find 5,000 concept nodes");
            assert!(duration.as_millis() < 100, "Query should complete within 100ms");
        }
        _ => panic!("Expected NodeList result"),
    }

    Ok(())
}

/// Test concurrent queries don't interfere
#[tokio::test]
async fn test_concurrent_queries() -> DomainResult<()> {
    use tokio::task;
    use std::sync::Arc;

    // Arrange - Shared projection
    let projection = Arc::new(tokio::sync::RwLock::new(NodeListProjection::new()));
    let graph_id = GraphId::new();

    // Add some nodes
    for i in 0..100 {
        let event = create_node_added_event(graph_id, NodeType::Concept);
        projection.write().await.apply_event(&event).await?;
    }

    // Act - Run multiple queries concurrently
    let handles: Vec<_> = (0..10).map(|_| {
        let proj = projection.clone();
        let gid = graph_id;

        task::spawn(async move {
            let query = GraphQuery::ListNodes {
                graph_id: gid,
                offset: 0,
                limit: 10,
            };

            let projection_read = proj.read().await;
            QueryHandler::handle_query(query, &*projection_read)
        })
    }).collect();

    // Wait for all queries
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await??);
    }

    // Assert - All queries should succeed with same result
    assert_eq!(results.len(), 10);
    for result in results {
        match result {
            cim_domain_graph::QueryResult::NodeList(nodes) => {
                assert_eq!(nodes.len(), 10);
            }
            _ => panic!("Expected NodeList result"),
        }
    }

    Ok(())
}

/// Test query with complex filters
#[tokio::test]
async fn test_complex_filter_query() -> DomainResult<()> {
    // Arrange
    let mut projection = NodeListProjection::new();
    let graph_id = GraphId::new();

    // Add nodes with different metadata
    for i in 0..20 {
        let mut metadata = HashMap::new();
        metadata.insert("priority".to_string(), (i % 3).to_string());
        metadata.insert("status".to_string(), if i < 10 { "active" } else { "inactive" }.to_string());

        let event = DomainEvent::Graph(GraphDomainEvent::NodeAdded {
            graph_id,
            node_id: NodeId::new(),
            node_type: NodeType::Concept,
            position: Position3D::default(),
            conceptual_point: Default::default(),
            metadata,
        });
        projection.apply_event(&event).await?;
    }

    // Act - Query with filters
    let query = GraphQuery::FindNodesWithFilter {
        graph_id,
        filters: vec![
            cim_domain_graph::QueryFilter::MetadataEquals {
                key: "status".to_string(),
                value: "active".to_string(),
            },
            cim_domain_graph::QueryFilter::MetadataEquals {
                key: "priority".to_string(),
                value: "0".to_string(),
            },
        ],
    };

    let result = QueryHandler::handle_query(query, &projection)?;

    // Assert
    match result {
        cim_domain_graph::QueryResult::NodeList(nodes) => {
            // Should find active nodes with priority 0
            assert!(nodes.len() > 0);
            for node in nodes {
                assert_eq!(node.metadata.get("status"), Some(&"active".to_string()));
                assert_eq!(node.metadata.get("priority"), Some(&"0".to_string()));
            }
        }
        _ => panic!("Expected NodeList result"),
    }

    Ok(())
}

// Helper functions

fn create_node_added_event(graph_id: GraphId, node_type: NodeType) -> DomainEvent {
    DomainEvent::Graph(GraphDomainEvent::NodeAdded {
        graph_id,
        node_id: NodeId::new(),
        node_type,
        position: Position3D::default(),
        conceptual_point: Default::default(),
        metadata: Default::default(),
    })
}
