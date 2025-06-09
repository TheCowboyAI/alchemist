//! Conceptual visualization components for ConceptGraph
//!
//! These components provide visual representations of conceptual graphs,
//! including quality dimensions, semantic relationships, and interactive elements.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid;

use crate::domain::conceptual_graph::{ConceptNode, ConceptEdge, QualityDimension, ConceptualPoint};
use crate::domain::value_objects::{NodeId, EdgeId, GraphId};

/// Visual representation of a concept node in 3D space
#[derive(Component, Debug, Clone)]
pub struct ConceptualNodeVisual {
    /// The concept ID this visual represents
    pub concept_id: NodeId,

    /// Type of concept node
    pub node_type: ConceptNodeType,

    /// Position in quality dimension space
    pub quality_position: ConceptualPoint,

    /// Visual styling information
    pub visual_style: NodeVisualStyle,

    /// Whether this node is currently selected
    pub selected: bool,

    /// Whether this node is being hovered over
    pub hovered: bool,
}

/// Types of concept nodes for visualization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConceptNodeType {
    /// Atomic concept
    Atom {
        category: String,
        properties: HashMap<String, serde_json::Value>,
    },

    /// Composite concept containing other concepts
    Composite {
        child_count: usize,
        composition_type: String,
    },

    /// Function concept that transforms other concepts
    Function {
        input_types: Vec<String>,
        output_type: String,
    },

    /// Context boundary node
    Context {
        context_name: String,
        mapping_type: String,
    },
}

/// Visual styling for concept nodes
#[derive(Debug, Clone)]
pub struct NodeVisualStyle {
    /// Base color of the node
    pub base_color: Color,

    /// Emissive color for highlighting
    pub emissive_color: Color,

    /// Size multiplier
    pub scale: f32,

    /// Shape type
    pub shape: NodeShape,

    /// Transparency
    pub alpha: f32,
}

impl Default for NodeVisualStyle {
    fn default() -> Self {
        Self {
            base_color: Color::srgb(0.3, 0.5, 0.8),
            emissive_color: Color::srgb(0.0, 0.0, 0.0),
            scale: 1.0,
            shape: NodeShape::Sphere,
            alpha: 1.0,
        }
    }
}

/// Available node shapes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeShape {
    Sphere,
    Cube,
    Cylinder,
    Cone,
    Torus,
    Icosahedron,
}

/// Visual representation of a concept edge
#[derive(Component, Debug, Clone)]
pub struct ConceptualEdgeVisual {
    /// The edge ID this visual represents
    pub edge_id: EdgeId,

    /// Source node entity
    pub source_entity: Entity,

    /// Target node entity
    pub target_entity: Entity,

    /// Type of relationship
    pub relationship: ConceptRelationship,

    /// Visual styling
    pub visual_style: EdgeVisualStyle,

    /// Animation progress (0.0 to 1.0)
    pub animation_progress: f32,
}

/// Types of conceptual relationships
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConceptRelationship {
    /// Semantic similarity
    Similarity { strength: f32 },

    /// Hierarchical relationship
    Hierarchy { parent_to_child: bool },

    /// Functional dependency
    Dependency { dependency_type: String },

    /// Morphism between concepts
    Morphism { morphism_type: String },

    /// Context bridge
    ContextBridge { mapping_type: String },
}

/// Visual styling for edges
#[derive(Debug, Clone)]
pub struct EdgeVisualStyle {
    /// Color of the edge
    pub color: Color,

    /// Width of the edge line
    pub width: f32,

    /// Whether to show arrow heads
    pub show_arrows: bool,

    /// Dash pattern (empty for solid line)
    pub dash_pattern: Vec<f32>,

    /// Curve factor (0.0 for straight, positive for curved)
    pub curve_factor: f32,
}

impl Default for EdgeVisualStyle {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.6, 0.6, 0.6),
            width: 0.1,
            show_arrows: true,
            dash_pattern: vec![],
            curve_factor: 0.0,
        }
    }
}

/// Visual representation of a quality dimension axis
#[derive(Component, Debug, Clone)]
pub struct QualityDimensionAxis {
    /// The quality dimension this axis represents
    pub dimension: QualityDimension,

    /// Direction in 3D space this dimension maps to
    pub axis_direction: Vec3,

    /// Scale factor for this dimension
    pub scale: f32,

    /// Color for this axis
    pub color: Color,

    /// Whether to show axis labels
    pub show_labels: bool,

    /// Label text entities
    pub label_entities: Vec<Entity>,
}

/// Visual representation of a conceptual space
#[derive(Component, Debug, Clone)]
pub struct ConceptualSpaceVisual {
    /// Unique identifier for this space
    pub space_id: SpaceId,

    /// Quality dimensions in this space
    pub dimensions: Vec<QualityDimensionAxis>,

    /// Origin point in 3D space
    pub origin: Vec3,

    /// Bounding box of the space
    pub bounds: SpaceBounds,

    /// Grid settings
    pub grid_settings: GridSettings,
}

/// Unique identifier for a conceptual space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpaceId(pub uuid::Uuid);

impl SpaceId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

/// Bounds of a conceptual space
#[derive(Debug, Clone)]
pub struct SpaceBounds {
    pub min: Vec3,
    pub max: Vec3,
}

impl Default for SpaceBounds {
    fn default() -> Self {
        Self {
            min: Vec3::new(-10.0, -10.0, -10.0),
            max: Vec3::new(10.0, 10.0, 10.0),
        }
    }
}

/// Grid visualization settings
#[derive(Debug, Clone)]
pub struct GridSettings {
    /// Whether to show the grid
    pub visible: bool,

    /// Grid spacing
    pub spacing: f32,

    /// Grid color
    pub color: Color,

    /// Grid line width
    pub line_width: f32,

    /// Number of subdivisions
    pub subdivisions: u32,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            visible: true,
            spacing: 1.0,
            color: Color::srgba(0.5, 0.5, 0.5, 0.3),
            line_width: 0.01,
            subdivisions: 10,
        }
    }
}

/// Component for nodes that can be dragged
#[derive(Component, Debug, Clone)]
pub struct DraggableNode {
    /// Constraints on dragging
    pub constraints: DragConstraints,

    /// Whether to snap to grid
    pub snap_to_grid: bool,

    /// Grid size for snapping
    pub grid_size: f32,

    /// Whether dragging is currently active
    pub is_dragging: bool,

    /// Drag offset from node center
    pub drag_offset: Vec3,
}

impl Default for DraggableNode {
    fn default() -> Self {
        Self {
            constraints: DragConstraints::default(),
            snap_to_grid: false,
            grid_size: 0.5,
            is_dragging: false,
            drag_offset: Vec3::ZERO,
        }
    }
}

/// Constraints for node dragging
#[derive(Debug, Clone)]
pub struct DragConstraints {
    /// Minimum position
    pub min_position: Option<Vec3>,

    /// Maximum position
    pub max_position: Option<Vec3>,

    /// Allowed axes (x, y, z)
    pub allowed_axes: (bool, bool, bool),

    /// Whether to constrain to conceptual space bounds
    pub constrain_to_space: bool,
}

impl Default for DragConstraints {
    fn default() -> Self {
        Self {
            min_position: None,
            max_position: None,
            allowed_axes: (true, true, true),
            constrain_to_space: true,
        }
    }
}

/// Component for nodes that can be connected
#[derive(Component, Debug, Clone)]
pub struct ConnectableNode {
    /// Allowed connection types
    pub allowed_connections: Vec<ConceptRelationship>,

    /// Maximum number of connections
    pub max_connections: Option<usize>,

    /// Current connection count
    pub connection_count: usize,

    /// Whether this node can be a source
    pub can_be_source: bool,

    /// Whether this node can be a target
    pub can_be_target: bool,
}

impl Default for ConnectableNode {
    fn default() -> Self {
        Self {
            allowed_connections: vec![],
            max_connections: None,
            connection_count: 0,
            can_be_source: true,
            can_be_target: true,
        }
    }
}

/// Component for selectable graphs
#[derive(Component, Debug, Clone)]
pub struct SelectableGraph {
    /// Graph identifier
    pub graph_id: GraphId,

    /// Current selection mode
    pub selection_mode: SelectionMode,

    /// Currently selected entities
    pub selected_entities: Vec<Entity>,

    /// Selection box start position (for box selection)
    pub selection_start: Option<Vec2>,
}

/// Selection modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionMode {
    /// Single entity selection
    Single,

    /// Multiple entity selection
    Multiple,

    /// Box selection
    Box,

    /// Lasso selection
    Lasso,
}

/// Marker component for highlighted entities
#[derive(Component, Debug, Clone, Copy)]
pub struct Highlighted {
    /// Highlight color
    pub color: Color,

    /// Highlight intensity
    pub intensity: f32,
}

/// Marker component for entities being previewed
#[derive(Component, Debug, Clone, Copy)]
pub struct Preview {
    /// Preview opacity
    pub opacity: f32,

    /// Whether the preview is valid
    pub valid: bool,
}

/// Animation component for smooth transitions
#[derive(Component, Debug, Clone)]
pub struct TransitionAnimation {
    /// Start position
    pub start: Vec3,

    /// End position
    pub end: Vec3,

    /// Animation duration
    pub duration: f32,

    /// Elapsed time
    pub elapsed: f32,

    /// Easing function
    pub easing: EasingFunction,
}

/// Easing functions for animations
#[derive(Debug, Clone, Copy)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
}

impl EasingFunction {
    /// Apply easing function to a value between 0 and 1
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t,
            Self::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Self::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Self::Bounce => {
                let n1 = 7.5625;
                let d1 = 2.75;

                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }
            Self::Elastic => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let p = 0.3;
                    let s = p / 4.0;
                    let t = t - 1.0;
                    -(2.0_f32.powf(10.0 * t) * ((t - s) * (2.0 * std::f32::consts::PI) / p).sin())
                }
            }
        }
    }
}

/// Resource for managing conceptual space visualization
#[derive(Resource, Debug, Clone)]
pub struct ConceptualVisualizationSettings {
    /// Whether to show quality dimension axes
    pub show_axes: bool,

    /// Whether to show grid
    pub show_grid: bool,

    /// Whether to show node labels
    pub show_labels: bool,

    /// Whether to show edge labels
    pub show_edge_labels: bool,

    /// Node size multiplier
    pub node_scale: f32,

    /// Edge width multiplier
    pub edge_scale: f32,

    /// Animation speed multiplier
    pub animation_speed: f32,

    /// Whether to use smooth transitions
    pub smooth_transitions: bool,
}

impl Default for ConceptualVisualizationSettings {
    fn default() -> Self {
        Self {
            show_axes: true,
            show_grid: true,
            show_labels: true,
            show_edge_labels: false,
            node_scale: 1.0,
            edge_scale: 1.0,
            animation_speed: 1.0,
            smooth_transitions: true,
        }
    }
}
