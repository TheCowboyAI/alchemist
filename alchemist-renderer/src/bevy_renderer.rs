//! Bevy 3D renderer implementation

use anyhow::Result;
use bevy::prelude::*;
use alchemist::renderer::{RenderRequest, RenderData, GraphNode as AlchemistNode, GraphEdge as AlchemistEdge};

pub fn run(request: RenderRequest) -> Result<()> {
    let mut app = App::new();
    
    // Configure window
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: request.title.clone(),
            resolution: (request.config.width as f32, request.config.height as f32).into(),
            resizable: request.config.resizable,
            ..default()
        }),
        ..default()
    }));
    
    // Add render data as resource
    app.insert_resource(request);
    
    // Add our systems
    app.add_systems(Startup, setup_scene);
    app.add_systems(Update, (
        handle_input,
        rotate_camera,
        update_graph_layout,
    ));
    
    // Add graph-specific components and systems based on data type
    app.add_systems(Startup, setup_graph_visualization);
    
    app.run();
    
    Ok(())
}

#[derive(Component)]
struct GraphNodeMarker {
    id: String,
}

#[derive(Component)]
struct GraphEdgeMarker {
    source: String,
    target: String,
}

#[derive(Component)]
struct MainCamera;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Add light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    
    // Add camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera,
    ));
    
    // Add ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 10.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.2),
            ..default()
        })),
    ));
}

fn setup_graph_visualization(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    request: Res<RenderRequest>,
) {
    if let RenderData::Graph3D { nodes, edges } = &request.data {
        // Create spheres for nodes
        for (i, node) in nodes.iter().enumerate() {
            let position = if let Some(pos) = node.position {
                Vec3::new(pos[0], pos[1], pos[2])
            } else {
                // Auto-layout in a circle
                let angle = (i as f32) * std::f32::consts::TAU / (nodes.len() as f32);
                Vec3::new(angle.cos() * 3.0, 0.5, angle.sin() * 3.0)
            };
            
            let color = if let Some(col) = node.color {
                Color::srgba(col[0], col[1], col[2], col[3])
            } else {
                Color::srgb(0.3, 0.7, 1.0)
            };
            
            let size = node.size.unwrap_or(1.0);
            
            // Spawn node entity
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(size * 0.2))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    emissive: color.as_linear() * 0.2,
                    ..default()
                })),
                Transform::from_translation(position),
                GraphNodeMarker { id: node.id.clone() },
            ));
            
            // Add label
            commands.spawn((
                Text3d::new(node.label.clone()),
                Transform::from_xyz(position.x, position.y + size * 0.3, position.z)
                    .with_scale(Vec3::splat(0.1)),
            ));
        }
        
        // Create lines for edges
        // Note: In a real implementation, we'd create proper line meshes
        // For now, we'll use thin cylinders
        for edge in edges {
            // Find node positions
            let source_pos = nodes.iter()
                .find(|n| n.id == edge.source)
                .and_then(|n| n.position)
                .map(|p| Vec3::new(p[0], p[1], p[2]))
                .unwrap_or(Vec3::ZERO);
                
            let target_pos = nodes.iter()
                .find(|n| n.id == edge.target)
                .and_then(|n| n.position)
                .map(|p| Vec3::new(p[0], p[1], p[2]))
                .unwrap_or(Vec3::ZERO);
            
            let midpoint = (source_pos + target_pos) * 0.5;
            let direction = target_pos - source_pos;
            let distance = direction.length();
            
            if distance > 0.0 {
                let color = if let Some(col) = edge.color {
                    Color::srgba(col[0], col[1], col[2], col[3])
                } else {
                    Color::srgb(0.5, 0.5, 0.5)
                };
                
                // Create edge as a thin cylinder
                commands.spawn((
                    Mesh3d(meshes.add(Cylinder::new(0.05, distance))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: color,
                        ..default()
                    })),
                    Transform::from_translation(midpoint)
                        .looking_at(target_pos, Vec3::Y),
                    GraphEdgeMarker {
                        source: edge.source.clone(),
                        target: edge.target.clone(),
                    },
                ));
            }
        }
    }
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut app_exit: EventWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        app_exit.send(AppExit::Success);
    }
}

fn rotate_camera(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<MainCamera>>,
) {
    if keyboard.pressed(KeyCode::Space) {
        for mut transform in query.iter_mut() {
            let rotation = Quat::from_rotation_y(time.delta_secs() * 0.5);
            transform.rotate_around(Vec3::ZERO, rotation);
        }
    }
}

fn update_graph_layout(
    time: Res<Time>,
    mut nodes: Query<(&GraphNodeMarker, &mut Transform)>,
) {
    // Simple animation: make nodes bob up and down
    for (_node, mut transform) in nodes.iter_mut() {
        transform.translation.y = 0.5 + (time.elapsed_secs() * 2.0).sin() * 0.1;
    }
}