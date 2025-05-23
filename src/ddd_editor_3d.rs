use bevy::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::ddd_editor::DddEditor;
use crate::graph::{GraphEdge, GraphNode};
use crate::graph_editor_ui::GraphEditorTheme;

// Plugin for the DDD Editor 3D view
#[derive(Default)]
pub struct DddEditor3dPlugin;

impl Plugin for DddEditor3dPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ddd_3d_scene)
            .add_systems(Update, (update_ddd_node_positions, handle_ddd_node_clicks));
    }
}

// Components to track entities
#[derive(Component)]
struct DddNodeComponent {
    id: Uuid,
}

#[derive(Component)]
struct DddEdgeComponent;

// Resource to track entity mappings
#[derive(Resource, Default)]
struct DddEditorEntities {
    node_entities: HashMap<Uuid, Entity>,
    edge_entities: HashMap<Uuid, Entity>,
}

// Setup the 3D scene for DDD visualization
fn setup_ddd_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ddd_editor: Res<DddEditor>,
    theme: Res<GraphEditorTheme>,
) {
    // Note: Camera is handled by GraphEditor3DPlugin, so we don't spawn one here

    // Add lighting (only if not already present)
    commands.spawn((
        PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Initialize entity tracking
    let mut node_entities = HashMap::new();
    let mut edge_entities = HashMap::new();

    // Create entities for each node
    for (id, node) in ddd_editor.graph.nodes.iter() {
        let position = if let Some(pos) = ddd_editor.graph.node_positions.get(id) {
            Vec3::new(pos.x, 0.0, pos.y)
        } else {
            Vec3::new(0.0, 0.0, 0.0) // Default position
        };

        let node_entity = commands
            .spawn((
                Mesh3d(get_mesh_for_ddd_node(node, &mut meshes)),
                MeshMaterial3d(get_material_for_ddd_node(node, &mut materials, &theme)),
                Transform::from_translation(position),
                DddNodeComponent { id: *id },
                Name::new(format!("DDD Node: {}", node.name)),
            ))
            .id();

        node_entities.insert(*id, node_entity);
    }

    // Create entities for each edge
    for (id, edge) in ddd_editor.graph.edges.iter() {
        // Create a simple line for the edge
        let edge_entity = commands
            .spawn((
                Mesh3d(meshes.add(Cylinder::default())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: get_color_for_ddd_edge(edge, &theme),
                    ..default()
                })),
                Transform::default(),
                DddEdgeComponent,
                Name::new(format!("DDD Edge: {:?}", edge.labels)),
            ))
            .id();

        edge_entities.insert(*id, edge_entity);
    }

    // Store entity mappings
    commands.insert_resource(DddEditorEntities {
        node_entities,
        edge_entities,
    });
}

// Get appropriate mesh for different DDD node types
fn get_mesh_for_ddd_node(node: &GraphNode, meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
    // Different node types get different meshes
    if node.labels.contains(&"BoundedContext".to_string()) {
        meshes.add(Cuboid::default())
    } else if node.labels.contains(&"Aggregate".to_string()) {
        meshes.add(Sphere::default().mesh().ico(5).unwrap())
    } else if node.labels.contains(&"Entity".to_string()) {
        meshes.add(Sphere::default().mesh().uv(16, 16))
    } else if node.labels.contains(&"ValueObject".to_string()) {
        meshes.add(Sphere::default().mesh().uv(12, 12))
    } else {
        // Default
        meshes.add(Sphere::default().mesh().uv(12, 12))
    }
}

// Get material with appropriate color for different DDD node types
fn get_material_for_ddd_node(
    node: &GraphNode,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    theme: &Res<GraphEditorTheme>,
) -> Handle<StandardMaterial> {
    let base_theme = &theme.current_theme;

    let color = if node.labels.contains(&"BoundedContext".to_string()) {
        // Convert from egui::Color32 to bevy::Color
        let c = base_theme.base08; // Red for bounded contexts
        Color::srgb_u8(c.r(), c.g(), c.b())
    } else if node.labels.contains(&"Aggregate".to_string()) {
        let c = base_theme.base0D; // Blue for aggregates
        Color::srgb_u8(c.r(), c.g(), c.b())
    } else if node.labels.contains(&"Entity".to_string()) {
        let c = base_theme.base0B; // Green for entities
        Color::srgb_u8(c.r(), c.g(), c.b())
    } else if node.labels.contains(&"ValueObject".to_string()) {
        let c = base_theme.base0A; // Yellow for value objects
        Color::srgb_u8(c.r(), c.g(), c.b())
    } else {
        let c = base_theme.base04; // Gray for other nodes
        Color::srgb_u8(c.r(), c.g(), c.b())
    };

    materials.add(StandardMaterial {
        base_color: color,
        ..default()
    })
}

// Get appropriate color for edge based on relationship type
fn get_color_for_ddd_edge(edge: &GraphEdge, theme: &Res<GraphEditorTheme>) -> Color {
    let base_theme = &theme.current_theme;

    let c = if edge.labels.contains(&"contains".to_string()) {
        base_theme.base0C // Aqua for containment
    } else if edge.labels.contains(&"references".to_string()) {
        base_theme.base0D // Blue for references
    } else if edge.labels.contains(&"implements".to_string()) {
        base_theme.base0E // Purple for implementation
    } else {
        base_theme.base03 // Comments color for other edges
    };

    Color::srgb_u8(c.r(), c.g(), c.b())
}

// Update node positions based on forces
fn update_ddd_node_positions(
    _commands: Commands,
    ddd_editor: Res<DddEditor>,
    ddd_entities: Res<DddEditorEntities>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
) {
    // Simple force-directed layout
    // In a real implementation, this would be more sophisticated
    let mut forces = HashMap::<Uuid, Vec3>::new();

    // Initialize forces
    for id in ddd_editor.graph.nodes.keys() {
        forces.insert(*id, Vec3::ZERO);
    }

    // Node repulsion
    for (id1, _node1) in ddd_editor.graph.nodes.iter() {
        if let Some(entity1) = ddd_entities.node_entities.get(id1) {
            if let Ok(transform1) = transforms.get(*entity1) {
                let pos1 = transform1.translation;

                for (id2, _node2) in ddd_editor.graph.nodes.iter() {
                    if id1 != id2 {
                        if let Some(entity2) = ddd_entities.node_entities.get(id2) {
                            if let Ok(transform2) = transforms.get(*entity2) {
                                let pos2 = transform2.translation;
                                let direction = pos1 - pos2;
                                let distance = direction.length();

                                if distance < 0.001 {
                                    continue; // Avoid division by zero
                                }

                                let repulsion = 5.0 / (distance * distance);
                                let repulsion_force = direction.normalize() * repulsion;

                                if let Some(force) = forces.get_mut(id1) {
                                    *force += repulsion_force;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Edge attraction
    for (_, edge) in ddd_editor.graph.edges.iter() {
        if let (Some(source_entity), Some(target_entity)) = (
            ddd_entities.node_entities.get(&edge.source),
            ddd_entities.node_entities.get(&edge.target),
        ) {
            if let (Ok(source_transform), Ok(target_transform)) = (
                transforms.get(*source_entity),
                transforms.get(*target_entity),
            ) {
                let source_pos = source_transform.translation;
                let target_pos = target_transform.translation;
                let direction = target_pos - source_pos;
                let distance = direction.length();

                if distance < 0.001 {
                    continue; // Avoid division by zero
                }

                let attraction = 0.05 * distance;
                let attraction_force = direction.normalize() * attraction;

                if let Some(force) = forces.get_mut(&edge.source) {
                    *force += attraction_force;
                }

                if let Some(force) = forces.get_mut(&edge.target) {
                    *force -= attraction_force;
                }
            }
        }
    }

    // Apply forces to node positions
    let delta_seconds = time.delta_secs();
    let damping = 0.8;
    for (id, force) in forces.iter() {
        if let Some(entity) = ddd_entities.node_entities.get(id) {
            if let Ok(mut transform) = transforms.get_mut(*entity) {
                transform.translation += *force * delta_seconds * damping;
                // Keep nodes at same height
                transform.translation.y = 0.0;
            }
        }
    }

    // Update edge transforms to connect nodes
    for (id, edge) in ddd_editor.graph.edges.iter() {
        if let Some(edge_entity) = ddd_entities.edge_entities.get(id) {
            if let (Some(source_entity), Some(target_entity)) = (
                ddd_entities.node_entities.get(&edge.source),
                ddd_entities.node_entities.get(&edge.target),
            ) {
                // Get source and target positions first (immutable borrow)
                let source_pos;
                let target_pos;

                if let Ok(source_transform) = transforms.get(*source_entity) {
                    source_pos = source_transform.translation;
                } else {
                    continue;
                }

                if let Ok(target_transform) = transforms.get(*target_entity) {
                    target_pos = target_transform.translation;
                } else {
                    continue;
                }

                // Now update the edge transform (mutable borrow)
                if let Ok(mut edge_transform) = transforms.get_mut(*edge_entity) {
                    let midpoint = (source_pos + target_pos) / 2.0;
                    let direction = target_pos - source_pos;
                    let distance = direction.length();

                    if distance < 0.001 {
                        continue; // Avoid division by zero
                    }

                    // Point the cylinder in the right direction
                    let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());

                    edge_transform.translation = midpoint;
                    edge_transform.rotation = rotation;
                    edge_transform.scale = Vec3::new(1.0, distance, 1.0);
                }
            }
        }
    }
}

// Handle mouse clicks on nodes
fn handle_ddd_node_clicks(// TODO: Implement this in a future revision
) {
    // This would be implemented to allow interaction with the nodes
}
