use bevy::prelude::*;
use std::collections::HashMap;
use crate::presentation::components::{GraphNode, SubgraphMember};
use crate::domain::value_objects::Position3D;

/// Component for subgraph root entities that act as transform parents
#[derive(Component)]
pub struct SubgraphRoot {
    pub subgraph_id: usize,
    pub name: String,
}

/// Resource for spatial mapping of subgraphs
#[derive(Resource, Default)]
pub struct SubgraphSpatialMap {
    /// Maps subgraph ID to its root entity
    pub roots: HashMap<usize, Entity>,
    /// Spatial index for efficient queries
    pub spatial_index: HashMap<usize, Transform>,
}

/// System to create subgraph hierarchy from imported nodes
pub fn build_subgraph_hierarchy(
    mut commands: Commands,
    mut spatial_map: ResMut<SubgraphSpatialMap>,
    nodes: Query<(Entity, &GraphNode, &SubgraphMember, &Transform), Added<SubgraphMember>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Group nodes by subgraph
    let mut subgraph_nodes: HashMap<usize, Vec<(Entity, Position3D)>> = HashMap::new();

    for (entity, _node, member, transform) in nodes.iter() {
        let relative_pos = Position3D {
            x: member.relative_position.x,
            y: member.relative_position.y,
            z: member.relative_position.z,
        };

        subgraph_nodes
            .entry(member.subgraph_id)
            .or_insert_with(Vec::new)
            .push((entity, relative_pos));
    }

    // Create or update subgraph roots
    for (subgraph_id, node_list) in subgraph_nodes {
        let root_entity = spatial_map.roots.entry(subgraph_id).or_insert_with(|| {
            // Create a new root entity for this subgraph
            let root = commands.spawn((
                SubgraphRoot {
                    subgraph_id,
                    name: format!("Subgraph_{}", subgraph_id),
                },
                Transform::default(),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                Name::new(format!("SubgraphRoot_{}", subgraph_id)),
            )).id();

            // Optionally add a visual indicator for the subgraph origin
            commands.entity(root).with_children(|parent| {
                parent.spawn((
                    Mesh3d(meshes.add(Cuboid::new(10.0, 0.5, 10.0))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgba(0.3, 0.3, 0.3, 0.3),
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    })),
                    Transform::from_xyz(0.0, -5.0, 0.0),
                    Name::new("SubgraphFloor"),
                ));
            });

            root
        });

        // Set nodes as children of the subgraph root
        for (node_entity, relative_pos) in node_list {
            commands.entity(*root_entity).add_child(node_entity);

            // Update the node's transform to be relative to the subgraph root
            commands.entity(node_entity).insert(
                Transform::from_translation(Vec3::new(
                    relative_pos.x,
                    relative_pos.y,
                    relative_pos.z,
                ))
            );
        }
    }
}

/// System to move entire subgraphs by moving their root transform
pub fn move_subgraph_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut roots: Query<(&SubgraphRoot, &mut Transform)>,
    time: Res<Time>,
) {
    // Example: Move subgraph 0 with arrow keys
    let speed = 100.0 * time.delta_secs();

    for (root, mut transform) in roots.iter_mut() {
        if root.subgraph_id == 0 {
            if keyboard.pressed(KeyCode::ArrowLeft) {
                transform.translation.x -= speed;
            }
            if keyboard.pressed(KeyCode::ArrowRight) {
                transform.translation.x += speed;
            }
            if keyboard.pressed(KeyCode::ArrowUp) {
                transform.translation.y += speed;
            }
            if keyboard.pressed(KeyCode::ArrowDown) {
                transform.translation.y -= speed;
            }
        }
    }
}

/// System to update spatial index for subgraph roots
pub fn update_subgraph_spatial_index(
    mut spatial_map: ResMut<SubgraphSpatialMap>,
    roots: Query<(&SubgraphRoot, &Transform), Changed<Transform>>,
) {
    for (root, transform) in roots.iter() {
        spatial_map.spatial_index.insert(root.subgraph_id, *transform);
    }
}

/// Plugin to handle subgraph spatial mapping
pub struct SubgraphSpatialMapPlugin;

impl Plugin for SubgraphSpatialMapPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SubgraphSpatialMap>()
            .add_systems(Update, (
                build_subgraph_hierarchy,
                move_subgraph_system,
                update_subgraph_spatial_index,
            ).chain());
    }
}
