//! Systems for node visual representation
//!
//! These systems handle:
//! - Node mesh generation based on type
//! - Visual updates based on state
//! - Icon and label rendering

use bevy::prelude::*;

use crate::{
    components::*,
    events::*,
    resources::*,
};

/// System that updates node visuals based on domain type changes
pub fn update_node_visuals(
    changed_nodes: Query<
        (Entity, &DomainNodeType, &Handle<Mesh>),
        Changed<DomainNodeType>
    >,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, domain_type, mesh_handle) in changed_nodes.iter() {
        // Update mesh based on new domain type
        if let Some(mesh) = meshes.get_mut(mesh_handle) {
            *mesh = create_mesh_for_type(domain_type);
        }
    }
}

/// System that handles node visual scaling based on importance
pub fn scale_nodes_by_importance(
    nodes: Query<(&Transform, &NodeId), With<NodeImportance>>,
    importance_query: Query<&NodeImportance, Changed<NodeImportance>>,
    mut transforms: Query<&mut Transform, Without<NodeImportance>>,
) {
    for importance in importance_query.iter() {
        // Scale nodes based on importance metric
        // This is a placeholder - actual implementation would be more sophisticated
    }
}

/// System that adds visual indicators for node state
pub fn update_node_state_indicators(
    mut commands: Commands,
    nodes_with_errors: Query<Entity, (With<NodeId>, With<ValidationError>)>,
    nodes_processing: Query<Entity, (With<NodeId>, With<Processing>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Add error indicators
    for entity in nodes_with_errors.iter() {
        // Add red outline or glow effect
    }

    // Add processing indicators
    for entity in nodes_processing.iter() {
        // Add pulsing or spinning effect
    }
}

// Helper functions

fn create_mesh_for_type(domain_type: &DomainNodeType) -> Mesh {
    match domain_type {
        DomainNodeType::Entity | DomainNodeType::Aggregate => {
            Mesh::from(shape::Cube { size: 1.0 })
        }
        DomainNodeType::Event => {
            Mesh::from(shape::UVSphere {
                radius: 0.5,
                sectors: 16,
                stacks: 8,
            })
        }
        DomainNodeType::Command | DomainNodeType::Query => {
            Mesh::from(shape::Cylinder {
                radius: 0.5,
                height: 1.0,
                resolution: 16,
                segments: 1,
            })
        }
        _ => {
            Mesh::from(shape::Icosphere {
                radius: 0.5,
                subdivisions: 2,
            })
        }
    }
}
