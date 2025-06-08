//! Subgraph Spatial Mapping Demo
//!
//! This example demonstrates the subgraph visualization system with:
//! - Creating multiple subgraphs with their own origins
//! - Moving entire subgraphs as units
//! - Visualizing subgraph boundaries

use bevy::prelude::*;
use bevy::pbr::StandardMaterial;
use bevy::render::mesh::Mesh;
use ia::domain::value_objects::{NodeId, GraphId};
use ia::presentation::bevy_systems::{
    SubgraphOrigin, SubgraphMember, SubgraphSpatialMap,
    visualize_subgraph_boundaries,
};
use ia::presentation::components::{GraphNode, NodeLabel};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<SubgraphSpatialMap>()
        .add_systems(Startup, setup_demo)
        .add_systems(Update, (
            animate_subgraphs,
            visualize_subgraph_boundaries,
            handle_input,
        ))
        .run();
}

fn setup_demo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spatial_map: ResMut<SubgraphSpatialMap>,
) {
    // Create camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(20.0, 30.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add lighting
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
        affects_lightmapped_meshes: false,
    });

    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Create three subgraphs at different positions
    let colors = [
        Color::srgb(0.8, 0.3, 0.3), // Red
        Color::srgb(0.3, 0.8, 0.3), // Green
        Color::srgb(0.3, 0.3, 0.8), // Blue
    ];

    let positions = [
        Vec3::new(-15.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(15.0, 0.0, 0.0),
    ];

    for i in 0..3 {
        let graph_id = GraphId::new();
        let base_position = positions[i];

        // Create subgraph origin (invisible parent entity)
        let origin_entity = commands.spawn((
            SubgraphOrigin {
                graph_id,
                base_position,
            },
            Transform::from_translation(base_position),
            GlobalTransform::default(),
            Visibility::Hidden,
        )).id();

        // Update spatial map
        spatial_map.origins.insert(graph_id, origin_entity);
        spatial_map.positions.insert(graph_id, base_position);

        // Create nodes in a circular pattern
        let node_count = 5;
        let radius = 5.0;

        for j in 0..node_count {
            let angle = (j as f32) * 2.0 * std::f32::consts::PI / (node_count as f32);
            let relative_position = Vec3::new(
                radius * angle.cos(),
                0.0,
                radius * angle.sin(),
            );

            let node_entity = commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.5))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: colors[i],
                    metallic: 0.8,
                    perceptual_roughness: 0.2,
                    ..default()
                })),
                Transform::from_translation(relative_position),
                GlobalTransform::default(),
                GraphNode {
                    node_id: NodeId::new(),
                    graph_id,
                },
                NodeLabel {
                    text: format!("Node {}-{}", i + 1, j + 1),
                },
                SubgraphMember { graph_id },
            )).id();

            // Make it a child of the origin
            commands.entity(origin_entity).add_child(node_entity);
        }

        // Add a label for the subgraph using 2D text in world space
        commands.spawn((
            Text2d::new(format!("Subgraph {}", i + 1)),
            Transform::from_translation(base_position + Vec3::new(0.0, 8.0, 0.0))
                .with_scale(Vec3::splat(0.1)),
        ));
    }

    // Add instructions
    commands.spawn((
        Text::new("Press 1/2/3 to select subgraph\nArrow keys to move selected subgraph\nSpace to animate"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

#[derive(Resource)]
struct SelectedSubgraph(usize);

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected: Local<usize>,
    mut spatial_map: ResMut<SubgraphSpatialMap>,
    mut origins: Query<&mut Transform, With<SubgraphOrigin>>,
) {
    // Select subgraph
    if keyboard.just_pressed(KeyCode::Digit1) {
        *selected = 0;
        println!("Selected subgraph 1");
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        *selected = 1;
        println!("Selected subgraph 2");
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        *selected = 2;
        println!("Selected subgraph 3");
    }

    // Get the selected graph ID
    let graph_ids: Vec<GraphId> = spatial_map.origins.keys().cloned().collect();
    if *selected >= graph_ids.len() {
        return;
    }
    let graph_id = graph_ids[*selected];

    // Move selected subgraph
    let move_speed = 0.5;
    let mut movement = Vec3::ZERO;

    if keyboard.pressed(KeyCode::ArrowLeft) {
        movement.x -= move_speed;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        movement.x += move_speed;
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        movement.z -= move_speed;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        movement.z += move_speed;
    }

    if movement != Vec3::ZERO {
        if let Some(origin_entity) = spatial_map.origins.get(&graph_id) {
            if let Ok(mut transform) = origins.get_mut(*origin_entity) {
                transform.translation += movement;
                spatial_map.positions.insert(graph_id, transform.translation);
            }
        }
    }
}

fn animate_subgraphs(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut origins: Query<&mut Transform, With<SubgraphOrigin>>,
) {
    if !keyboard.pressed(KeyCode::Space) {
        return;
    }

    let elapsed = time.elapsed_secs();

    // Animate subgraphs in different patterns
    for (i, mut transform) in origins.iter_mut().enumerate() {
        match i {
            0 => {
                // Circular motion
                let radius = 15.0;
                let angle = elapsed * 0.5;
                transform.translation.x = -15.0 + radius * angle.cos() * 0.3;
                transform.translation.z = radius * angle.sin() * 0.3;
            }
            1 => {
                // Up and down motion
                transform.translation.y = 5.0 * (elapsed * 0.7).sin();
            }
            2 => {
                // Figure-8 motion
                let t = elapsed * 0.4;
                transform.translation.x = 15.0 + 5.0 * (t * 2.0).sin();
                transform.translation.z = 5.0 * t.sin();
            }
            _ => {}
        }
    }
}
