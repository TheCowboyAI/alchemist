//! Systems for loading graph data from files
//!
//! These systems handle:
//! - JSON file parsing
//! - Graph reconstruction from saved data
//! - Error handling and validation

use bevy::prelude::*;
use std::fs;
use std::path::Path;
use serde_json;

use crate::{
    components::*,
    events::*,
    resources::*,
};

/// System that handles file load events
pub fn handle_load_file(
    mut events: EventReader<LoadJsonFileEvent>,
    mut create_node_events: EventWriter<CreateNodeEvent>,
    mut deferred_edge_events: EventWriter<DeferredEdgeEvent>,
    mut notification_events: EventWriter<ShowNotificationEvent>,
    mut file_operation_events: EventWriter<FileOperationCompleteEvent>,
) {
    for event in events.read() {
        match load_graph_from_file(&event.file_path) {
            Ok(graph_data) => {
                // Clear existing graph first
                // In practice, would send clear event

                // Create nodes
                for node in graph_data.nodes {
                    create_node_events.send(CreateNodeEvent {
                        id: node.id,
                        position: node.position,
                        domain_type: node.domain_type,
                        name: node.name,
                        labels: node.labels,
                        properties: node.properties,
                        subgraph_id: node.subgraph_id,
                        color: node.style.as_ref().and_then(|s| s.color.clone()),
                    });
                }

                // Create edges (deferred until nodes exist)
                for edge in graph_data.edges {
                    deferred_edge_events.send(DeferredEdgeEvent {
                        id: edge.id,
                        source_uuid: edge.source,
                        target_uuid: edge.target,
                        edge_type: edge.edge_type,
                        labels: edge.labels,
                        properties: edge.properties,
                        retry_count: 0,
                    });
                }

                // Send success notification
                notification_events.send(ShowNotificationEvent {
                    message: format!(
                        "Loaded graph: {} nodes, {} edges",
                        graph_data.nodes.len(),
                        graph_data.edges.len()
                    ),
                    notification_type: NotificationType::Success,
                    duration_seconds: 3.0,
                });

                // Send completion event
                file_operation_events.send(FileOperationCompleteEvent {
                    operation: FileOperation::Load,
                    success: true,
                    message: format!("Successfully loaded {}", event.file_path),
                });
            }
            Err(error) => {
                // Send error notification
                notification_events.send(ShowNotificationEvent {
                    message: format!("Failed to load file: {}", error),
                    notification_type: NotificationType::Error,
                    duration_seconds: 5.0,
                });

                // Send completion event
                file_operation_events.send(FileOperationCompleteEvent {
                    operation: FileOperation::Load,
                    success: false,
                    message: error.to_string(),
                });
            }
        }
    }
}

/// System that validates loaded graph data
pub fn validate_loaded_data(
    mut events: EventReader<FileOperationCompleteEvent>,
    mut validation_events: EventWriter<ValidateGraphEvent>,
) {
    for event in events.read() {
        if event.success && matches!(event.operation, FileOperation::Load) {
            // Trigger graph validation after successful load
            validation_events.send(ValidateGraphEvent);
        }
    }
}

// Helper functions

fn load_graph_from_file(file_path: &str) -> Result<GraphData, Box<dyn std::error::Error>> {
    let path = Path::new(file_path);

    if !path.exists() {
        return Err("File does not exist".into());
    }

    let contents = fs::read_to_string(path)?;
    let graph_data: GraphData = serde_json::from_str(&contents)?;

    // Basic validation
    if graph_data.version != "1.0" {
        return Err(format!("Unsupported file version: {}", graph_data.version).into());
    }

    Ok(graph_data)
}

// Data structures for JSON serialization
#[derive(serde::Deserialize)]
struct GraphData {
    version: String,
    nodes: Vec<NodeData>,
    edges: Vec<EdgeData>,
    metadata: Option<GraphMetadata>,
}

#[derive(serde::Deserialize)]
struct NodeData {
    id: uuid::Uuid,
    position: Vec3,
    domain_type: DomainNodeType,
    name: String,
    labels: Vec<String>,
    properties: std::collections::HashMap<String, String>,
    subgraph_id: Option<uuid::Uuid>,
    style: Option<NodeStyle>,
}

#[derive(serde::Deserialize)]
struct EdgeData {
    id: uuid::Uuid,
    source: uuid::Uuid,
    target: uuid::Uuid,
    edge_type: DomainEdgeType,
    labels: Vec<String>,
    properties: std::collections::HashMap<String, String>,
}

#[derive(serde::Deserialize)]
struct NodeStyle {
    color: Option<String>,
    icon: Option<String>,
    size: Option<f32>,
}

#[derive(serde::Deserialize)]
struct GraphMetadata {
    name: String,
    description: Option<String>,
    created_at: String,
    modified_at: String,
    author: Option<String>,
}
