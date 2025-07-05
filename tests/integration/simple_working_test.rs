//! Simple working integration test

use cim_domain::{DomainResult, GraphId, NodeId, EdgeId};

#[tokio::test]
async fn test_basic_types_creation() -> DomainResult<()> {
    // Test basic type creation
    let graph_id = GraphId::new();
    let node_id = NodeId::new();
    let edge_id = EdgeId::new();
    
    // Verify they're unique
    assert_ne!(graph_id.as_uuid(), node_id.as_uuid());
    assert_ne!(node_id.as_uuid(), edge_id.as_uuid());
    assert_ne!(graph_id.as_uuid(), edge_id.as_uuid());
    
    Ok(())
}

#[tokio::test]
async fn test_domain_graph_creation() -> DomainResult<()> {
    use cim_domain_graph::{DomainGraph, GraphType};
    
    // Create a domain graph
    let graph_id = GraphId::new();
    let graph = DomainGraph::new(graph_id, "Test Graph".to_string(), GraphType::Directed);
    
    // Verify basic properties
    assert_eq!(graph.id(), graph_id);
    assert_eq!(graph.name(), "Test Graph");
    
    Ok(())
}

#[tokio::test]
async fn test_cross_domain_types() -> DomainResult<()> {
    // Test that we can use types from different domains
    use cim_domain_person::Person;
    use cim_domain_agent::{Agent, AgentType};
    
    // Create a person
    let person_id = NodeId::new();
    let person = Person::new(person_id, "Alice".to_string(), "Smith".to_string());
    
    // Create an agent owned by the person
    let agent_id = NodeId::new();
    let agent = Agent::new(agent_id, AgentType::Human, person_id);
    
    // Verify relationships
    assert_eq!(agent.owner_id(), person_id);
    assert_eq!(agent.agent_type(), AgentType::Human);
    
    Ok(())
}

#[test]
fn test_sync_operations() {
    // Test synchronous operations
    let id1 = NodeId::new();
    let id2 = NodeId::new();
    
    // IDs should be unique
    assert_ne!(id1, id2);
    
    // Test display
    let id_string = id1.to_string();
    assert!(!id_string.is_empty());
} 