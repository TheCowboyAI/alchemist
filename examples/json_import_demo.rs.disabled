//! # JSON Import Demo
//!
//! This demo shows how to import external graph data from JSON files into our ContentGraph system.
//! It demonstrates:
//! 1. Reading and parsing JSON graph formats
//! 2. Transforming external data to our domain model
//! 3. Generating proper domain events
//! 4. Creating a ContentGraph with CID

use cim_ipld::types::ContentType;
use colored::*;
use ia::domain::{
    aggregates::content_graph::{ContentGraph, NodeContent},
    events::{
        DomainEvent,
        content_graph::{ContentAdded, ContentGraphCreated, RelationshipEstablished},
    },
    value_objects::{EdgeId, GraphId, NodeId, Position3D, RelatedBy},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::time::SystemTime;

/// External JSON format (Neo4j-style export)
#[derive(Debug, Deserialize)]
struct ExternalGraph {
    nodes: Vec<ExternalNode>,
    relationships: Vec<ExternalRelationship>,
    style: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ExternalNode {
    id: String,
    position: Position2D,
    caption: String,
    labels: Vec<String>,
    properties: HashMap<String, Value>,
    style: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct Position2D {
    x: f64,
    y: f64,
}

#[derive(Debug, Deserialize)]
struct ExternalRelationship {
    id: String,
    #[serde(rename = "fromId")]
    from_id: String,
    #[serde(rename = "toId")]
    to_id: String,
    #[serde(rename = "type")]
    relationship_type: String,
    properties: HashMap<String, Value>,
    style: Option<HashMap<String, String>>,
}

/// Import result tracking
struct ImportResult {
    graph: ContentGraph,
    events: Vec<DomainEvent>,
    node_mapping: HashMap<String, NodeId>,
    warnings: Vec<String>,
}

fn main() {
    println!("{}", "=== JSON Import Demo ===".bright_blue().bold());
    println!();

    // Read the JSON file
    let json_path = "assets/models/CIM.json";
    println!("📁 Reading JSON file: {}", json_path.yellow());

    let json_content = match fs::read_to_string(json_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Failed to read file: {}", e);
            return;
        }
    };

    // Parse the JSON
    println!("🔍 Parsing JSON structure...");
    let external_graph: ExternalGraph = match serde_json::from_str(&json_content) {
        Ok(graph) => graph,
        Err(e) => {
            eprintln!("❌ Failed to parse JSON: {}", e);
            return;
        }
    };

    println!(
        "✅ Found {} nodes and {} relationships",
        external_graph.nodes.len().to_string().green(),
        external_graph.relationships.len().to_string().green()
    );

    // Import the graph
    println!("\n{}", "📥 Importing to ContentGraph...".bright_cyan());
    let result = import_external_graph(external_graph);

    // Display import results
    display_import_results(&result);

    // Show the generated CID
    println!("\n{}", "🔐 Content Addressing:".bright_magenta());
    let cid = result.graph.cid();
    println!("   CID: {}", cid.to_string().bright_yellow());
    println!("   This CID uniquely identifies this imported graph");

    // Export back to JSON to show round-trip capability
    println!("\n{}", "📤 Export Capability:".bright_cyan());
    export_to_json(&result.graph);
}

fn import_external_graph(external: ExternalGraph) -> ImportResult {
    let graph_id = GraphId::new();
    let mut events = Vec::new();
    let mut node_mapping = HashMap::new();
    let mut warnings = Vec::new();

    // Create the graph
    let create_event = ContentGraphCreated {
        graph_id: graph_id.clone(),
        created_at: SystemTime::now(),
    };
    events.push(DomainEvent::ContentGraphCreated(create_event.clone()));

    let mut graph = ContentGraph::new(graph_id.clone());

    // Apply the creation event
    if let Err(e) = graph.apply_event(&DomainEvent::ContentGraphCreated(create_event)) {
        warnings.push(format!("Failed to apply creation event: {}", e));
    }

    // Import nodes
    println!("\n{}", "Importing Nodes:".bright_green());
    for ext_node in external.nodes {
        let node_id = NodeId::new();
        node_mapping.insert(ext_node.id.clone(), node_id.clone());

        // Transform position from 2D to 3D
        let position = Position3D {
            x: ext_node.position.x as f32,
            y: ext_node.position.y as f32,
            z: 0.0, // Default Z coordinate
        };

        // Create node content
        let content = NodeContent::Value {
            content_type: ContentType::Json,
            data: json!({
                "caption": ext_node.caption,
                "labels": ext_node.labels,
                "properties": ext_node.properties,
                "style": ext_node.style,
                "original_id": ext_node.id,
            }),
        };

        // Convert metadata to HashMap<String, Value>
        let mut metadata = HashMap::new();
        metadata.insert("imported".to_string(), json!(true));
        metadata.insert("source_format".to_string(), json!("neo4j"));

        let add_event = ContentAdded {
            graph_id: graph_id.clone(),
            node_id: node_id.clone(),
            content,
            position,
            metadata,
            content_cid: None,
        };

        events.push(DomainEvent::ContentAdded(add_event.clone()));
        if let Err(e) = graph.apply_event(&DomainEvent::ContentAdded(add_event)) {
            warnings.push(format!("Failed to add node {}: {}", ext_node.caption, e));
        } else {
            println!(
                "   ✅ {} ({})",
                ext_node.caption.green(),
                ext_node.id.dimmed()
            );
        }
    }

    // Import relationships
    println!("\n{}", "Importing Relationships:".bright_green());
    for ext_rel in external.relationships {
        // Map external IDs to our NodeIds
        let source_id = match node_mapping.get(&ext_rel.from_id) {
            Some(id) => id.clone(),
            None => {
                warnings.push(format!(
                    "Source node {} not found for relationship {}",
                    ext_rel.from_id, ext_rel.id
                ));
                continue;
            }
        };

        let target_id = match node_mapping.get(&ext_rel.to_id) {
            Some(id) => id.clone(),
            None => {
                warnings.push(format!(
                    "Target node {} not found for relationship {}",
                    ext_rel.to_id, ext_rel.id
                ));
                continue;
            }
        };

        // Map relationship type
        let relationship = map_relationship_type(&ext_rel.relationship_type);

        let edge_event = RelationshipEstablished {
            graph_id: graph_id.clone(),
            edge_id: EdgeId::new(),
            source: source_id,
            target: target_id,
            relationship,
            strength: 1.0, // Default strength
        };

        events.push(DomainEvent::RelationshipEstablished(edge_event.clone()));
        if let Err(e) = graph.apply_event(&DomainEvent::RelationshipEstablished(edge_event)) {
            warnings.push(format!("Failed to add relationship: {}", e));
        } else {
            println!(
                "   ✅ {} → {} ({})",
                ext_rel.from_id.green(),
                ext_rel.to_id.green(),
                ext_rel.relationship_type.yellow()
            );
        }
    }

    ImportResult {
        graph,
        events,
        node_mapping,
        warnings,
    }
}

fn map_relationship_type(external_type: &str) -> RelatedBy {
    match external_type.to_uppercase().as_str() {
        "K8S" | "KUBERNETES" => RelatedBy::Custom("K8S".to_string()),
        "CONTAINS" => RelatedBy::Contains,
        "DEPENDS_ON" | "DEPENDSON" => RelatedBy::DependsOn,
        "" => RelatedBy::Custom("Connected".to_string()),
        other => RelatedBy::Custom(other.to_string()),
    }
}

fn display_import_results(result: &ImportResult) {
    println!("\n{}", "📊 Import Summary:".bright_magenta());
    println!(
        "   Total Events Generated: {}",
        result.events.len().to_string().green()
    );
    println!(
        "   Nodes Imported: {}",
        result.graph.nodes.len().to_string().green()
    );
    println!(
        "   Edges Imported: {}",
        result.graph.edges.len().to_string().green()
    );

    if !result.warnings.is_empty() {
        println!("\n{}", "⚠️  Warnings:".yellow());
        for warning in &result.warnings {
            println!("   - {}", warning);
        }
    }

    // Show graph structure
    println!("\n{}", "🌳 Graph Structure:".bright_cyan());

    // Group nodes by type/caption
    let mut node_types: HashMap<String, Vec<String>> = HashMap::new();
    for (node_id, node) in &result.graph.nodes {
        if let NodeContent::Value { data, .. } = &node.content {
            if let Some(caption) = data.get("caption").and_then(|v| v.as_str()) {
                let original_id = data
                    .get("original_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                node_types
                    .entry(caption.to_string())
                    .or_insert_with(Vec::new)
                    .push(original_id.to_string());
            }
        }
    }

    for (node_type, ids) in node_types {
        println!(
            "   {} {}: {}",
            "▶".green(),
            node_type.bright_white(),
            ids.join(", ").dimmed()
        );
    }

    // Show relationship statistics
    let mut rel_stats: HashMap<String, usize> = HashMap::new();
    for edge in result.graph.edges.values() {
        let rel_type = match &edge.relationship {
            RelatedBy::Contains => "Contains",
            RelatedBy::DependsOn => "DependsOn",
            RelatedBy::Similar => "Similar",
            RelatedBy::SameCategory => "SameCategory",
            RelatedBy::DerivedFrom => "DerivedFrom",
            RelatedBy::FlowsTo => "FlowsTo",
            RelatedBy::References => "References",
            RelatedBy::Alternative => "Alternative",
            RelatedBy::Custom(s) => s,
        };
        *rel_stats.entry(rel_type.to_string()).or_insert(0) += 1;
    }

    println!("\n{}", "🔗 Relationship Types:".bright_cyan());
    for (rel_type, count) in rel_stats {
        println!(
            "   {} {}: {}",
            "→".green(),
            rel_type.bright_white(),
            count.to_string().yellow()
        );
    }
}

fn export_to_json(graph: &ContentGraph) {
    // Create export structure
    let mut export = json!({
        "graph_id": graph.id.to_string(),
        "nodes": [],
        "edges": [],
    });

    // Export nodes
    let nodes: Vec<Value> = graph.nodes.iter().map(|(id, node)| {
        json!({
            "id": id.to_string(),
            "content": match &node.content {
                NodeContent::Value { data, .. } => data.clone(),
                NodeContent::Graph { graph_id, .. } => json!({ "graph_ref": graph_id.to_string() }),
                NodeContent::Reference { graph_id, node_id, .. } => json!({
                    "ref_graph": graph_id.to_string(),
                    "ref_node": node_id.to_string()
                }),
            },
            "position": node.position,
        })
    }).collect();

    // Export edges
    let edges: Vec<Value> = graph
        .edges
        .iter()
        .map(|(id, edge)| {
            json!({
                "id": id.to_string(),
                "source": edge.source.to_string(),
                "target": edge.target.to_string(),
                "relationship": format!("{:?}", edge.relationship),
            })
        })
        .collect();

    export["nodes"] = json!(nodes);
    export["edges"] = json!(edges);

    // Add CID
    export["cid"] = json!(graph.cid().to_string());

    println!("   ✅ Graph can be exported back to JSON");
    println!("   📝 Export includes CID for content verification");

    // Save to file (optional)
    let export_path = "examples/exported_graph.json";
    match fs::write(export_path, serde_json::to_string_pretty(&export).unwrap()) {
        Ok(_) => println!("   💾 Exported to: {}", export_path.green()),
        Err(e) => println!("   ⚠️  Failed to save export: {}", e),
    }
}
