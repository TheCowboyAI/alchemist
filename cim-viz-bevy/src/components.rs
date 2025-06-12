//! Visual Components: Value Objects in the Bevy ECS Category
//!
//! These components are the objects in the Bevy category that correspond
//! to objects in the CIM-ContextGraph category. They maintain the isomorphism
//! by preserving the essential structure while adding visual properties.

use bevy::prelude::*;
use cim_contextgraph::{NodeId, EdgeId, ContextGraphId as GraphId};
use serde::{Deserialize, Serialize};

// ============================================================================
// Graph Visual Components (Objects in the Visual Category)
// ============================================================================

/// Visual representation of a graph - preserves graph identity
#[derive(Component, Debug, Clone)]
pub struct GraphVisual {
    pub graph_id: GraphId,
    pub layout_type: LayoutType,
}

/// Visual representation of a node - preserves node identity
#[derive(Component, Debug, Clone)]
pub struct NodeVisual {
    pub node_id: NodeId,
    pub graph_id: GraphId,
}

/// Visual representation of an edge - preserves edge identity and relationships
#[derive(Component, Debug, Clone)]
pub struct EdgeVisual {
    pub edge_id: EdgeId,
    pub graph_id: GraphId,
    pub source_entity: Entity,
    pub target_entity: Entity,
}

// ============================================================================
// Visual Properties (Additional structure in the visual category)
// ============================================================================

/// Visual selection state - exists only in visual category
#[derive(Component, Debug, Clone, Default)]
pub struct Selected;

/// Visual hover state - exists only in visual category
#[derive(Component, Debug, Clone, Default)]
pub struct Hovered;

/// Visual dragging state - exists only in visual category
#[derive(Component, Debug, Clone)]
pub struct Dragging {
    pub offset: Vec3,
    pub start_position: Vec3,
}

/// Visual highlight state - exists only in visual category
#[derive(Component, Debug, Clone)]
pub struct Highlighted {
    pub color: Color,
    pub intensity: f32,
}

// ============================================================================
// Layout Types (Morphisms in the visual category)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutType {
    ForceDirected,
    Hierarchical,
    Circular,
    Grid,
    Random,
    Manual,
}

// ============================================================================
// Bundles (Composite objects in the visual category)
// ============================================================================

/// Bundle for creating a visual node entity
#[derive(Bundle)]
pub struct NodeVisualBundle {
    pub node: NodeVisual,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl NodeVisualBundle {
    pub fn new(node_id: NodeId, graph_id: GraphId, position: Vec3) -> Self {
        Self {
            node: NodeVisual { node_id, graph_id },
            transform: Transform::from_translation(position),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}

/// Bundle for creating a visual edge entity
#[derive(Bundle)]
pub struct EdgeVisualBundle {
    pub edge: EdgeVisual,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl EdgeVisualBundle {
    pub fn new(
        edge_id: EdgeId,
        graph_id: GraphId,
        source_entity: Entity,
        target_entity: Entity,
    ) -> Self {
        Self {
            edge: EdgeVisual {
                edge_id,
                graph_id,
                source_entity,
                target_entity,
            },
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}

// ============================================================================
// Marker Components (Category-specific type markers)
// ============================================================================

/// Marks the main camera for graph visualization
#[derive(Component)]
pub struct GraphCamera;

/// Marks UI elements related to graph visualization
#[derive(Component)]
pub struct GraphUI;

/// Marks temporary visual effects
#[derive(Component)]
pub struct TemporaryVisual {
    pub lifetime: Timer,
}

/// Visual style for nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStyle {
    pub shape: NodeShape,
    pub size: f32,
    pub color: Color,
    pub border_color: Option<Color>,
    pub border_width: f32,
}

impl Default for NodeStyle {
    fn default() -> Self {
        Self {
            shape: NodeShape::Circle,
            size: 1.0,
            color: Color::srgb(0.5, 0.5, 0.5),
            border_color: None,
            border_width: 0.0,
        }
    }
}

/// Node shape variants
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NodeShape {
    Circle,
    Square,
    Diamond,
    Triangle,
    Hexagon,
}

/// Visual style for edges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeStyle {
    pub curve_type: EdgeCurveType,
    pub thickness: f32,
    pub color: Color,
    pub arrow_size: f32,
    pub dashed: bool,
}

impl Default for EdgeStyle {
    fn default() -> Self {
        Self {
            curve_type: EdgeCurveType::Straight,
            thickness: 0.1,
            color: Color::srgb(0.3, 0.3, 0.3),
            arrow_size: 0.2,
            dashed: false,
        }
    }
}

/// Edge curve types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EdgeCurveType {
    Straight,
    Bezier,
    Arc,
    Step,
}

/// Component marking an entity as needing layout update
#[derive(Component, Default)]
pub struct NeedsLayout;

/// Component for animated transitions
#[derive(Component)]
pub struct AnimatedTransition {
    pub start_position: Vec3,
    pub target_position: Vec3,
    pub progress: f32,
    pub duration: f32,
}

/// Component for workflow visualization state
#[derive(Component)]
pub struct WorkflowState {
    pub current_step: Option<NodeId>,
    pub completed_steps: Vec<NodeId>,
    pub execution_path: Vec<EdgeId>,
}

/// Component for camera focus targets
#[derive(Component)]
pub struct CameraFocusTarget {
    pub priority: f32,
    pub zoom_level: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_visual_bundle_creation() {
        let node_id = NodeId::new();
        let graph_id = GraphId::new();
        let position = Vec3::new(1.0, 2.0, 3.0);

        let bundle = NodeVisualBundle::new(node_id, graph_id, position);

        assert_eq!(bundle.node.node_id, node_id);
        assert_eq!(bundle.node.graph_id, graph_id);
        assert_eq!(bundle.transform.translation, position);
    }

    #[test]
    fn test_edge_visual_bundle_creation() {
        let edge_id = EdgeId::new();
        let graph_id = GraphId::new();
        let source = Entity::from_raw(1);
        let target = Entity::from_raw(2);

        let bundle = EdgeVisualBundle::new(edge_id, graph_id, source, target);

        assert_eq!(bundle.edge.edge_id, edge_id);
        assert_eq!(bundle.edge.graph_id, graph_id);
        assert_eq!(bundle.edge.source_entity, source);
        assert_eq!(bundle.edge.target_entity, target);
    }
}
