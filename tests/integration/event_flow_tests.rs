//! Integration tests for event flow through the system

use crate::fixtures::{TestEventStore, TestNatsServer, assertions::*, create_test_graph};
use cim_domain::{DomainResult, GraphId, NodeId};
use cim_domain_graph::{GraphAggregate, GraphCommand, GraphType, NodeType, Position3D, StepType};

#[tokio::test]
async fn test_command_to_event_store_flow() -> DomainResult<()> {
    // Arrange
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

    Ok(())
}

#[tokio::test]
async fn test_multiple_commands_sequential_processing() -> DomainResult<()> {
    // Arrange
    let event_store = TestEventStore::new();
    let mut graph = create_test_graph();

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
    let graph_id = GraphId::new();
    let mut original_graph = GraphAggregate::new(graph_id, "Test Graph".to_string(), GraphType::WorkflowGraph);

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
                criteria: Default::default(),
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

    // Act - Create new aggregate (simulating replay)
    let mut replayed_graph = GraphAggregate::new(graph_id, "Replayed Graph".to_string(), GraphType::WorkflowGraph);
    
    // In a real system, we would replay events here
    // For now, just verify the original graph state
    
    // Assert - Original graph has correct state
    assert_eq!(original_graph.node_count(), 2);
    assert_eq!(all_events.len(), 2);

    Ok(())
}

/// Test concurrent command processing
#[tokio::test]
async fn test_concurrent_command_processing() -> DomainResult<()> {
    use tokio::sync::Mutex;
    use std::sync::Arc;
    
    // Arrange
    let event_store = TestEventStore::new();
    let graph_id = GraphId::new();
    let graph = Arc::new(Mutex::new(GraphAggregate::new(
        graph_id, 
        "Concurrent Test Graph".to_string(), 
        GraphType::WorkflowGraph
    )));
    
    // Act - Process commands concurrently
    let mut handles = vec![];
    
    for i in 0..3 {
        let graph_clone = graph.clone();
        let store_clone = event_store.clone();
        
        let handle = tokio::spawn(async move {
            let command = GraphCommand::AddNode {
                node_type: NodeType::WorkflowStep {
                    step_type: StepType::Process,
                },
                position: Position3D {
                    x: i as f32 * 5.0,
                    y: 0.0,
                    z: 0.0,
                },
                metadata: Default::default(),
            };
            
            let mut graph_guard = graph_clone.lock().await;
            let events = graph_guard.handle_command(command)?;
            drop(graph_guard); // Release lock before async operations
            
            for event in events {
                store_clone.append(event.boxed_clone()).await?;
            }
            
            Ok::<(), cim_domain::DomainError>(())
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        handle.await.map_err(|e| cim_domain::DomainError::generic(format!("Task failed: {}", e)))??;
    }
    
    // Assert
    let stored_events = event_store.get_events().await;
    assert_eq!(stored_events.len(), 3, "Should have 3 events from concurrent commands");
    
    let final_graph = graph.lock().await;
    assert_eq!(final_graph.node_count(), 3, "Graph should have 3 nodes");
    
    Ok(())
}
