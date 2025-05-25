use bevy::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::components::{DomainEdgeType, EdgeVisual, OutgoingEdge};

/// Bundle for edge visualization entities
/// Note: Edges in the graph are stored as components on nodes (OutgoingEdge)
/// This bundle is for the visual representation of edges
#[derive(Bundle)]
pub struct EdgeVisualizationBundle {
    pub visual: EdgeVisual,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl EdgeVisualizationBundle {
    pub fn new(start: Vec3, end: Vec3, visual: EdgeVisual) -> Self {
        // Calculate edge position and rotation
        let midpoint = (start + end) / 2.0;
        let _direction = (end - start).normalize();
        let length = start.distance(end);

        // Create transform that positions and orients the edge
        let transform = Transform::from_translation(midpoint)
            .looking_at(end, Vec3::Y)
            .with_scale(Vec3::new(visual.width, visual.width, length));

        Self {
            visual,
            transform,
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            view_visibility: ViewVisibility::default(),
            inherited_visibility: InheritedVisibility::default(),
        }
    }
}

/// Helper to create an OutgoingEdge component
pub fn create_outgoing_edge(
    id: Uuid,
    target: Entity,
    edge_type: DomainEdgeType,
    labels: Vec<String>,
    properties: HashMap<String, String>,
) -> OutgoingEdge {
    OutgoingEdge {
        id,
        target,
        edge_type,
        labels,
        properties,
    }
}

/// Bundle for different edge types
pub struct DataFlowEdgeBundle {
    pub edge: OutgoingEdge,
}

impl DataFlowEdgeBundle {
    pub fn new(id: Uuid, target: Entity) -> Self {
        Self {
            edge: create_outgoing_edge(
                id,
                target,
                DomainEdgeType::DataFlow,
                vec!["dataflow".to_string()],
                HashMap::new(),
            ),
        }
    }
}

pub struct ControlFlowEdgeBundle {
    pub edge: OutgoingEdge,
}

impl ControlFlowEdgeBundle {
    pub fn new(id: Uuid, target: Entity) -> Self {
        Self {
            edge: create_outgoing_edge(
                id,
                target,
                DomainEdgeType::ControlFlow,
                vec!["controlflow".to_string()],
                HashMap::new(),
            ),
        }
    }
}

pub struct DependencyEdgeBundle {
    pub edge: OutgoingEdge,
}

impl DependencyEdgeBundle {
    pub fn new(id: Uuid, target: Entity) -> Self {
        Self {
            edge: create_outgoing_edge(
                id,
                target,
                DomainEdgeType::Dependency,
                vec!["dependency".to_string()],
                HashMap::new(),
            ),
        }
    }
}
