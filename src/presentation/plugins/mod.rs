//! Bevy Plugins for the Presentation Layer

use crate::application::command_handlers::process_commands;
use crate::application::{CommandEvent, EventNotification};
use crate::presentation::components::*;
use crate::domain::events::{DomainEvent, GraphEvent, NodeEvent, EdgeEvent};
use crate::domain::value_objects::{NodeId, EdgeId, GraphId, Position3D, NodeContent, NodeType, EdgeRelationship, RelationshipType};
use crate::domain::commands::{Command, NodeCommand, EdgeCommand};
use bevy::prelude::*;
use tracing::info;

/// Main plugin for the graph editor
pub struct GraphEditorPlugin;

impl Plugin for GraphEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register events
            .add_event::<CommandEvent>()
            .add_event::<EventNotification>()
            // Add systems
            .add_systems(Startup, (setup_camera, setup_lighting))
            .add_systems(
                Update,
                (
                    // Command processing
                    process_commands,
                    execute_scheduled_commands,
                    // Event handling
                    handle_domain_events,
                    record_events,
                    replay_events,
                    // Visualization updates
                    update_edge_positions,
                ),
            );
    }
}

/// Setup basic 3D camera
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Setup basic lighting
fn setup_lighting(mut commands: Commands) {
    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
        affects_lightmapped_meshes: false,
    });

    // Directional light
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Handle domain events and update the world
fn handle_domain_events(
    mut commands: Commands,
    mut events: EventReader<EventNotification>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for event in events.read() {
        info!("Received domain event: {:?}", event.event);

        match &event.event {
            DomainEvent::Graph(GraphEvent::GraphCreated { id, metadata }) => {
                create_graph_visualization(&mut commands, *id, &metadata.name);
                // Schedule demo nodes creation
                schedule_demo_graph(&mut commands, *id, &time);

                // Start recording events for this graph
                commands.spawn(EventRecorder {
                    events: Vec::new(),
                    recording_start_time: time.elapsed_secs(),
                });
            }
            DomainEvent::Node(NodeEvent::NodeAdded { graph_id, node_id, content, position }) => {
                spawn_node(&mut commands, &mut meshes, &mut materials, *graph_id, *node_id, content, *position);
            }
            DomainEvent::Edge(EdgeEvent::EdgeConnected { graph_id, edge_id, source, target, .. }) => {
                spawn_edge(&mut commands, &mut meshes, &mut materials, *graph_id, *edge_id, *source, *target);
            }
            _ => {}
        }
    }
}

/// Create the graph container
fn create_graph_visualization(commands: &mut Commands, graph_id: GraphId, name: &str) {
    commands.spawn((
        GraphContainer {
            graph_id,
            name: name.to_string(),
        },
        Transform::default(),
        Visibility::default(),
    ));

    info!("Created graph visualization for: {}", name);
}

/// Schedule demo graph creation
fn schedule_demo_graph(
    commands: &mut Commands,
    graph_id: GraphId,
    time: &Time,
) {
    // Create K7 complete graph - 7 nodes arranged in a circle
    let num_nodes = 7;
    let radius = 4.0;
    let animation_duration = 15.0;
    let nodes_duration = 5.0;
    let edges_duration = 10.0;
    let current_time = time.elapsed_secs();

    // Calculate positions for nodes arranged in a circle
    let mut node_ids = Vec::new();
    for i in 0..num_nodes {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / (num_nodes as f32);
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        let position = Position3D { x, y: 0.0, z };

        // Schedule node creation
        let node_id = NodeId::new();
        node_ids.push(node_id);

        let delay = (i as f32 / num_nodes as f32) * nodes_duration;
        let execute_at = current_time + delay;

        commands.spawn(ScheduledCommand {
            execute_at,
            command: Command::Node(NodeCommand::AddNode {
                graph_id,
                node_id,
                content: NodeContent {
                    label: format!("Node {}", i + 1),
                    node_type: NodeType::Custom("demo".to_string()),
                    properties: std::collections::HashMap::new(),
                },
                position,
            }),
        });
    }

    // Schedule edge creation
    let mut edge_index = 0;
    let total_edges = (num_nodes * (num_nodes - 1)) / 2;

    for i in 0..num_nodes {
        for j in (i + 1)..num_nodes {
            let edge_id = EdgeId::new();
            let delay = nodes_duration + (edge_index as f32 / total_edges as f32) * edges_duration;
            let execute_at = current_time + delay;

            commands.spawn(ScheduledCommand {
                execute_at,
                command: Command::Edge(EdgeCommand::ConnectEdge {
                    graph_id,
                    edge_id,
                    source: node_ids[i],
                    target: node_ids[j],
                    relationship: EdgeRelationship {
                        relationship_type: RelationshipType::Custom("demo".to_string()),
                        properties: std::collections::HashMap::new(),
                        bidirectional: false,
                    },
                }),
            });

            edge_index += 1;
        }
    }

    info!("Scheduled K7 complete graph creation with {} nodes and {} edges over {} seconds",
          num_nodes, total_edges, animation_duration);
}

/// System to execute scheduled commands when their time comes
fn execute_scheduled_commands(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &ScheduledCommand)>,
    mut command_events: EventWriter<CommandEvent>,
) {
    let current_time = time.elapsed_secs();

    for (entity, scheduled) in query.iter() {
        if current_time >= scheduled.execute_at {
            // Send the command
            command_events.write(CommandEvent {
                command: scheduled.command.clone(),
            });

            // Remove the scheduled command entity
            commands.entity(entity).despawn();
        }
    }
}

/// Spawn a node entity
fn spawn_node(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    graph_id: GraphId,
    node_id: NodeId,
    content: &NodeContent,
    position: Position3D,
) {
    let node_mesh = meshes.add(Sphere::new(0.5));
    let node_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.5, 0.8),
        metallic: 0.3,
        perceptual_roughness: 0.6,
        ..default()
    });

    commands.spawn((
        GraphNode {
            node_id,
            graph_id,
        },
        NodeLabel {
            text: content.label.clone(),
        },
        Mesh3d(node_mesh),
        MeshMaterial3d(node_material),
        Transform::from_translation(position.into()),
    ));
}

/// Spawn an edge entity
fn spawn_edge(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    graph_id: GraphId,
    edge_id: EdgeId,
    source: NodeId,
    target: NodeId,
) {
    let edge_mesh = meshes.add(Cylinder::new(0.05, 1.0));
    let edge_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.6, 0.6),
        metallic: 0.1,
        perceptual_roughness: 0.8,
        ..default()
    });

    commands.spawn((
        GraphEdge {
            edge_id,
            graph_id,
            source,
            target,
        },
        Mesh3d(edge_mesh),
        MeshMaterial3d(edge_material),
        Transform::default(),
    ));
}

/// Update edge positions to connect nodes
fn update_edge_positions(
    node_query: Query<(&GraphNode, &Transform), Without<GraphEdge>>,
    mut edge_query: Query<(&GraphEdge, &mut Transform), Without<GraphNode>>,
) {
    for (edge, mut edge_transform) in edge_query.iter_mut() {
        // Find source and target positions
        let mut source_pos = None;
        let mut target_pos = None;

        for (node, transform) in node_query.iter() {
            if node.node_id == edge.source {
                source_pos = Some(transform.translation);
            }
            if node.node_id == edge.target {
                target_pos = Some(transform.translation);
            }
        }

        if let (Some(source), Some(target)) = (source_pos, target_pos) {
            // Calculate edge position and rotation
            let midpoint = (source + target) / 2.0;
            let direction = target - source;
            let distance = direction.length();

            if distance > 0.0 {
                let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());

                edge_transform.translation = midpoint;
                edge_transform.rotation = rotation;
                edge_transform.scale = Vec3::new(1.0, distance, 1.0);
            }
        }
    }
}

/// System to record events as they occur
fn record_events(
    mut commands: Commands,
    mut recorders: Query<(Entity, &mut EventRecorder)>,
    mut events: EventReader<EventNotification>,
    time: Res<Time>,
) {
    if let Ok((recorder_entity, mut recorder)) = recorders.single_mut() {
        for event in events.read() {
            let timestamp = time.elapsed_secs() - recorder.recording_start_time;
            recorder.events.push(RecordedEvent {
                event: event.event.clone(),
                timestamp,
            });

            // Check if we've finished recording (all nodes and edges created)
            if matches!(&event.event, DomainEvent::Edge(EdgeEvent::EdgeConnected { .. })) {
                // Check if this is the last edge (21st edge for K7)
                if recorder.events.iter().filter(|e| matches!(&e.event, DomainEvent::Edge(_))).count() == 21 {
                    info!("Recording complete! Captured {} events", recorder.events.len());

                    // Start replay after a short delay
                    let replay_delay = 3.0; // Wait 3 seconds before replay
                    let replay_start = time.elapsed_secs() + replay_delay;

                    commands.spawn(EventReplayer {
                        events: recorder.events.clone(),
                        replay_start_time: replay_start,
                        current_index: 0,
                        speed_multiplier: 2.0, // Replay at 2x speed
                    });

                    info!("Replay will start in {} seconds at 2x speed", replay_delay);

                    // Remove the recorder
                    commands.entity(recorder_entity).despawn();
                }
            }
        }
    }
}

/// System to replay recorded events
fn replay_events(
    mut commands: Commands,
    mut replayers: Query<(Entity, &mut EventReplayer)>,
    mut events: EventWriter<EventNotification>,
    time: Res<Time>,
    nodes: Query<Entity, With<GraphNode>>,
    edges: Query<Entity, With<GraphEdge>>,
) {
    for (entity, mut replayer) in replayers.iter_mut() {
        let elapsed = (time.elapsed_secs() - replayer.replay_start_time) * replayer.speed_multiplier;

        // Clear existing graph entities when replay starts
        if replayer.current_index == 0 && elapsed >= 0.0 {
            // Despawn all nodes
            for node_entity in nodes.iter() {
                commands.entity(node_entity).despawn();
            }
            // Despawn all edges
            for edge_entity in edges.iter() {
                commands.entity(edge_entity).despawn();
            }
            info!("Cleared graph for replay");
        }

        // Replay events whose time has come
        while replayer.current_index < replayer.events.len() {
            let recorded_event = &replayer.events[replayer.current_index];

            if elapsed >= recorded_event.timestamp {
                // Skip graph creation events during replay
                if !matches!(&recorded_event.event, DomainEvent::Graph(GraphEvent::GraphCreated { .. })) {
                    events.write(EventNotification {
                        event: recorded_event.event.clone(),
                    });
                    info!("Replaying event at {:.2}s: {:?}", elapsed, recorded_event.event.event_type());
                }
                replayer.current_index += 1;
            } else {
                break;
            }
        }

        // Remove replayer when done
        if replayer.current_index >= replayer.events.len() {
            info!("Replay complete!");
            commands.entity(entity).despawn();
        }
    }
}
