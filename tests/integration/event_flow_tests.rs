//! Integration tests for event flow through the system

use crate::fixtures::{TestEventStore, TestNatsServer, assertions::*, create_test_graph};
use cim_domain::{DomainEvent, DomainResult, GraphId, NodeId, Position3D};
use cim_domain_graph::{GraphAggregate, GraphCommand};
use cim_domain_workflow::{DecisionCriteria, NodeType, StepType};

#[tokio::test]
async fn test_command_to_event_store_flow() -> DomainResult<()> {
    // Arrange
    let nats = TestNatsServer::start().await?;
    let event_store = TestEventStore::new();
    let mut graph = create_test_graph();

    // Act - Process command
    let command = GraphCommand::AddNode {
        node_type: NodeType::WorkflowStep {
            step_type: StepType::Process,
        },
        position: Position3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        metadata: Default::default(),
    };

    let events = graph.handle_command(command)?;

    // Store events
    for event in &events {
        event_store.append(event.boxed_clone()).await?;
    }

    // Assert
    let stored_events = event_store.get_events().await;
    assert_event_count(&stored_events, 1);
    assert_event_published(&stored_events, "NodeAdded");

    // Cleanup
    nats.cleanup().await?;

    Ok(())
}

#[tokio::test]
async fn test_event_projection_update() -> DomainResult<()> {
    use cim_domain_graph::{GraphSummaryProjection, Projection};

    // Arrange
    let mut projection = GraphSummaryProjection::new();
    let graph_id = GraphId::new();

    // Create a node added event
    let event = Box::new(cim_domain::DomainEvent::from(
        cim_domain_graph::GraphDomainEvent::NodeAdded {
            graph_id,
            node_id: NodeId::new(),
            node_type: NodeType::WorkflowStep {
                step_type: StepType::Process,
            },
            position: Position3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            conceptual_point: cim_domain_graph::ConceptualPoint::default(),
            metadata: Default::default(),
        },
    ));

    // Act - Apply event to projection
    projection.apply_event(&*event).await?;

    // Assert
    let summary = projection.get_summary(&graph_id)?;
    assert_eq!(summary.node_count, 1);
    assert_eq!(summary.edge_count, 0);

    Ok(())
}

#[tokio::test]
async fn test_multiple_commands_sequential_processing() -> DomainResult<()> {
    // Arrange
    let event_store = TestEventStore::new();
    let mut graph = create_test_graph();
    let node_id1 = NodeId::new();
    let node_id2 = NodeId::new();

    // Act - Process multiple commands
    let commands = vec![
        GraphCommand::AddNode {
            node_type: NodeType::WorkflowStep {
                step_type: StepType::Start,
            },
            position: Position3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            metadata: Default::default(),
        },
        GraphCommand::AddNode {
            node_type: NodeType::WorkflowStep {
                step_type: StepType::End,
            },
            position: Position3D {
                x: 10.0,
                y: 0.0,
                z: 0.0,
            },
            metadata: Default::default(),
        },
    ];

    let mut all_events = Vec::new();
    for command in commands {
        let events = graph.handle_command(command)?;
        for event in events {
            event_store.append(event.boxed_clone()).await?;
            all_events.push(event);
        }
    }

    // Assert
    let stored_events = event_store.get_events().await;
    assert_event_count(&stored_events, 2);
    assert_eq!(graph.node_count(), 2);

    Ok(())
}

#[tokio::test]
async fn test_event_replay_consistency() -> DomainResult<()> {
    // Arrange
    let event_store = TestEventStore::new();
    let mut original_graph = create_test_graph();

    // Create some events
    let commands = vec![
        GraphCommand::AddNode {
            node_type: NodeType::WorkflowStep {
                step_type: StepType::Process,
            },
            position: Position3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            metadata: Default::default(),
        },
        GraphCommand::AddNode {
            node_type: NodeType::Decision {
                criteria: DecisionCriteria::default(),
            },
            position: Position3D {
                x: 5.0,
                y: 5.0,
                z: 0.0,
            },
            metadata: Default::default(),
        },
    ];

    // Process commands and store events
    let mut all_events = Vec::new();
    for command in commands {
        let events = original_graph.handle_command(command)?;
        for event in events {
            event_store.append(event.boxed_clone()).await?;
            all_events.push(event);
        }
    }

    // Act - Replay events on new aggregate
    let mut replayed_graph = create_test_graph();
    let stored_events = event_store.get_events().await;

    for event in stored_events.iter() {
        replayed_graph.apply_event(&**event)?;
    }

    // Assert - Both graphs should have same state
    assert_eq!(original_graph.node_count(), replayed_graph.node_count());
    assert_eq!(original_graph.version(), replayed_graph.version());

    Ok(())
}

/// Test that demonstrates the full CQRS flow
#[tokio::test]
async fn test_full_cqrs_flow() -> DomainResult<()> {
    use cim_domain_graph::{GraphSummaryProjection, NodeListProjection, Projection};

    // Arrange
    let event_store = TestEventStore::new();
    let mut graph = create_test_graph();
    let mut summary_projection = GraphSummaryProjection::new();
    let mut node_list_projection = NodeListProjection::new();

    // Act - Process command
    let command = GraphCommand::AddNode {
        node_type: NodeType::WorkflowStep {
            step_type: StepType::Process,
        },
        position: Position3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        metadata: Default::default(),
    };

    let events = graph.handle_command(command)?;

    // Store events and update projections
    for event in events {
        event_store.append(event.boxed_clone()).await?;
        summary_projection.apply_event(&*event).await?;
        node_list_projection.apply_event(&*event).await?;
    }

    // Query projections
    let summary = summary_projection.get_summary(&graph.id())?;
    let nodes = node_list_projection.get_nodes(&graph.id())?;

    // Assert
    assert_eq!(summary.node_count, 1);
    assert_eq!(nodes.len(), 1);

    Ok(())
}
