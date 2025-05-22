use bevy::prelude::*;
use bevy::math::primitives::{Cuboid, Sphere, Capsule3d, Cylinder};
use uuid::Uuid;
use std::collections::HashMap;

use crate::graph::{AlchemistGraph, GraphNode, GraphEdge};
use crate::ddd_editor::{DddEditor, UpdateDDDGraphEvent};
use crate::graph_editor_ui::GraphEditorTheme;

// Plugin for the DDD Editor 3D view
#[derive(Default)]
pub struct DddEditor3dPlugin;

impl Plugin for DddEditor3dPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_ddd_3d_scene)
            .add_systems(Update, (
                update_ddd_node_positions,
                handle_ddd_node_clicks,
            ));
    }
}

// Resource to keep track of entity mappings for the 3D scene
#[derive(Resource, Default)]
pub struct DddEditorEntities {
    pub node_entities: HashMap<Uuid, Entity>,
    pub edge_entities: HashMap<Uuid, Entity>,
    pub camera_entity: Option<Entity>,
}

// DDD-specific node types for visualization
#[derive(Component)]
pub struct BoundedContextNode;

#[derive(Component)]
pub struct AggregateNode;

#[derive(Component)]
pub struct EntityNode;

#[derive(Component)]
pub struct ValueObjectNode;

#[derive(Component)]
pub struct DddEdgeComponent;

// Component to tag the DDD 3D scene
#[derive(Component)]
pub struct DddScene;

fn setup_ddd_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ddd_editor: Res<DddEditor>,
    theme: Res<GraphEditorTheme>,
) {
    // Create entities for all nodes in the DDD graph
    let mut node_entities = HashMap::new();
    let mut edge_entities = HashMap::new();
    
    // Parent entity for the entire DDD scene
    let scene_entity = commands
        .spawn((
            SpatialBundle::default(),
            DddScene,
            Name::new("DDD Scene"),
        ))
        .id();
    
    // Create camera for the 3D view
    let camera_entity = commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 5.0, 10.0)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            Name::new("DDD Camera"),
        ))
        .id();
    
    // Light for the scene
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    
    // Create entities for nodes
    for (id, node) in ddd_editor.graph.nodes.iter() {
        let mesh = get_mesh_for_ddd_node(node, &mut meshes);
        let material = get_material_for_ddd_node(node, &mut materials, &theme);
        
        let position = ddd_editor.node_positions.get(id)
            .copied()
            .unwrap_or_else(|| Vec3::new(
                rand::random::<f32>() * 5.0 - 2.5,
                rand::random::<f32>() * 5.0,
                rand::random::<f32>() * 5.0 - 2.5,
            ));
        
        let mut entity_commands = commands.spawn((
            PbrBundle {
                mesh,
                material,
                transform: Transform::from_translation(position),
                ..default()
            },
            Name::new(format!("DDD Node: {}", node.name)),
        ));
        
        // Add specific component based on node type
        if node.labels.contains(&"BoundedContext".to_string()) {
            entity_commands.insert(BoundedContextNode);
        } else if node.labels.contains(&"Aggregate".to_string()) {
            entity_commands.insert(AggregateNode);
        } else if node.labels.contains(&"Entity".to_string()) {
            entity_commands.insert(EntityNode);
        } else if node.labels.contains(&"ValueObject".to_string()) {
            entity_commands.insert(ValueObjectNode);
        }
        
        let entity = entity_commands.id();
        node_entities.insert(*id, entity);
        
        // Parent to the scene
        commands.entity(scene_entity).add_child(entity);
    }
    
    // Create entities for edges
    for (id, edge) in ddd_editor.graph.edges.iter() {
        if let (Some(source_entity), Some(target_entity)) = (
            node_entities.get(&edge.source),
            node_entities.get(&edge.target),
        ) {
            // Create a simple line for the edge
            let edge_entity = commands
                .spawn((
                    PbrBundle {
                        mesh: meshes.add(Cylinder::new(0.05, 1.0).into()),
                        material: materials.add(StandardMaterial {
                            base_color: get_color_for_ddd_edge(edge, &theme),
                            ..default()
                        }),
                        transform: Transform::default(),
                        ..default()
                    },
                    DddEdgeComponent,
                    Name::new(format!("DDD Edge: {:?}", edge.labels)),
                ))
                .id();
            
            edge_entities.insert(*id, edge_entity);
            
            // Parent to the scene
            commands.entity(scene_entity).add_child(edge_entity);
        }
    }
    
    // Store entity mappings
    commands.insert_resource(DddEditorEntities {
        node_entities,
        edge_entities,
        camera_entity: Some(camera_entity),
    });
}

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
                                let dir = pos1 - pos2;
                                let distance = dir.length();
                                if distance < 0.001 {
                                    continue;
                                }
                                
                                let repulsion_strength = 2.0;
                                let force = dir.normalize() * repulsion_strength / distance.powf(0.5);
                                
                                *forces.entry(*id1).or_insert(Vec3::ZERO) += force;
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
                let dir = target_pos - source_pos;
                let distance = dir.length();
                
                let attraction_strength = 0.3;
                let ideal_length = 3.0;
                let force = dir.normalize() * attraction_strength * (distance - ideal_length);
                
                *forces.entry(edge.source).or_insert(Vec3::ZERO) += force;
                *forces.entry(edge.target).or_insert(Vec3::ZERO) -= force;
            }
        }
    }
    
    // Apply forces to update positions
    for (id, force) in forces.iter() {
        if let Some(entity) = ddd_entities.node_entities.get(id) {
            if let Ok(mut transform) = transforms.get_mut(*entity) {
                transform.translation += *force * time.delta().as_secs_f32();
                
                // Damping to prevent oscillation
                transform.translation.y = transform.translation.y.clamp(0.0, 10.0);
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
                // Get source and target positions first (immutable borrows)
                let (source_pos, target_pos) = if let (Ok(source_transform), Ok(target_transform)) = (
                    transforms.get(*source_entity),
                    transforms.get(*target_entity),
                ) {
                    (source_transform.translation, target_transform.translation)
                } else {
                    continue; // Skip if we can't get either transform
                };

                // Now get the edge transform (mutable borrow)
                if let Ok(mut edge_transform) = transforms.get_mut(*edge_entity) {
                    let mid_point = (source_pos + target_pos) / 2.0;
                    let direction = target_pos - source_pos;
                    let distance = direction.length();
                    
                    if distance > 0.001 {
                        // Position at midpoint between nodes
                        edge_transform.translation = mid_point;
                        
                        // Orient towards target
                        edge_transform.rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());
                        
                        // Scale to match distance
                        edge_transform.scale = Vec3::new(1.0, distance, 1.0);
                    }
                }
            }
        }
    }
}

fn handle_ddd_node_clicks(
    // This would handle 3D picking and interaction
    // For simplicity, this is left as a placeholder
) {
    // 3D node selection would be implemented here
}

fn get_mesh_for_ddd_node(node: &GraphNode, meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
    // Different node types get different meshes
    if node.labels.contains(&"BoundedContext".to_string()) {
        meshes.add(Cuboid::new(1.0, 1.0, 1.0).into())
    } else if node.labels.contains(&"Aggregate".to_string()) {
        meshes.add(Sphere::new(0.8).into())
    } else if node.labels.contains(&"Entity".to_string()) {
        meshes.add(Sphere::new(0.6).into())
    } else if node.labels.contains(&"ValueObject".to_string()) {
        meshes.add(Capsule3d::new(0.5, 0.3).into())
    } else {
        // Default
        meshes.add(Sphere::new(0.5).into())
    }
}

fn get_material_for_ddd_node(node: &GraphNode, materials: &mut ResMut<Assets<StandardMaterial>>, theme: &Res<GraphEditorTheme>) -> Handle<StandardMaterial> {
    let base_theme = &theme.current_theme;
    
    let color = if node.labels.contains(&"BoundedContext".to_string()) {
        // Convert from egui::Color32 to bevy::Color
        let c = base_theme.base08; // Red for bounded contexts
        Color::srgba(c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, c.a() as f32 / 255.0)
    } else if node.labels.contains(&"Aggregate".to_string()) {
        let c = base_theme.base0D; // Blue for aggregates
        Color::srgba(c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, c.a() as f32 / 255.0)
    } else if node.labels.contains(&"Entity".to_string()) {
        let c = base_theme.base0B; // Green for entities
        Color::srgba(c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, c.a() as f32 / 255.0)
    } else if node.labels.contains(&"ValueObject".to_string()) {
        let c = base_theme.base0A; // Yellow for value objects
        Color::srgba(c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, c.a() as f32 / 255.0)
    } else {
        let c = base_theme.base05; // Default foreground
        Color::srgba(c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, c.a() as f32 / 255.0)
    };
    
    materials.add(StandardMaterial {
        base_color: color,
        metallic: 0.2,
        perceptual_roughness: 0.3,
        ..default()
    })
}

fn get_color_for_ddd_edge(edge: &GraphEdge, theme: &Res<GraphEditorTheme>) -> Color {
    let base_theme = &theme.current_theme;
    
    if edge.labels.contains(&"Contains".to_string()) {
        let c = base_theme.base03; // Comments color for containment
        Color::srgba(c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, c.a() as f32 / 255.0)
    } else if edge.labels.contains(&"References".to_string()) {
        let c = base_theme.base0D; // Blue for references
        Color::srgba(c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, c.a() as f32 / 255.0)
    } else if edge.labels.contains(&"Has".to_string()) {
        let c = base_theme.base0B; // Green for has-a
        Color::srgba(c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, c.a() as f32 / 255.0)
    } else if edge.labels.contains(&"Uses".to_string()) {
        let c = base_theme.base09; // Orange for uses
        Color::srgba(c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, c.a() as f32 / 255.0)
    } else {
        let c = base_theme.base04; // Default dark foreground
        Color::srgba(c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, c.a() as f32 / 255.0)
    }
} 