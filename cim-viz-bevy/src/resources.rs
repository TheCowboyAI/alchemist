//! Resources for graph visualization
//!
//! Following event-driven architecture: Resources are ONLY for read models and configuration.
//! All state changes must go through events.

use bevy::prelude::*;
use cim_contextgraph::{NodeId, EdgeId, ContextGraphId as GraphId};
use std::collections::HashMap;

/// Resource tracking the currently active graph
#[derive(Resource, Default)]
pub struct ActiveGraph {
    pub graph_id: Option<GraphId>,
}

/// Resource tracking selected entities
#[derive(Resource, Default)]
pub struct Selection {
    pub nodes: Vec<(Entity, NodeId)>,
    pub edges: Vec<(Entity, EdgeId)>,
}

impl Selection {
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }

    pub fn contains_node(&self, node_id: &NodeId) -> bool {
        self.nodes.iter().any(|(_, id)| id == node_id)
    }

    pub fn contains_edge(&self, edge_id: &EdgeId) -> bool {
        self.edges.iter().any(|(_, id)| id == edge_id)
    }
}

/// Read-only configuration for visualization
#[derive(Resource)]
pub struct VisualizationConfig {
    pub default_animation_speed: f32,
    pub default_camera_smoothing: f32,
    pub enable_physics: bool,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            default_animation_speed: 1.0,
            default_camera_smoothing: 0.1,
            enable_physics: true,
        }
    }
}

/// Read-only layout configuration
#[derive(Resource)]
pub struct GraphLayoutConfig {
    pub force_directed_strength: f32,
    pub force_directed_distance: f32,
    pub hierarchical_layer_spacing: f32,
    pub circular_radius: f32,
    pub grid_spacing: f32,
}

impl Default for GraphLayoutConfig {
    fn default() -> Self {
        Self {
            force_directed_strength: 100.0,
            force_directed_distance: 0.1,
            hierarchical_layer_spacing: 100.0,
            circular_radius: 200.0,
            grid_spacing: 50.0,
        }
    }
}

/// Configuration for force-directed layout
#[derive(Debug, Clone)]
pub struct ForceDirectedConfig {
    pub repulsion_strength: f32,
    pub attraction_strength: f32,
    pub damping: f32,
    pub ideal_edge_length: f32,
    pub iterations: u32,
}

impl Default for ForceDirectedConfig {
    fn default() -> Self {
        Self {
            repulsion_strength: 100.0,
            attraction_strength: 0.1,
            damping: 0.9,
            ideal_edge_length: 50.0,
            iterations: 100,
        }
    }
}

/// Configuration for hierarchical layout
#[derive(Debug, Clone)]
pub struct HierarchicalConfig {
    pub layer_spacing: f32,
    pub node_spacing: f32,
    pub direction: LayoutDirection,
}

impl Default for HierarchicalConfig {
    fn default() -> Self {
        Self {
            layer_spacing: 100.0,
            node_spacing: 50.0,
            direction: LayoutDirection::TopToBottom,
        }
    }
}

/// Layout direction for hierarchical layouts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutDirection {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
}

/// Configuration for circular layout
#[derive(Debug, Clone)]
pub struct CircularConfig {
    pub radius: f32,
    pub start_angle: f32,
    pub clockwise: bool,
}

impl Default for CircularConfig {
    fn default() -> Self {
        Self {
            radius: 200.0,
            start_angle: 0.0,
            clockwise: true,
        }
    }
}

/// Resource for camera control state
#[derive(Resource, Default)]
pub struct CameraState {
    pub is_panning: bool,
    pub is_rotating: bool,
    pub is_zooming: bool,
    pub target_position: Option<Vec3>,
    pub target_zoom: Option<f32>,
}

/// Read-only performance metrics
#[derive(Resource, Default)]
pub struct PerformanceMetrics {
    pub node_count: usize,
    pub edge_count: usize,
    pub visible_nodes: usize,
    pub visible_edges: usize,
    pub layout_time_ms: f32,
    pub render_time_ms: f32,
}

/// Resource mapping graph IDs to their visual bounds
#[derive(Resource, Default)]
pub struct GraphBounds {
    pub bounds: HashMap<GraphId, BoundingBox>,
}

/// Read-only theme configuration
#[derive(Resource)]
pub struct ThemeConfig {
    pub background_color: Color,
    pub grid_color: Color,
    pub default_node_color: Color,
    pub default_edge_color: Color,
    pub selection_color: Color,
    pub highlight_color: Color,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background_color: Color::srgb(0.1, 0.1, 0.1),
            grid_color: Color::srgba(0.3, 0.3, 0.3, 0.3),
            default_node_color: Color::srgb(0.3, 0.7, 0.3),
            default_edge_color: Color::srgb(0.5, 0.5, 0.5),
            selection_color: Color::srgb(0.0, 0.7, 1.0),
            highlight_color: Color::srgb(1.0, 0.7, 0.0),
        }
    }
}

/// Read-only spatial index for performance optimization
#[derive(Resource, Default)]
pub struct SpatialIndex {
    graph_bounds: HashMap<GraphId, BoundingBox>,
}

impl SpatialIndex {
    pub fn get_bounds(&self, graph_id: &GraphId) -> Option<&BoundingBox> {
        self.graph_bounds.get(graph_id)
    }

    pub fn update_bounds(&mut self, graph_id: GraphId, bounds: BoundingBox) {
        self.graph_bounds.insert(graph_id, bounds);
    }
}

/// Bounding box for spatial queries
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
}

/// Resource for interaction state
#[derive(Resource, Default)]
pub struct InteractionState {
    pub mouse_world_position: Vec3,
    pub drag_start_position: Option<Vec3>,
    pub hovered_entity: Option<Entity>,
    pub interaction_mode: InteractionMode,
}

/// Interaction modes
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum InteractionMode {
    #[default]
    Select,
    Pan,
    CreateNode,
    CreateEdge,
    Delete,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_operations() {
        let mut selection = Selection::default();
        let entity = Entity::from_raw(1);
        let node_id = NodeId::new();

        selection.nodes.push((entity, node_id));

        assert!(!selection.is_empty());
        assert!(selection.contains_node(&node_id));

        selection.clear();
        assert!(selection.is_empty());
    }

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox::new(Vec3::ZERO, Vec3::ONE);

        assert_eq!(bbox.center(), Vec3::splat(0.5));
        assert_eq!(bbox.size(), Vec3::ONE);
        assert!(bbox.contains(Vec3::splat(0.5)));
        assert!(!bbox.contains(Vec3::splat(2.0)));
    }
}

