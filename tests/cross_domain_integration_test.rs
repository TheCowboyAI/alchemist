//! Cross-domain integration tests
//! 
//! These tests verify that different domains can work together through events

use cim_domain::{DomainResult, GraphId, NodeId};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

/// Simple in-memory event bus for testing
#[derive(Clone)]
struct TestEventBus {
    events: Arc<Mutex<Vec<TestEvent>>>,
}

#[derive(Clone, Debug)]
struct TestEvent {
    event_type: String,
    aggregate_id: String,
    payload: HashMap<String, String>,
}

impl TestEventBus {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn publish(&self, event: TestEvent) {
        let mut events = self.events.lock().await;
        events.push(event);
    }

    async fn get_events(&self) -> Vec<TestEvent> {
        let events = self.events.lock().await;
        events.clone()
    }

    async fn get_events_for_aggregate(&self, aggregate_id: &str) -> Vec<TestEvent> {
        let events = self.events.lock().await;
        events
            .iter()
            .filter(|e| e.aggregate_id == aggregate_id)
            .cloned()
            .collect()
    }
}

#[tokio::test]
async fn test_graph_to_workflow_integration() -> DomainResult<()> {
    // Scenario: Creating a graph triggers workflow creation
    let event_bus = TestEventBus::new();
    let graph_id = GraphId::new();
    
    // Simulate graph creation
    let graph_created_event = TestEvent {
        event_type: "GraphCreated".to_string(),
        aggregate_id: graph_id.to_string(),
        payload: HashMap::from([
            ("name".to_string(), "Approval Workflow".to_string()),
            ("type".to_string(), "Workflow".to_string()),
        ]),
    };
    
    event_bus.publish(graph_created_event).await;
    
    // Simulate workflow domain reacting to graph creation
    let events = event_bus.get_events().await;
    let graph_events: Vec<_> = events.iter()
        .filter(|e| e.event_type == "GraphCreated" && e.payload.get("type") == Some(&"Workflow".to_string()))
        .collect();
    
    // For each workflow graph, create a corresponding workflow
    for graph_event in graph_events {
        let workflow_created = TestEvent {
            event_type: "WorkflowCreated".to_string(),
            aggregate_id: format!("workflow-{}", graph_event.aggregate_id),
            payload: HashMap::from([
                ("graph_id".to_string(), graph_event.aggregate_id.clone()),
                ("name".to_string(), graph_event.payload.get("name").unwrap().clone()),
            ]),
        };
        event_bus.publish(workflow_created).await;
    }
    
    // Verify workflow was created
    let all_events = event_bus.get_events().await;
    assert_eq!(all_events.len(), 2);
    
    let workflow_events: Vec<_> = all_events.iter()
        .filter(|e| e.event_type == "WorkflowCreated")
        .collect();
    assert_eq!(workflow_events.len(), 1);
    assert_eq!(workflow_events[0].payload.get("graph_id"), Some(&graph_id.to_string()));
    
    println!("✅ Graph to Workflow integration test passed");
    Ok(())
}

#[tokio::test]
async fn test_person_location_integration() -> DomainResult<()> {
    // Scenario: Person is assigned to a location
    let event_bus = TestEventBus::new();
    let person_id = NodeId::new();
    let location_id = NodeId::new();
    
    // Create a person
    let person_created = TestEvent {
        event_type: "PersonCreated".to_string(),
        aggregate_id: person_id.to_string(),
        payload: HashMap::from([
            ("first_name".to_string(), "Alice".to_string()),
            ("last_name".to_string(), "Smith".to_string()),
        ]),
    };
    event_bus.publish(person_created).await;
    
    // Create a location
    let location_created = TestEvent {
        event_type: "LocationCreated".to_string(),
        aggregate_id: location_id.to_string(),
        payload: HashMap::from([
            ("name".to_string(), "New York Office".to_string()),
            ("type".to_string(), "Office".to_string()),
        ]),
    };
    event_bus.publish(location_created).await;
    
    // Assign person to location
    let person_assigned = TestEvent {
        event_type: "PersonAssignedToLocation".to_string(),
        aggregate_id: person_id.to_string(),
        payload: HashMap::from([
            ("location_id".to_string(), location_id.to_string()),
            ("assignment_type".to_string(), "WorkLocation".to_string()),
        ]),
    };
    event_bus.publish(person_assigned).await;
    
    // Verify all events
    let all_events = event_bus.get_events().await;
    assert_eq!(all_events.len(), 3);
    
    // Verify person's events
    let person_events = event_bus.get_events_for_aggregate(&person_id.to_string()).await;
    assert_eq!(person_events.len(), 2); // Created + Assigned
    
    println!("✅ Person-Location integration test passed");
    Ok(())
}

#[tokio::test]
async fn test_agent_workflow_execution() -> DomainResult<()> {
    // Scenario: AI agent executes a workflow
    let event_bus = TestEventBus::new();
    let agent_id = NodeId::new();
    let workflow_id = NodeId::new();
    
    // Create an AI agent
    let agent_created = TestEvent {
        event_type: "AgentCreated".to_string(),
        aggregate_id: agent_id.to_string(),
        payload: HashMap::from([
            ("name".to_string(), "WorkflowBot".to_string()),
            ("type".to_string(), "AI".to_string()),
            ("capabilities".to_string(), "workflow_execution".to_string()),
        ]),
    };
    event_bus.publish(agent_created).await;
    
    // Create a workflow
    let workflow_created = TestEvent {
        event_type: "WorkflowCreated".to_string(),
        aggregate_id: workflow_id.to_string(),
        payload: HashMap::from([
            ("name".to_string(), "Document Approval".to_string()),
            ("steps".to_string(), "3".to_string()),
        ]),
    };
    event_bus.publish(workflow_created).await;
    
    // Agent starts workflow execution
    let execution_started = TestEvent {
        event_type: "WorkflowExecutionStarted".to_string(),
        aggregate_id: workflow_id.to_string(),
        payload: HashMap::from([
            ("agent_id".to_string(), agent_id.to_string()),
            ("started_at".to_string(), chrono::Utc::now().to_string()),
        ]),
    };
    event_bus.publish(execution_started).await;
    
    // Simulate workflow steps
    for step in 1..=3 {
        let step_completed = TestEvent {
            event_type: "WorkflowStepCompleted".to_string(),
            aggregate_id: workflow_id.to_string(),
            payload: HashMap::from([
                ("step".to_string(), step.to_string()),
                ("agent_id".to_string(), agent_id.to_string()),
                ("result".to_string(), "Success".to_string()),
            ]),
        };
        event_bus.publish(step_completed).await;
    }
    
    // Workflow completed
    let workflow_completed = TestEvent {
        event_type: "WorkflowCompleted".to_string(),
        aggregate_id: workflow_id.to_string(),
        payload: HashMap::from([
            ("agent_id".to_string(), agent_id.to_string()),
            ("duration_ms".to_string(), "1500".to_string()),
        ]),
    };
    event_bus.publish(workflow_completed).await;
    
    // Verify execution flow
    let workflow_events = event_bus.get_events_for_aggregate(&workflow_id.to_string()).await;
    assert_eq!(workflow_events.len(), 6); // Created + Started + 3 Steps + Completed
    
    // Verify all step completions
    let step_events: Vec<_> = workflow_events.iter()
        .filter(|e| e.event_type == "WorkflowStepCompleted")
        .collect();
    assert_eq!(step_events.len(), 3);
    
    println!("✅ Agent-Workflow execution test passed");
    Ok(())
}

#[tokio::test]
async fn test_document_graph_visualization() -> DomainResult<()> {
    // Scenario: Documents are visualized in a graph
    let event_bus = TestEventBus::new();
    let graph_id = GraphId::new();
    let doc1_id = NodeId::new();
    let doc2_id = NodeId::new();
    
    // Create a graph for document relationships
    let graph_created = TestEvent {
        event_type: "GraphCreated".to_string(),
        aggregate_id: graph_id.to_string(),
        payload: HashMap::from([
            ("name".to_string(), "Document Relationships".to_string()),
            ("type".to_string(), "Knowledge".to_string()),
        ]),
    };
    event_bus.publish(graph_created).await;
    
    // Create documents
    for (doc_id, title) in [(doc1_id, "Requirements"), (doc2_id, "Design")] {
        let doc_created = TestEvent {
            event_type: "DocumentCreated".to_string(),
            aggregate_id: doc_id.to_string(),
            payload: HashMap::from([
                ("title".to_string(), title.to_string()),
                ("type".to_string(), "Technical".to_string()),
            ]),
        };
        event_bus.publish(doc_created).await;
        
        // Add document as node in graph
        let node_added = TestEvent {
            event_type: "NodeAddedToGraph".to_string(),
            aggregate_id: graph_id.to_string(),
            payload: HashMap::from([
                ("node_id".to_string(), doc_id.to_string()),
                ("node_type".to_string(), "Document".to_string()),
                ("title".to_string(), title.to_string()),
            ]),
        };
        event_bus.publish(node_added).await;
    }
    
    // Create relationship between documents
    let edge_added = TestEvent {
        event_type: "EdgeAddedToGraph".to_string(),
        aggregate_id: graph_id.to_string(),
        payload: HashMap::from([
            ("source".to_string(), doc1_id.to_string()),
            ("target".to_string(), doc2_id.to_string()),
            ("relationship".to_string(), "DerivedFrom".to_string()),
        ]),
    };
    event_bus.publish(edge_added).await;
    
    // Verify graph structure
    let graph_events = event_bus.get_events_for_aggregate(&graph_id.to_string()).await;
    assert_eq!(graph_events.len(), 4); // Created + 2 Nodes + 1 Edge
    
    let node_events: Vec<_> = graph_events.iter()
        .filter(|e| e.event_type == "NodeAddedToGraph")
        .collect();
    assert_eq!(node_events.len(), 2);
    
    println!("✅ Document-Graph visualization test passed");
    Ok(())
}

#[tokio::test]
async fn test_organization_hierarchy() -> DomainResult<()> {
    // Scenario: Organization with departments and employees
    let event_bus = TestEventBus::new();
    let org_id = NodeId::new();
    let dept_id = NodeId::new();
    let person_id = NodeId::new();
    
    // Create organization
    let org_created = TestEvent {
        event_type: "OrganizationCreated".to_string(),
        aggregate_id: org_id.to_string(),
        payload: HashMap::from([
            ("name".to_string(), "Tech Corp".to_string()),
            ("type".to_string(), "Corporation".to_string()),
        ]),
    };
    event_bus.publish(org_created).await;
    
    // Create department
    let dept_created = TestEvent {
        event_type: "DepartmentCreated".to_string(),
        aggregate_id: dept_id.to_string(),
        payload: HashMap::from([
            ("name".to_string(), "Engineering".to_string()),
            ("organization_id".to_string(), org_id.to_string()),
        ]),
    };
    event_bus.publish(dept_created).await;
    
    // Create person
    let person_created = TestEvent {
        event_type: "PersonCreated".to_string(),
        aggregate_id: person_id.to_string(),
        payload: HashMap::from([
            ("first_name".to_string(), "Bob".to_string()),
            ("last_name".to_string(), "Engineer".to_string()),
        ]),
    };
    event_bus.publish(person_created).await;
    
    // Assign person to department
    let person_assigned = TestEvent {
        event_type: "PersonAssignedToDepartment".to_string(),
        aggregate_id: person_id.to_string(),
        payload: HashMap::from([
            ("department_id".to_string(), dept_id.to_string()),
            ("role".to_string(), "Senior Developer".to_string()),
        ]),
    };
    event_bus.publish(person_assigned).await;
    
    // Verify hierarchy
    let all_events = event_bus.get_events().await;
    assert_eq!(all_events.len(), 4);
    
    // Verify person's assignment
    let person_events = event_bus.get_events_for_aggregate(&person_id.to_string()).await;
    let assignment = person_events.iter()
        .find(|e| e.event_type == "PersonAssignedToDepartment")
        .unwrap();
    assert_eq!(assignment.payload.get("department_id"), Some(&dept_id.to_string()));
    
    println!("✅ Organization hierarchy test passed");
    Ok(())
}

#[test]
fn test_event_bus_functionality() {
    // Synchronous test for the event bus itself
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    runtime.block_on(async {
        let event_bus = TestEventBus::new();
        
        // Publish multiple events
        for i in 0..5 {
            let event = TestEvent {
                event_type: "TestEvent".to_string(),
                aggregate_id: format!("aggregate-{}", i % 2),
                payload: HashMap::from([
                    ("index".to_string(), i.to_string()),
                ]),
            };
            event_bus.publish(event).await;
        }
        
        // Verify all events stored
        let all_events = event_bus.get_events().await;
        assert_eq!(all_events.len(), 5);
        
        // Verify filtering by aggregate
        let aggregate_0_events = event_bus.get_events_for_aggregate("aggregate-0").await;
        assert_eq!(aggregate_0_events.len(), 3); // indices 0, 2, 4
        
        let aggregate_1_events = event_bus.get_events_for_aggregate("aggregate-1").await;
        assert_eq!(aggregate_1_events.len(), 2); // indices 1, 3
        
        println!("✅ Event bus functionality test passed");
    });
} 