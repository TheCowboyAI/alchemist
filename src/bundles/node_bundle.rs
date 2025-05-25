use bevy::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::components::{
    DomainNodeType, GraphNode, GraphPosition, NodeVisual, UIInteractable,
};

/// Standard bundle for spawning graph nodes
#[derive(Bundle)]
pub struct GraphNodeBundle {
    pub node: GraphNode,
    pub position: GraphPosition,
    pub visual: NodeVisual,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl GraphNodeBundle {
    pub fn new(
        id: Uuid,
        domain_type: DomainNodeType,
        position: Vec3,
        color: Color,
        name: String,
        labels: Vec<String>,
        properties: HashMap<String, String>,
    ) -> Self {
        Self {
            node: GraphNode {
                id,
                domain_type,
                name,
                labels,
                properties,
            },
            position: GraphPosition(position),
            visual: NodeVisual {
                base_color: color,
                current_color: color,
            },
            transform: Transform::from_translation(position),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            view_visibility: ViewVisibility::default(),
            inherited_visibility: InheritedVisibility::default(),
        }
    }
}

/// Bundle for decision nodes
#[derive(Bundle)]
pub struct DecisionNodeBundle {
    pub node: GraphNode,
    pub position: GraphPosition,
    pub visual: NodeVisual,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub interactable: UIInteractable,
}

impl DecisionNodeBundle {
    pub fn new(id: Uuid, position: Vec3, name: String) -> Self {
        let labels = vec!["decision".to_string()];
        let properties = HashMap::new();
        let color = Color::srgb(1.0, 0.7, 0.0); // Orange

        Self {
            node: GraphNode {
                id,
                domain_type: DomainNodeType::Decision,
                name,
                labels,
                properties,
            },
            position: GraphPosition(position),
            visual: NodeVisual {
                base_color: color,
                current_color: color,
            },
            transform: Transform::from_translation(position),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            view_visibility: ViewVisibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            interactable: UIInteractable,
        }
    }
}

/// Bundle for event nodes
#[derive(Bundle)]
pub struct EventNodeBundle {
    pub node: GraphNode,
    pub position: GraphPosition,
    pub visual: NodeVisual,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub interactable: UIInteractable,
}

impl EventNodeBundle {
    pub fn new(id: Uuid, position: Vec3, name: String) -> Self {
        let labels = vec!["event".to_string()];
        let properties = HashMap::new();
        let color = Color::srgb(0.0, 0.8, 0.8); // Cyan

        Self {
            node: GraphNode {
                id,
                domain_type: DomainNodeType::Event,
                name,
                labels,
                properties,
            },
            position: GraphPosition(position),
            visual: NodeVisual {
                base_color: color,
                current_color: color,
            },
            transform: Transform::from_translation(position),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            view_visibility: ViewVisibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            interactable: UIInteractable,
        }
    }
}

/// Bundle for process nodes
#[derive(Bundle)]
pub struct ProcessNodeBundle {
    pub node: GraphNode,
    pub position: GraphPosition,
    pub visual: NodeVisual,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub interactable: UIInteractable,
}

impl ProcessNodeBundle {
    pub fn new(id: Uuid, position: Vec3, name: String) -> Self {
        let labels = vec!["process".to_string()];
        let properties = HashMap::new();
        let color = Color::srgb(0.0, 0.6, 1.0); // Blue

        Self {
            node: GraphNode {
                id,
                domain_type: DomainNodeType::Process,
                name,
                labels,
                properties,
            },
            position: GraphPosition(position),
            visual: NodeVisual {
                base_color: color,
                current_color: color,
            },
            transform: Transform::from_translation(position),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            view_visibility: ViewVisibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            interactable: UIInteractable,
        }
    }
}
