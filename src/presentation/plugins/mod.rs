//! Bevy Plugins for the Presentation Layer

use bevy::{
    prelude::*,
    render::view::{ViewVisibility, InheritedVisibility},
};
use tracing::{info, warn};
use crate::application::{CommandEvent, EventNotification};
use crate::domain::commands::{Command, EdgeCommand, GraphCommand, NodeCommand};
use crate::domain::events::{DomainEvent, EdgeEvent, GraphEvent, NodeEvent, SubgraphEvent};
use crate::domain::value_objects::{
    EdgeId, EdgeRelationship, GraphId, NodeContent, NodeId, NodeType, Position3D, RelationshipType,
    SubgraphId,
};
use std::collections::HashMap;
// use crate::presentation::components::*; // Unused - specific imports below
use crate::presentation::events::{ImportResultEvent, ImportRequestEvent};
use crate::presentation::systems::{
    ImportPlugin, update_subgraph_boundaries,
};
// use std::time::SystemTime; // Unused
use crate::presentation::components::{
    EdgeDrawAnimation, EventRecorder, EventReplayer, ForceLayoutParticipant, ForceLayoutSettings,
    ForceNode, GraphContainer, GraphEdge, GraphNode, NodeAppearanceAnimation, NodeLabel,
    RecordedEvent, ScheduledCommand, OrbitCamera, PendingEdge, NodeLabelEntity,
    SubgraphOrigins, SubgraphInfo, SubgraphMember, VoronoiSettings, SubgraphOrigin,
};
use crate::presentation::systems::subgraph_spatial_map::SubgraphSpatialMap;
use std::collections::HashSet;
use uuid;
use crate::presentation::systems::{
    conceptual_visualization::ConceptualVisualizationPlugin,
    subgraph_visualization::SubgraphVisualizationPlugin,
    voronoi_tessellation::VoronoiTessellationPlugin,
    workflow_visualization::WorkflowVisualizationPlugin,
};

pub mod subgraph_plugin;
pub mod graph_editor;
pub mod graph_editor_plugin;
pub mod conceptual_graph_plugin;
pub mod graph_plugin;
pub mod workflow_designer_plugin;

pub use subgraph_plugin::SubgraphPlugin;
pub use conceptual_graph_plugin::ConceptualGraphPlugin;
pub use graph_plugin::GraphPlugin;
pub use graph_editor_plugin::GraphEditorPlugin as ConceptualGraphEditorPlugin;
pub use workflow_designer_plugin::WorkflowDesignerPlugin;

/// Main plugin for the graph editor
pub struct GraphEditorPlugin;

impl Plugin for GraphEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<ForceLayoutSettings>()
            .init_resource::<SubgraphOrigins>()
            .init_resource::<VoronoiSettings>()
            .init_resource::<SubgraphSpatialMap>()

            // Events
            .add_event::<CommandEvent>()
            .add_event::<EventNotification>()
            .add_event::<ImportResultEvent>()
            .add_event::<ImportRequestEvent>()
            .add_event::<DomainEvent>()

            // Systems
            .add_systems(Startup, (setup_graph_editor, setup_camera, setup_lighting))
            .add_systems(Update, (
                handle_domain_events,
                handle_import_requests,
                handle_import_results,
                update_force_layout,
                update_edge_positions,
                update_subgraph_boundaries,
                process_pending_edges,
                animate_node_appearance,
                animate_edge_drawing,
                create_node_labels,
                update_label_positions,
                apply_force_layout,
                update_node_positions,
                debug_node_visibility,
                record_events,
                replay_events,
                execute_scheduled_commands,
            ).chain())

            // Plugins
            .add_plugins((
                ImportPlugin,
                ConceptualVisualizationPlugin,
                SubgraphVisualizationPlugin,
                VoronoiTessellationPlugin,
                WorkflowVisualizationPlugin,
                crate::presentation::systems::subgraph_collapse_expand::SubgraphCollapseExpandPlugin,
                crate::presentation::systems::subgraph_drag_drop::SubgraphDragDropPlugin,
                crate::presentation::systems::subgraph_merge_split::SubgraphMergeSplitPlugin,
            ));
    }
}

/// Setup camera
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(50.0, 100.0, 150.0).looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera {
            focus: Vec3::ZERO,
            distance: 200.0,
            ..default()
        },
    ));
}

/// Setup basic lighting
fn setup_lighting(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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

    // Ground plane for reference
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(200.0, 200.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.05, 0.05),
            metallic: 0.0,
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(0.0, -2.0, 0.0),
    ));

    // Grid lines for better spatial reference
    let grid_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.2),
        emissive: Color::srgb(0.1, 0.1, 0.1).into(),
        ..default()
    });

    // Create grid lines
    let grid_size = 100.0;
    let grid_spacing = 10.0;
    let line_thickness = 0.1;

    for i in -10..=10 {
        let offset = i as f32 * grid_spacing;

        // X-axis lines
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(grid_size * 2.0, line_thickness, line_thickness))),
            MeshMaterial3d(grid_material.clone()),
            Transform::from_xyz(0.0, -1.9, offset),
        ));

        // Z-axis lines
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(line_thickness, line_thickness, grid_size * 2.0))),
            MeshMaterial3d(grid_material.clone()),
            Transform::from_xyz(offset, -1.9, 0.0),
        ));
    }
}

/// Handle domain events and update the world
fn handle_domain_events(
    mut commands: Commands,
    mut events: EventReader<EventNotification>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    node_query: Query<(Entity, &GraphNode)>,
    edge_query: Query<(Entity, &GraphEdge)>,
    mut subgraph_origins: ResMut<SubgraphOrigins>,
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
                    position,
                    &time,
                    metadata,
                    &mut subgraph_origins,
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
            DomainEvent::Node(NodeEvent::NodeRemoved { node_id, .. }) => {
                eprintln!("handle_domain_events: Removing node {:?}", node_id);
                // Find and despawn the node entity
                for (entity, node) in node_query.iter() {
                    if node.node_id == *node_id {
                        commands.entity(entity).despawn_recursive();
                        break;
                    }
                }
            }
            DomainEvent::Edge(EdgeEvent::EdgeRemoved { edge_id, .. }) => {
                eprintln!("handle_domain_events: Removing edge {:?}", edge_id);
                // Find and despawn the edge entity
                for (entity, edge) in edge_query.iter() {
                    if edge.edge_id == *edge_id {
                        commands.entity(entity).despawn_recursive();
                        break;
                    }
                }
            }
            DomainEvent::Subgraph(SubgraphEvent::SubgraphCreated { graph_id, subgraph_id, name, base_position, metadata }) => {
                eprintln!("handle_domain_events: Creating subgraph {:?} at position {:?}", subgraph_id, base_position);

                // Create subgraph origin entity
                let origin_entity = commands.spawn((
                    SubgraphOrigin {
                        subgraph_id: *subgraph_id,
                        subgraph_name: name.clone(),
                        node_count: 0,
                    },
                    crate::presentation::systems::subgraph_drag_drop::DropZone {
                        subgraph_id: *subgraph_id,
                        accepts: crate::presentation::systems::subgraph_drag_drop::DropAcceptance::All,
                        highlight_on_hover: true,
                    },
                    Transform::from_translation(Vec3::new(base_position.x, base_position.y, base_position.z)),
                    GlobalTransform::default(),
                    Visibility::Hidden,
                    Name::new(format!("SubgraphOrigin_{}", name)),
                )).id();

                // Update spatial map will be done in a separate system
                // For now, just spawn the origin entity
            }
            DomainEvent::Subgraph(SubgraphEvent::SubgraphRemoved { graph_id, subgraph_id }) => {
                eprintln!("handle_domain_events: Removing subgraph {:?}", subgraph_id);

                // Find and remove subgraph origin - this needs to be done in a separate system
                // that has access to the SubgraphSpatialMap resource
            }
            DomainEvent::Subgraph(SubgraphEvent::SubgraphMoved { graph_id, subgraph_id, old_position, new_position }) => {
                eprintln!("handle_domain_events: Moving subgraph {:?} from {:?} to {:?}", subgraph_id, old_position, new_position);

                // Update subgraph origin position - this needs to be done in a separate system
            }
            DomainEvent::Subgraph(SubgraphEvent::NodeAddedToSubgraph { graph_id, subgraph_id, node_id, relative_position }) => {
                eprintln!("handle_domain_events: Adding node {:?} to subgraph {:?}", node_id, subgraph_id);

                // Find the node entity and make it a child of the subgraph origin
                // This needs to be done in a separate system that has access to SubgraphSpatialMap
                for (entity, node) in node_query.iter() {
                    if node.node_id == *node_id {
                        // Convert SubgraphId to usize using hash
                        let subgraph_id_hash = {
                            use std::hash::{Hash, Hasher};
                            let mut hasher = std::collections::hash_map::DefaultHasher::new();
                            subgraph_id.hash(&mut hasher);
                            hasher.finish() as usize
                        };

                        commands.entity(entity).insert(SubgraphMember {
                            subgraph_ids: {
                                let mut set = HashSet::new();
                                set.insert(*subgraph_id);
                                set
                            },
                            relative_position: relative_position.clone(),
                        });
                        break;
                    }
                }
            }
            DomainEvent::Subgraph(SubgraphEvent::NodeRemovedFromSubgraph { graph_id, subgraph_id, node_id }) => {
                eprintln!("handle_domain_events: Removing node {:?} from subgraph {:?}", node_id, subgraph_id);

                // Find the node entity and remove it from the subgraph
                for (entity, node) in node_query.iter() {
                    if node.node_id == *node_id {
                        commands.entity(entity).remove::<SubgraphMember>();
                        commands.entity(entity).remove_parent();
                        break;
                    }
                }
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
    position: &Position3D,
    time: &Res<Time>,
    metadata: &HashMap<String, serde_json::Value>,
    subgraph_origins: &mut ResMut<SubgraphOrigins>,
) {
    eprintln!("spawn_node: Spawning node {:?} at position {:?}", node_id, position);

    // Extract label from metadata
    let label = metadata.get("label")
        .and_then(|v| v.as_str())
        .unwrap_or("Node")
        .to_string();

    // Check if this node belongs to a subgraph
    let subgraph_id = metadata.get("subgraph_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .map(SubgraphId::from_uuid);

    let subgraph_name = metadata.get("subgraph")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    // Get or extract the subgraph origin
    let (spawn_position, relative_position) = if let Some(sg_id) = subgraph_id {
        // Extract the subgraph origin from metadata
        let origin = metadata.get("subgraph_origin")
            .and_then(|v| {
                let x = v.get("x")?.as_f64()? as f32;
                let y = v.get("y")?.as_f64()? as f32;
                let z = v.get("z")?.as_f64()? as f32;
                Some(Position3D { x, y, z })
            })
            .unwrap_or_else(|| Position3D { x: 0.0, y: 0.0, z: 0.0 });

        // Calculate relative position (node position should already include origin offset)
        let relative = Position3D {
            x: position.x - origin.x,
            y: position.y - origin.y,
            z: position.z - origin.z,
        };

        // Update subgraph info
        if !subgraph_origins.origins.contains_key(&sg_id) {
            // Create a visual marker for the subgraph origin (optional)
            let origin_entity = commands.spawn((
                Transform::from_translation(Vec3::new(origin.x, origin.y, origin.z)),
                GlobalTransform::default(),
                Name::new(format!("SubgraphOrigin_{}", sg_id)),
            )).id();

            subgraph_origins.origins.insert(sg_id, origin_entity);
            subgraph_origins.subgraph_info.insert(sg_id, SubgraphInfo {
                name: subgraph_name.clone(),
                origin,
                node_count: 0,
                member_entities: Vec::new(),
            });
        }

        (*position, relative)
    } else {
        (*position, Position3D { x: 0.0, y: 0.0, z: 0.0 })
    };

    // Larger nodes for better visibility
    let node_mesh = meshes.add(Sphere::new(5.0));

    // Color based on metadata type if available
    let node_type = metadata
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("default");

    let base_color = match node_type {
        "Actor" => Color::srgb(0.9, 0.3, 0.3),      // Red for actors
        "System" => Color::srgb(0.3, 0.9, 0.3),     // Green for systems
        "Command" => Color::srgb(0.3, 0.3, 0.9),    // Blue for commands
        "Event" => Color::srgb(0.9, 0.9, 0.3),      // Yellow for events
        "Policy" => Color::srgb(0.9, 0.3, 0.9),     // Magenta for policies
        _ => Color::srgb(0.5, 0.7, 0.9),           // Light blue default
    };

    let node_material = materials.add(StandardMaterial {
        base_color,
        metallic: 0.2,
        perceptual_roughness: 0.4,
        ..default()
    });

    // Convert position - preserve all coordinates including Y for grid layouts
    let spawn_position_vec: Vec3 = spawn_position.into();

    let mut entity = commands.spawn((
        GraphNode {
            node_id,
            graph_id,
        },
        Mesh3d(node_mesh),
        MeshMaterial3d(node_material),
        Transform::from_translation(spawn_position_vec),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        NodeAppearanceAnimation {
            start_time: time.elapsed_secs(),
            duration: 0.5,
            start_scale: 0.0,
            target_scale: 1.0,
        },
        ForceNode {
            velocity: Vec3::ZERO,
            mass: 1.0,
            charge: 1.0,
        },
        ForceLayoutParticipant,
        NodeLabel {
            text: label,
        },
        Name::new(format!("Node_{}", node_id)),
    ));

    // Add subgraph membership if applicable
    if let Some(sg_id) = subgraph_id {
        entity.insert(SubgraphMember {
            subgraph_ids: {
                let mut set = HashSet::new();
                set.insert(sg_id);
                set
            },
            relative_position,
        });
    }

    let entity_id = entity.id();

    // Update subgraph info after spawning
    if let Some(sg_id) = subgraph_id {
        if let Some(info) = subgraph_origins.subgraph_info.get_mut(&sg_id) {
            info.node_count += 1;
            info.member_entities.push(entity_id);
        }
    }
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

    eprintln!("spawn_edge: Looking for source {:?} and target {:?}", source_id, target_id);
    eprintln!("spawn_edge: Total nodes in query: {}", node_query.iter().count());

    for (entity, node) in node_query.iter() {
        eprintln!("spawn_edge: Checking node {:?}", node.node_id);
        if node.node_id == source_id {
            source_entity = Some(entity);
            eprintln!("spawn_edge: Found source entity {:?}", entity);
        }
        if node.node_id == target_id {
            target_entity = Some(entity);
            eprintln!("spawn_edge: Found target entity {:?}", entity);
        }
    }

    if let (Some(source), Some(target)) = (source_entity, target_entity) {
        eprintln!("spawn_edge: Creating edge between {:?} and {:?}", source, target);
        let edge_mesh = meshes.add(Cylinder::new(0.2, 1.0));
        let edge_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.8, 0.8),
            metallic: 0.3,
            perceptual_roughness: 0.5,
            emissive: Color::srgb(0.1, 0.1, 0.1).into(),
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
    } else {
        eprintln!("spawn_edge: Failed to find source/target entities for edge {:?}", edge_id);
        eprintln!("  Source entity: {:?}, Target entity: {:?}", source_entity, target_entity);
        eprintln!("  Creating pending edge to retry later");

        // Create a pending edge entity that will be processed later
        commands.spawn(PendingEdge {
            edge_id,
            graph_id,
            source_id,
            target_id,
            spawn_time: time.elapsed_secs(),
        });
    }
}

/// Process pending edges that couldn't be created earlier
fn process_pending_edges(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    pending_edges: Query<(Entity, &PendingEdge)>,
    node_query: Query<(Entity, &GraphNode)>,
    time: Res<Time>,
) {
    for (pending_entity, pending) in pending_edges.iter() {
        // Try to find source and target nodes
        let mut source_entity = None;
        let mut target_entity = None;

        for (entity, node) in node_query.iter() {
            if node.node_id == pending.source_id {
                source_entity = Some(entity);
            }
            if node.node_id == pending.target_id {
                target_entity = Some(entity);
            }
        }

        if let (Some(source), Some(target)) = (source_entity, target_entity) {
            eprintln!("process_pending_edges: Creating edge {:?} (was pending)", pending.edge_id);

            let edge_mesh = meshes.add(Cylinder::new(0.2, 1.0));
            let edge_material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.8, 0.8),
                metallic: 0.3,
                perceptual_roughness: 0.5,
                emissive: Color::srgb(0.1, 0.1, 0.1).into(),
                ..default()
            });

            commands.spawn((
                GraphEdge {
                    edge_id: pending.edge_id,
                    graph_id: pending.graph_id,
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

            // Remove the pending edge
            commands.entity(pending_entity).despawn();
        } else if time.elapsed_secs() - pending.spawn_time > 5.0 {
            // Give up after 5 seconds
            eprintln!("process_pending_edges: Giving up on edge {:?} after 5 seconds", pending.edge_id);
            commands.entity(pending_entity).despawn();
        }
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
    nodes: Query<(Entity, &GraphNode, &NodeLabel, &Transform), Added<NodeLabel>>,
    asset_server: Res<AssetServer>,
) {
    let label_count = nodes.iter().count();
    if label_count > 0 {
        eprintln!("create_node_labels: Creating labels for {} nodes", label_count);
    }

    // Load font
    let font = asset_server.load("fonts/FiraCodeNerdFont-Regular.ttf");

    for (entity, graph_node, label, node_transform) in nodes.iter() {
        eprintln!("create_node_labels: Creating label '{}' for entity {:?}", label.text, entity);

        // Create UI text that will be positioned in world space
        let label_entity = commands.spawn((
            Text::new(&label.text),
            TextFont {
                font: font.clone(),
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            NodeLabelEntity {
                parent_node: entity,
                offset: Vec3::new(0.0, 30.0, 0.0), // Offset in screen pixels
            },
            // Store the world position for conversion
            Transform::from_translation(node_transform.translation),
        )).id();

        eprintln!("create_node_labels: Created label entity {:?} for node {:?}", label_entity, entity);
        eprintln!("  Label text: '{}', World position: {:?}", label.text, node_transform.translation);
    }
}

/// Update label positions to follow their parent nodes
fn update_label_positions(
    node_query: Query<(Entity, &Transform), With<GraphNode>>,
    mut label_query: Query<(&NodeLabelEntity, &mut Node, &mut Transform), Without<GraphNode>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    for (label_entity, mut label_node, mut label_transform) in label_query.iter_mut() {
        // Find the parent node's position
        if let Ok((_, node_transform)) = node_query.get(label_entity.parent_node) {
            // Update the stored world position
            label_transform.translation = node_transform.translation;

            // Convert world position to screen coordinates
            if let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, node_transform.translation) {
                // Apply the label to the viewport position with offset
                label_node.left = Val::Px(viewport_pos.x - 50.0); // Center the text
                label_node.top = Val::Px(viewport_pos.y - label_entity.offset.y);
            }
        }
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

/// Setup system for the graph editor
fn setup_graph_editor(mut commands: Commands) {
    // Create a graph container entity
    commands.spawn((
        GraphContainer {
            graph_id: GraphId::new(),
            name: "Main Graph".to_string(),
        },
        Transform::default(),
        GlobalTransform::default(),
    ));
}

/// Handle import requests
fn handle_import_requests(
    mut import_requests: EventReader<ImportRequestEvent>,
    mut import_results: EventWriter<ImportResultEvent>,
) {
    for request in import_requests.read() {
        // For now, just echo back the event
        import_results.send(ImportResultEvent {
            event: request.event.clone(),
        });
    }
}

/// Handle import results
fn handle_import_results(
    mut import_results: EventReader<ImportResultEvent>,
) {
    for result in import_results.read() {
        match &result.event {
            DomainEvent::Graph(crate::domain::events::GraphEvent::GraphImportCompleted {
                imported_nodes,
                imported_edges,
                ..
            }) => {
                info!("Import successful: {} nodes, {} edges", imported_nodes, imported_edges);
            }
            DomainEvent::Graph(crate::domain::events::GraphEvent::GraphImportFailed {
                error,
                ..
            }) => {
                warn!("Import failed: {}", error);
            }
            _ => {}
        }
    }
}

/// Update force-directed layout
fn update_force_layout(
    mut nodes: Query<(&mut Transform, &ForceNode), With<GraphNode>>,
    edges: Query<(&GraphEdge,)>,
    settings: Res<ForceLayoutSettings>,
    time: Res<Time>,
) {
    // Simple force-directed layout update
    let dt = time.delta_secs();

    for (mut transform, force_node) in nodes.iter_mut() {
        // Apply velocity
        transform.translation += force_node.velocity * dt;

        // Apply damping
        // Note: This is a simplified version
    }
}

