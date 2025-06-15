//! Domain integration tests across multiple bounded contexts

use cim_domain::{DomainResult, GraphId, NodeId};
use cim_domain_graph::{GraphAggregate, GraphCommand, NodeType, StepType};
use cim_domain_workflow::{WorkflowAggregate, WorkflowCommand};

#[tokio::test]
async fn test_graph_workflow_integration() -> DomainResult<()> {
    // This test demonstrates how graph and workflow domains can work together

    // Arrange - Create a graph that represents a workflow
    let graph_id = GraphId::new();
    let mut graph = GraphAggregate::new(
        graph_id,
        "Order Processing Workflow".to_string(),
        cim_domain_graph::GraphType::WorkflowGraph,
    );

    // Add workflow nodes to graph
    let start_node = NodeId::new();
    let process_node = NodeId::new();
    let end_node = NodeId::new();

    // Act - Build workflow structure in graph
    let commands = vec![
        GraphCommand::AddNode {
            node_type: NodeType::WorkflowStep {
                step_type: StepType::Start,
            },
            position: cim_domain_graph::Position3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            metadata: Default::default(),
        },
        GraphCommand::AddNode {
            node_type: NodeType::WorkflowStep {
                step_type: StepType::Process,
            },
            position: cim_domain_graph::Position3D {
                x: 5.0,
                y: 0.0,
                z: 0.0,
            },
            metadata: Default::default(),
        },
        GraphCommand::AddNode {
            node_type: NodeType::WorkflowStep {
                step_type: StepType::End,
            },
            position: cim_domain_graph::Position3D {
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
        all_events.extend(events);
    }

    // Assert - Graph should contain workflow structure
    assert_eq!(graph.node_count(), 3);
    assert_eq!(all_events.len(), 3);

    // Now create corresponding workflow aggregate
    let workflow_id = cim_domain::WorkflowId::new();
    let mut workflow = WorkflowAggregate::new(
        workflow_id,
        "Order Processing".to_string(),
        graph_id, // Link to graph representation
    );

    // Workflow can reference the graph for visualization
    assert_eq!(workflow.graph_id(), graph_id);

    Ok(())
}

#[tokio::test]
async fn test_location_in_graph() -> DomainResult<()> {
    use cim_domain_location::{LocationAggregate, LocationCommand, LocationType};

    // Arrange - Create a graph with location nodes
    let mut graph = cim_domain_graph::GraphAggregate::new(
        GraphId::new(),
        "Facility Layout".to_string(),
        cim_domain_graph::GraphType::ConceptualGraph,
    );

    // Create location aggregate
    let location_id = cim_domain::NodeId::new(); // Locations can be nodes
    let mut location = LocationAggregate::new(
        location_id,
        "Warehouse A".to_string(),
        LocationType::Facility,
    );

    // Act - Add location as node in graph
    let graph_events = graph.handle_command(GraphCommand::AddNode {
        node_type: NodeType::Integration {
            system: "LocationSystem".to_string(),
        },
        position: cim_domain_graph::Position3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        metadata: {
            let mut meta = std::collections::HashMap::new();
            meta.insert(
                "location_id".to_string(),
                serde_json::json!(location_id.to_string()),
            );
            meta.insert("location_type".to_string(), serde_json::json!("Facility"));
            meta
        },
    })?;

    // Update location coordinates
    let location_events = location.handle_command(LocationCommand::UpdateCoordinates {
        latitude: 37.7749,
        longitude: -122.4194,
    })?;

    // Assert
    assert_eq!(graph.node_count(), 1);
    assert!(!graph_events.is_empty());
    assert!(!location_events.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_policy_enforcement_across_domains() -> DomainResult<()> {
    use cim_domain_policy::{PolicyAggregate, PolicyCommand, PolicyRule, PolicyType};

    // Arrange - Create a policy for graph modifications
    let policy_id = cim_domain::NodeId::new();
    let mut policy = PolicyAggregate::new(
        policy_id,
        "Graph Modification Policy".to_string(),
        PolicyType::Security,
    );

    // Add rule that limits graph size
    let rule = PolicyRule {
        id: cim_domain::NodeId::new(),
        name: "MaxNodesRule".to_string(),
        condition: "graph.node_count < 100".to_string(),
        action: "ALLOW".to_string(),
        priority: 1,
    };

    let policy_events = policy.handle_command(PolicyCommand::AddRule { rule: rule.clone() })?;

    // Create graph that respects policy
    let mut graph = cim_domain_graph::GraphAggregate::new(
        GraphId::new(),
        "Policy-Constrained Graph".to_string(),
        cim_domain_graph::GraphType::WorkflowGraph,
    );

    // Act - Try to add nodes (in real system, policy would be checked)
    let mut node_count = 0;
    let max_nodes = 5; // Simulating policy limit

    for i in 0..10 {
        if node_count < max_nodes {
            let events = graph.handle_command(GraphCommand::AddNode {
                node_type: NodeType::WorkflowStep {
                    step_type: StepType::Process,
                },
                position: cim_domain_graph::Position3D {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },
                metadata: Default::default(),
            })?;

            if !events.is_empty() {
                node_count += 1;
            }
        }
    }

    // Assert
    assert_eq!(graph.node_count(), max_nodes);
    assert!(policy.is_active());

    Ok(())
}

#[tokio::test]
async fn test_document_attached_to_graph_node() -> DomainResult<()> {
    use cim_domain_document::{DocumentAggregate, DocumentCommand, DocumentType};

    // Arrange - Create graph and document
    let mut graph = cim_domain_graph::GraphAggregate::new(
        GraphId::new(),
        "Documentation Graph".to_string(),
        cim_domain_graph::GraphType::ConceptualGraph,
    );

    let document_id = cim_domain::NodeId::new();
    let mut document = DocumentAggregate::new(
        document_id,
        "Architecture Diagram".to_string(),
        DocumentType::Diagram,
    );

    // Act - Add document as node in graph
    let node_events = graph.handle_command(GraphCommand::AddNode {
        node_type: NodeType::Integration {
            system: "DocumentSystem".to_string(),
        },
        position: cim_domain_graph::Position3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        metadata: {
            let mut meta = std::collections::HashMap::new();
            meta.insert(
                "document_id".to_string(),
                serde_json::json!(document_id.to_string()),
            );
            meta.insert("document_type".to_string(), serde_json::json!("Diagram"));
            meta
        },
    })?;

    // Update document content
    let doc_events = document.handle_command(DocumentCommand::UpdateContent {
        content: "graph TD; A-->B; B-->C;".to_string(),
        content_type: "text/vnd.mermaid".to_string(),
    })?;

    // Assert
    assert_eq!(graph.node_count(), 1);
    assert!(!node_events.is_empty());
    assert!(!doc_events.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_agent_person_organization_hierarchy() -> DomainResult<()> {
    use cim_domain_agent::{Agent, AgentStatus, AgentType};
    use cim_domain_organization::{Organization, OrganizationType};
    use cim_domain_person::Person;

    // Arrange - Create organizational hierarchy
    let org_id = cim_domain::NodeId::new();
    let org = Organization::new(
        org_id,
        "Tech Corp".to_string(),
        OrganizationType::Corporation,
    );

    let person_id = cim_domain::NodeId::new();
    let person = Person::new(person_id, "Alice".to_string(), "Smith".to_string());

    let agent_id = cim_domain::NodeId::new();
    let agent = Agent::new(
        agent_id,
        AgentType::Human,
        person_id, // Agent owned by person
    );

    // Create graph to visualize relationships
    let mut graph = cim_domain_graph::GraphAggregate::new(
        GraphId::new(),
        "Organizational Structure".to_string(),
        cim_domain_graph::GraphType::ConceptualGraph,
    );

    // Act - Add all entities as nodes
    let org_node_events = graph.handle_command(GraphCommand::AddNode {
        node_type: NodeType::Concept {
            embedding: cim_domain_graph::ConceptEmbedding::default(),
        },
        position: cim_domain_graph::Position3D {
            x: 0.0,
            y: 10.0,
            z: 0.0,
        },
        metadata: {
            let mut meta = std::collections::HashMap::new();
            meta.insert("entity_type".to_string(), serde_json::json!("Organization"));
            meta.insert(
                "entity_id".to_string(),
                serde_json::json!(org_id.to_string()),
            );
            meta
        },
    })?;

    let person_node_events = graph.handle_command(GraphCommand::AddNode {
        node_type: NodeType::Concept {
            embedding: cim_domain_graph::ConceptEmbedding::default(),
        },
        position: cim_domain_graph::Position3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        metadata: {
            let mut meta = std::collections::HashMap::new();
            meta.insert("entity_type".to_string(), serde_json::json!("Person"));
            meta.insert(
                "entity_id".to_string(),
                serde_json::json!(person_id.to_string()),
            );
            meta
        },
    })?;

    let agent_node_events = graph.handle_command(GraphCommand::AddNode {
        node_type: NodeType::Concept {
            embedding: cim_domain_graph::ConceptEmbedding::default(),
        },
        position: cim_domain_graph::Position3D {
            x: 0.0,
            y: -10.0,
            z: 0.0,
        },
        metadata: {
            let mut meta = std::collections::HashMap::new();
            meta.insert("entity_type".to_string(), serde_json::json!("Agent"));
            meta.insert(
                "entity_id".to_string(),
                serde_json::json!(agent_id.to_string()),
            );
            meta
        },
    })?;

    // Assert - All entities represented in graph
    assert_eq!(graph.node_count(), 3);
    assert!(!org_node_events.is_empty());
    assert!(!person_node_events.is_empty());
    assert!(!agent_node_events.is_empty());

    // Verify relationships
    assert_eq!(agent.owner_id(), person_id);
    assert_eq!(agent.agent_type(), AgentType::Human);
    assert_eq!(agent.status(), AgentStatus::Initializing);

    Ok(())
}
