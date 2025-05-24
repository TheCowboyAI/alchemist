//! Example demonstrating graph algorithms, change detection, and UI
//!
//! Run with: nix run .#examples -- graph_algorithms_demo

use alchemist::graph_core::{
    CreateEdgeEvent, CreateNodeEvent, DomainEdgeType, DomainNodeType, GraphAlgorithms, GraphData,
    GraphInspectorState, GraphPlugin,
};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use std::collections::HashMap;
use uuid::Uuid;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(GraphPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (create_demo_graph, run_algorithm_demos, keyboard_shortcuts),
        )
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));
}

/// Create a demo graph with interesting topology
fn create_demo_graph(
    mut commands: Commands,
    mut create_node_events: EventWriter<CreateNodeEvent>,
    mut create_edge_events: EventWriter<CreateEdgeEvent>,
    mut created: Local<bool>,
) {
    if *created {
        return;
    }
    *created = true;

    // Create a small workflow graph
    let nodes = vec![
        ("Start", Vec3::new(-4.0, 0.0, 0.0), DomainNodeType::Bounded),
        (
            "Process A",
            Vec3::new(-2.0, 2.0, 0.0),
            DomainNodeType::Process,
        ),
        (
            "Process B",
            Vec3::new(-2.0, -2.0, 0.0),
            DomainNodeType::Process,
        ),
        (
            "Decision",
            Vec3::new(0.0, 0.0, 0.0),
            DomainNodeType::Gateway,
        ),
        (
            "Action 1",
            Vec3::new(2.0, 2.0, 0.0),
            DomainNodeType::Command,
        ),
        (
            "Action 2",
            Vec3::new(2.0, -2.0, 0.0),
            DomainNodeType::Command,
        ),
        ("End", Vec3::new(4.0, 0.0, 0.0), DomainNodeType::Bounded),
    ];

    let mut node_ids = Vec::new();

    // Create nodes
    for (name, position, node_type) in nodes {
        let id = Uuid::new_v4();
        node_ids.push(id);

        create_node_events.send(CreateNodeEvent {
            id,
            name: name.to_string(),
            domain_type: node_type,
            position,
            labels: vec!["demo".to_string()],
            properties: HashMap::new(),
        });
    }

    // Create edges to form an interesting workflow
    let edges = vec![
        (0, 1, "start->a"),     // Start -> Process A
        (0, 2, "start->b"),     // Start -> Process B
        (1, 3, "a->decision"),  // Process A -> Decision
        (2, 3, "b->decision"),  // Process B -> Decision
        (3, 4, "decision->1"),  // Decision -> Action 1
        (3, 5, "decision->2"),  // Decision -> Action 2
        (4, 6, "action1->end"), // Action 1 -> End
        (5, 6, "action2->end"), // Action 2 -> End
    ];

    // Create edges
    for (source_idx, target_idx, label) in edges {
        if let (Some(&source_id), Some(&target_id)) =
            (node_ids.get(source_idx), node_ids.get(target_idx))
        {
            create_edge_events.send(CreateEdgeEvent {
                id: Uuid::new_v4(),
                source_id,
                target_id,
                edge_type: DomainEdgeType::Flow,
                labels: vec![label.to_string()],
                properties: HashMap::new(),
            });
        }
    }

    info!(
        "Demo graph created with {} nodes and {} edges",
        node_ids.len(),
        edges.len()
    );
}

/// Run algorithm demonstrations periodically
fn run_algorithm_demos(
    graph_data: Res<GraphData>,
    time: Res<Time>,
    mut last_run: Local<f32>,
    mut demo_index: Local<usize>,
) {
    // Run a demo every 5 seconds
    if time.elapsed_secs() - *last_run < 5.0 {
        return;
    }
    *last_run = time.elapsed_secs();

    if graph_data.node_count() < 2 {
        return;
    }

    // Get all nodes
    let nodes: Vec<_> = graph_data
        .nodes()
        .map(|(_, data)| (data.id, data.name.clone()))
        .collect();

    match *demo_index % 4 {
        0 => {
            // Demo: Find shortest path
            if nodes.len() >= 2 {
                let (start_id, start_name) = &nodes[0];
                let (end_id, end_name) = &nodes[nodes.len() - 1];

                match GraphAlgorithms::shortest_path(&graph_data, *start_id, *end_id) {
                    Some((path, cost)) => {
                        info!(
                            "📍 Shortest path from '{}' to '{}': {} nodes, cost: {}",
                            start_name,
                            end_name,
                            path.len(),
                            cost
                        );
                    }
                    None => {
                        info!("❌ No path found from '{}' to '{}'", start_name, end_name);
                    }
                }
            }
        }
        1 => {
            // Demo: Check connectivity
            let components = GraphAlgorithms::find_components(&graph_data);
            info!("🔗 Graph has {} connected components", components.len());

            if components.len() == 1 {
                info!("✅ Graph is fully connected!");
            } else {
                info!(
                    "⚠️ Graph is disconnected into {} components",
                    components.len()
                );
            }
        }
        2 => {
            // Demo: Cycle detection
            if GraphAlgorithms::has_cycles(&graph_data) {
                info!("🔄 Graph contains cycles");
            } else {
                info!("➡️ Graph is acyclic (DAG)");

                // Try topological sort
                match GraphAlgorithms::topological_sort(&graph_data) {
                    Ok(order) => {
                        let names: Vec<_> = order
                            .iter()
                            .filter_map(|id| nodes.iter().find(|(nid, _)| nid == id))
                            .map(|(_, name)| name.as_str())
                            .collect();
                        info!("📋 Topological order: {:?}", names);
                    }
                    Err(e) => {
                        warn!("Failed to get topological order: {}", e);
                    }
                }
            }
        }
        3 => {
            // Demo: Centrality analysis
            let centrality = GraphAlgorithms::degree_centrality(&graph_data);

            // Find most connected node
            let most_connected = centrality
                .iter()
                .max_by_key(|(_, (_, _, total))| total)
                .and_then(|(id, (in_deg, out_deg, total))| {
                    nodes
                        .iter()
                        .find(|(nid, _)| nid == id)
                        .map(|(_, name)| (name, in_deg, out_deg, total))
                });

            if let Some((name, in_deg, out_deg, total)) = most_connected {
                info!(
                    "⭐ Most connected node: '{}' (in: {}, out: {}, total: {})",
                    name, in_deg, out_deg, total
                );
            }
        }
        _ => {}
    }

    *demo_index += 1;
}

/// Keyboard shortcuts for testing
fn keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut inspector_state: ResMut<GraphInspectorState>,
    graph_data: Res<GraphData>,
) {
    // Toggle statistics window
    if keys.just_pressed(KeyCode::KeyS) {
        inspector_state.show_stats = !inspector_state.show_stats;
        info!(
            "Statistics window: {}",
            if inspector_state.show_stats {
                "ON"
            } else {
                "OFF"
            }
        );
    }

    // Toggle algorithms window
    if keys.just_pressed(KeyCode::KeyA) {
        inspector_state.show_algorithms = !inspector_state.show_algorithms;
        info!(
            "Algorithms window: {}",
            if inspector_state.show_algorithms {
                "ON"
            } else {
                "OFF"
            }
        );
    }

    // Select random nodes for pathfinding
    if keys.just_pressed(KeyCode::KeyR) {
        let nodes: Vec<_> = graph_data
            .nodes()
            .map(|(_, data)| (data.id, data.name.clone()))
            .collect();

        if nodes.len() >= 2 {
            inspector_state.pathfind_source = Some(nodes[0].0);
            inspector_state.pathfind_target = Some(nodes[nodes.len() - 1].0);
            info!(
                "Selected '{}' as source and '{}' as target",
                nodes[0].1,
                nodes[nodes.len() - 1].1
            );
        }
    }

    // Clear selection
    if keys.just_pressed(KeyCode::Escape) {
        inspector_state.selected_node = None;
        inspector_state.selected_edge = None;
        inspector_state.pathfind_source = None;
        inspector_state.pathfind_target = None;
        info!("Cleared all selections");
    }
}
