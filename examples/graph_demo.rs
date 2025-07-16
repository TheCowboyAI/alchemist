//! Simple Graph Visualization Demo for Alchemist
//! 
//! This creates a basic 3D graph visualization using Bevy

use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
struct GraphNode {
    id: String,
    label: String,
}

#[derive(Component)]
struct GraphEdge {
    source: String,
    target: String,
}

#[derive(Resource)]
struct GraphData {
    nodes: HashMap<String, Entity>,
}

fn main() {
    println!("Starting Alchemist Graph Visualization...");
    
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Alchemist - Graph Visualization".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(GraphData { nodes: HashMap::new() })
        .add_systems(Startup, (setup_scene, create_graph))
        .add_systems(Update, (rotate_camera, update_edges))
        .run();
}

fn setup_scene(mut commands: Commands) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            1.0,
            -std::f32::consts::FRAC_PI_4,
        )),
        ..default()
    });

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.3, 0.3, 0.4),
        brightness: 200.0,
    });
}

fn create_graph(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut graph_data: ResMut<GraphData>,
) {
    let node_mesh = meshes.add(Sphere::new(0.5));
    let edge_mesh = meshes.add(Cylinder::new(0.05, 1.0));

    // Define nodes
    let nodes = vec![
        ("graph", "Graph Domain", Vec3::new(0.0, 0.0, 0.0), Color::srgb(0.8, 0.2, 0.2)),
        ("agent", "Agent Domain", Vec3::new(4.0, 0.0, 0.0), Color::srgb(0.2, 0.8, 0.2)),
        ("workflow", "Workflow Domain", Vec3::new(-4.0, 0.0, 0.0), Color::srgb(0.2, 0.2, 0.8)),
        ("dialog", "Dialog Domain", Vec3::new(0.0, 4.0, 0.0), Color::srgb(0.8, 0.8, 0.2)),
        ("nix", "Nix Domain", Vec3::new(0.0, -4.0, 0.0), Color::srgb(0.8, 0.2, 0.8)),
        ("bevy", "Bevy Domain", Vec3::new(0.0, 0.0, 4.0), Color::srgb(0.2, 0.8, 0.8)),
    ];

    // Create nodes
    for (id, label, position, color) in nodes {
        let entity = commands.spawn((
            PbrBundle {
                mesh: node_mesh.clone(),
                material: materials.add(StandardMaterial {
                    base_color: color,
                    metallic: 0.3,
                    perceptual_roughness: 0.7,
                    ..default()
                }),
                transform: Transform::from_translation(position),
                ..default()
            },
            GraphNode {
                id: id.to_string(),
                label: label.to_string(),
            },
        )).id();

        // Add text label
        commands.spawn((
            TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                ..default()
            }),
        ));

        graph_data.nodes.insert(id.to_string(), entity);
    }

    // Define edges
    let edges = vec![
        ("graph", "agent"),
        ("graph", "workflow"),
        ("graph", "bevy"),
        ("agent", "dialog"),
        ("workflow", "nix"),
        ("dialog", "bevy"),
    ];

    // Create edges
    let edge_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.5, 0.5, 0.8, 0.8),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    for (source, target) in edges {
        commands.spawn((
            PbrBundle {
                mesh: edge_mesh.clone(),
                material: edge_material.clone(),
                ..default()
            },
            GraphEdge {
                source: source.to_string(),
                target: target.to_string(),
            },
        ));
    }
}

fn rotate_camera(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    for mut transform in query.iter_mut() {
        let angle = time.elapsed_seconds() * 0.5;
        transform.translation.x = angle.cos() * 15.0;
        transform.translation.z = angle.sin() * 15.0;
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

fn update_edges(
    graph_data: Res<GraphData>,
    mut edge_query: Query<(&mut Transform, &GraphEdge)>,
    node_query: Query<&GlobalTransform, With<GraphNode>>,
) {
    for (mut edge_transform, edge) in edge_query.iter_mut() {
        if let (Some(&source_entity), Some(&target_entity)) = 
            (graph_data.nodes.get(&edge.source), graph_data.nodes.get(&edge.target)) {
            
            if let (Ok(source_transform), Ok(target_transform)) = 
                (node_query.get(source_entity), node_query.get(target_entity)) {
                
                let source_pos = source_transform.translation();
                let target_pos = target_transform.translation();
                
                // Position edge between nodes
                let midpoint = (source_pos + target_pos) / 2.0;
                let direction = target_pos - source_pos;
                let distance = direction.length();
                
                edge_transform.translation = midpoint;
                edge_transform.scale = Vec3::new(1.0, distance, 1.0);
                
                // Rotate to align with direction
                if distance > 0.001 {
                    edge_transform.rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());
                }
            }
        }
    }
}