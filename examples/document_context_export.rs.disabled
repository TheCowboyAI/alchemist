//! # Document Context Export Demo
//!
//! This example creates the same Document Context as the main demo
//! but exports it as JSON for inspection.

use cim_ipld::ContentType;
use ia::domain::{
    aggregates::content_graph::{ContentGraph, GraphType, NodeContent},
    commands::ContentGraphCommand,
    events::DomainEvent,
    value_objects::{EdgeId, GraphId, NodeId, Position3D, RelatedBy},
};
use serde_json::json;
use std::collections::HashMap;

fn main() {
    println!("Creating and exporting Document Context as JSON...\n");

    // Create the graph (same as main demo)
    let context_id = GraphId::new();
    let mut context_graph = ContentGraph::new(context_id);
    let mut all_events = Vec::new();

    // Initialize
    let events = context_graph
        .handle_command(ContentGraphCommand::CreateGraph {
            graph_id: context_id,
        })
        .unwrap();
    all_events.extend(events.clone());
    for event in &events {
        context_graph.apply_event(event).unwrap();
    }

    // Build the graph structure
    let doc_aggregate_id = create_document_aggregate(&mut context_graph, &mut all_events);
    let req_aggregate_id =
        create_document_requirement_aggregate(&mut context_graph, &mut all_events);
    connect_aggregates(
        &mut context_graph,
        doc_aggregate_id,
        req_aggregate_id,
        &mut all_events,
    );
    create_domain_events(&mut context_graph, doc_aggregate_id, &mut all_events);

    // Export as JSON
    match serde_json::to_string_pretty(&context_graph) {
        Ok(json) => {
            // Save to file
            let filename = "document_context_graph.json";
            match std::fs::write(filename, &json) {
                Ok(_) => println!("Graph exported to: {}", filename),
                Err(e) => println!("Failed to write file: {}", e),
            }

            // Show preview
            println!("\nJSON Preview (first 50 lines):");
            println!("================================");
            for (i, line) in json.lines().enumerate() {
                if i >= 50 {
                    println!("... (truncated, see {} for full content)", filename);
                    break;
                }
                println!("{}", line);
            }
        }
        Err(e) => println!("Failed to serialize graph: {}", e),
    }

    // Also export events
    match serde_json::to_string_pretty(&all_events) {
        Ok(json) => {
            let filename = "document_context_events.json";
            match std::fs::write(filename, &json) {
                Ok(_) => println!("\nEvents exported to: {}", filename),
                Err(e) => println!("Failed to write events file: {}", e),
            }
        }
        Err(e) => println!("Failed to serialize events: {}", e),
    }

    println!("\nExport complete!");
}

// Include all the helper functions from the main demo
fn create_document_aggregate(graph: &mut ContentGraph, events: &mut Vec<DomainEvent>) -> NodeId {
    let aggregate_id = NodeId::new();

    let command = ContentGraphCommand::AddContent {
        node_id: aggregate_id,
        content: NodeContent::Graph {
            graph_id: GraphId::new(),
            graph_type: GraphType::Aggregate {
                aggregate_type: "DocumentAggregate".to_string(),
            },
            summary: "Document aggregate root".to_string(),
        },
        position: Position3D::new(0.0, 0.0, 0.0).unwrap(),
        metadata: HashMap::new(),
    };

    let new_events = graph.handle_command(command).unwrap();
    for event in &new_events {
        graph.apply_event(event).unwrap();
    }
    events.extend(new_events);

    // Add fields
    let fields = vec![
        ("DocumentId", "Unique identifier"),
        ("DocumentType", "Type of document"),
        ("DocumentStatus", "Current status"),
    ];

    for (i, (field_name, description)) in fields.iter().enumerate() {
        let field_id = NodeId::new();
        let command = ContentGraphCommand::AddContent {
            node_id: field_id,
            content: NodeContent::Value {
                content_type: ContentType::Custom(0x300201),
                data: json!({
                    "field": field_name,
                    "description": description,
                    "type": "String",
                }),
            },
            position: Position3D::new(1.0, i as f32 * 0.5, 0.0).unwrap(),
            metadata: HashMap::new(),
        };

        let new_events = graph.handle_command(command).unwrap();
        for event in &new_events {
            graph.apply_event(event).unwrap();
        }
        events.extend(new_events.clone());

        let edge_id = EdgeId::new();
        let command = ContentGraphCommand::EstablishRelationship {
            edge_id,
            source: aggregate_id,
            target: field_id,
            relationship: RelatedBy::Contains,
        };

        let new_events = graph.handle_command(command).unwrap();
        for event in &new_events {
            graph.apply_event(event).unwrap();
        }
        events.extend(new_events);
    }

    aggregate_id
}

fn create_document_requirement_aggregate(
    graph: &mut ContentGraph,
    events: &mut Vec<DomainEvent>,
) -> NodeId {
    let aggregate_id = NodeId::new();

    let command = ContentGraphCommand::AddContent {
        node_id: aggregate_id,
        content: NodeContent::Graph {
            graph_id: GraphId::new(),
            graph_type: GraphType::Aggregate {
                aggregate_type: "DocumentRequirementAggregate".to_string(),
            },
            summary: "Document requirement aggregate".to_string(),
        },
        position: Position3D::new(0.0, 5.0, 0.0).unwrap(),
        metadata: HashMap::new(),
    };

    let new_events = graph.handle_command(command).unwrap();
    for event in &new_events {
        graph.apply_event(event).unwrap();
    }
    events.extend(new_events);

    aggregate_id
}

fn connect_aggregates(
    graph: &mut ContentGraph,
    doc_id: NodeId,
    req_id: NodeId,
    events: &mut Vec<DomainEvent>,
) {
    let edge_id = EdgeId::new();
    let command = ContentGraphCommand::EstablishRelationship {
        edge_id,
        source: req_id,
        target: doc_id,
        relationship: RelatedBy::DependsOn,
    };

    let new_events = graph.handle_command(command).unwrap();
    for event in &new_events {
        graph.apply_event(event).unwrap();
    }
    events.extend(new_events);
}

fn create_domain_events(
    graph: &mut ContentGraph,
    doc_aggregate_id: NodeId,
    events: &mut Vec<DomainEvent>,
) {
    let event_id = NodeId::new();
    let command = ContentGraphCommand::AddContent {
        node_id: event_id,
        content: NodeContent::Graph {
            graph_id: GraphId::new(),
            graph_type: GraphType::Event {
                event_type: "DocumentUploadedEvent".to_string(),
            },
            summary: "Document was uploaded".to_string(),
        },
        position: Position3D::new(4.0, 1.0, 0.0).unwrap(),
        metadata: HashMap::new(),
    };

    let new_events = graph.handle_command(command).unwrap();
    for event in &new_events {
        graph.apply_event(event).unwrap();
    }
    events.extend(new_events);
}
