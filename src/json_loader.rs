use crate::graph::{GraphEdge, GraphNode};
use crate::unified_graph_editor::{BaseGraphResource, SubgraphInfo};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// JSON format structures matching the KECO event storming format
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonGraphData {
    pub nodes: Vec<JsonNode>,
    pub relationships: Vec<JsonRelationship>,
    #[serde(default)]
    pub style: JsonStyle,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonNode {
    pub id: String,
    #[serde(default)]
    pub position: JsonPosition,
    pub caption: String,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub properties: HashMap<String, String>,
    #[serde(default)]
    pub style: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRelationship {
    pub id: String,
    #[serde(rename = "fromId")]
    pub from_id: String,
    #[serde(rename = "toId")]
    pub to_id: String,
    #[serde(rename = "type")]
    pub relationship_type: String,
    #[serde(default)]
    pub properties: HashMap<String, String>,
    #[serde(default)]
    pub style: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct JsonPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct JsonStyle {
    #[serde(rename = "font-family", default)]
    pub font_family: String,
    #[serde(rename = "background-color", default)]
    pub background_color: String,
    // Add other style properties as needed
}

/// Convert JSON format to internal graph format
pub fn json_to_base_graph(json_data: JsonGraphData) -> Result<BaseGraphResource, String> {
    let mut base_graph = BaseGraphResource::default();
    let mut node_id_map: HashMap<String, Uuid> = HashMap::new();

    // Convert nodes
    for json_node in json_data.nodes {
        let node_id = Uuid::new_v4();
        node_id_map.insert(json_node.id.clone(), node_id);

        let graph_node = GraphNode {
            id: node_id,
            name: json_node.caption,
            properties: json_node.properties,
            labels: json_node.labels,
            radius: 1.0, // Default radius
        };

        base_graph.graph.nodes.insert(node_id, graph_node);

        // Store position information for later use
        base_graph.node_positions.insert(
            node_id,
            Vec3::new(
                json_node.position.x / 100.0,
                0.0,                          // Keep Y at 0 for horizontal plane
                json_node.position.y / 100.0, // Map 2D Y to 3D Z
            ),
        );

        // Store style information in node_styles (if it exists)
        if let Some(color) = json_node.style.get("node-color") {
            if let Some(node) = base_graph.graph.nodes.get_mut(&node_id) {
                node.properties.insert("node-color".to_string(), color.clone());
            }
        }
    }

    // Convert relationships
    for json_rel in json_data.relationships {
        let edge_id = Uuid::new_v4();

        // Map the string IDs to UUIDs
        let source_id = node_id_map
            .get(&json_rel.from_id)
            .ok_or_else(|| format!("Source node not found: {}", json_rel.from_id))?;
        let target_id = node_id_map
            .get(&json_rel.to_id)
            .ok_or_else(|| format!("Target node not found: {}", json_rel.to_id))?;

        let labels = vec![json_rel.relationship_type];

        let graph_edge = GraphEdge {
            id: edge_id,
            source: *source_id,
            target: *target_id,
            properties: json_rel.properties,
            labels,
            weight: 1.0, // Default weight
        };

        base_graph.graph.edges.insert(edge_id, graph_edge);
    }

    // Create a single subgraph containing all imported nodes
    if !base_graph.graph.nodes.is_empty() {
        let subgraph_id = Uuid::new_v4();
        base_graph.next_subgraph_id += 1;

        let subgraph = SubgraphInfo {
            id: subgraph_id,
            name: "Imported Graph".to_string(),
            pattern_type: "imported".to_string(),
            nodes: base_graph.graph.nodes.keys().cloned().collect(),
            color: Color::srgba(0.3, 0.6, 0.9, 1.0), // Blue color for imported data
        };

        base_graph.subgraphs.insert(subgraph_id, subgraph);
    }

    Ok(base_graph)
}

/// Convert internal graph format to JSON format
pub fn base_graph_to_json(base_graph: &BaseGraphResource) -> JsonGraphData {
    let mut json_nodes = Vec::new();
    let mut json_relationships = Vec::new();
    let mut id_counter = 1;
    let mut uuid_to_string: HashMap<Uuid, String> = HashMap::new();

    // Convert nodes
    for (uuid, node) in &base_graph.graph.nodes {
        let string_id = format!("n{}", id_counter);
        uuid_to_string.insert(*uuid, string_id.clone());
        id_counter += 1;

        let position = base_graph
            .node_positions
            .get(uuid)
            .map(|pos| JsonPosition {
                x: pos.x * 100.0,
                y: pos.z * 100.0,
            })
            .unwrap_or_default();

        let json_node = JsonNode {
            id: string_id,
            position,
            caption: node.name.clone(),
            labels: node.labels.clone(),
            properties: node.properties.clone(),
            style: HashMap::new(),
        };

        json_nodes.push(json_node);
    }

    // Convert relationships
    for (_, edge) in &base_graph.graph.edges {
        let source_id = uuid_to_string
            .get(&edge.source)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let target_id = uuid_to_string
            .get(&edge.target)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let relationship_type = edge
            .labels
            .first()
            .cloned()
            .unwrap_or_else(|| "connected".to_string());

        let json_rel = JsonRelationship {
            id: format!("r{}", id_counter),
            from_id: source_id,
            to_id: target_id,
            relationship_type,
            properties: edge.properties.clone(),
            style: HashMap::new(),
        };

        json_relationships.push(json_rel);
        id_counter += 1;
    }

    JsonGraphData {
        nodes: json_nodes,
        relationships: json_relationships,
        style: JsonStyle::default(),
    }
}

/// Load JSON file from disk
pub fn load_json_file(file_path: &str) -> Result<JsonGraphData, String> {
    let content =
        std::fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    let json_data: JsonGraphData =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(json_data)
}

/// Save JSON file to disk
pub fn save_json_file(file_path: &str, json_data: &JsonGraphData) -> Result<(), String> {
    let content = serde_json::to_string_pretty(json_data)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    std::fs::write(file_path, content).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

/// Events for file operations
#[derive(Event)]
pub struct LoadJsonFileEvent {
    pub file_path: String,
}

#[derive(Event)]
pub struct SaveJsonFileEvent {
    pub file_path: String,
}

#[derive(Event)]
pub struct ClearGraphEvent;

#[derive(Event)]
pub struct JsonFileLoadedEvent {
    pub success: bool,
    pub message: String,
}

#[derive(Event)]
pub struct JsonFileSavedEvent {
    pub success: bool,
    pub message: String,
}

/// Resource to track file operations
#[derive(Resource, Default)]
pub struct FileOperationState {
    pub current_file_path: Option<String>,
    pub last_operation_message: String,
    pub available_files: Vec<String>,
}

impl FileOperationState {
    pub fn scan_models_directory(&mut self) {
        self.available_files.clear();

        info!("Scanning assets/models directory for JSON files...");

        // Scan the assets/models directory for JSON files
        if let Ok(entries) = std::fs::read_dir("assets/models") {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".json") {
                        let file_path = format!("assets/models/{}", file_name);
                        info!("Found JSON file: {}", file_path);
                        self.available_files.push(file_path);
                    }
                }
            }
        } else {
            warn!("Failed to read assets/models directory");
        }

        // Sort for consistent ordering
        self.available_files.sort();
        info!("Found {} JSON files total", self.available_files.len());
    }
}

/// System to handle file loading
pub fn handle_json_file_loading(
    mut load_events: EventReader<LoadJsonFileEvent>,
    mut loaded_events: EventWriter<JsonFileLoadedEvent>,
    mut base_graph: ResMut<BaseGraphResource>,
    mut file_state: ResMut<FileOperationState>,
) {
    for event in load_events.read() {
        match load_json_file(&event.file_path) {
            Ok(json_data) => match json_to_base_graph(json_data) {
                Ok(new_base_graph) => {
                    *base_graph = new_base_graph;
                    file_state.current_file_path = Some(event.file_path.clone());
                    file_state.last_operation_message =
                        format!("Successfully loaded {}", event.file_path);

                    loaded_events.write(JsonFileLoadedEvent {
                        success: true,
                        message: format!(
                            "Loaded {} nodes and {} edges from {}",
                            base_graph.graph.nodes.len(),
                            base_graph.graph.edges.len(),
                            event.file_path
                        ),
                    });
                }
                Err(e) => {
                    file_state.last_operation_message = format!("Failed to convert graph: {}", e);
                    loaded_events.write(JsonFileLoadedEvent {
                        success: false,
                        message: e,
                    });
                }
            },
            Err(e) => {
                file_state.last_operation_message = format!("Failed to load file: {}", e);
                loaded_events.write(JsonFileLoadedEvent {
                    success: false,
                    message: e,
                });
            }
        }
    }
}

/// System to handle file saving
pub fn handle_json_file_saving(
    mut save_events: EventReader<SaveJsonFileEvent>,
    mut saved_events: EventWriter<JsonFileSavedEvent>,
    base_graph: Res<BaseGraphResource>,
    mut file_state: ResMut<FileOperationState>,
) {
    for event in save_events.read() {
        let json_data = base_graph_to_json(&base_graph);

        match save_json_file(&event.file_path, &json_data) {
            Ok(()) => {
                file_state.current_file_path = Some(event.file_path.clone());
                file_state.last_operation_message =
                    format!("Successfully saved {}", event.file_path);

                saved_events.write(JsonFileSavedEvent {
                    success: true,
                    message: format!(
                        "Saved {} nodes and {} edges to {}",
                        base_graph.graph.nodes.len(),
                        base_graph.graph.edges.len(),
                        event.file_path
                    ),
                });
            }
            Err(e) => {
                file_state.last_operation_message = format!("Failed to save file: {}", e);
                saved_events.write(JsonFileSavedEvent {
                    success: false,
                    message: e,
                });
            }
        }
    }
}
