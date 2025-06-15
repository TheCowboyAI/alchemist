//! # Document Context Demo
//!
//! This example demonstrates how to model a Document Management bounded context
//! using ContentGraph's recursive structure. It shows how DDD patterns map naturally
//! into our recursive ContentGraph structure.

use cim_ipld::ContentType;
use ia::domain::{
    aggregates::content_graph::{ContentGraph, GraphType, NodeContent},
    commands::ContentGraphCommand,
    events::DomainEvent,
    value_objects::{EdgeId, GraphId, NodeId, Position3D, RelatedBy},
};
use serde_json::json;
use std::collections::HashMap;

fn pause_for_enter() {
    println!("\nPress Enter to continue...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

fn main() {
    println!("Creating Document Context as ContentGraph...\n");

    // Create the root context graph
    let context_id = GraphId::new();
    let mut context_graph = ContentGraph::new(context_id);
    let mut all_events = Vec::new();

    // Track events as we build the graph
    let events = context_graph
        .handle_command(ContentGraphCommand::CreateGraph {
            graph_id: context_id,
        })
        .unwrap();
    all_events.extend(events.clone());
    for event in &events {
        context_graph.apply_event(event).unwrap();
    }

    // Create the Document Aggregate
    let doc_aggregate_id = create_document_aggregate(&mut context_graph, &mut all_events);

    // Create the Document Requirement Aggregate
    let req_aggregate_id =
        create_document_requirement_aggregate(&mut context_graph, &mut all_events);

    // Connect the aggregates
    connect_aggregates(
        &mut context_graph,
        doc_aggregate_id,
        req_aggregate_id,
        &mut all_events,
    );

    // Create some domain events
    create_domain_events(&mut context_graph, doc_aggregate_id, &mut all_events);

    // Print the events that created the graph
    print_events(&all_events);

    // Uncomment to pause between sections
    // pause_for_enter();

    // Print the resulting graph structure
    print_graph_structure(&context_graph);

    // Uncomment to pause between sections
    // pause_for_enter();

    // Show some interesting queries
    demonstrate_queries(&context_graph);

    // Demonstrate CID changes
    demonstrate_cid_changes(&mut context_graph);

    println!("\nDemo complete!");
}

fn create_document_aggregate(graph: &mut ContentGraph, events: &mut Vec<DomainEvent>) -> NodeId {
    let aggregate_id = NodeId::new();

    // Add the aggregate node (which is itself a graph)
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

    // Add entity fields
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

        // Connect field to aggregate
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

    // Add value objects
    let file_metadata_id = create_file_metadata(graph, aggregate_id, events);
    let verification_status_id = create_verification_status(graph, aggregate_id, events);

    aggregate_id
}

fn create_file_metadata(
    graph: &mut ContentGraph,
    aggregate_id: NodeId,
    events: &mut Vec<DomainEvent>,
) -> NodeId {
    let metadata_id = NodeId::new();

    // FileMetadata is a value object (also a graph)
    let command = ContentGraphCommand::AddContent {
        node_id: metadata_id,
        content: NodeContent::Graph {
            graph_id: GraphId::new(),
            graph_type: GraphType::ValueObject {
                value_type: "FileMetadata".to_string(),
            },
            summary: "File metadata value object".to_string(),
        },
        position: Position3D::new(2.0, 0.0, 0.0).unwrap(),
        metadata: HashMap::new(),
    };

    let new_events = graph.handle_command(command).unwrap();
    for event in &new_events {
        graph.apply_event(event).unwrap();
    }
    events.extend(new_events);

    // Connect to aggregate
    let edge_id = EdgeId::new();
    let command = ContentGraphCommand::EstablishRelationship {
        edge_id,
        source: aggregate_id,
        target: metadata_id,
        relationship: RelatedBy::Contains,
    };

    let new_events = graph.handle_command(command).unwrap();
    for event in &new_events {
        graph.apply_event(event).unwrap();
    }
    events.extend(new_events);

    // Add fields to FileMetadata
    let field_id = NodeId::new();
    let command = ContentGraphCommand::AddContent {
        node_id: field_id,
        content: NodeContent::Value {
            content_type: ContentType::Custom(0x300202),
            data: json!({
                "field": "fileName",
                "type": "String",
                "constraints": ["required", "max_length:255"],
            }),
        },
        position: Position3D::new(3.0, 0.0, 0.0).unwrap(),
        metadata: HashMap::new(),
    };

    let new_events = graph.handle_command(command).unwrap();
    for event in &new_events {
        graph.apply_event(event).unwrap();
    }
    events.extend(new_events);

    // Connect field to metadata
    let edge_id = EdgeId::new();
    let command = ContentGraphCommand::EstablishRelationship {
        edge_id,
        source: metadata_id,
        target: field_id,
        relationship: RelatedBy::Contains,
    };

    let new_events = graph.handle_command(command).unwrap();
    for event in &new_events {
        graph.apply_event(event).unwrap();
    }
    events.extend(new_events);

    metadata_id
}

fn create_verification_status(
    graph: &mut ContentGraph,
    aggregate_id: NodeId,
    events: &mut Vec<DomainEvent>,
) -> NodeId {
    let status_id = NodeId::new();

    // VerificationStatus value object
    let command = ContentGraphCommand::AddContent {
        node_id: status_id,
        content: NodeContent::Graph {
            graph_id: GraphId::new(),
            graph_type: GraphType::ValueObject {
                value_type: "VerificationStatus".to_string(),
            },
            summary: "Document verification status".to_string(),
        },
        position: Position3D::new(2.0, 2.0, 0.0).unwrap(),
        metadata: HashMap::new(),
    };

    let new_events = graph.handle_command(command).unwrap();
    for event in &new_events {
        graph.apply_event(event).unwrap();
    }
    events.extend(new_events);

    // Connect to aggregate
    let edge_id = EdgeId::new();
    let command = ContentGraphCommand::EstablishRelationship {
        edge_id,
        source: aggregate_id,
        target: status_id,
        relationship: RelatedBy::Contains,
    };

    let new_events = graph.handle_command(command).unwrap();
    for event in &new_events {
        graph.apply_event(event).unwrap();
    }
    events.extend(new_events);

    // Add verification rule
    let rule_id = create_verification_rule(graph, status_id, events);

    status_id
}

fn create_verification_rule(
    graph: &mut ContentGraph,
    status_id: NodeId,
    events: &mut Vec<DomainEvent>,
) -> NodeId {
    let rule_id = NodeId::new();

    let command = ContentGraphCommand::AddContent {
        node_id: rule_id,
        content: NodeContent::Value {
            content_type: ContentType::Custom(0x300203),
            data: json!({
                "rule": "DocumentMustBeSignedByAuthorizedParty",
                "severity": "Required",
                "validation": "signature.isValid() && authorizedParties.contains(signature.party)",
            }),
        },
        position: Position3D::new(3.0, 2.0, 0.0).unwrap(),
        metadata: HashMap::new(),
    };

    let new_events = graph.handle_command(command).unwrap();
    for event in &new_events {
        graph.apply_event(event).unwrap();
    }
    events.extend(new_events);

    // Connect to status
    let edge_id = EdgeId::new();
    let command = ContentGraphCommand::EstablishRelationship {
        edge_id,
        source: status_id,
        target: rule_id,
        relationship: RelatedBy::Custom("EnforcesRule".to_string()),
    };

    let new_events = graph.handle_command(command).unwrap();
    for event in &new_events {
        graph.apply_event(event).unwrap();
    }
    events.extend(new_events);

    rule_id
}

fn create_document_requirement_aggregate(
    graph: &mut ContentGraph,
    events: &mut Vec<DomainEvent>,
) -> NodeId {
    let aggregate_id = NodeId::new();

    // Add the requirement aggregate
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

    // Add fields
    let fields = vec![
        ("RequirementId", "Unique requirement identifier"),
        ("RequirementStatus", "Current status of requirement"),
        ("dueDate", "When document is required by"),
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
                }),
            },
            position: Position3D::new(1.0, 5.0 + i as f32 * 0.5, 0.0).unwrap(),
            metadata: HashMap::new(),
        };

        let new_events = graph.handle_command(command).unwrap();
        for event in &new_events {
            graph.apply_event(event).unwrap();
        }
        events.extend(new_events);

        // Connect to aggregate
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

    // Add entity
    let doc_type_req = create_document_type_requirement(graph, aggregate_id, events);

    aggregate_id
}

fn create_document_type_requirement(
    graph: &mut ContentGraph,
    aggregate_id: NodeId,
    events: &mut Vec<DomainEvent>,
) -> NodeId {
    let entity_id = NodeId::new();

    let command = ContentGraphCommand::AddContent {
        node_id: entity_id,
        content: NodeContent::Graph {
            graph_id: GraphId::new(),
            graph_type: GraphType::Entity {
                entity_type: "DocumentTypeRequirement".to_string(),
            },
            summary: "Specific document type requirement".to_string(),
        },
        position: Position3D::new(2.0, 6.0, 0.0).unwrap(),
        metadata: HashMap::new(),
    };

    let new_events = graph.handle_command(command).unwrap();
    for event in &new_events {
        graph.apply_event(event).unwrap();
    }
    events.extend(new_events);

    // Connect to aggregate
    let edge_id = EdgeId::new();
    let command = ContentGraphCommand::EstablishRelationship {
        edge_id,
        source: aggregate_id,
        target: entity_id,
        relationship: RelatedBy::Contains,
    };

    let new_events = graph.handle_command(command).unwrap();
    for event in &new_events {
        graph.apply_event(event).unwrap();
    }
    events.extend(new_events);

    entity_id
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
    // Create DocumentUploadedEvent
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

    // Connect event to aggregate
    let edge_id = EdgeId::new();
    let command = ContentGraphCommand::EstablishRelationship {
        edge_id,
        source: doc_aggregate_id,
        target: event_id,
        relationship: RelatedBy::Custom("EmitsEvent".to_string()),
    };

    let new_events = graph.handle_command(command).unwrap();
    for event in &new_events {
        graph.apply_event(event).unwrap();
    }
    events.extend(new_events);
}

fn print_events(events: &[DomainEvent]) {
    println!("Events that created the graph:");
    println!("==============================\n");

    for (i, event) in events.iter().enumerate() {
        println!("Event #{}: {}", i + 1, format_event(event));
    }

    println!("\nTotal events: {}\n", events.len());
}

fn format_event(event: &DomainEvent) -> String {
    match event {
        DomainEvent::ContentGraphCreated(e) => {
            format!("ContentGraphCreated {{ graph_id: {} }}", e.graph_id)
        }
        DomainEvent::ContentAdded(e) => {
            let content_type = match &e.content {
                NodeContent::Value { .. } => "Value",
                NodeContent::Graph { graph_type, .. } => match graph_type {
                    GraphType::Aggregate { aggregate_type } => {
                        &format!("Aggregate<{}>", aggregate_type)
                    }
                    GraphType::Entity { entity_type } => &format!("Entity<{}>", entity_type),
                    GraphType::ValueObject { value_type } => {
                        &format!("ValueObject<{}>", value_type)
                    }
                    GraphType::Event { event_type } => &format!("Event<{}>", event_type),
                    _ => "Graph",
                },
                NodeContent::Reference { .. } => "Reference",
            };
            format!(
                "ContentAdded {{ node_id: {}, type: {} }}",
                e.node_id, content_type
            )
        }
        DomainEvent::RelationshipEstablished(e) => {
            format!(
                "RelationshipEstablished {{ {} --[{:?}]--> {} }}",
                e.source, e.relationship, e.target
            )
        }
        _ => format!("{:?}", event),
    }
}

fn print_graph_structure(graph: &ContentGraph) {
    println!("\nResulting Graph Structure:");
    println!("==========================\n");

    println!("Document Context Graph created with:");
    println!("- {} nodes", graph.nodes.len());
    println!("- {} edges", graph.edges.len());

    // Highlight the CID
    println!("\n🔐 Content-Addressed Identifier (CID):");
    println!("   {}", graph.cid());
    println!("   This CID uniquely identifies this exact graph state.");
    println!("   Any change to the graph will produce a different CID.\n");

    // Group nodes by type
    let mut aggregates = Vec::new();
    let mut entities = Vec::new();
    let mut value_objects = Vec::new();
    let mut events = Vec::new();
    let mut values = Vec::new();

    for (node_id, node) in &graph.nodes {
        match &node.content {
            NodeContent::Graph { graph_type, .. } => match graph_type {
                GraphType::Aggregate { aggregate_type } => {
                    aggregates.push((node_id, aggregate_type));
                }
                GraphType::Entity { entity_type } => {
                    entities.push((node_id, entity_type));
                }
                GraphType::ValueObject { value_type } => {
                    value_objects.push((node_id, value_type));
                }
                GraphType::Event { event_type } => {
                    events.push((node_id, event_type));
                }
                _ => {}
            },
            NodeContent::Value { data, .. } => {
                if let Some(field) = data.get("field") {
                    values.push((node_id, field.as_str().unwrap_or("unknown")));
                }
            }
            _ => {}
        }
    }

    // Print structured view
    println!("Aggregates:");
    for (id, name) in &aggregates {
        println!("  - {} ({})", name, id);
        print_children(graph, id, 2);
    }

    if !events.is_empty() {
        println!("\nDomain Events:");
        for (id, name) in &events {
            println!("  - {} ({})", name, id);
        }
    }

    // Show nested graphs
    let nested_graphs = graph.get_nested_graphs();
    println!("\nNested Graphs: {}", nested_graphs.len());
    for (node_id, graph_id, graph_type) in &nested_graphs[..5.min(nested_graphs.len())] {
        println!("  - {:?} at node {}", graph_type, node_id);
    }
}

fn print_children(graph: &ContentGraph, parent_id: &NodeId, indent: usize) {
    let children: Vec<_> = graph
        .edges
        .values()
        .filter(|edge| edge.source == *parent_id)
        .collect();

    for edge in children {
        if let Some(child_node) = graph.nodes.get(&edge.target) {
            let prefix = " ".repeat(indent) + "└─ ";
            match &child_node.content {
                NodeContent::Graph { graph_type, .. } => {
                    println!("{}{:?}", prefix, graph_type);
                }
                NodeContent::Value { data, .. } => {
                    if let Some(field) = data.get("field") {
                        println!("{}Field: {:?}", prefix, field);
                    } else {
                        println!("{}Value node", prefix);
                    }
                }
                _ => {}
            }
            // Recursively print children
            print_children(graph, &edge.target, indent + 2);
        }
    }
}

fn demonstrate_queries(graph: &ContentGraph) {
    println!("\nInteresting Queries:");
    println!("===================\n");

    // Count relationships by type
    let mut relationship_counts = HashMap::new();
    for edge in graph.edges.values() {
        *relationship_counts.entry(&edge.relationship).or_insert(0) += 1;
    }

    println!("Relationships by type:");
    for (rel_type, count) in relationship_counts {
        println!("  - {:?}: {}", rel_type, count);
    }

    // Find all value objects
    let value_object_count = graph
        .nodes
        .values()
        .filter(|node| {
            matches!(
                &node.content,
                NodeContent::Graph {
                    graph_type: GraphType::ValueObject { .. },
                    ..
                }
            )
        })
        .count();

    println!("\nValue Objects: {}", value_object_count);

    // Calculate graph metrics
    let metrics = graph.metrics();
    println!("\nGraph Metrics:");
    println!("  - Average degree: {:.2}", metrics.average_degree);
    println!("  - Pattern count: {}", metrics.pattern_count);
}

fn demonstrate_cid_changes(graph: &mut ContentGraph) {
    println!("\nCID Demonstration:");
    println!("==================\n");

    // Get initial CID
    let initial_cid = graph.cid().to_string();
    println!("Initial CID: {}", initial_cid);

    // Add a new node
    let test_node_id = NodeId::new();
    let command = ContentGraphCommand::AddContent {
        node_id: test_node_id,
        content: NodeContent::Value {
            content_type: ContentType::Custom(0x300204),
            data: json!({
                "test": "CID demonstration",
                "timestamp": "2024-01-15T12:00:00Z",
            }),
        },
        position: Position3D::new(10.0, 10.0, 0.0).unwrap(),
        metadata: HashMap::new(),
    };

    let events = graph.handle_command(command).unwrap();
    for event in &events {
        graph.apply_event(event).unwrap();
    }

    // Get new CID
    let new_cid = graph.cid().to_string();
    println!("After adding node: {}", new_cid);
    println!("CIDs are different: {}", initial_cid != new_cid);

    // Remove the node
    let command = ContentGraphCommand::RemoveContent {
        node_id: test_node_id,
    };

    let events = graph.handle_command(command).unwrap();
    for event in &events {
        graph.apply_event(event).unwrap();
    }

    // Get CID after removal
    let final_cid = graph.cid().to_string();
    println!("After removing node: {}", final_cid);
    println!("Returned to original: {}", initial_cid == final_cid);
}
