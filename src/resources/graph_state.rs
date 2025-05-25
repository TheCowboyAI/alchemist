use bevy::prelude::*;
use uuid::Uuid;

/// Resource to track the overall graph state
#[derive(Resource, Default)]
pub struct GraphState {
    pub node_count: usize,
    pub edge_count: usize,
    pub selected_nodes: Vec<Entity>,
    pub hovered_entity: Option<Entity>,
}

/// Resource for graph metadata
#[derive(Resource, Default)]
pub struct GraphMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub domain: String,
}

/// Resource for graph bounds (for camera calculations)
#[derive(Resource, Default)]
pub struct GraphBounds {
    pub min: Vec3,
    pub max: Vec3,
    pub center: Vec3,
    pub max_y: f32,
}

/// Resource for tracking graph inspector state
#[derive(Resource, Default)]
pub struct GraphInspectorState {
    /// Currently selected node
    pub selected_node: Option<Uuid>,
    /// Currently selected edge
    pub selected_edge: Option<Uuid>,
    /// Show graph statistics
    pub show_stats: bool,
    /// Show algorithm controls
    pub show_algorithms: bool,
    /// Path finding source
    pub pathfind_source: Option<Uuid>,
    /// Path finding target
    pub pathfind_target: Option<Uuid>,
    /// Search filter
    pub search_filter: String,
}
