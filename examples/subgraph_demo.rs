//! Subgraph Spatial Mapping Demo
//!
//! This example demonstrates the subgraph visualization system with:
//! - Creating multiple subgraphs with their own origins
//! - Moving entire subgraphs as units
//! - Visualizing subgraph boundaries
//! - Hierarchical graph organization

use bevy::prelude::*;
use ia::domain::value_objects::{NodeId, GraphId, Position3D};
use ia::presentation::bevy_systems::{
    create_subgraph_origin, add_node_to_subgraph, move_subgraph,
    circular_layout, SubgraphSpatialMap, layout_subgraph_nodes,
    SubgraphOrigin,
};
use ia::presentation::plugins::{GraphEditorPlugin, SubgraphPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GraphEditorPlugin)
        .add_plugins(SubgraphPlugin)
        .add_systems(Startup, (setup_demo, position_subgraphs).chain())
        .add_systems(Update, animate_subgraphs)
        .run();
}

/// Demo state for tracking subgraphs
#[derive(Resource)]
struct DemoState {
    subgraphs: Vec<GraphId>,
    time: f32,
}

fn setup_demo(
    mut commands: Commands,
    mut spatial_map: ResMut<SubgraphSpatialMap>,
) {
    // Create camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(30.0, 40.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add lighting
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
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

    // Create three subgraphs
    let mut subgraphs = Vec::new();

    for i in 0..3 {
        // Create subgraph origin
        let origin_entity = create_subgraph_origin(&mut commands, &mut spatial_map);
        let graph_id = spatial_map.origins.iter()
            .find(|(_, &entity)| entity == origin_entity)
            .map(|(id, _)| *id)
            .unwrap();

        subgraphs.push(graph_id);

        // Position subgraphs in a triangle
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / 3.0;
        let radius = 20.0;
        let position = Vec3::new(
            radius * angle.cos(),
            0.0,
            radius * angle.sin(),
        );

        // We'll move the subgraph after all entities are created
        spatial_map.positions.insert(graph_id, position);

        // Add nodes to each subgraph
        let node_count = 5 + i * 2; // Different sizes for variety
        for j in 0..node_count {
            let node_id = NodeId::new();
            let label = format!("S{}-N{}", i + 1, j + 1);

            // Use circular layout for nodes
            let layout = circular_layout(5.0, node_count);
            let relative_pos = layout(j);

            add_node_to_subgraph(
                &mut commands,
                &spatial_map,
                graph_id,
                node_id,
                relative_pos,
                label,
            );
        }
    }

    // Store demo state
    commands.insert_resource(DemoState {
        subgraphs,
        time: 0.0,
    });

    // Add help text
    commands.spawn((
        Text::new("Subgraph Spatial Mapping Demo\n\nSubgraphs move as units\nPress SPACE to pause animation"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        },
    ));
}

// Add a startup system to position the subgraphs after creation
fn position_subgraphs(
    mut spatial_map: ResMut<SubgraphSpatialMap>,
    mut transforms: Query<&mut Transform, With<SubgraphOrigin>>,
) {
    for (&graph_id, &position) in spatial_map.positions.iter() {
        if let Some(&origin_entity) = spatial_map.origins.get(&graph_id) {
            if let Ok(mut transform) = transforms.get_mut(origin_entity) {
                transform.translation = position;
            }
        }
    }
}

/// Animate subgraphs moving in different patterns
fn animate_subgraphs(
    mut demo_state: ResMut<DemoState>,
    mut spatial_map: ResMut<SubgraphSpatialMap>,
    mut transforms: Query<&mut Transform, With<SubgraphOrigin>>,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Pause on spacebar
    if keyboard.pressed(KeyCode::Space) {
        return;
    }

    demo_state.time += time.delta_secs();

    // Animate each subgraph with different patterns
    for (i, &graph_id) in demo_state.subgraphs.iter().enumerate() {
        let pattern = match i {
            0 => {
                // Circular motion
                let angle = demo_state.time * 0.5;
                let radius = 20.0;
                Vec3::new(
                    radius * angle.cos(),
                    5.0 * (demo_state.time * 2.0).sin(),
                    radius * angle.sin(),
                )
            }
            1 => {
                // Figure-8 motion
                let t = demo_state.time * 0.3;
                let x = 25.0 * (t * 2.0).sin();
                let z = 12.5 * (t * 4.0).sin();
                let y = 3.0 * (t * 3.0).cos();
                Vec3::new(x, y, z)
            }
            2 => {
                // Lissajous curve
                let t = demo_state.time * 0.4;
                let x = 20.0 * (t * 3.0).sin();
                let z = 20.0 * (t * 2.0).sin();
                let y = 10.0 * (t).sin();
                Vec3::new(x, y, z)
            }
            _ => Vec3::ZERO,
        };

        move_subgraph(&mut spatial_map, &mut transforms, graph_id, pattern);
    }
}
