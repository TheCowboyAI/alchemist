//! Bevy Plugin for Graph Visualization and Persistence
//!
//! This plugin integrates all graph systems including:
//! - File parsing (JSON, Nix, Markdown)
//! - ECS components and systems
//! - Graph theory algorithms (connected components)
//! - JetStream persistence
//! - Real-time synchronization

use bevy::prelude::*;
use crate::{
    graph_components::*,
    graph_systems::*,
    graph_algorithms::identify_graph_components_system,
    jetstream_persistence::*,
    nats_client::NatsClient,
};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

/// Main plugin for graph functionality
pub struct AlchemistGraphPlugin {
    pub nats_url: Option<String>,
    pub enable_persistence: bool,
}

impl Default for AlchemistGraphPlugin {
    fn default() -> Self {
        Self {
            nats_url: Some("nats://localhost:4222".to_string()),
            enable_persistence: true,
        }
    }
}

impl Plugin for AlchemistGraphPlugin {
    fn build(&self, app: &mut App) {
        // Resources
        app.insert_resource(GraphManager::default())
           .insert_resource(GraphReplayManager::default())
           .insert_resource(GraphLayoutSettings::default());
        
        // Events
        app.add_event::<GraphOperationEvent>()
           .add_event::<GraphLoadRequest>()
           .add_event::<GraphSaveRequest>()
           .add_event::<ComponentDetectionRequest>();
        
        // Core graph systems
        app.add_systems(
            Update,
            (
                // Graph operations
                handle_graph_operations,
                update_node_connections,
                
                // Component detection
                identify_graph_components_system,
                
                // Layout
                apply_force_directed_layout,
                // apply_hierarchical_layout, // TODO: implement
                
                // Persistence (if enabled)
                // persist_graph_events_system.run_if(resource_exists::<GraphPersistence>), // TODO
                // sync_from_jetstream_system.run_if(resource_exists::<GraphPersistence>), // TODO
                
                // File loading
                handle_graph_load_requests,
                handle_graph_save_requests,
                
                // Visualization
                // render_graph_edges, // TODO
                // highlight_selected_nodes, // TODO
                // handle_node_interaction, // TODO
            )
            .chain()
        );
        
        // Initialize persistence if enabled
        if self.enable_persistence {
            if let Some(nats_url) = &self.nats_url {
                let url = nats_url.clone();
                app.add_systems(Startup, move |mut commands: Commands, runtime: Res<crate::TokioRuntime>| {
                    let url = url.clone();
                    runtime.0.spawn(async move {
                        match NatsClient::new(&url).await {
                            Ok(client) => {
                                match GraphPersistence::new(client).await {
                                    Ok(persistence) => {
                                        info!("Graph persistence initialized");
                                        // Note: In a real app, we'd send this back to Bevy
                                    }
                                    Err(e) => {
                                        error!("Failed to initialize graph persistence: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("NATS not available for graph persistence: {}", e);
                            }
                        }
                    });
                });
            }
        }
        
        // Add startup system for initial setup
        app.add_systems(Startup, setup_graph_environment);
    }
}

/// Event for requesting graph file loading
#[derive(Event)]
pub struct GraphLoadRequest {
    pub file_path: String,
    pub graph_id: Option<String>,
}

/// Event for requesting graph save
#[derive(Event)]
pub struct GraphSaveRequest {
    pub graph_id: String,
    pub file_path: String,
    pub format: GraphExportFormat,
}

#[derive(Debug, Clone, Copy)]
pub enum GraphExportFormat {
    Json,
    Cytoscape,
    Graphviz,
    Gexf,
}

/// Event for requesting component detection
#[derive(Event)]
pub struct ComponentDetectionRequest {
    pub highlight_components: bool,
    pub separate_components: bool,
}

/// System to handle graph load requests
fn handle_graph_load_requests(
    mut events: EventReader<GraphLoadRequest>,
    mut commands: Commands,
    mut graph_manager: ResMut<GraphManager>,
    mut graph_ops: EventWriter<GraphOperationEvent>,
) {
    for request in events.read() {
        info!("Loading graph from: {}", request.file_path);
        
        match std::fs::read_to_string(&request.file_path) {
            Ok(content) => {
                let graph_id = request.graph_id.clone()
                    .unwrap_or_else(|| format!("graph_{}", uuid::Uuid::new_v4()));
                
                // Parse based on file extension
                let parse_result = if request.file_path.ends_with(".json") {
                    crate::graph_parser::parse_json_graph(&content)
                } else if request.file_path.ends_with(".nix") {
                    crate::graph_parser::parse_nix_graph(&content)
                } else if request.file_path.ends_with(".md") {
                    crate::graph_parser::parse_markdown_graph(&content)
                } else {
                    // Try to auto-detect format
                    crate::graph_parser::parse_json_graph(&content)
                };
                
                match parse_result {
                    Ok((nodes, edges)) => {
                        info!("Parsed {} nodes and {} edges", nodes.len(), edges.len());
                        
                        // Clear existing graph if needed
                        graph_ops.send(GraphOperationEvent {
                            graph_id: graph_id.clone(),
                            operation: GraphOperation::Clear,
                            entities: vec![],
                        });
                        
                        // Add nodes
                        for node in nodes {
                            graph_ops.send(GraphOperationEvent {
                                graph_id: graph_id.clone(),
                                operation: GraphOperation::CreateNode {
                                    id: node.id,
                                    label: node.label,
                                    position: Vec3::from_array(node.position),
                                },
                                entities: vec![],
                            });
                        }
                        
                        // Add edges (would need entity mapping in real implementation)
                        // For now, we store edge data for later processing
                        for edge in edges {
                            info!("Would create edge: {} -> {}", edge.source_id, edge.target_id);
                        }
                        
                        // Request layout
                        graph_ops.send(GraphOperationEvent {
                            graph_id: graph_id.clone(),
                            operation: GraphOperation::ApplyLayout {
                                layout_type: LayoutType::ForceDirected,
                            },
                            entities: vec![],
                        });
                    }
                    Err(e) => {
                        error!("Failed to parse graph file: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to read file {}: {}", request.file_path, e);
            }
        }
    }
}

/// System to handle graph save requests
fn handle_graph_save_requests(
    mut events: EventReader<GraphSaveRequest>,
    nodes: Query<(&GraphNode, &Transform, &NodeMetadata)>,
    edges: Query<(Entity, &GraphEdge)>,
    graph_manager: Res<GraphManager>,
) {
    for request in events.read() {
        info!("Saving graph {} to: {}", request.graph_id, request.file_path);
        
        // Collect nodes and edges for the specified graph
        let mut node_data = Vec::new();
        let mut edge_data = Vec::new();
        
        for (node, transform, metadata) in nodes.iter() {
            if node.graph_id == request.graph_id {
                node_data.push(crate::graph_parser::NodeData {
                    id: node.id.clone(),
                    label: node.label.clone(),
                    position: transform.translation.to_array(),
                    color: None,
                    size: None,
                    metadata: serde_json::to_value(&metadata.properties).unwrap_or(serde_json::Value::Null),
                });
            }
        }
        
        for (edge_entity, edge) in edges.iter() {
            if edge.graph_id == request.graph_id {
                edge_data.push(crate::graph_parser::EdgeData {
                    id: format!("edge_{}", edge_entity.index()),
                    source_id: edge.source.to_string(),
                    target_id: edge.target.to_string(),
                    label: edge.label.clone(),
                    weight: edge.weight,
                    color: None,
                    metadata: serde_json::Value::Null,
                });
            }
        }
        
        // Export based on format
        let export_result: Result<String, Box<dyn std::error::Error>> = match request.format {
            GraphExportFormat::Json => {
                export_as_json(&node_data, &edge_data).map_err(|e| e.into())
            }
            GraphExportFormat::Cytoscape => {
                export_as_cytoscape(&node_data, &edge_data).map_err(|e| e.into())
            }
            GraphExportFormat::Graphviz => {
                export_as_graphviz(&node_data, &edge_data)
            }
            GraphExportFormat::Gexf => {
                export_as_gexf(&node_data, &edge_data)
            }
        };
        
        match export_result {
            Ok(content) => {
                if let Err(e) = std::fs::write(&request.file_path, content) {
                    error!("Failed to write file: {}", e);
                } else {
                    info!("Graph saved successfully");
                }
            }
            Err(e) => {
                error!("Failed to export graph: {}", e);
            }
        }
    }
}

/// Setup initial graph environment
fn setup_graph_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 20.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-0.5)),
    ));
    
    // Grid floor
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, -1.0, 0.0),
    ));
}

// Export format implementations
fn export_as_json(
    nodes: &[crate::graph_parser::NodeData],
    edges: &[crate::graph_parser::EdgeData],
) -> Result<String, serde_json::Error> {
    let graph = serde_json::json!({
        "nodes": nodes,
        "edges": edges,
        "metadata": {
            "created": chrono::Utc::now().to_rfc3339(),
            "format": "alchemist-graph-v1"
        }
    });
    serde_json::to_string_pretty(&graph)
}

fn export_as_cytoscape(
    nodes: &[crate::graph_parser::NodeData],
    edges: &[crate::graph_parser::EdgeData],
) -> Result<String, serde_json::Error> {
    let elements: Vec<_> = nodes.iter().map(|node| {
        serde_json::json!({
            "data": {
                "id": node.id,
                "label": node.label
            },
            "position": {
                "x": node.position[0],
                "y": node.position[2]
            }
        })
    }).chain(edges.iter().map(|edge| {
        serde_json::json!({
            "data": {
                "id": edge.id,
                "source": edge.source_id,
                "target": edge.target_id,
                "label": edge.label
            }
        })
    })).collect();
    
    let cytoscape = serde_json::json!({
        "elements": elements
    });
    serde_json::to_string_pretty(&cytoscape)
}

fn export_as_graphviz(
    nodes: &[crate::graph_parser::NodeData],
    edges: &[crate::graph_parser::EdgeData],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut dot = String::from("digraph G {\n");
    dot.push_str("  rankdir=TB;\n");
    dot.push_str("  node [shape=box];\n\n");
    
    for node in nodes {
        dot.push_str(&format!("  \"{}\" [label=\"{}\"];\n", node.id, node.label));
    }
    
    dot.push_str("\n");
    
    for edge in edges {
        let label = edge.label.as_ref()
            .map(|l| format!(" [label=\"{}\"]", l))
            .unwrap_or_default();
        dot.push_str(&format!("  \"{}\" -> \"{}\"{}\n", 
            edge.source_id, edge.target_id, label));
    }
    
    dot.push_str("}\n");
    Ok(dot)
}

fn export_as_gexf(
    nodes: &[crate::graph_parser::NodeData],
    edges: &[crate::graph_parser::EdgeData],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut gexf = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<gexf xmlns="http://www.gexf.net/1.2draft" version="1.2">
  <graph mode="static" defaultedgetype="directed">
    <nodes>
"#);
    
    for node in nodes {
        gexf.push_str(&format!(
            "      <node id=\"{}\" label=\"{}\" />\n",
            node.id, node.label
        ));
    }
    
    gexf.push_str("    </nodes>\n    <edges>\n");
    
    for (i, edge) in edges.iter().enumerate() {
        gexf.push_str(&format!(
            "      <edge id=\"{}\" source=\"{}\" target=\"{}\" />\n",
            i, edge.source_id, edge.target_id
        ));
    }
    
    gexf.push_str("    </edges>\n  </graph>\n</gexf>");
    Ok(gexf)
}

/// Helper to check if a resource exists
fn resource_exists<T: Resource>(resource: Option<Res<T>>) -> bool {
    resource.is_some()
}