//! Visual Events: Morphisms (Arrows) in the Bevy ECS Category
//!
//! These events are the morphisms in the visual category that correspond
//! to operations in the CIM-ContextGraph category. They preserve the
//! operational structure while adding visual-specific semantics.

use bevy::prelude::*;
use cim_contextgraph::{NodeId, EdgeId, ContextGraphId as GraphId};

/// Edge relationship types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EdgeRelationship {
    /// Generic connection
    Connected,
    /// Hierarchical parent-child
    ParentChild,
    /// Dependency relationship
    DependsOn,
    /// Data flow
    DataFlow,
    /// Control flow
    ControlFlow,
    /// Custom relationship
    Custom(String),
}

// ============================================================================
// Creation Morphisms (Domain → Visual)
// ============================================================================

/// Morphism: Create visual representation of a node
#[derive(Event, Debug, Clone)]
pub struct CreateNodeVisual {
    pub node_id: NodeId,
    pub graph_id: GraphId,
    pub position: Vec3,
    pub metadata: serde_json::Value,
}

/// Morphism: Create visual representation of an edge
#[derive(Event, Debug, Clone)]
pub struct CreateEdgeVisual {
    pub graph_id: GraphId,
    pub edge_id: EdgeId,
    pub source_id: NodeId,
    pub target_id: NodeId,
    pub relationship: EdgeRelationship,
}

/// Morphism: Create visual representation of a graph
#[derive(Event, Debug, Clone)]
pub struct CreateGraphVisual {
    pub graph_id: GraphId,
    pub name: String,
    pub metadata: serde_json::Value,
}

// ============================================================================
// Deletion Morphisms (Domain → Visual)
// ============================================================================

/// Morphism: Remove visual representation of a node
#[derive(Event, Debug, Clone)]
pub struct RemoveNodeVisual {
    pub node_id: NodeId,
    pub graph_id: GraphId,
}

/// Morphism: Remove visual representation of an edge
#[derive(Event, Debug, Clone)]
pub struct RemoveEdgeVisual {
    pub edge_id: EdgeId,
    pub graph_id: GraphId,
}

/// Morphism: Remove visual representation of a graph
#[derive(Event, Debug, Clone)]
pub struct RemoveGraphVisual {
    pub graph_id: GraphId,
}

// ============================================================================
// Interaction Morphisms (Visual → Domain)
// ============================================================================

/// Morphism: User clicked on a node
#[derive(Event, Debug, Clone)]
pub struct NodeClicked {
    pub entity: Entity,
    pub node_id: NodeId,
    pub graph_id: GraphId,
    pub world_position: Vec3,
}

/// Morphism: User hovered over a node
#[derive(Event, Debug, Clone)]
pub struct NodeHovered {
    pub entity: Entity,
    pub node_id: NodeId,
    pub graph_id: GraphId,
}

/// Morphism: User stopped hovering over a node
#[derive(Event, Debug, Clone)]
pub struct NodeUnhovered {
    pub entity: Entity,
    pub node_id: NodeId,
    pub graph_id: GraphId,
}

/// Morphism: User clicked on an edge
#[derive(Event, Debug, Clone)]
pub struct EdgeClicked {
    pub entity: Entity,
    pub edge_id: EdgeId,
    pub graph_id: GraphId,
}

/// Morphism: User clicked on background
#[derive(Event, Debug, Clone)]
pub struct BackgroundClicked {
    pub world_position: Vec3,
}

// ============================================================================
// Drag Morphisms (Visual state transitions)
// ============================================================================

/// Morphism: Node drag operation started
#[derive(Event, Debug, Clone)]
pub struct NodeDragStart {
    pub entity: Entity,
    pub node_id: NodeId,
    pub graph_id: GraphId,
    pub start_position: Vec3,
}

/// Morphism: Node being dragged
#[derive(Event, Debug, Clone)]
pub struct NodeDragging {
    pub entity: Entity,
    pub node_id: NodeId,
    pub graph_id: GraphId,
    pub current_position: Vec3,
    pub delta: Vec3,
}

/// Morphism: Node drag operation ended
#[derive(Event, Debug, Clone)]
pub struct NodeDragEnd {
    pub entity: Entity,
    pub node_id: NodeId,
    pub graph_id: GraphId,
    pub new_position: Vec3,
}

// ============================================================================
// Selection Morphisms (Visual state transitions)
// ============================================================================

/// Morphism: Selection state changed
#[derive(Event, Debug, Clone)]
pub struct SelectionChanged {
    pub selected_nodes: Vec<(Entity, NodeId)>,
    pub selected_edges: Vec<(Entity, EdgeId)>,
    pub graph_id: GraphId,
}

/// Morphism: Request to select all
#[derive(Event, Debug, Clone)]
pub struct SelectAll {
    pub graph_id: GraphId,
}

/// Morphism: Request to clear selection
#[derive(Event, Debug, Clone)]
pub struct ClearSelection {
    pub graph_id: GraphId,
}

// ============================================================================
// Layout Morphisms (Visual transformations)
// ============================================================================

/// Morphism: Request layout recalculation
#[derive(Event, Debug, Clone)]
pub struct RequestLayout {
    pub graph_id: GraphId,
    pub layout_type: crate::components::LayoutType,
}

/// Morphism: Layout calculation completed
#[derive(Event, Debug, Clone)]
pub struct LayoutCompleted {
    pub graph_id: GraphId,
    pub node_positions: Vec<(NodeId, Vec3)>,
}

// ============================================================================
// Camera Morphisms (Visual navigation)
// ============================================================================

/// Morphism: Focus camera on entities
#[derive(Event, Debug, Clone)]
pub struct FocusCamera {
    pub target_entities: Vec<Entity>,
    pub transition_duration: f32,
}

/// Morphism: Reset camera to default view
#[derive(Event, Debug, Clone)]
pub struct ResetCamera {
    pub transition_duration: f32,
}

// ============================================================================
// Animation Morphisms (Visual effects)
// ============================================================================

/// Morphism: Animate node appearance
#[derive(Event, Debug, Clone)]
pub struct AnimateNodeAppear {
    pub entity: Entity,
    pub duration: f32,
}

/// Morphism: Animate edge connection
#[derive(Event, Debug, Clone)]
pub struct AnimateEdgeConnect {
    pub entity: Entity,
    pub duration: f32,
}

/// Morphism: Highlight path through graph
#[derive(Event, Debug, Clone)]
pub struct HighlightPath {
    pub nodes: Vec<(Entity, NodeId)>,
    pub edges: Vec<(Entity, EdgeId)>,
    pub color: Color,
    pub duration: f32,
}

// ============================================================================
// Domain Request Morphisms (Visual → Domain operations)
// ============================================================================

/// Morphism: Request node creation at position
#[derive(Event, Debug, Clone)]
pub struct RequestNodeCreation {
    pub graph_id: GraphId,
    pub position: Vec3,
    pub metadata: serde_json::Value,
}

/// Morphism: Request edge creation between nodes
#[derive(Event, Debug, Clone)]
pub struct RequestEdgeCreation {
    pub graph_id: GraphId,
    pub source_id: NodeId,
    pub target_id: NodeId,
    pub relationship: EdgeRelationship,
}

/// Morphism: Request deletion of selected entities
#[derive(Event, Debug, Clone)]
pub struct RequestDeleteSelected {
    pub graph_id: GraphId,
}

// ============================================================================
// Update Morphisms (Domain → Visual synchronization)
// ============================================================================

/// Morphism: Node position changed in domain
#[derive(Event, Debug, Clone)]
pub struct NodePositionChanged {
    pub node_id: NodeId,
    pub graph_id: GraphId,
    pub old_position: Vec3,
    pub new_position: Vec3,
}

/// Morphism: Node metadata changed in domain
#[derive(Event, Debug, Clone)]
pub struct NodeMetadataChanged {
    pub node_id: NodeId,
    pub graph_id: GraphId,
    pub metadata: serde_json::Value,
}

/// Morphism: Edge metadata changed in domain
#[derive(Event, Debug, Clone)]
pub struct EdgeMetadataChanged {
    pub edge_id: EdgeId,
    pub graph_id: GraphId,
    pub metadata: serde_json::Value,
}

/// Domain events that flow from the domain layer to the visualization
#[derive(Debug, Clone, Event)]
pub enum DomainEvent {
    /// A node was added to the graph
    NodeAdded {
        graph_id: GraphId,
        node_id: NodeId,
        position: Option<Vec3>,
        metadata: serde_json::Value,
    },
    /// A node was removed from the graph
    NodeRemoved {
        graph_id: GraphId,
        node_id: NodeId,
    },
    /// An edge was added to the graph
    EdgeAdded {
        graph_id: GraphId,
        edge_id: EdgeId,
        source: NodeId,
        target: NodeId,
        relationship: EdgeRelationship,
    },
    /// An edge was removed from the graph
    EdgeRemoved {
        graph_id: GraphId,
        edge_id: EdgeId,
    },
    /// Node metadata was updated
    NodeMetadataUpdated {
        graph_id: GraphId,
        node_id: NodeId,
        metadata: serde_json::Value,
    },
    /// Edge metadata was updated
    EdgeMetadataUpdated {
        graph_id: GraphId,
        edge_id: EdgeId,
        metadata: serde_json::Value,
    },
}

/// Commands from visualization to domain
#[derive(Debug, Clone, Event)]
pub enum VisualizationCommand {
    /// Create a new node
    CreateNode {
        graph_id: GraphId,
        position: Vec3,
        metadata: Option<serde_json::Value>,
    },
    /// Delete a node
    DeleteNode {
        graph_id: GraphId,
        node_id: NodeId,
    },
    /// Update node position
    UpdateNodePosition {
        graph_id: GraphId,
        node_id: NodeId,
        position: Vec3,
    },
    /// Create an edge
    CreateEdge {
        graph_id: GraphId,
        source: NodeId,
        target: NodeId,
        relationship: EdgeRelationship,
    },
    /// Delete an edge
    DeleteEdge {
        graph_id: GraphId,
        edge_id: EdgeId,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let entity = Entity::from_raw(1);
        let node_id = NodeId::new();
        let graph_id = GraphId::new();

        let event = NodeClicked {
            entity,
            node_id,
            graph_id,
            world_position: Vec3::ZERO,
        };

        assert_eq!(event.entity, entity);
        assert_eq!(event.node_id, node_id);
        assert_eq!(event.graph_id, graph_id);
    }

    #[test]
    fn test_selection_events() {
        let graph_id = GraphId::new();
        let node_id = NodeId::new();
        let entity = Entity::from_raw(1);

        let selection_changed = SelectionChanged {
            selected_nodes: vec![(entity, node_id)],
            selected_edges: vec![],
            graph_id,
        };

        assert_eq!(selection_changed.selected_nodes.len(), 1);
        assert_eq!(selection_changed.selected_edges.len(), 0);
        assert_eq!(selection_changed.graph_id, graph_id);
    }
}
