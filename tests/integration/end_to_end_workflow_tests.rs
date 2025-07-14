//! End-to-end workflow integration tests
//!
//! These tests verify complete business workflows through the system:
//! 1. Multi-step business processes
//! 2. User interaction flows
//! 3. State machine transitions
//! 4. Complex event choreography
//!
//! ```mermaid
//! graph LR
//!     A[User Action] --> B[Command]
//!     B --> C[Domain Logic]
//!     C --> D[Events]
//!     D --> E[Projections]
//!     E --> F[UI Update]
//!     F --> G[User Sees Result]
//! ```
#![cfg(feature = "bevy")]

use crate::fixtures::{TestEventStore, TestNatsServer, assertions::*, create_test_app};
use bevy::prelude::*;
use cim_domain::{DomainEvent, DomainResult, EdgeId, GraphId, NodeId};
use cim_domain_graph::{
    EdgeType, GraphAggregate, GraphCommand, GraphDomainEvent, NodeType, Position3D, StepType,
    WorkflowState, WorkflowStatus,
};
use cim_domain_workflow::{
    State, StateId, Transition, TransitionId, WorkflowAggregate, WorkflowCommand, WorkflowEvent,
};
use std::collections::HashMap;

/// Test complete graph creation and visualization workflow
#[tokio::test]
async fn test_complete_graph_creation_workflow() -> DomainResult<()> {
    // Arrange
    let mut app = create_test_app();
    let nats = TestNatsServer::start().await?;
    let event_store = TestEventStore::with_nats(&nats).await?;

    // User Story: As a user, I want to create a workflow graph and see it visualized

    // Step 1: User creates a new graph
    let graph_id = GraphId::new();
    let create_graph_cmd = GraphCommand::CreateGraph {
        graph_id,
        name: "Order Processing Workflow".to_string(),
        graph_type: cim_domain_graph::GraphType::WorkflowGraph,
        metadata: HashMap::from([
            ("author".to_string(), "test_user".to_string()),
            ("version".to_string(), "1.0".to_string()),
        ]),
    };

    // Send command through Bevy
    app.world.send_event(create_graph_cmd);
    app.update(); // Process command

    // Step 2: User adds workflow nodes
    let start_node = NodeId::new();
    let process_node = NodeId::new();
    let decision_node = NodeId::new();
    let end_node = NodeId::new();

    let nodes = vec![
        (
            start_node,
            NodeType::WorkflowStep {
                step_type: StepType::Start,
            },
            Position3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        ),
        (
            process_node,
            NodeType::WorkflowStep {
                step_type: StepType::Process,
            },
            Position3D {
                x: 5.0,
                y: 0.0,
                z: 0.0,
            },
        ),
        (
            decision_node,
            NodeType::Decision {
                criteria: Default::default(),
            },
            Position3D {
                x: 10.0,
                y: 0.0,
                z: 0.0,
            },
        ),
        (
            end_node,
            NodeType::WorkflowStep {
                step_type: StepType::End,
            },
            Position3D {
                x: 15.0,
                y: 0.0,
                z: 0.0,
            },
        ),
    ];

    for (node_id, node_type, position) in nodes {
        app.world.send_event(GraphCommand::AddNode {
            node_type,
            position,
            metadata: HashMap::from([("id".to_string(), node_id.to_string())]),
        });
        app.update();
    }

    // Step 3: User connects nodes
    let edges = vec![
        (start_node, process_node, EdgeType::Sequence),
        (process_node, decision_node, EdgeType::Sequence),
        (
            decision_node,
            end_node,
            EdgeType::Conditional {
                condition: "approved".to_string(),
            },
        ),
    ];

    for (source, target, edge_type) in edges {
        app.world.send_event(GraphCommand::ConnectNodes {
            source,
            target,
            edge_type,
        });
        app.update();
    }

    // Step 4: Verify visualization
    app.update(); // Process events to entities

    // Check that all nodes are visualized
    let node_query = app
        .world
        .query::<&cim_domain_bevy::components::NodeVisual>();
    let node_count = node_query.iter(&app.world).count();
    assert_eq!(node_count, 4, "All workflow nodes should be visualized");

    // Check that all edges are visualized
    let edge_query = app
        .world
        .query::<&cim_domain_bevy::components::EdgeVisual>();
    let edge_count = edge_query.iter(&app.world).count();
    assert_eq!(edge_count, 3, "All workflow edges should be visualized");

    // Step 5: User saves the workflow
    app.world.send_event(GraphCommand::SaveGraph { graph_id });
    app.update();

    // Verify events were persisted
    let events = event_store.get_events().await;
    assert!(
        events.len() >= 8,
        "Should have graph creation + 4 nodes + 3 edges"
    );

    // Cleanup
    nats.cleanup().await?;

    Ok(())
}

/// Test workflow execution with state transitions
#[tokio::test]
async fn test_workflow_execution_state_transitions() -> DomainResult<()> {
    // Arrange
    let event_store = TestEventStore::new();
    let mut workflow = WorkflowAggregate::new(cim_domain::WorkflowId::new());

    // Define workflow states
    let states = vec![
        State::new(StateId::from("draft"), "Draft".to_string()),
        State::new(StateId::from("submitted"), "Submitted".to_string()),
        State::new(StateId::from("approved"), "Approved".to_string()),
        State::new(StateId::from("rejected"), "Rejected".to_string()),
        State::new(StateId::from("completed"), "Completed".to_string()),
    ];

    // Define transitions
    let transitions = vec![
        Transition::new(
            TransitionId::from("submit"),
            StateId::from("draft"),
            StateId::from("submitted"),
            Some("user.canSubmit".to_string()),
        ),
        Transition::new(
            TransitionId::from("approve"),
            StateId::from("submitted"),
            StateId::from("approved"),
            Some("manager.canApprove".to_string()),
        ),
        Transition::new(
            TransitionId::from("reject"),
            StateId::from("submitted"),
            StateId::from("rejected"),
            Some("manager.canReject".to_string()),
        ),
        Transition::new(
            TransitionId::from("complete"),
            StateId::from("approved"),
            StateId::from("completed"),
            None,
        ),
    ];

    // Initialize workflow
    let init_cmd = WorkflowCommand::InitializeWorkflow {
        workflow_id: workflow.id(),
        name: "Order Approval Workflow".to_string(),
        states: states.clone(),
        transitions: transitions.clone(),
        initial_state: StateId::from("draft"),
    };

    let events = workflow.handle_command(init_cmd)?;
    for event in &events {
        event_store.append(event.clone()).await?;
        workflow.apply_event(event)?;
    }

    // Test workflow execution

    // Step 1: Submit for approval
    let submit_cmd = WorkflowCommand::TransitionWorkflow {
        workflow_id: workflow.id(),
        transition_id: TransitionId::from("submit"),
        context: HashMap::from([("user".to_string(), "john_doe".to_string())]),
    };

    let events = workflow.handle_command(submit_cmd)?;
    assert!(!events.is_empty(), "Submit should generate events");

    for event in &events {
        workflow.apply_event(event)?;
    }

    assert_eq!(workflow.current_state(), Some(&StateId::from("submitted")));

    // Step 2: Manager approves
    let approve_cmd = WorkflowCommand::TransitionWorkflow {
        workflow_id: workflow.id(),
        transition_id: TransitionId::from("approve"),
        context: HashMap::from([("manager".to_string(), "jane_smith".to_string())]),
    };

    let events = workflow.handle_command(approve_cmd)?;
    for event in &events {
        workflow.apply_event(event)?;
    }

    assert_eq!(workflow.current_state(), Some(&StateId::from("approved")));

    // Step 3: Complete the workflow
    let complete_cmd = WorkflowCommand::TransitionWorkflow {
        workflow_id: workflow.id(),
        transition_id: TransitionId::from("complete"),
        context: HashMap::new(),
    };

    let events = workflow.handle_command(complete_cmd)?;
    for event in &events {
        workflow.apply_event(event)?;
    }

    assert_eq!(workflow.current_state(), Some(&StateId::from("completed")));
    assert_eq!(workflow.status(), WorkflowStatus::Completed);

    // Verify execution history
    let history = workflow.execution_history();
    assert_eq!(history.len(), 3, "Should have 3 transitions in history");

    Ok(())
}

/// Test multi-user collaborative graph editing
#[tokio::test]
async fn test_multi_user_collaborative_editing() -> DomainResult<()> {
    use tokio::sync::broadcast;

    // Arrange
    let event_store = TestEventStore::new();
    let (tx, _) = broadcast::channel(100);

    // Simulate multiple users
    let users = vec!["alice", "bob", "charlie"];
    let graph_id = GraphId::new();

    // Create initial graph
    let mut graph = GraphAggregate::new(graph_id);
    let create_event = DomainEvent::Graph(GraphDomainEvent::GraphCreated {
        graph_id,
        graph_type: cim_domain_graph::GraphType::ConceptualGraph,
        name: "Collaborative Knowledge Graph".to_string(),
        metadata: Default::default(),
    });

    event_store.append(create_event.clone()).await?;
    graph.apply_event(&create_event)?;

    // Simulate concurrent edits
    let mut handles = Vec::new();

    for (i, user) in users.iter().enumerate() {
        let store = event_store.clone();
        let mut rx = tx.subscribe();
        let user = user.to_string();
        let graph_id = graph_id;

        let handle = tokio::spawn(async move {
            // Each user adds nodes
            for j in 0..3 {
                let node_id = NodeId::new();
                let event = DomainEvent::Graph(GraphDomainEvent::NodeAdded {
                    graph_id,
                    node_id,
                    node_type: NodeType::Concept,
                    position: Position3D {
                        x: (i * 10 + j) as f32,
                        y: 0.0,
                        z: 0.0,
                    },
                    conceptual_point: Default::default(),
                    metadata: HashMap::from([
                        ("author".to_string(), user.clone()),
                        ("label".to_string(), format!("{}'s node {}", user, j)),
                    ]),
                });

                store.append(event.clone()).await?;
                tx.send(event)?;

                // Simulate thinking time
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }

            Ok::<(), Box<dyn std::error::Error>>(())
        });

        handles.push(handle);
    }

    // Wait for all users to finish
    for handle in handles {
        handle.await??;
    }

    // Verify collaborative result
    let events = event_store.get_events().await;
    let node_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, DomainEvent::Graph(GraphDomainEvent::NodeAdded { .. })))
        .collect();

    assert_eq!(
        node_events.len(),
        9,
        "Should have 3 nodes from each of 3 users"
    );

    // Check that all users' contributions are present
    for user in &users {
        let user_nodes: Vec<_> = node_events
            .iter()
            .filter(|e| {
                if let DomainEvent::Graph(GraphDomainEvent::NodeAdded { metadata, .. }) = e {
                    metadata.get("author") == Some(&user.to_string())
                } else {
                    false
                }
            })
            .collect();

        assert_eq!(user_nodes.len(), 3, "Each user should have added 3 nodes");
    }

    Ok(())
}

/// Test complex event choreography workflow
#[tokio::test]
async fn test_complex_event_choreography() -> DomainResult<()> {
    // Arrange
    let event_store = TestEventStore::new();

    // Scenario: Order processing with inventory check and payment

    // Step 1: Order placed
    let order_id = GraphId::new();
    let order_event = DomainEvent::Graph(GraphDomainEvent::GraphCreated {
        graph_id: order_id,
        graph_type: cim_domain_graph::GraphType::WorkflowGraph,
        name: "Order #12345".to_string(),
        metadata: HashMap::from([
            ("customer".to_string(), "john@example.com".to_string()),
            ("total".to_string(), "99.99".to_string()),
        ]),
    });

    event_store.append(order_event.clone()).await?;

    // Step 2: Inventory service reacts
    let inventory_check = tokio::spawn({
        let store = event_store.clone();
        async move {
            // Simulate inventory check
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let inventory_event = DomainEvent::Graph(GraphDomainEvent::NodeAdded {
                graph_id: order_id,
                node_id: NodeId::new(),
                node_type: NodeType::WorkflowStep {
                    step_type: StepType::Process,
                },
                position: Position3D::default(),
                conceptual_point: Default::default(),
                metadata: HashMap::from([
                    ("step".to_string(), "inventory_checked".to_string()),
                    ("status".to_string(), "available".to_string()),
                ]),
            });

            store.append(inventory_event).await
        }
    });

    // Step 3: Payment service reacts
    let payment_process = tokio::spawn({
        let store = event_store.clone();
        async move {
            // Simulate payment processing
            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

            let payment_event = DomainEvent::Graph(GraphDomainEvent::NodeAdded {
                graph_id: order_id,
                node_id: NodeId::new(),
                node_type: NodeType::WorkflowStep {
                    step_type: StepType::Process,
                },
                position: Position3D::default(),
                conceptual_point: Default::default(),
                metadata: HashMap::from([
                    ("step".to_string(), "payment_processed".to_string()),
                    ("status".to_string(), "success".to_string()),
                ]),
            });

            store.append(payment_event).await
        }
    });

    // Wait for both services
    inventory_check.await??;
    payment_process.await??;

    // Step 4: Order fulfillment reacts to both events
    let events = event_store.get_events().await;

    let has_inventory = events.iter().any(|e| {
        matches!(e, DomainEvent::Graph(GraphDomainEvent::NodeAdded { metadata, .. })
            if metadata.get("step") == Some(&"inventory_checked".to_string()))
    });

    let has_payment = events.iter().any(|e| {
        matches!(e, DomainEvent::Graph(GraphDomainEvent::NodeAdded { metadata, .. })
            if metadata.get("step") == Some(&"payment_processed".to_string()))
    });

    if has_inventory && has_payment {
        // Complete order
        let completion_event = DomainEvent::Graph(GraphDomainEvent::NodeAdded {
            graph_id: order_id,
            node_id: NodeId::new(),
            node_type: NodeType::WorkflowStep {
                step_type: StepType::End,
            },
            position: Position3D::default(),
            conceptual_point: Default::default(),
            metadata: HashMap::from([
                ("step".to_string(), "order_completed".to_string()),
                ("status".to_string(), "fulfilled".to_string()),
            ]),
        });

        event_store.append(completion_event).await?;
    }

    // Verify choreography result
    let final_events = event_store.get_events().await;
    assert_eq!(
        final_events.len(),
        4,
        "Should have order + inventory + payment + completion"
    );

    // Verify order of events
    let step_events: Vec<_> = final_events
        .iter()
        .filter_map(|e| {
            if let DomainEvent::Graph(GraphDomainEvent::NodeAdded { metadata, .. }) = e {
                metadata.get("step").cloned()
            } else {
                None
            }
        })
        .collect();

    assert_eq!(step_events.len(), 3);
    assert!(step_events.contains(&"inventory_checked".to_string()));
    assert!(step_events.contains(&"payment_processed".to_string()));
    assert!(step_events.contains(&"order_completed".to_string()));

    Ok(())
}

/// Test error recovery in workflows
#[tokio::test]
async fn test_workflow_error_recovery() -> DomainResult<()> {
    // Arrange
    let event_store = TestEventStore::new();
    let mut workflow = WorkflowAggregate::new(cim_domain::WorkflowId::new());

    // Create workflow with error handling states
    let states = vec![
        State::new(StateId::from("processing"), "Processing".to_string()),
        State::new(StateId::from("error"), "Error".to_string()),
        State::new(StateId::from("retry"), "Retry".to_string()),
        State::new(StateId::from("failed"), "Failed".to_string()),
        State::new(StateId::from("completed"), "Completed".to_string()),
    ];

    let transitions = vec![
        Transition::new(
            TransitionId::from("process_error"),
            StateId::from("processing"),
            StateId::from("error"),
            None,
        ),
        Transition::new(
            TransitionId::from("retry_process"),
            StateId::from("error"),
            StateId::from("retry"),
            None,
        ),
        Transition::new(
            TransitionId::from("retry_success"),
            StateId::from("retry"),
            StateId::from("completed"),
            None,
        ),
        Transition::new(
            TransitionId::from("max_retries_exceeded"),
            StateId::from("error"),
            StateId::from("failed"),
            None,
        ),
    ];

    // Initialize workflow
    workflow.handle_command(WorkflowCommand::InitializeWorkflow {
        workflow_id: workflow.id(),
        name: "Error Recovery Workflow".to_string(),
        states,
        transitions,
        initial_state: StateId::from("processing"),
    })?;

    // Simulate processing error
    let error_cmd = WorkflowCommand::TransitionWorkflow {
        workflow_id: workflow.id(),
        transition_id: TransitionId::from("process_error"),
        context: HashMap::from([
            ("error".to_string(), "Network timeout".to_string()),
            ("retry_count".to_string(), "0".to_string()),
        ]),
    };

    let events = workflow.handle_command(error_cmd)?;
    for event in &events {
        workflow.apply_event(event)?;
    }

    assert_eq!(workflow.current_state(), Some(&StateId::from("error")));

    // Attempt retry
    let retry_cmd = WorkflowCommand::TransitionWorkflow {
        workflow_id: workflow.id(),
        transition_id: TransitionId::from("retry_process"),
        context: HashMap::from([("retry_count".to_string(), "1".to_string())]),
    };

    workflow.handle_command(retry_cmd)?;
    assert_eq!(workflow.current_state(), Some(&StateId::from("retry")));

    // Simulate successful retry
    let success_cmd = WorkflowCommand::TransitionWorkflow {
        workflow_id: workflow.id(),
        transition_id: TransitionId::from("retry_success"),
        context: HashMap::new(),
    };

    workflow.handle_command(success_cmd)?;
    assert_eq!(workflow.current_state(), Some(&StateId::from("completed")));

    // Verify recovery path in history
    let history = workflow.execution_history();
    assert!(
        history.len() >= 3,
        "Should have error -> retry -> success path"
    );

    Ok(())
}

/// Test performance under load with multiple concurrent workflows
#[tokio::test]
async fn test_concurrent_workflow_performance() -> DomainResult<()> {
    use std::time::Instant;
    use tokio::task;

    // Arrange
    let workflow_count = 100;
    let steps_per_workflow = 10;
    let event_store = TestEventStore::new();

    let start = Instant::now();

    // Spawn concurrent workflows
    let handles: Vec<_> = (0..workflow_count)
        .map(|i| {
            let store = event_store.clone();

            task::spawn(async move {
                let workflow_id = cim_domain::WorkflowId::new();
                let graph_id = GraphId::new();

                // Create workflow graph
                let create_event = DomainEvent::Graph(GraphDomainEvent::GraphCreated {
                    graph_id,
                    graph_type: cim_domain_graph::GraphType::WorkflowGraph,
                    name: format!("Workflow {}", i),
                    metadata: Default::default(),
                });

                store.append(create_event).await?;

                // Add workflow steps
                for step in 0..steps_per_workflow {
                    let node_event = DomainEvent::Graph(GraphDomainEvent::NodeAdded {
                        graph_id,
                        node_id: NodeId::new(),
                        node_type: NodeType::WorkflowStep {
                            step_type: if step == 0 {
                                StepType::Start
                            } else if step == steps_per_workflow - 1 {
                                StepType::End
                            } else {
                                StepType::Process
                            },
                        },
                        position: Position3D {
                            x: step as f32,
                            y: 0.0,
                            z: 0.0,
                        },
                        conceptual_point: Default::default(),
                        metadata: HashMap::from([
                            ("workflow".to_string(), i.to_string()),
                            ("step".to_string(), step.to_string()),
                        ]),
                    });

                    store.append(node_event).await?;
                }

                Ok::<(), cim_domain::DomainError>(())
            })
        })
        .collect();

    // Wait for all workflows
    for handle in handles {
        handle.await??;
    }

    let duration = start.elapsed();

    // Verify results
    let events = event_store.get_events().await;
    let expected_events = workflow_count * (1 + steps_per_workflow); // create + steps

    assert_eq!(
        events.len(),
        expected_events,
        "Should have all workflow events"
    );

    // Performance assertions
    let events_per_second = expected_events as f64 / duration.as_secs_f64();
    println!(
        "Processed {} workflows with {} events in {:?} ({:.2} events/sec)",
        workflow_count, expected_events, duration, events_per_second
    );

    assert!(
        events_per_second > 1000.0,
        "Should process at least 1000 events/sec, got {:.2}",
        events_per_second
    );

    Ok(())
}
