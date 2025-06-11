//! Demo showing how graph-composition library integrates with our modular architecture

use graph_composition::{GraphComposition, BaseNodeType, BaseRelationshipType};
use ia::contexts::graph::domain::{
    ContextGraph,
    context_graph::{ContextType, DefaultInvariantValidator, DefaultPositionCalculator},
};
use ia::shared::types::{GraphId, NodeId};

fn main() {
    println!("=== Graph Composition Integration Demo ===\n");

    // 1. Create a standalone GraphComposition (from the library)
    println!("1. Creating a standalone GraphComposition:");
    let workflow = create_workflow_graph();
    println!("   Created workflow graph with ID: {}", workflow.id);
    println!("   Nodes: {}", workflow.nodes.len());
    println!("   Edges: {}", workflow.edges.len());

    // 2. Create a ContextGraph (our domain model that wraps GraphComposition)
    println!("\n2. Creating a ContextGraph (domain model):");
    let context_graph = create_context_graph();
    println!("   Created context graph with ID: {}", context_graph.id());
    println!("   Context type: {:?}", context_graph.context_type());
    println!("   Version: {}", context_graph.version());

    // 3. Show how they relate
    println!("\n3. Key Insights:");
    println!("   - GraphComposition is the pure data structure (from graph-composition crate)");
    println!("   - ContextGraph is our domain aggregate that adds behavior");
    println!("   - ContextGraph internally uses GraphComposition for storage");
    println!("   - All domain logic (commands, events, invariants) lives in ContextGraph");
    println!("   - GraphComposition provides the mathematical foundation");

    // 4. Show composition operations
    println!("\n4. Composition Operations:");
    let validate = GraphComposition::composite("ValidateOrder");
    let process = GraphComposition::composite("ProcessPayment");

    match validate.then(&process) {
        Ok(sequential) => {
            println!("   Sequential composition created: {} nodes", sequential.nodes.len());
        }
        Err(e) => println!("   Composition failed: {}", e),
    }

    match validate.parallel(&process) {
        Ok(parallel) => {
            println!("   Parallel composition created: {} nodes", parallel.nodes.len());
        }
        Err(e) => println!("   Composition failed: {}", e),
    }

    println!("\n✅ Demo complete - graph-composition library is properly integrated!");
}

/// Create a workflow using the graph-composition library directly
fn create_workflow_graph() -> GraphComposition {
    GraphComposition::composite("OrderProcessing")
        .add_node(BaseNodeType::Command, "ReceiveOrder", serde_json::json!({}))
        .add_node(BaseNodeType::Service, "ValidateOrder", serde_json::json!({}))
        .add_node(BaseNodeType::Service, "ProcessPayment", serde_json::json!({}))
        .add_node(BaseNodeType::Event, "OrderCompleted", serde_json::json!({}))
        .add_edge_by_label("ReceiveOrder", "ValidateOrder", BaseRelationshipType::Sequence)
        .add_edge_by_label("ValidateOrder", "ProcessPayment", BaseRelationshipType::Sequence)
        .add_edge_by_label("ProcessPayment", "OrderCompleted", BaseRelationshipType::Sequence)
}

/// Create a context graph using our domain model
fn create_context_graph() -> ContextGraph {
    let graph_id = GraphId::new();
    let root_id = NodeId::new();

    // Create with dependency injection
    let graph = ContextGraph::new(
        graph_id,
        "Order Processing Context".to_string(),
        ContextType::BoundedContext {
            name: "OrderManagement".to_string(),
            domain: "Sales".to_string(),
        },
        root_id,
        Box::new(DefaultInvariantValidator),
        Box::new(DefaultPositionCalculator),
    ).expect("Failed to create context graph");

    // Note: In a real application, you would use commands to modify the graph
    // Commands would be processed through command handlers that emit events
    // For this demo, we're just showing the structure

    println!("   Note: Graph modification requires commands in the real system");
    println!("   Commands -> Command Handlers -> Events -> Event Store");

    graph
}
