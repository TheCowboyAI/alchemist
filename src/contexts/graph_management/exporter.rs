//! Graph file exporter for saving graphs to various formats
//!
//! This module provides functionality to export graph data to various formats,
//! starting with JSON format that preserves all graph data for round-trip operations.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::contexts::graph_management::domain::*;
use crate::contexts::graph_management::repositories::GraphData;
use crate::contexts::graph_management::storage::GraphStorage;

/// JSON representation of a node for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonNode {
    pub id: String,
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub category: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// JSON representation of an edge for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub category: String,
    pub strength: f32,
    pub properties: HashMap<String, serde_json::Value>,
}

/// JSON representation of a graph for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonGraph {
    pub id: String,
    pub name: String,
    pub description: String,
    pub domain: String,
    pub version: String,
    pub nodes: Vec<JsonNode>,
    pub edges: Vec<JsonEdge>,
    pub tags: Vec<String>,
}

/// Service for exporting graphs to various formats
#[derive(Debug, Clone)]
pub struct GraphExporter;

impl GraphExporter {
    /// Export a graph to JSON format
    pub fn export_to_json(graph_data: &GraphData) -> Result<String, String> {
        let json_nodes: Vec<JsonNode> = graph_data
            .nodes
            .iter()
            .map(|node| JsonNode {
                id: node.identity.0.to_string(),
                label: node.content.label.clone(),
                x: node.position.coordinates_3d.x,
                y: node.position.coordinates_3d.y,
                z: node.position.coordinates_3d.z,
                category: node.content.category.clone(),
                properties: node.content.properties.clone(),
            })
            .collect();

        let json_edges: Vec<JsonEdge> = graph_data
            .edges
            .iter()
            .map(|edge| JsonEdge {
                id: edge.identity.0.to_string(),
                source: edge.relationship.source.0.to_string(),
                target: edge.relationship.target.0.to_string(),
                category: edge.relationship.category.clone(),
                strength: edge.relationship.strength,
                properties: edge.relationship.properties.clone(),
            })
            .collect();

        let json_graph = JsonGraph {
            id: graph_data.identity.0.to_string(),
            name: graph_data.metadata.name.clone(),
            description: graph_data.metadata.description.clone(),
            domain: graph_data.metadata.domain.clone(),
            version: "1.0.0".to_string(),
            nodes: json_nodes,
            edges: json_edges,
            tags: graph_data.metadata.tags.clone(),
        };

        serde_json::to_string_pretty(&json_graph)
            .map_err(|e| format!("Failed to serialize graph: {}", e))
    }

    /// Export a graph to a JSON file
    pub fn export_to_file(path: &Path, graph_data: &GraphData) -> Result<(), String> {
        let json_content = Self::export_to_json(graph_data)?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        fs::write(path, json_content).map_err(|e| format!("Failed to write file: {}", e))
    }
}

/// System to handle export requests
pub fn handle_export_request(
    keyboard: Res<ButtonInput<KeyCode>>,
    storage: Res<GraphStorage>,
    graphs: Query<(&GraphIdentity, &GraphMetadata, &GraphJourney)>,
    nodes: Query<(&NodeIdentity, &NodeContent, &SpatialPosition), With<Node>>,
    edges: Query<(&EdgeIdentity, &EdgeRelationship), With<Edge>>,
    mut export_events: EventWriter<ExportGraphEvent>,
) {
    // Ctrl+S for save/export
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyS) {
        // Find the first graph (for now)
        if let Some((graph_id, _, _)) = graphs.iter().next() {
            export_events.send(ExportGraphEvent {
                graph_id: *graph_id,
            });
        } else {
            warn!("No graph to export");
        }
    }
}

/// Event triggered when a graph export is requested
#[derive(Event, Debug, Clone)]
pub struct ExportGraphEvent {
    pub graph_id: GraphIdentity,
}

/// Event triggered when a graph export completes
#[derive(Event, Debug, Clone)]
pub struct GraphExportedEvent {
    pub graph_id: GraphIdentity,
    pub path: String,
    pub success: bool,
    pub message: String,
}

/// System to process export events
pub fn process_export_events(
    mut export_events: EventReader<ExportGraphEvent>,
    mut exported_events: EventWriter<GraphExportedEvent>,
    graphs: Query<(&GraphIdentity, &GraphMetadata, &GraphJourney)>,
    nodes: Query<(&NodeIdentity, &NodeContent, &SpatialPosition), With<Node>>,
    edges: Query<(&EdgeIdentity, &EdgeRelationship), With<Edge>>,
) {
    for event in export_events.read() {
        // Find the graph
        let graph_data = graphs.iter().find(|(id, _, _)| **id == event.graph_id).map(
            |(id, metadata, journey)| {
                // Collect nodes for this graph
                let graph_nodes: Vec<_> = nodes
                    .iter()
                    .filter(|(_, _, _)| true) // TODO: Filter by graph when we have proper parent tracking
                    .map(|(id, content, pos)| {
                        crate::contexts::graph_management::repositories::NodeData {
                            identity: *id,
                            content: content.clone(),
                            position: *pos,
                        }
                    })
                    .collect();

                // Collect edges for this graph
                let graph_edges: Vec<_> = edges
                    .iter()
                    .filter(|(_, _)| true) // TODO: Filter by graph when we have proper parent tracking
                    .map(
                        |(id, rel)| crate::contexts::graph_management::repositories::EdgeData {
                            identity: *id,
                            relationship: rel.clone(),
                        },
                    )
                    .collect();

                GraphData {
                    identity: *id,
                    metadata: metadata.clone(),
                    journey: journey.clone(),
                    nodes: graph_nodes,
                    edges: graph_edges,
                }
            },
        );

        if let Some(graph_data) = graph_data {
            // Use file dialog to get save location
            let safe_name = graph_data
                .metadata
                .name
                .replace(' ', "_")
                .replace('/', "_")
                .replace('\\', "_");

            // Create file dialog
            let file_dialog = rfd::FileDialog::new()
                .set_file_name(&format!("{}.json", safe_name))
                .add_filter("JSON files", &["json"])
                .add_filter("All files", &["*"]);

            // Show save dialog (blocking)
            if let Some(path) = file_dialog.save_file() {
                match GraphExporter::export_to_file(&path, &graph_data) {
                    Ok(_) => {
                        exported_events.send(GraphExportedEvent {
                            graph_id: event.graph_id,
                            path: path.display().to_string(),
                            success: true,
                            message: format!("Graph exported successfully to {}", path.display()),
                        });
                    }
                    Err(e) => {
                        exported_events.send(GraphExportedEvent {
                            graph_id: event.graph_id,
                            path: path.display().to_string(),
                            success: false,
                            message: format!("Export failed: {}", e),
                        });
                    }
                }
            } else {
                // User cancelled the dialog
                exported_events.send(GraphExportedEvent {
                    graph_id: event.graph_id,
                    path: String::new(),
                    success: false,
                    message: "Export cancelled by user".to_string(),
                });
            }
        } else {
            exported_events.send(GraphExportedEvent {
                graph_id: event.graph_id,
                path: String::new(),
                success: false,
                message: "Graph not found".to_string(),
            });
        }
    }
}

/// System to display export feedback
pub fn display_export_feedback(mut exported_events: EventReader<GraphExportedEvent>) {
    for event in exported_events.read() {
        if event.success {
            info!("✅ {}", event.message);
        } else {
            error!("❌ {}", event.message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_export_to_json() {
        let graph_data = GraphData {
            identity: GraphIdentity(Uuid::new_v4()),
            metadata: GraphMetadata {
                name: "Test Graph".to_string(),
                description: "A test graph".to_string(),
                domain: "test".to_string(),
                created: std::time::SystemTime::now(),
                modified: std::time::SystemTime::now(),
                tags: vec!["test".to_string()],
            },
            journey: GraphJourney::default(),
            nodes: vec![crate::contexts::graph_management::repositories::NodeData {
                identity: NodeIdentity(Uuid::new_v4()),
                content: NodeContent {
                    label: "Node 1".to_string(),
                    category: "default".to_string(),
                    properties: HashMap::new(),
                },
                position: SpatialPosition::at_3d(100.0, 0.0, 200.0),
            }],
            edges: vec![],
        };

        let result = GraphExporter::export_to_json(&graph_data);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json.contains("Test Graph"));
        assert!(json.contains("Node 1"));
    }

    #[test]
    fn test_json_round_trip() {
        let node_id = NodeIdentity(Uuid::new_v4());
        let graph_data = GraphData {
            identity: GraphIdentity(Uuid::new_v4()),
            metadata: GraphMetadata {
                name: "Round Trip Test".to_string(),
                description: "Testing round trip".to_string(),
                domain: "test".to_string(),
                created: std::time::SystemTime::now(),
                modified: std::time::SystemTime::now(),
                tags: vec!["round-trip".to_string()],
            },
            journey: GraphJourney::default(),
            nodes: vec![crate::contexts::graph_management::repositories::NodeData {
                identity: node_id,
                content: NodeContent {
                    label: "First".to_string(),
                    category: "test".to_string(),
                    properties: HashMap::new(),
                },
                position: SpatialPosition::at_3d(50.0, 0.0, 50.0),
            }],
            edges: vec![],
        };

        let json = GraphExporter::export_to_json(&graph_data).unwrap();
        let parsed: JsonGraph = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, "Round Trip Test");
        assert_eq!(parsed.nodes.len(), 1);
        assert_eq!(parsed.nodes[0].label, "First");
        assert_eq!(parsed.nodes[0].x, 50.0);
        assert_eq!(parsed.nodes[0].z, 50.0);
    }
}
