//! Basic integration test - minimal functionality verification

#[test]
fn test_basic_types() {
    use cim_domain::{GraphId, NodeId, EdgeId};
    
    // Create basic types
    let graph_id = GraphId::new();
    let node_id = NodeId::new();
    let edge_id = EdgeId::new();
    
    // Verify they're unique
    assert_ne!(graph_id.to_string(), node_id.to_string());
    assert_ne!(node_id.to_string(), edge_id.to_string());
    
    println!("✅ Basic types test passed");
}

#[tokio::test]
async fn test_async_functionality() {
    // Simple async test
    let result = async {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        42
    }.await;
    
    assert_eq!(result, 42);
    println!("✅ Async test passed");
}

#[test]
fn test_cross_domain_types() {
    use cim_domain::NodeId;
    
    // Test person domain
    use cim_domain_person::Person;
    let person_id = NodeId::new();
    let person = Person::new(person_id, "Alice".to_string(), "Smith".to_string());
    assert_eq!(person.first_name(), "Alice");
    assert_eq!(person.last_name(), "Smith");
    
    // Test agent domain
    use cim_domain_agent::{Agent, AgentType};
    let agent_id = NodeId::new();
    let agent = Agent::new(agent_id, AgentType::Human, person_id);
    assert_eq!(agent.owner_id(), person_id);
    
    println!("✅ Cross-domain types test passed");
}

#[test]
fn test_graph_events() {
    use cim_domain_graph::{GraphCreated, NodeAdded, Position3D};
    use cim_domain::{GraphId, NodeId};
    use std::collections::HashMap;
    
    // Create events
    let graph_id = GraphId::new();
    let node_id = NodeId::new();
    
    let graph_event = GraphCreated {
        graph_id,
        name: "Test".to_string(),
        description: "Test graph".to_string(),
        graph_type: None,
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
    };
    
    let node_event = NodeAdded {
        graph_id,
        node_id,
        position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
        node_type: "test".to_string(),
        metadata: HashMap::new(),
    };
    
    // Verify
    assert_eq!(graph_event.graph_id, graph_id);
    assert_eq!(node_event.node_id, node_id);
    
    println!("✅ Graph events test passed");
}

#[test]
fn test_all_domains_compile() {
    // Just verify we can import from all domains
    use cim_domain_graph::GraphCreated;
    use cim_domain_person::Person;
    use cim_domain_agent::Agent;
    use cim_domain_workflow::WorkflowCreated;
    use cim_domain_location::LocationCreated;
    use cim_domain_identity::IdentityCreated;
    use cim_domain_organization::OrganizationCreated;
    use cim_domain_document::DocumentCreated;
    
    println!("✅ All domains compile and export expected types");
} 