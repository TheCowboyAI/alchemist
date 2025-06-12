//! Workflow Visualization Demo
//!
//! This demo shows how to visualize a business workflow using cim-viz-bevy.
//! It demonstrates a document approval workflow with different node types
//! and animated state transitions.
//!
//! Run with: cargo run --example workflow_demo --features bevy/dynamic_linking

use bevy::prelude::*;
use cim_viz_bevy::*;
use cim_contextgraph::{NodeId, EdgeId, ContextGraphId as GraphId};
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CIM Workflow Visualization".to_string(),
                resolution: (1400.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CimVizPlugin::default())
        .insert_resource(WorkflowDemo::default())
        .insert_resource(NodeEntityMap::default())
        .add_systems(Startup, (setup_scene, create_workflow))
        .add_systems(Update, (
            handle_node_creation,
            handle_edge_creation,
            render_edges,
            animate_workflow,
            handle_interaction,
            update_ui,
        ).chain())
        .run();
}

#[derive(Resource, Default)]
struct WorkflowDemo {
    graph_id: Option<GraphId>,
    workflow_state: WorkflowState,
    current_step: usize,
    node_states: HashMap<NodeId, NodeState>,
}

#[derive(Default, Clone, Copy, PartialEq)]
enum WorkflowState {
    #[default]
    NotStarted,
    Running,
    Completed,
    Failed,
}

#[derive(Clone, Copy, PartialEq)]
enum NodeState {
    Pending,
    Active,
    Completed,
    Failed,
}

#[derive(Component)]
struct WorkflowNode {
    node_type: WorkflowNodeType,
    state: NodeState,
}

#[derive(Clone, Copy, PartialEq)]
enum WorkflowNodeType {
    Start,
    Task,
    Decision,
    End,
}

#[derive(Resource, Default)]
struct NodeEntityMap {
    map: HashMap<NodeId, Entity>,
}

#[derive(Component)]
struct EdgeLine;

#[derive(Component)]
struct UIText;

fn setup_scene(
    mut commands: Commands,
) {
    // Camera - orthographic for 2D-style workflow view
    commands.spawn((
        Camera3d {
            projection: Projection::Orthographic(OrthographicProjection {
                scaling_mode: bevy::render::camera::ScalingMode::FixedVertical {
                    viewport_height: 20.0,
                },
                ..OrthographicProjection::default_3d()
            }),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        GraphCamera,
    ));

    // Lighting
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });

    // UI
    commands.spawn((
        Text::new("Document Approval Workflow\n\nClick 'Start' to begin\nClick active nodes to progress"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
        UIText,
    ));
}

fn create_workflow(
    mut demo: ResMut<WorkflowDemo>,
    bridge: Res<CategoricalBridge>,
) {
    let graph_id = GraphId::new();
    demo.graph_id = Some(graph_id);

    // Define workflow nodes
    let nodes = vec![
        ("Start", WorkflowNodeType::Start, Vec3::new(-8.0, 0.0, 0.0)),
        ("Submit Document", WorkflowNodeType::Task, Vec3::new(-4.0, 0.0, 0.0)),
        ("Review", WorkflowNodeType::Task, Vec3::new(0.0, 0.0, 0.0)),
        ("Approval Decision", WorkflowNodeType::Decision, Vec3::new(4.0, 0.0, 0.0)),
        ("Revise", WorkflowNodeType::Task, Vec3::new(0.0, -4.0, 0.0)),
        ("Approved", WorkflowNodeType::End, Vec3::new(8.0, 0.0, 0.0)),
        ("Rejected", WorkflowNodeType::End, Vec3::new(8.0, -4.0, 0.0)),
    ];

    let sender = bridge.domain_sender();
    let mut node_ids = Vec::new();

    // Create nodes
    for (name, node_type, position) in nodes {
        let node_id = NodeId::new();
        node_ids.push(node_id);

        demo.node_states.insert(node_id, NodeState::Pending);

        let event = DomainEvent::NodeAdded {
            graph_id,
            node_id,
            position: Some(position),
            metadata: serde_json::json!({
                "name": name,
                "type": format!("{:?}", node_type),
            }),
        };

        let _ = sender.send(event);
    }

    // Create edges
    let edges = vec![
        (0, 1, "Start Process"),
        (1, 2, "Submit"),
        (2, 3, "Review Complete"),
        (3, 5, "Approve"),
        (3, 4, "Needs Revision"),
        (3, 6, "Reject"),
        (4, 2, "Resubmit"),
    ];

    for (from, to, label) in edges {
        let event = DomainEvent::EdgeAdded {
            graph_id,
            edge_id: EdgeId::new(),
            source_id: node_ids[from],
            target_id: node_ids[to],
            metadata: serde_json::json!({
                "label": label,
            }),
        };

        let _ = sender.send(event);
    }
}

fn handle_node_creation(
    mut commands: Commands,
    mut create_events: EventReader<CreateNodeVisual>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut node_map: ResMut<NodeEntityMap>,
) {
    for event in create_events.read() {
        // Parse node type from metadata
        let node_type = match event.position.x {
            x if x < -6.0 => WorkflowNodeType::Start,
            x if x > 6.0 => WorkflowNodeType::End,
            x if x > 2.0 && x < 6.0 => WorkflowNodeType::Decision,
            _ => WorkflowNodeType::Task,
        };

        let (mesh, color) = match node_type {
            WorkflowNodeType::Start => (
                meshes.add(Circle::new(0.8).mesh()),
                Color::srgb(0.2, 0.8, 0.2),
            ),
            WorkflowNodeType::End => (
                meshes.add(Circle::new(0.8).mesh()),
                Color::srgb(0.8, 0.2, 0.2),
            ),
            WorkflowNodeType::Decision => (
                meshes.add(RegularPolygon::new(1.0, 4).mesh()),
                Color::srgb(0.8, 0.8, 0.2),
            ),
            WorkflowNodeType::Task => (
                meshes.add(Rectangle::new(2.0, 1.5).mesh()),
                Color::srgb(0.2, 0.5, 0.8),
            ),
        };

        let entity = commands.spawn((
            NodeVisualBundle::new(event.node_id, event.graph_id, event.position),
            Mesh3d(mesh),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color * 0.5, // Dimmed initially
                unlit: true,
                ..default()
            })),
            WorkflowNode {
                node_type,
                state: NodeState::Pending,
            },
        )).id();

        node_map.map.insert(event.node_id, entity);
    }
}

fn handle_edge_creation(
    mut commands: Commands,
    mut create_events: EventReader<CreateEdgeVisual>,
    node_map: Res<NodeEntityMap>,
) {
    for event in create_events.read() {
        if let (Some(&source_entity), Some(&target_entity)) = (
            node_map.map.get(&event.source_id),
            node_map.map.get(&event.target_id),
        ) {
            commands.spawn((
                EdgeVisualBundle::new(event.edge_id, event.graph_id, source_entity, target_entity),
                EdgeLine,
            ));
        }
    }
}

fn render_edges(
    mut gizmos: Gizmos,
    edges: Query<&EdgeVisual, With<EdgeLine>>,
    nodes: Query<&Transform, With<NodeVisual>>,
) {
    for edge in edges.iter() {
        if let (Ok(source_transform), Ok(target_transform)) = (
            nodes.get(edge.source_entity),
            nodes.get(edge.target_entity),
        ) {
            let start = source_transform.translation;
            let end = target_transform.translation;

            // Draw arrow
            gizmos.arrow(start, end, Color::srgb(0.6, 0.6, 0.6));
        }
    }
}

fn animate_workflow(
    mut demo: ResMut<WorkflowDemo>,
    mut nodes: Query<(&NodeVisual, &WorkflowNode, &mut Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    if demo.workflow_state != WorkflowState::Running {
        return;
    }

    // Simple animation: pulse the active node
    for (node_visual, workflow_node, material_handle) in nodes.iter_mut() {
        if let Some(&state) = demo.node_states.get(&node_visual.node_id) {
            if state == NodeState::Active {
                if let Some(material) = materials.get_mut(material_handle) {
                    let pulse = (time.elapsed_secs() * 3.0).sin() * 0.5 + 0.5;
                    let base_color = match workflow_node.node_type {
                        WorkflowNodeType::Start => Color::srgb(0.2, 0.8, 0.2),
                        WorkflowNodeType::End => Color::srgb(0.8, 0.2, 0.2),
                        WorkflowNodeType::Decision => Color::srgb(0.8, 0.8, 0.2),
                        WorkflowNodeType::Task => Color::srgb(0.2, 0.5, 0.8),
                    };
                    material.base_color = base_color * (0.5 + pulse * 0.5);
                }
            }
        }
    }
}

fn handle_interaction(
    mut demo: ResMut<WorkflowDemo>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
    mut nodes: Query<(Entity, &NodeVisual, &Transform, &WorkflowNode, &mut Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.get_single() else { return };
    let Ok((camera, camera_transform)) = camera.get_single() else { return };

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            // Check for node clicks
            for (entity, node_visual, transform, workflow_node, material_handle) in nodes.iter_mut() {
                let distance = ray.closest_point_to_point(transform.translation)
                    .distance(transform.translation);

                if distance < 1.0 {
                    if let Some(&state) = demo.node_states.get(&node_visual.node_id) {
                        // Handle click based on node state
                        match (workflow_node.node_type, state) {
                            (WorkflowNodeType::Start, NodeState::Pending) => {
                                // Start the workflow
                                demo.workflow_state = WorkflowState::Running;
                                demo.node_states.insert(node_visual.node_id, NodeState::Completed);

                                // Activate next node
                                if let Some((next_id, _)) = demo.node_states.iter()
                                    .find(|(_, &s)| s == NodeState::Pending)
                                    .map(|(id, s)| (*id, *s))
                                {
                                    demo.node_states.insert(next_id, NodeState::Active);
                                }
                            }
                            (_, NodeState::Active) => {
                                // Progress the workflow
                                demo.node_states.insert(node_visual.node_id, NodeState::Completed);

                                // Simple progression logic
                                demo.current_step += 1;

                                // Update material
                                if let Some(material) = materials.get_mut(material_handle) {
                                    material.base_color = Color::srgb(0.2, 0.8, 0.2);
                                }
                            }
                            _ => {}
                        }
                    }
                    break;
                }
            }
        }
    }
}

fn update_ui(
    mut text_query: Query<&mut Text, With<UIText>>,
    demo: Res<WorkflowDemo>,
) {
    if demo.is_changed() {
        if let Ok(mut text) = text_query.get_single_mut() {
            let status = match demo.workflow_state {
                WorkflowState::NotStarted => "Not Started",
                WorkflowState::Running => "In Progress",
                WorkflowState::Completed => "Completed",
                WorkflowState::Failed => "Failed",
            };

            text.0 = format!(
                "Document Approval Workflow\n\
                Status: {}\n\
                Step: {}\n\n\
                Click nodes to progress through the workflow",
                status,
                demo.current_step
            );
        }
    }
}
