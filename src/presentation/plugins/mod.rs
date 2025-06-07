//! Bevy Plugins for the Presentation Layer

use bevy::{
    input::mouse::MouseWheel,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use tracing::info;
use crate::application::command_handlers::process_commands;
use crate::application::{CommandEvent, EventNotification};
use crate::domain::commands::{Command, EdgeCommand, NodeCommand};
use crate::domain::events::{DomainEvent, EdgeEvent, GraphEvent, NodeEvent};
use crate::domain::value_objects::{
    EdgeId, EdgeRelationship, GraphId, NodeContent, NodeId, NodeType, Position3D, RelationshipType,
};
use crate::presentation::components::*;
use crate::presentation::events::{ImportResultEvent, ImportRequestEvent};
use crate::presentation::systems::{
    forward_import_requests,
    process_graph_import_requests, forward_import_results,
    update_orbit_camera, orbit_camera_mouse_rotation, orbit_camera_zoom,
    orbit_camera_pan, reset_camera_view, focus_camera_on_selection,
    update_subgraph_boundaries, create_subgraph_from_selection,
    toggle_subgraph_boundary_type,
    ImportPlugin, display_import_help, display_camera_help,
};
use std::time::SystemTime;
use crate::presentation::components::{
    EdgeDrawAnimation, EventRecorder, EventReplayer, ForceLayoutParticipant, ForceLayoutSettings,
    ForceNode, GraphContainer, GraphEdge, GraphNode, NodeAppearanceAnimation, NodeLabel,
    RecordedEvent, ScheduledCommand, OrbitCamera,
};
use crate::presentation::systems::subgraph_visualization::{SubgraphVisualizationPlugin, display_subgraph_help};
use crate::presentation::systems::voronoi_tessellation::VoronoiTessellationPlugin;

/// Main plugin for the graph editor
pub struct GraphEditorPlugin;

impl Plugin for GraphEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register events
            .add_event::<CommandEvent>()
            .add_event::<EventNotification>()
            .add_event::<ImportResultEvent>()
            .add_event::<ImportRequestEvent>()
            // Add resources
            .insert_resource(ForceLayoutSettings::default())
            // Add import plugin
            .add_plugins((
                ImportPlugin,
                SubgraphVisualizationPlugin,
                VoronoiTessellationPlugin,
            ))
            // Add systems
            .add_systems(Startup, (setup_camera, setup_lighting, display_import_help, display_camera_help, display_subgraph_help))
            .add_systems(
                Update,
                (
                    // Command processing chain - MUST run in order
                    (
                        process_commands,
                        forward_import_requests,
                        process_graph_import_requests,
                        forward_import_results,
                    ).chain(),
                    // Other systems can run in parallel
                    execute_scheduled_commands,
                    handle_domain_events,
                    record_events,
                    replay_events,
                    // Animation systems
                    animate_node_appearance,
                    animate_edge_drawing,
                    // Force-directed layout
                    apply_force_layout,
                    // update_node_positions,
                    // Visualization updates
                    update_edge_positions,
                    // Label rendering
                    create_node_labels,
                    // Debug system
                    debug_node_visibility,
                    // Camera controller systems
                    update_orbit_camera,
                    orbit_camera_mouse_rotation,
                    orbit_camera_zoom,
                    orbit_camera_pan,
                    reset_camera_view,
                    focus_camera_on_selection,
                    // Subgraph visualization systems
                    update_subgraph_boundaries,
                    create_subgraph_from_selection,
                    toggle_subgraph_boundary_type,
                ),
            );
    }
}

/// Setup camera
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(100.0, 150.0, 150.0).looking_at(Vec3::new(100.0, 0.0, 0.0), Vec3::Y),
        OrbitCamera::default(),
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
    node_query: Query<(Entity, &GraphNode)>,
) {
    let event_count = events.len();
    if event_count > 0 {
        eprintln!("handle_domain_events: Processing {} events", event_count);
    }

    for event in events.read() {
        info!("Received domain event: {:?}", event.event);
        eprintln!("handle_domain_events: Received {:?}", event.event.event_type());

        match &event.event {
            DomainEvent::Graph(GraphEvent::GraphCreated { id, metadata }) => {
                eprintln!("handle_domain_events: Creating graph visualization");
                create_graph_visualization(&mut commands, *id, &metadata.name);
                // Don't automatically create demo nodes - let imports handle it
                // schedule_demo_graph(&mut commands, *id, &time);

                // Start recording events for this graph
                commands.spawn(EventRecorder {
                    events: Vec::new(),
                    recording_start_time: time.elapsed_secs(),
                });
            }
            DomainEvent::Node(NodeEvent::NodeAdded {
                graph_id,
                node_id,
                metadata,
                position,
            }) => {
                eprintln!("handle_domain_events: Spawning node {:?} at position {:?}", node_id, position);
                spawn_node(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    *graph_id,
                    *node_id,
                    metadata,
                    *position,
                    &time,
                );
            }
            DomainEvent::Edge(EdgeEvent::EdgeConnected {
                graph_id,
                edge_id,
                source,
                target,
                ..
            }) => {
                eprintln!("handle_domain_events: Spawning edge {:?}", edge_id);
                spawn_edge(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    *graph_id,
                    *edge_id,
                    *source,
                    *target,
                    &time,
                    &node_query,
                );
            }
            _ => {
                eprintln!("handle_domain_events: Unhandled event type: {:?}", event.event.event_type());
            }
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
fn schedule_demo_graph(commands: &mut Commands, graph_id: GraphId, time: &Time) {
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

    info!(
        "Scheduled K7 complete graph creation with {} nodes and {} edges over {} seconds",
        num_nodes, total_edges, animation_duration
    );
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
#[allow(clippy::too_many_arguments)]
fn spawn_node(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    graph_id: GraphId,
    node_id: NodeId,
    metadata: &std::collections::HashMap<String, serde_json::Value>,
    position: Position3D,
    time: &Time,
) {
    eprintln!("spawn_node: Starting to spawn node {:?}", node_id);

    // Extract label from metadata
    let label = metadata
        .get("label")
        .and_then(|v| v.as_str())
        .unwrap_or("Unnamed")
        .to_string();

    eprintln!("spawn_node: Node label: {}", label);

    let node_mesh = meshes.add(Sphere::new(0.5));
    let node_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.5, 0.8),
        metallic: 0.3,
        perceptual_roughness: 0.6,
        ..default()
    });

    // Convert position but keep y at 0 for consistency with force layout
    let mut spawn_position: Vec3 = position.into();
    spawn_position.y = 0.0;

    let entity = commands.spawn((
        GraphNode { node_id, graph_id },
        NodeLabel { text: label.clone() },
        Mesh3d(node_mesh),
        MeshMaterial3d(node_material),
        Transform::from_translation(spawn_position).with_scale(Vec3::splat(1.0)), // Start at full size
        NodeAppearanceAnimation {
            start_time: time.elapsed_secs(),
            duration: 0.5,
            start_scale: 1.0,  // Already at full size
            target_scale: 1.0,
        },
        // Temporarily disable force layout for imported nodes
        // ForceNode {
        //     velocity: Vec3::ZERO,
        //     mass: 1.0,
        //     charge: 1.0,
        // },
        // ForceLayoutParticipant,
        Visibility::Visible,  // Explicitly set visibility
    )).id();

    eprintln!("spawn_node: Spawned node {:?} as entity {:?} at position {:?}", node_id, entity, spawn_position);
}

/// Spawn an edge entity
#[allow(clippy::too_many_arguments)]
fn spawn_edge(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    graph_id: GraphId,
    edge_id: EdgeId,
    source_id: NodeId,
    target_id: NodeId,
    time: &Time,
    node_query: &Query<(Entity, &GraphNode)>,
) {
    // Find source and target entities
    let mut source_entity = None;
    let mut target_entity = None;

    for (entity, node) in node_query.iter() {
        if node.node_id == source_id {
            source_entity = Some(entity);
        }
        if node.node_id == target_id {
            target_entity = Some(entity);
        }
    }

    if let (Some(source), Some(target)) = (source_entity, target_entity) {
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
            EdgeDrawAnimation {
                start_time: time.elapsed_secs(),
                duration: 0.3,
                progress: 0.0,
            },
        ));
    }
}

/// Update edge positions to connect nodes
fn update_edge_positions(
    node_query: Query<(Entity, &Transform), With<GraphNode>>,
    mut edge_query: Query<
        (&GraphEdge, &mut Transform, Option<&EdgeDrawAnimation>),
        Without<GraphNode>,
    >,
) {
    for (edge, mut edge_transform, edge_animation) in edge_query.iter_mut() {
        // Find source and target positions
        let mut source_pos = None;
        let mut target_pos = None;

        for (entity, transform) in node_query.iter() {
            if entity == edge.source {
                source_pos = Some(transform.translation);
            }
            if entity == edge.target {
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

                // Apply animation progress to scale
                let scale_y = if let Some(animation) = edge_animation {
                    distance * animation.progress
                } else {
                    distance
                };

                edge_transform.scale = Vec3::new(1.0, scale_y, 1.0);
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
            if matches!(
                &event.event,
                DomainEvent::Edge(EdgeEvent::EdgeConnected { .. })
            ) {
                // Check if this is the last edge (21st edge for K7)
                if recorder
                    .events
                    .iter()
                    .filter(|e| matches!(&e.event, DomainEvent::Edge(_)))
                    .count()
                    == 21
                {
                    info!(
                        "Recording complete! Captured {} events",
                        recorder.events.len()
                    );

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
        let elapsed =
            (time.elapsed_secs() - replayer.replay_start_time) * replayer.speed_multiplier;

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
                if !matches!(
                    &recorded_event.event,
                    DomainEvent::Graph(GraphEvent::GraphCreated { .. })
                ) {
                    events.write(EventNotification {
                        event: recorded_event.event.clone(),
                    });
                    info!(
                        "Replaying event at {:.2}s: {:?}",
                        elapsed,
                        recorded_event.event.event_type()
                    );
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

/// Smooth easing function (ease-out cubic)
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Animate node appearance with smooth scaling
fn animate_node_appearance(
    mut query: Query<(&mut Transform, &NodeAppearanceAnimation)>,
    time: Res<Time>,
) {
    for (mut transform, animation) in query.iter_mut() {
        let elapsed = time.elapsed_secs() - animation.start_time;
        let progress = (elapsed / animation.duration).clamp(0.0, 1.0);

        // Apply easing
        let eased_progress = ease_out_cubic(progress);

        // Interpolate scale
        let scale = animation.start_scale
            + (animation.target_scale - animation.start_scale) * eased_progress;
        transform.scale = Vec3::splat(scale);
    }
}

/// Animate edge drawing with smooth progress
fn animate_edge_drawing(mut query: Query<&mut EdgeDrawAnimation>, time: Res<Time>) {
    for mut animation in query.iter_mut() {
        let elapsed = time.elapsed_secs() - animation.start_time;
        let progress = (elapsed / animation.duration).clamp(0.0, 1.0);
        animation.progress = ease_out_cubic(progress);
    }
}

/// Apply force-directed layout physics
fn apply_force_layout(
    mut nodes: Query<
        (Entity, &Transform, &mut ForceNode, &GraphNode),
        With<ForceLayoutParticipant>,
    >,
    edges: Query<&GraphEdge>,
    settings: Res<ForceLayoutSettings>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();

    // Reset forces
    for (_, _, mut force_node, _) in nodes.iter_mut() {
        force_node.velocity *= settings.damping;
    }

    // Collect node positions for force calculations
    let node_positions: Vec<(Entity, Vec3, f32)> = nodes
        .iter()
        .map(|(entity, transform, force_node, _)| {
            (entity, transform.translation, force_node.charge)
        })
        .collect();

    // Apply repulsion forces between all nodes
    for i in 0..node_positions.len() {
        for j in (i + 1)..node_positions.len() {
            let (entity_i, pos_i, charge_i) = node_positions[i];
            let (entity_j, pos_j, charge_j) = node_positions[j];

            let diff = pos_i - pos_j;
            let distance = diff.length().max(0.1); // Avoid division by zero
            let force_magnitude =
                settings.repulsion_strength * charge_i * charge_j / (distance * distance);
            let force = diff.normalize() * force_magnitude;

            // Apply forces
            if let Ok((_, _, mut force_node_i, _)) = nodes.get_mut(entity_i) {
                let mass = force_node_i.mass;
                force_node_i.velocity += force * delta_time / mass;
            }
            if let Ok((_, _, mut force_node_j, _)) = nodes.get_mut(entity_j) {
                let mass = force_node_j.mass;
                force_node_j.velocity -= force * delta_time / mass;
            }
        }
    }

    // Apply spring forces for edges
    for edge in edges.iter() {
        let mut source_pos = None;
        let mut target_pos = None;
        let mut source_entity = None;
        let mut target_entity = None;

        // Find source and target positions
        for (entity, transform, _, _) in nodes.iter() {
            if entity == edge.source {
                source_pos = Some(transform.translation);
                source_entity = Some(entity);
            }
            if entity == edge.target {
                target_pos = Some(transform.translation);
                target_entity = Some(entity);
            }
        }

        if let (Some(source_pos), Some(target_pos), Some(source_entity), Some(target_entity)) =
            (source_pos, target_pos, source_entity, target_entity)
        {
            let diff = target_pos - source_pos;
            let distance = diff.length();
            let force_magnitude = settings.spring_strength * (distance - settings.spring_length);
            let force = diff.normalize() * force_magnitude;

            // Apply spring forces
            if let Ok((_, _, mut force_node, _)) = nodes.get_mut(source_entity) {
                let mass = force_node.mass;
                force_node.velocity += force * delta_time / mass;
            }
            if let Ok((_, _, mut force_node, _)) = nodes.get_mut(target_entity) {
                let mass = force_node.mass;
                force_node.velocity -= force * delta_time / mass;
            }
        }
    }

    // Apply center force to keep graph centered
    for (_, transform, mut force_node, _) in nodes.iter_mut() {
        let center_force = -transform.translation * settings.center_force;
        let mass = force_node.mass;
        force_node.velocity += center_force * delta_time / mass;
    }
}

/// Update node positions based on forces
fn update_node_positions(
    mut query: Query<(&mut Transform, &ForceNode), With<ForceLayoutParticipant>>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();

    for (mut transform, force_node) in query.iter_mut() {
        // Update position based on velocity
        transform.translation += force_node.velocity * delta_time;

        // Keep nodes on the same Y plane
        transform.translation.y = 0.0;
    }
}

/// System to create text labels for nodes
fn create_node_labels(
    mut commands: Commands,
    nodes: Query<(Entity, &GraphNode, &NodeLabel), Without<Text>>,
) {
    for (entity, _graph_node, label) in nodes.iter() {
        // For now, just log that we would create a label
        // TODO: Implement proper 3D text rendering
        eprintln!("Would create label '{}' for node {:?}", label.text, entity);
    }
}

/// Debug system to monitor node visibility
fn debug_node_visibility(
    nodes: Query<(Entity, &GraphNode, &Transform, Option<&Visibility>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut last_report: Local<f32>,
) {
    // Report every 5 seconds or when V is pressed
    let should_report = keyboard.just_pressed(KeyCode::KeyV) ||
                       (time.elapsed_secs() - *last_report > 5.0);

    if should_report {
        *last_report = time.elapsed_secs();

        let count = nodes.iter().count();
        eprintln!("\n=== NODE VISIBILITY REPORT ===");
        eprintln!("Total nodes: {}", count);

        for (entity, node, transform, visibility) in nodes.iter() {
            eprintln!("Entity {:?}:", entity);
            eprintln!("  Position: ({:.2}, {:.2}, {:.2})",
                transform.translation.x,
                transform.translation.y,
                transform.translation.z
            );
            eprintln!("  Scale: ({:.2}, {:.2}, {:.2})",
                transform.scale.x,
                transform.scale.y,
                transform.scale.z
            );
            eprintln!("  Visibility: {:?}", visibility);
        }
        eprintln!("==============================\n");
    }
}
