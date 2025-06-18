//! Git Repository Graph Visualization Demo
//!
//! This demo demonstrates the complete CIM pipeline:
//! 1. Analyze the current Git repository using cim-domain-git
//! 2. Convert Git data to a ContextGraph representation
//! 3. Visualize the graph using Bevy with cim-domain-bevy
//!
//! ## Mermaid Graph - Demo Architecture
//!
//! ```mermaid
//! graph TD
//!     A[Git Repository] -->|analyze| B[Git Domain Events]
//!     B -->|convert| C[ContextGraph]
//!     C -->|visualize| D[Bevy ECS Components]
//!     D -->|render| E[3D Visualization]
//!     
//!     F[User Input] -->|camera controls| D
//!     F -->|node interaction| D
//!     
//!     subgraph "Git Domain"
//!         B1[CommitAnalyzed]
//!         B2[BranchCreated]
//!         B3[RepositoryAnalyzed]
//!     end
//!     
//!     subgraph "Graph Domain"
//!         C1[NodeAdded]
//!         C2[EdgeAdded]
//!         C3[GraphCreated]
//!     end
//!     
//!     subgraph "Bevy Domain"
//!         D1[NodeVisual]
//!         D2[EdgeVisual]
//!         D3[CameraController]
//!     end
//! ```

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use cim_domain_git::{
    events::GitDomainEvent, handlers::RepositoryCommandHandler, value_objects::CommitHash,
};
use std::collections::HashMap;

fn main() {
    println!("🚀 Starting Git Repository Graph Visualization Demo");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CIM - Git Repository Graph Visualization".into(),
                resolution: (1200.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        // Note: Using basic setup since cim-domain-bevy plugins aren't fully implemented yet
        .add_systems(Startup, setup_demo)
        .add_systems(
            Update,
            (
                camera_controller,
                update_info_text,
                handle_node_interactions,
            ),
        )
        .insert_resource(GitGraphData::default())
        .run();
}

#[derive(Resource, Default)]
struct GitGraphData {
    commit_nodes: HashMap<CommitHash, Entity>,
    repository_info: Option<String>,
    node_count: usize,
    edge_count: usize,
}

#[derive(Component)]
struct CommitNode {
    commit_hash: CommitHash,
    message: String,
    author: String,
}

#[derive(Component)]
struct CommitEdge {
    parent: CommitHash,
    child: CommitHash,
}

#[derive(Component)]
struct CameraController {
    sensitivity: f32,
    speed: f32,
}

#[derive(Component)]
struct InfoText;

async fn analyze_git_repository()
-> Result<(String, Vec<GitDomainEvent>), Box<dyn std::error::Error>> {
    let handler = RepositoryCommandHandler::new();

    match handler.analyze_current_repository().await {
        Ok((repo_id, events)) => {
            let repo_info = format!("Repository ID: {:?}\nEvents: {}", repo_id, events.len());
            Ok((repo_info, events))
        }
        Err(e) => Err(format!("Failed to analyze repository: {}", e).into()),
    }
}

fn setup_demo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut git_data: ResMut<GitGraphData>,
) {
    println!("🔍 Analyzing Git repository...");

    // Setup camera with controller
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraController {
            sensitivity: 100.0,  // Increase sensitivity for better mouse look
            speed: 15.0,         // Increase speed for better movement
        },
    ));

    // Setup lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 4.0)),
    ));

    // Add ambient light (fixed for Bevy 0.15+)
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 100.0,
        affects_lightmapped_meshes: true,
    });

    // Create info text UI
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("Analyzing Git repository..."),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                InfoText,
            ));
        });

    // Analyze repository in blocking manner for demo
    // Note: In a real application, this should be async
    let rt = tokio::runtime::Runtime::new().unwrap();

    match rt.block_on(analyze_git_repository()) {
        Ok((repo_info, events)) => {
            println!("✅ Successfully analyzed repository!");
            println!("📊 Repository info: {}", repo_info);

            git_data.repository_info = Some(repo_info);

            // Convert git events to visual graph
            create_visual_graph(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut git_data,
                events,
            );
        }
        Err(e) => {
            eprintln!("❌ Failed to analyze repository: {}", e);

            // Create a fallback demo graph
            create_fallback_graph(&mut commands, &mut meshes, &mut materials, &mut git_data);
        }
    }
}

fn create_visual_graph(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    git_data: &mut ResMut<GitGraphData>,
    events: Vec<GitDomainEvent>,
) {
    let mut commits = Vec::new();
    let mut commit_positions = HashMap::new();

    // Extract commit information from events
    for event in events {
        match event {
            GitDomainEvent::CommitAnalyzed(commit_event) => {
                commits.push(commit_event);
            }
            _ => {} // Handle other event types if needed
        }
    }

    println!("📈 Creating visualization for {} commits", commits.len());

    // Create node materials
    let commit_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.8, 0.3),
        metallic: 0.1,
        perceptual_roughness: 0.4,
        ..default()
    });

    let edge_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.6, 0.9),
        metallic: 0.0,
        perceptual_roughness: 0.8,
        ..default()
    });

    // Create sphere mesh for commits
    let sphere_mesh = meshes.add(Sphere::new(0.5));
    let cylinder_mesh = meshes.add(Cylinder::new(0.05, 1.0));

    // Position commits in a spiral layout for better visualization
    let mut y_offset = 0.0;
    let radius = 8.0;

    for (index, commit) in commits.iter().enumerate() {
        let angle = (index as f32) * 0.8; // Spacing between commits
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        y_offset += 0.5; // Vertical progression

        let position = Vec3::new(x, y_offset, z);
        commit_positions.insert(commit.commit_hash.clone(), position);

        // Create commit node visual
        let entity = commands
            .spawn((
                Mesh3d(sphere_mesh.clone()),
                MeshMaterial3d(commit_material.clone()),
                Transform::from_translation(position),
                CommitNode {
                    commit_hash: commit.commit_hash.clone(),
                    message: commit.message.clone(),
                    author: commit.author.name.clone(),
                },
            ))
            .id();

        git_data
            .commit_nodes
            .insert(commit.commit_hash.clone(), entity);
        git_data.node_count += 1;
    }

    // Create edges between commits and their parents
    for commit in &commits {
        if let Some(current_pos) = commit_positions.get(&commit.commit_hash) {
            for parent_hash in &commit.parents {
                if let Some(parent_pos) = commit_positions.get(parent_hash) {
                    // Calculate edge position and rotation
                    let edge_pos = (*current_pos + *parent_pos) / 2.0;
                    let direction = (*parent_pos - *current_pos).normalize();
                    let length = current_pos.distance(*parent_pos);

                    // Create transform for the edge
                    let up = Vec3::Y;
                    let rotation = if direction.cross(up).length() < 0.01 {
                        Quat::IDENTITY
                    } else {
                        Quat::from_rotation_arc(up, direction)
                    };

                    commands.spawn((
                        Mesh3d(cylinder_mesh.clone()),
                        MeshMaterial3d(edge_material.clone()),
                        Transform {
                            translation: edge_pos,
                            rotation,
                            scale: Vec3::new(1.0, length, 1.0),
                        },
                        CommitEdge {
                            parent: parent_hash.clone(),
                            child: commit.commit_hash.clone(),
                        },
                    ));

                    git_data.edge_count += 1;
                }
            }
        }
    }

    println!(
        "✨ Created visualization with {} nodes and {} edges",
        git_data.node_count, git_data.edge_count
    );
}

fn create_fallback_graph(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    git_data: &mut ResMut<GitGraphData>,
) {
    println!("🔄 Creating fallback demo graph");

    git_data.repository_info = Some("Fallback Demo Graph\n(Not a git repository)".to_string());

    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.3, 0.3),
        ..default()
    });

    let sphere_mesh = meshes.add(Sphere::new(0.8));

    // Create a simple demo graph
    let positions = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(3.0, 2.0, 0.0),
        Vec3::new(-3.0, 2.0, 0.0),
        Vec3::new(0.0, 4.0, 0.0),
    ];

    for pos in positions.iter() {
        commands.spawn((
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(*pos),
        ));
        git_data.node_count += 1;
    }
}

fn camera_controller(
    mut camera_query: Query<(&mut Transform, &CameraController), With<Camera>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, controller)) = camera_query.single_mut() {
        let mut rotation_delta = Vec2::ZERO;
        let mut translation_delta = Vec3::ZERO;

        // Mouse look (right mouse button) - only process motion when right button is held
        let is_right_mouse_pressed = mouse_button_input.pressed(MouseButton::Right);
        
        // Debug: Log right mouse button state changes
        if mouse_button_input.just_pressed(MouseButton::Right) {
            println!("🖱️ Right mouse button pressed - mouse look enabled");
        }
        if mouse_button_input.just_released(MouseButton::Right) {
            println!("🖱️ Right mouse button released - mouse look disabled");
        }
        
        // Read all mouse motion events but only use them if right mouse is pressed
        for event in mouse_motion_events.read() {
            if is_right_mouse_pressed {
                // Use more reasonable sensitivity - remove time.delta_secs() as it's too sensitive
                rotation_delta += event.delta * controller.sensitivity * 0.001;
            }
        }

        // Keyboard movement
        if keyboard_input.pressed(KeyCode::KeyW) {
            translation_delta -=
                transform.forward().as_vec3() * controller.speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            translation_delta +=
                transform.forward().as_vec3() * controller.speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            translation_delta -= transform.right().as_vec3() * controller.speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            translation_delta += transform.right().as_vec3() * controller.speed * time.delta_secs();
        }

        // Mouse wheel zoom - increase sensitivity significantly and remove time dependency
        for event in mouse_wheel_events.read() {
            let zoom_speed = controller.speed * 2.0; // Increase zoom sensitivity
            translation_delta += transform.forward().as_vec3() * event.y * zoom_speed;
            println!("🖱️ Mouse wheel: {} (zoom_speed: {})", event.y, zoom_speed);
        }

        // Apply rotation only if we actually have rotation input
        if rotation_delta.length_squared() > 0.0 {
            let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            let new_yaw = yaw - rotation_delta.x;
            let new_pitch = (pitch - rotation_delta.y).clamp(-1.5, 1.5);
            transform.rotation = Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, 0.0);
            println!("🎯 Camera rotation: yaw={:.2}, pitch={:.2}", new_yaw, new_pitch);
        }

        // Apply translation
        if translation_delta.length_squared() > 0.0 {
            transform.translation += translation_delta;
            println!("📍 Camera position: {:?}", transform.translation);
        }
    }
}

fn update_info_text(mut text_query: Query<&mut Text, With<InfoText>>, git_data: Res<GitGraphData>) {
    if let Ok(mut text) = text_query.single_mut() {
        if let Some(repo_info) = &git_data.repository_info {
            text.0 = format!(
                "Git Repository Graph Visualization\n\n{}\n\nNodes: {}\nEdges: {}\n\nControls:\n- WASD: Move camera\n- Right click + drag: Look around\n- Mouse wheel: Zoom in/out\n- Left click: Select node (debug)\n\nTip: Hold right mouse button while moving mouse to look around!",
                repo_info, git_data.node_count, git_data.edge_count
            );
        }
    }
}

fn handle_node_interactions(
    _commit_query: Query<(&CommitNode, &Transform)>,
    _camera_query: Query<&Transform, (With<Camera>, Without<CommitNode>)>,
    windows: Query<&Window>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Ok(window) = windows.single() {
            if let Some(_cursor_position) = window.cursor_position() {
                // TODO: Implement ray casting for node selection
                // This would require more complex intersection testing
                println!("🖱️  Mouse clicked - node selection not yet implemented");
            }
        }
    }
}
