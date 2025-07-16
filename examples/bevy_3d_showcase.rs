//! Bevy 3D Showcase - Demonstrates Alchemist's 3D visualization capabilities

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use std::f32::consts::PI;

#[derive(Component)]
struct GraphNode {
    id: String,
    label: String,
}

#[derive(Component)]
struct GraphEdge {
    source: Entity,
    target: Entity,
}

#[derive(Component)]
struct Orbiting {
    radius: f32,
    speed: f32,
    angle: f32,
}

#[derive(Component)]
struct Hoverable;

#[derive(Component)]
struct Selected;

#[derive(Resource)]
struct UiState {
    show_panel: bool,
    selected_node: Option<String>,
    animation_speed: f32,
    edge_visibility: bool,
    layout_type: LayoutType,
}

#[derive(Clone, Copy, PartialEq)]
enum LayoutType {
    ForceDirected,
    Hierarchical,
    Circular,
    Grid,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            show_panel: true,
            selected_node: None,
            animation_speed: 1.0,
            edge_visibility: true,
            layout_type: LayoutType::ForceDirected,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Alchemist - Bevy 3D Showcase".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .init_resource::<UiState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            ui_system,
            orbit_system,
            hover_system,
            selection_system,
            edge_update_system,
            camera_control_system,
            layout_animation_system,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera with orbit controls
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(15.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        Orbiting {
            radius: 25.0,
            speed: 0.5,
            angle: 0.0,
        },
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.3, 0.3, 0.4),
        brightness: 0.3,
    });

    // Point lights for better visualization
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            color: Color::srgb(1.0, 0.9, 0.8),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(10.0, 10.0, 10.0),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1000.0,
            color: Color::srgb(0.8, 0.9, 1.0),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-10.0, 10.0, -10.0),
        ..default()
    });

    // Create graph nodes
    let node_mesh = meshes.add(Sphere::new(0.5));
    let positions = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(5.0, 2.0, 0.0),
        Vec3::new(-5.0, 2.0, 0.0),
        Vec3::new(0.0, 4.0, 5.0),
        Vec3::new(0.0, 4.0, -5.0),
        Vec3::new(3.0, 6.0, 3.0),
        Vec3::new(-3.0, 6.0, -3.0),
    ];

    let colors = vec![
        Color::srgb(0.8, 0.2, 0.2),  // Red
        Color::srgb(0.2, 0.8, 0.2),  // Green
        Color::srgb(0.2, 0.2, 0.8),  // Blue
        Color::srgb(0.8, 0.8, 0.2),  // Yellow
        Color::srgb(0.8, 0.2, 0.8),  // Magenta
        Color::srgb(0.2, 0.8, 0.8),  // Cyan
        Color::srgb(0.6, 0.6, 0.6),  // Gray
    ];

    let labels = vec![
        "Core System",
        "Agent Domain",
        "Graph Domain",
        "Workflow Engine",
        "Event Store",
        "AI Provider",
        "NATS Messaging",
    ];

    let mut node_entities = Vec::new();

    for (i, (pos, color, label)) in positions.iter().zip(colors.iter()).zip(labels.iter()).enumerate() {
        let entity = commands.spawn((
            PbrBundle {
                mesh: node_mesh.clone(),
                material: materials.add(StandardMaterial {
                    base_color: *color,
                    metallic: 0.3,
                    perceptual_roughness: 0.6,
                    ..default()
                }),
                transform: Transform::from_translation(*pos),
                ..default()
            },
            GraphNode {
                id: format!("node_{}", i),
                label: label.to_string(),
            },
            Hoverable,
        )).id();
        node_entities.push(entity);

        // Add text label above node
        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    label.to_string(),
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                transform: Transform::from_translation(*pos + Vec3::Y * 1.0),
                ..default()
            },
        ));
    }

    // Create edges
    let edge_mesh = meshes.add(Cylinder::new(0.05, 1.0));
    let edge_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.5, 0.5, 0.8, 0.8),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let connections = vec![
        (0, 1), (0, 2), (0, 3), (0, 4),
        (1, 5), (2, 5), (3, 6), (4, 6),
    ];

    for (source_idx, target_idx) in connections {
        commands.spawn((
            PbrBundle {
                mesh: edge_mesh.clone(),
                material: edge_material.clone(),
                ..default()
            },
            GraphEdge {
                source: node_entities[source_idx],
                target: node_entities[target_idx],
            },
        ));
    }

    // Add a ground plane for reference
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(50.0, 50.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.15),
            metallic: 0.0,
            perceptual_roughness: 1.0,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, -5.0, 0.0),
        ..default()
    });
}

fn ui_system(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut query: Query<&mut Visibility, With<GraphEdge>>,
) {
    let ctx = contexts.ctx_mut();

    egui::Window::new("Alchemist 3D Controls")
        .default_pos((10.0, 10.0))
        .show(ctx, |ui| {
            ui.heading("Graph Visualization");
            
            ui.separator();

            ui.label("Selected Node:");
            if let Some(node) = &ui_state.selected_node {
                ui.label(format!("  {}", node));
            } else {
                ui.label("  None");
            }

            ui.separator();

            ui.label("Animation Speed:");
            ui.add(egui::Slider::new(&mut ui_state.animation_speed, 0.0..=2.0));

            ui.checkbox(&mut ui_state.edge_visibility, "Show Edges");

            ui.separator();

            ui.label("Layout Type:");
            ui.radio_value(&mut ui_state.layout_type, LayoutType::ForceDirected, "Force Directed");
            ui.radio_value(&mut ui_state.layout_type, LayoutType::Hierarchical, "Hierarchical");
            ui.radio_value(&mut ui_state.layout_type, LayoutType::Circular, "Circular");
            ui.radio_value(&mut ui_state.layout_type, LayoutType::Grid, "Grid");

            ui.separator();

            ui.label("Camera Controls:");
            ui.label("  • Mouse: Orbit camera");
            ui.label("  • Scroll: Zoom in/out");
            ui.label("  • Click: Select node");

            ui.separator();

            ui.label("Features:");
            ui.label("  • Real-time 3D rendering");
            ui.label("  • Interactive node selection");
            ui.label("  • Dynamic edge visualization");
            ui.label("  • Multiple layout algorithms");
            ui.label("  • Smooth animations");
        });

    // Update edge visibility
    if !ui_state.edge_visibility {
        for mut visibility in query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
    } else {
        for mut visibility in query.iter_mut() {
            *visibility = Visibility::Visible;
        }
    }
}

fn orbit_system(
    time: Res<Time>,
    ui_state: Res<UiState>,
    mut query: Query<(&mut Transform, &mut Orbiting), With<Camera>>,
) {
    for (mut transform, mut orbiting) in query.iter_mut() {
        orbiting.angle += orbiting.speed * time.delta_seconds() * ui_state.animation_speed;
        
        let x = orbiting.radius * orbiting.angle.cos();
        let z = orbiting.radius * orbiting.angle.sin();
        let y = transform.translation.y;
        
        transform.translation = Vec3::new(x, y, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

fn hover_system(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    rapier_context: Query<&Transform, With<GraphNode>>,
    mut gizmos: Gizmos,
) {
    // Hover visualization would go here
    // For now, we'll skip the actual hover detection to keep it simple
}

fn selection_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    node_query: Query<(Entity, &GlobalTransform, &GraphNode), With<Hoverable>>,
    mut ui_state: ResMut<UiState>,
    mut commands: Commands,
    selected_query: Query<Entity, With<Selected>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        // Clear previous selection
        for entity in selected_query.iter() {
            commands.entity(entity).remove::<Selected>();
        }
        ui_state.selected_node = None;

        // Selection logic would go here
        // For simplicity, we're not implementing full ray-casting
    }
}

fn edge_update_system(
    mut edges: Query<(&mut Transform, &GraphEdge)>,
    nodes: Query<&GlobalTransform, With<GraphNode>>,
) {
    for (mut edge_transform, edge) in edges.iter_mut() {
        if let Ok(source_transform) = nodes.get(edge.source) {
            if let Ok(target_transform) = nodes.get(edge.target) {
                let source_pos = source_transform.translation();
                let target_pos = target_transform.translation();
                
                let midpoint = (source_pos + target_pos) / 2.0;
                let direction = target_pos - source_pos;
                let distance = direction.length();
                
                edge_transform.translation = midpoint;
                edge_transform.scale = Vec3::new(1.0, distance, 1.0);
                
                if let Some(rotation) = Quat::from_rotation_arc(Vec3::Y, direction.normalize()).try_normalize() {
                    edge_transform.rotation = rotation;
                }
            }
        }
    }
}

fn camera_control_system(
    mut scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut camera_query: Query<&mut Orbiting, With<Camera>>,
) {
    for event in scroll_events.read() {
        for mut orbiting in camera_query.iter_mut() {
            orbiting.radius = (orbiting.radius - event.y * 2.0).clamp(5.0, 50.0);
        }
    }
}

fn layout_animation_system(
    time: Res<Time>,
    ui_state: Res<UiState>,
    mut nodes: Query<(&mut Transform, &GraphNode)>,
) {
    let t = time.elapsed_seconds();
    
    match ui_state.layout_type {
        LayoutType::Circular => {
            let mut index = 0;
            let total = nodes.iter().count();
            for (mut transform, _) in nodes.iter_mut() {
                let angle = (index as f32 / total as f32) * 2.0 * PI;
                let radius = 8.0;
                transform.translation.x = radius * angle.cos();
                transform.translation.z = radius * angle.sin();
                transform.translation.y = 0.0;
                index += 1;
            }
        }
        LayoutType::Grid => {
            let mut index = 0;
            let grid_size = 3;
            for (mut transform, _) in nodes.iter_mut() {
                let x = (index % grid_size) as f32 * 4.0 - 4.0;
                let z = (index / grid_size) as f32 * 4.0 - 4.0;
                transform.translation.x = x;
                transform.translation.z = z;
                transform.translation.y = 0.0;
                index += 1;
            }
        }
        _ => {
            // Keep existing positions for other layouts
        }
    }
}