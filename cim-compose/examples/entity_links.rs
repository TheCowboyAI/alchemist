//! Example demonstrating how entity links are represented through graph composition

use cim_compose::base_types::*;
use cim_compose::composition::*;

fn main() {
    println!("=== Entity Links through Graph Composition ===\n");

    // Create two graph entities - these have persistent identity
    let workflow_graph = create_workflow_graph();
    let data_model_graph = create_data_model_graph();

    println!("Created workflow graph with ID: {}", workflow_graph.id);
    println!("Created data model graph with ID: {}", data_model_graph.id);

    // Entity links are just graphs themselves!
    // They contain EntityReference nodes that point to other entities
    let dependency_graph = create_dependency_link_graph(
        workflow_graph.id,
        data_model_graph.id,
        "Workflow processes data from model"
    );

    println!("\nCreated dependency link graph with ID: {}", dependency_graph.id);
    println!("This graph represents the relationship between the two entities");

    // Show the structure of the link graph
    for (_, node) in &dependency_graph.nodes {
        if node.node_type == BaseNodeType::EntityReference {
            println!("  - Node '{}' references entity: {}",
                node.label,
                node.data.get("entity_id").unwrap_or(&serde_json::json!("unknown"))
            );
        }
    }

    // You can create more complex link graphs
    let system_overview = create_system_overview_graph(vec![
        (workflow_graph.id, "OrderWorkflow"),
        (data_model_graph.id, "OrderModel"),
        (dependency_graph.id, "Dependencies"),
    ]);

    println!("\nCreated system overview graph with {} entity references",
        system_overview.nodes.values()
            .filter(|n| n.node_type == BaseNodeType::EntityReference)
            .count()
    );

    // The key insight: everything is a graph!
    // - Workflows are graphs
    // - Data models are graphs
    // - The relationships between them are also graphs
    // - Even the overview of all these things is a graph

    println!("\nAll of these are entities with persistent IDs:");
    println!("  - Workflow: {}", workflow_graph.id);
    println!("  - Data Model: {}", data_model_graph.id);
    println!("  - Dependency Link: {}", dependency_graph.id);
    println!("  - System Overview: {}", system_overview.id);

    // When these graphs change, those changes would be tracked via IPLD/CID chains
    // in the event store, not in the graph structure itself
    println!("\nChanges to these graphs would be tracked through:");
    println!("  - Domain events (with CID chains for integrity)");
    println!("  - IPLD for large object storage");
    println!("  - NOT by making nodes/edges into entities");
}

fn create_workflow_graph() -> GraphComposition {
    GraphComposition::composite("OrderProcessingWorkflow")
        .add_node(BaseNodeType::Command, "ReceiveOrder", serde_json::json!({}))
        .add_node(BaseNodeType::Service, "ValidateOrder", serde_json::json!({}))
        .add_node(BaseNodeType::Service, "ProcessPayment", serde_json::json!({}))
        .add_node(BaseNodeType::Event, "OrderCompleted", serde_json::json!({}))
        .add_edge_by_label("ReceiveOrder", "ValidateOrder", BaseRelationshipType::Sequence)
        .add_edge_by_label("ValidateOrder", "ProcessPayment", BaseRelationshipType::Sequence)
        .add_edge_by_label("ProcessPayment", "OrderCompleted", BaseRelationshipType::Sequence)
}

fn create_data_model_graph() -> GraphComposition {
    GraphComposition::composite("OrderDataModel")
        .add_node(BaseNodeType::Aggregate, "Order", serde_json::json!({
            "fields": ["id", "customer_id", "items", "total"]
        }))
        .add_node(BaseNodeType::Value, "OrderItem", serde_json::json!({
            "fields": ["product_id", "quantity", "price"]
        }))
        .add_node(BaseNodeType::Value, "Money", serde_json::json!({
            "fields": ["amount", "currency"]
        }))
        .add_edge_by_label("Order", "OrderItem", BaseRelationshipType::Contains)
        .add_edge_by_label("Order", "Money", BaseRelationshipType::Contains)
}

fn create_dependency_link_graph(
    source_id: GraphId,
    target_id: GraphId,
    reason: &str
) -> GraphComposition {
    GraphComposition::composite("DependencyLink")
        .add_node(BaseNodeType::EntityReference, "source", serde_json::json!({
            "entity_id": source_id.to_string(),
            "entity_type": "Graph"
        }))
        .add_node(BaseNodeType::EntityReference, "target", serde_json::json!({
            "entity_id": target_id.to_string(),
            "entity_type": "Graph"
        }))
        .add_node(BaseNodeType::Value, "metadata", serde_json::json!({
            "reason": reason,
            "created_at": "2024-01-01T00:00:00Z"
        }))
        .add_edge_by_label("source", "target", BaseRelationshipType::DependsOn)
        .add_edge_by_label("root", "metadata", BaseRelationshipType::Contains)
}

fn create_system_overview_graph(entities: Vec<(GraphId, &str)>) -> GraphComposition {
    let mut graph = GraphComposition::composite("SystemOverview");

    for (entity_id, label) in entities {
        graph = graph.add_node(
            BaseNodeType::EntityReference,
            label,
            serde_json::json!({
                "entity_id": entity_id.to_string(),
                "entity_type": "Graph"
            })
        );
    }

    graph
}
