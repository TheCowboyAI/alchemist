//! Graph file importer for loading external graph formats
//!
//! This module provides functionality to import graph data from various formats,
//! starting with the JSON format used in the assets/models directory.

use crate::contexts::graph_management::domain::*;
use crate::contexts::graph_management::events::*;
use crate::contexts::graph_management::storage::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// External graph format used in model files
#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalGraphFormat {
    pub nodes: Vec<ExternalNode>,
    pub relationships: Vec<ExternalRelationship>,
    pub style: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalNode {
    pub id: String,
    pub position: Position2D,
    pub caption: String,
    pub labels: Vec<String>,
    pub properties: HashMap<String, serde_json::Value>,
    pub style: Option<NodeStyle>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Position2D {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeStyle {
    #[serde(rename = "node-color")]
    pub node_color: Option<String>,
    #[serde(rename = "caption-color")]
    pub caption_color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalRelationship {
    pub id: String,
    #[serde(rename = "fromId")]
    pub from_id: String,
    #[serde(rename = "toId")]
    pub to_id: String,
    #[serde(rename = "type")]
    pub edge_type: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub style: Option<serde_json::Value>,
}

/// Service to import graphs from external formats
pub struct GraphImporter;

impl GraphImporter {
    /// Load a graph from a JSON file
    pub fn load_from_file(
        path: &Path,
        commands: &mut Commands,
        storage: &mut GraphStorage,
        graph_created_events: &mut EventWriter<GraphCreated>,
        node_added_events: &mut EventWriter<NodeAdded>,
        edge_connected_events: &mut EventWriter<EdgeConnected>,
    ) -> Result<GraphIdentity, String> {
        // Read the file
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {e}"))?;

        // Parse JSON
        let external_graph: ExternalGraphFormat = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON: {e}"))?;

        // Create a new graph
        let graph_id = GraphIdentity::new();
        let graph_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Imported Graph")
            .to_string();

        let metadata = GraphMetadata {
            name: graph_name.clone(),
            description: format!("Imported from {}", path.display()),
            domain: "imported".to_string(),
            created: std::time::SystemTime::now(),
            modified: std::time::SystemTime::now(),
            tags: vec!["imported".to_string()],
        };

        // Create graph in storage
        storage.create_graph(graph_id)
            .map_err(|e| format!("Failed to create graph in storage: {e:?}"))?;

        // Spawn graph entity
        let graph_entity = commands.spawn((
            GraphBundle {
                graph: Graph {
                    identity: graph_id,
                    metadata: metadata.clone(),
                    journey: GraphJourney::default(),
                },
                identity: graph_id,
                metadata: metadata.clone(),
                journey: GraphJourney::default(),
            },
            Transform::default(),
            GlobalTransform::default(),
        )).id();

        // Emit graph created event
        graph_created_events.write(GraphCreated {
            graph: graph_id,
            metadata,
            timestamp: std::time::SystemTime::now(),
        });

        // Track node ID mappings (external ID -> our NodeIdentity)
        let mut node_mappings = HashMap::new();
        let mut node_entities = HashMap::new();  // Track node entities for parent-child relationships

        // Import nodes
        for external_node in external_graph.nodes {
            let node_id = NodeIdentity::new();
            node_mappings.insert(external_node.id.clone(), node_id);

            // Convert position (scale down the coordinates)
            let scale = 0.01; // Scale down large coordinates
            let position = SpatialPosition::at_3d(
                external_node.position.x * scale,
                0.0, // Use flat Y=0 for now
                -external_node.position.y * scale, // Flip Y to Z and negate
            );

            let content = NodeContent {
                label: external_node.caption,
                category: external_node.labels.first()
                    .cloned()
                    .unwrap_or_else(|| "default".to_string()),
                properties: external_node.properties,
            };

            // Add to storage
            let node_data = NodeData {
                identity: node_id,
                content: content.clone(),
                position,
            };

            storage.add_node(graph_id, node_data)
                .map_err(|e| format!("Failed to add node to storage: {e:?}"))?;

            // Spawn node entity
            let node_entity = commands.spawn((
                NodeBundle {
                    node: crate::contexts::graph_management::domain::Node {
                        identity: node_id,
                        graph: graph_id,
                        content: content.clone(),
                        position,
                    },
                    identity: node_id,
                    content: content.clone(),
                    position,
                    transform: Transform::from_translation(position.coordinates_3d),
                    global_transform: GlobalTransform::default(),
                },
            )).id();

            info!("Spawned node entity {:?} with ID {:?} at position {:?}",
                node_entity, node_id, position.coordinates_3d);

            // Establish parent-child relationship
            commands.entity(graph_entity).add_child(node_entity);

            // Store node entity for edge creation
            node_entities.insert(node_id, node_entity);

            // Emit node added event
            node_added_events.write(NodeAdded {
                node: node_id,
                graph: graph_id,
                content,
                position,
            });
        }

        // Import relationships
        let relationship_count = external_graph.relationships.len();
        for external_rel in external_graph.relationships {
            // Look up our node IDs
            let source_id = node_mappings.get(&external_rel.from_id)
                .ok_or_else(|| format!("Source node {} not found", external_rel.from_id))?;
            let target_id = node_mappings.get(&external_rel.to_id)
                .ok_or_else(|| format!("Target node {} not found", external_rel.to_id))?;

            let edge_id = EdgeIdentity::new();
            let relationship = EdgeRelationship {
                source: *source_id,
                target: *target_id,
                category: if external_rel.edge_type.is_empty() {
                    "default".to_string()
                } else {
                    external_rel.edge_type
                },
                strength: 1.0,
                properties: external_rel.properties,
            };

            // Add to storage
            let edge_data = EdgeData {
                identity: edge_id,
                relationship: relationship.clone(),
            };

            storage.add_edge(graph_id, *source_id, *target_id, edge_data)
                .map_err(|e| format!("Failed to add edge to storage: {e:?}"))?;

            // Spawn edge entity
            let edge_entity = commands.spawn((
                EdgeBundle {
                    edge: crate::contexts::graph_management::domain::Edge {
                        identity: edge_id,
                        graph: graph_id,
                        relationship: relationship.clone(),
                    },
                    identity: edge_id,
                    relationship: relationship.clone(),
                },
            )).id();

            info!("Spawned edge entity {:?} with ID {:?}", edge_entity, edge_id);

            // Add edge as child of graph for proper hierarchy
            commands.entity(graph_entity).add_child(edge_entity);

            // Emit edge connected event
            edge_connected_events.write(EdgeConnected {
                edge: edge_id,
                graph: graph_id,
                relationship,
            });
        }

        info!("Successfully imported graph '{}' with {} nodes and {} relationships",
            graph_name,
            node_mappings.len(),
            relationship_count
        );

        Ok(graph_id)
    }
}

/// System to handle file import requests
#[allow(clippy::too_many_arguments)]
pub fn import_graph_from_file(
    mut commands: Commands,
    mut storage: ResMut<GraphStorage>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut graph_created_events: EventWriter<GraphCreated>,
    mut node_added_events: EventWriter<NodeAdded>,
    mut edge_connected_events: EventWriter<EdgeConnected>,
    // Query existing entities to clean up
    existing_graphs: Query<Entity, With<GraphIdentity>>,
    existing_nodes: Query<Entity, With<NodeIdentity>>,
    existing_edges: Query<Entity, With<EdgeIdentity>>,
    // Query for visual entities that might not have identity components
    flow_particles: Query<Entity, With<crate::contexts::visualization::services::FlowParticle>>,
    edge_segments: Query<Entity, With<crate::contexts::visualization::services::EdgeSegment>>,
    all_entities_with_mesh: Query<Entity, With<Mesh3d>>,
) {
    // Check for Ctrl+O to open file
    if (keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight))
        && keyboard.just_pressed(KeyCode::KeyO)
    {
        // Clear existing graphs first
        info!("Clearing existing graphs...");

        // First, despawn all flow particles
        let particle_count = flow_particles.iter().count();
        if particle_count > 0 {
            info!("Despawning {} flow particles", particle_count);
            for entity in flow_particles.iter() {
                commands.entity(entity).despawn();
            }
        }

        // Despawn all edge segments (from Arc and Bezier edges)
        let segment_count = edge_segments.iter().count();
        if segment_count > 0 {
            info!("Despawning {} edge segments", segment_count);
            for entity in edge_segments.iter() {
                commands.entity(entity).despawn();
            }
        }

        // Despawn all existing graph entities (this will recursively despawn children)
        for entity in existing_graphs.iter() {
            commands.entity(entity).despawn();
        }

        // Despawn any orphaned nodes
        for entity in existing_nodes.iter() {
            commands.entity(entity).despawn();
        }

        // Despawn any orphaned edges
        for entity in existing_edges.iter() {
            commands.entity(entity).despawn();
        }

        // As a final cleanup, despawn any mesh entities that don't have graph components
        // This catches any orphaned visual entities
        let mut orphaned_count = 0;
        for entity in all_entities_with_mesh.iter() {
            // Check if this entity has any graph-related components
            if existing_graphs.get(entity).is_err()
                && existing_nodes.get(entity).is_err()
                && existing_edges.get(entity).is_err()
                && flow_particles.get(entity).is_err()
                && edge_segments.get(entity).is_err() {
                // This is likely an orphaned visual entity
                commands.entity(entity).despawn();
                orphaned_count += 1;
            }
        }

        if orphaned_count > 0 {
            info!("Despawned {} orphaned visual entities", orphaned_count);
        }

        // Clear storage
        storage.clear();

        // For now, we'll load a specific file. In a real app, you'd use a file dialog
        let file_path = Path::new("assets/models/CIM.json");

        info!("Loading graph from: {}", file_path.display());

        match GraphImporter::load_from_file(
            file_path,
            &mut commands,
            &mut storage,
            &mut graph_created_events,
            &mut node_added_events,
            &mut edge_connected_events,
        ) {
            Ok(graph_id) => {
                info!("Successfully loaded graph: {:?}", graph_id);
            }
            Err(e) => {
                error!("Failed to load graph: {}", e);
            }
        }
    }
}
