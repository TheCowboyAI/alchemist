//! Demo showing subgraph operations in action

use bevy::prelude::*;
use ia::domain::{
    commands::SubgraphOperationCommand,
    events::SubgraphOperationEvent,
    services::{SubgraphAnalyzer, SubgraphLayoutCalculator},
    value_objects::{
        CollapseStrategy, EdgeId, GraphId, LayoutStrategy, NodeId, Position3D, SubgraphId,
        SubgraphState,
    },
};
use ia::presentation::components::{GraphEdge, GraphNode, SubgraphMembership};
use std::collections::{HashMap, HashSet};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_keyboard_input,
                update_subgraph_visuals,
                animate_transitions,
            ),
        )
        .insert_resource(SubgraphManager::default())
        .run();
}

#[derive(Resource, Default)]
struct SubgraphManager {
    graph_id: GraphId,
    subgraphs: HashMap<SubgraphId, SubgraphInfo>,
    selected_subgraph: Option<SubgraphId>,
}

struct SubgraphInfo {
    state: SubgraphState,
    nodes: HashSet<NodeId>,
    color: Color,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut manager: ResMut<SubgraphManager>,
) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
        ..default()
    });

    // Create graph
    let graph_id = GraphId::new();
    manager.graph_id = graph_id;

    // Create two subgraphs
    let subgraph1 = SubgraphId::new();
    let subgraph2 = SubgraphId::new();

    // Spawn nodes for subgraph 1
    let mut nodes1 = HashSet::new();
    for i in 0..4 {
        let node_id = NodeId::new();
        nodes1.insert(node_id);

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Sphere::new(0.5)),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgb(0.3, 0.7, 1.0),
                    ..default()
                }),
                transform: Transform::from_xyz(-5.0 + i as f32 * 2.0, 0.0, 0.0),
                ..default()
            },
            GraphNode { node_id },
            SubgraphMembership {
                subgraph_id: subgraph1,
            },
        ));
    }

    // Spawn nodes for subgraph 2
    let mut nodes2 = HashSet::new();
    for i in 0..3 {
        let node_id = NodeId::new();
        nodes2.insert(node_id);

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Sphere::new(0.5)),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.7, 0.3),
                    ..default()
                }),
                transform: Transform::from_xyz(2.0 + i as f32 * 2.0, 0.0, 2.0),
                ..default()
            },
            GraphNode { node_id },
            SubgraphMembership {
                subgraph_id: subgraph2,
            },
        ));
    }

    // Store subgraph info
    manager.subgraphs.insert(
        subgraph1,
        SubgraphInfo {
            state: SubgraphState::Expanded,
            nodes: nodes1,
            color: Color::srgb(0.3, 0.7, 1.0),
        },
    );

    manager.subgraphs.insert(
        subgraph2,
        SubgraphInfo {
            state: SubgraphState::Expanded,
            nodes: nodes2,
            color: Color::srgb(1.0, 0.7, 0.3),
        },
    );

    manager.selected_subgraph = Some(subgraph1);

    // Instructions
    commands.spawn(
        TextBundle::from_section(
            "Subgraph Operations Demo\n\
             Press 1/2 to select subgraph\n\
             Press C to collapse selected subgraph\n\
             Press E to expand selected subgraph\n\
             Press M to merge subgraphs",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );
}

fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut manager: ResMut<SubgraphManager>,
    mut events: EventWriter<SubgraphOperationEvent>,
) {
    // Select subgraph
    if keyboard.just_pressed(KeyCode::Digit1) {
        if let Some(id) = manager.subgraphs.keys().nth(0) {
            manager.selected_subgraph = Some(*id);
            println!("Selected subgraph 1");
        }
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        if let Some(id) = manager.subgraphs.keys().nth(1) {
            manager.selected_subgraph = Some(*id);
            println!("Selected subgraph 2");
        }
    }

    // Operations
    if let Some(subgraph_id) = manager.selected_subgraph {
        if keyboard.just_pressed(KeyCode::KeyC) {
            // Collapse
            if let Some(info) = manager.subgraphs.get_mut(&subgraph_id) {
                if matches!(info.state, SubgraphState::Expanded) {
                    info.state = SubgraphState::Collapsed;
                    events.send(SubgraphOperationEvent::SubgraphCollapsed {
                        graph_id: manager.graph_id,
                        subgraph_id,
                        strategy: CollapseStrategy::WeightedCenter,
                        collapsed_position: Position3D {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        timestamp: chrono::Utc::now(),
                    });
                    println!("Collapsed subgraph");
                }
            }
        }

        if keyboard.just_pressed(KeyCode::KeyE) {
            // Expand
            if let Some(info) = manager.subgraphs.get_mut(&subgraph_id) {
                if matches!(info.state, SubgraphState::Collapsed) {
                    info.state = SubgraphState::Expanded;
                    events.send(SubgraphOperationEvent::SubgraphExpanded {
                        graph_id: manager.graph_id,
                        subgraph_id,
                        layout_strategy: LayoutStrategy::ForceDirected,
                        timestamp: chrono::Utc::now(),
                    });
                    println!("Expanded subgraph");
                }
            }
        }
    }

    if keyboard.just_pressed(KeyCode::KeyM) {
        // Merge all subgraphs
        let subgraph_ids: Vec<_> = manager.subgraphs.keys().cloned().collect();
        if subgraph_ids.len() >= 2 {
            events.send(SubgraphOperationEvent::SubgraphsMerged {
                graph_id: manager.graph_id,
                source_subgraphs: subgraph_ids,
                target_subgraph: SubgraphId::new(),
                merge_strategy: ia::domain::value_objects::MergeStrategy::Union,
                timestamp: chrono::Utc::now(),
            });
            println!("Merged subgraphs");
        }
    }
}

fn update_subgraph_visuals(
    manager: Res<SubgraphManager>,
    mut query: Query<(
        &SubgraphMembership,
        &mut Transform,
        &mut Handle<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (membership, mut transform, mut material_handle) in query.iter_mut() {
        if let Some(info) = manager.subgraphs.get(&membership.subgraph_id) {
            // Update visual based on state
            match info.state {
                SubgraphState::Collapsed => {
                    // Move all nodes to center when collapsed
                    transform.translation = transform.translation.lerp(Vec3::ZERO, 0.1);
                    transform.scale = Vec3::splat(0.3);
                }
                SubgraphState::Expanded => {
                    // Keep original positions when expanded
                    transform.scale = Vec3::splat(1.0);
                }
                SubgraphState::Transitioning { progress, .. } => {
                    // Animate transition
                    transform.scale = Vec3::splat(1.0 - progress * 0.7);
                }
            }

            // Update color to show selection
            if Some(membership.subgraph_id) == manager.selected_subgraph {
                if let Some(material) = materials.get_mut(&material_handle) {
                    material.emissive = info.color.into();
                }
            } else {
                if let Some(material) = materials.get_mut(&material_handle) {
                    material.emissive = LinearRgba::BLACK;
                }
            }
        }
    }
}

fn animate_transitions(time: Res<Time>, mut manager: ResMut<SubgraphManager>) {
    let delta = time.delta_seconds();

    for (_, info) in manager.subgraphs.iter_mut() {
        if let SubgraphState::Transitioning { progress, from, to } = &mut info.state {
            *progress += delta * 2.0; // 0.5 second transition

            if *progress >= 1.0 {
                // Transition complete
                info.state = *to.clone();
            }
        }
    }
}
