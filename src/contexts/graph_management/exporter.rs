//! Graph file exporter for saving graphs to various formats
//!
//! This module provides functionality to export graph data to various formats,
//! starting with JSON format that preserves all graph data for round-trip operations.

use crate::contexts::graph_management::domain::*;
use crate::contexts::graph_management::storage::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Internal graph format for export
#[derive(Debug, Serialize, Deserialize)]
pub struct InternalGraphFormat {
    pub version: String,
    pub metadata: GraphMetadata,
    pub nodes: Vec<InternalNode>,
    pub edges: Vec<InternalEdge>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalNode {
    pub id: String,
    pub position: Position3D,
    pub content: NodeContent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relationship: EdgeRelationship,
}

/// Service to export graphs
pub struct GraphExporter;

impl GraphExporter {
    /// Export a graph to JSON format
    pub fn export_to_json(
        graph_id: GraphIdentity,
        storage: &GraphStorage,
        nodes: &Query<(&NodeIdentity, &NodeContent, &SpatialPosition)>,
        edges: &Query<(&EdgeIdentity, &EdgeRelationship)>,
        graphs: &Query<(&GraphIdentity, &GraphMetadata)>,
    ) -> Result<String, ExportError> {
        // Get graph metadata
        let metadata = graphs
            .iter()
            .find(|(id, _)| **id == graph_id)
            .map(|(_, metadata)| metadata.clone())
            .ok_or(ExportError::GraphNotFound)?;

        // Collect nodes that belong to this graph
        // TODO: Filter by graph once we have proper parent tracking
        let mut internal_nodes = Vec::new();
        for (node_id, content, position) in nodes.iter() {
            internal_nodes.push(InternalNode {
                id: node_id.0.to_string(),
                position: Position3D {
                    x: position.coordinates_3d.x,
                    y: position.coordinates_3d.y,
                    z: position.coordinates_3d.z,
                },
                content: content.clone(),
            });
        }

        // Collect edges that belong to this graph
        // TODO: Filter by graph once we have proper parent tracking
        let mut internal_edges = Vec::new();
        for (edge_id, relationship) in edges.iter() {
            internal_edges.push(InternalEdge {
                id: edge_id.0.to_string(),
                source_id: relationship.source.0.to_string(),
                target_id: relationship.target.0.to_string(),
                relationship: relationship.clone(),
            });
        }

        // Create internal format
        let internal_format = InternalGraphFormat {
            version: "1.0.0".to_string(),
            metadata,
            nodes: internal_nodes,
            edges: internal_edges,
        };

        // Serialize to JSON
        serde_json::to_string_pretty(&internal_format)
            .map_err(|e| ExportError::SerializationError(e.to_string()))
    }

    /// Save JSON content to a file
    pub fn save_to_file(path: &Path, json_content: &str) -> Result<(), std::io::Error> {
        fs::write(path, json_content)
    }
}

#[derive(Debug)]
pub enum ExportError {
    GraphNotFound,
    SerializationError(String),
    IoError(std::io::Error),
}

impl From<std::io::Error> for ExportError {
    fn from(error: std::io::Error) -> Self {
        ExportError::IoError(error)
    }
}

/// System to handle export requests
pub fn export_graph_to_file(
    keyboard: Res<ButtonInput<KeyCode>>,
    storage: Res<GraphStorage>,
    graphs: Query<(&GraphIdentity, &GraphMetadata)>,
    nodes: Query<(&NodeIdentity, &NodeContent, &SpatialPosition)>,
    edges: Query<(&EdgeIdentity, &EdgeRelationship)>,
) {
    // Check for Ctrl+S
    if (keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight))
        && keyboard.just_pressed(KeyCode::KeyS)
    {
        info!("Export requested via Ctrl+S");

        // Find active graph (for now, just use the first one)
        if let Some((graph_id, _metadata)) = graphs.iter().next() {
            match GraphExporter::export_to_json(*graph_id, &storage, &nodes, &edges, &graphs) {
                Ok(json) => {
                    // For now, save to a fixed location
                    let path = Path::new("exported_graph.json");
                    match GraphExporter::save_to_file(path, &json) {
                        Ok(_) => info!("Graph exported successfully to {:?}", path),
                        Err(e) => error!("Failed to save file: {}", e),
                    }
                }
                Err(e) => error!("Failed to export graph: {:?}", e),
            }
        } else {
            warn!("No graph to export");
        }
    }
}

/// Condition to check if export was requested
pub fn export_requested(keyboard: Res<ButtonInput<KeyCode>>) -> bool {
    (keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight))
        && keyboard.just_pressed(KeyCode::KeyS)
}
