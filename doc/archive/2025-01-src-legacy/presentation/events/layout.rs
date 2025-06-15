//! Layout calculation events that aggregate into domain position updates
//!
//! Force-directed layouts and other algorithms generate many position updates
//! that are aggregated before being sent to the domain.

use super::PresentationEvent;
use crate::domain::value_objects::NodeId;
use bevy::prelude::*;

/// Force-directed layout iteration
#[derive(Event, Clone, Debug)]
pub struct ForceLayoutIteration {
    pub iteration: u32,
    pub total_energy: f32,
    pub max_displacement: f32,
    pub node_updates: Vec<(NodeId, Vec3, Vec3)>, // (id, old_pos, new_pos)
}

impl PresentationEvent for ForceLayoutIteration {
    fn requires_aggregation(&self) -> bool {
        true // Aggregate many iterations into final positions
    }
}

/// Layout algorithm completed
#[derive(Event, Clone, Debug)]
pub struct LayoutComplete {
    pub layout_type: LayoutType,
    pub iterations_performed: u32,
    pub final_positions: Vec<(NodeId, Vec3)>,
    pub convergence_achieved: bool,
}

impl PresentationEvent for LayoutComplete {}

#[derive(Clone, Debug, PartialEq)]
pub enum LayoutType {
    ForceDirected,
    Hierarchical,
    Circular,
    Grid,
    Conceptual, // Based on conceptual space dimensions
    Strategic,  // Based on game theory components
    Custom(String),
}

/// Temporary node positioning during layout
#[derive(Event, Clone, Debug)]
pub struct TemporaryNodePosition {
    pub node_id: NodeId,
    pub entity: Entity,
    pub position: Vec3,
    pub is_preview: bool,
}

impl PresentationEvent for TemporaryNodePosition {
    fn requires_aggregation(&self) -> bool {
        true // Don't send individual positions
    }
}

/// Visual clustering animation
#[derive(Event, Clone, Debug)]
pub struct ClusteringUpdate {
    pub cluster_id: u32,
    pub member_nodes: Vec<NodeId>,
    pub centroid: Vec3,
    pub radius: f32,
    pub iteration: u32,
}

impl PresentationEvent for ClusteringUpdate {}

/// Grid snapping preview
#[derive(Event, Clone, Debug)]
pub struct GridSnapPreview {
    pub node_id: NodeId,
    pub original_position: Vec3,
    pub snapped_position: Vec3,
    pub grid_size: f32,
}

impl PresentationEvent for GridSnapPreview {
    fn requires_aggregation(&self) -> bool {
        false // Preview only, not committed
    }
}

/// Alignment operation preview
#[derive(Event, Clone, Debug)]
pub struct AlignmentPreview {
    pub alignment_type: AlignmentType,
    pub affected_nodes: Vec<(NodeId, Vec3, Vec3)>, // (id, current, aligned)
}

#[derive(Clone, Debug)]
pub enum AlignmentType {
    Horizontal,
    Vertical,
    CircularArc,
    DistributeEvenly,
    AlignToGrid,
}

impl PresentationEvent for AlignmentPreview {
    fn requires_aggregation(&self) -> bool {
        false // Preview until confirmed
    }
}

/// Layout constraint update
#[derive(Event, Clone, Debug)]
pub struct LayoutConstraintUpdate {
    pub constraint_type: LayoutConstraint,
    pub affected_nodes: Vec<NodeId>,
    pub active: bool,
}

#[derive(Clone, Debug)]
pub enum LayoutConstraint {
    FixedPosition { node_id: NodeId, position: Vec3 },
    MinimumDistance { distance: f32 },
    MaximumDistance { distance: f32 },
    BoundingBox { min: Vec3, max: Vec3 },
    KeepOnPlane { normal: Vec3, offset: f32 },
}

impl PresentationEvent for LayoutConstraintUpdate {}

/// Conceptual force calculation
#[derive(Event, Clone, Debug)]
pub struct ConceptualForceUpdate {
    pub node_a: NodeId,
    pub node_b: NodeId,
    pub semantic_similarity: f32,
    pub force_vector: Vec3,
    pub force_type: ConceptualForceType,
}

#[derive(Clone, Debug)]
pub enum ConceptualForceType {
    SemanticAttraction,
    CategoryRepulsion,
    TemporalProximity,
    StrategicAlignment,
}

impl PresentationEvent for ConceptualForceUpdate {
    fn requires_aggregation(&self) -> bool {
        true // Part of larger layout calculation
    }
}
