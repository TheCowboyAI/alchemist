//! Demo: Markdown Import with NATS Replay
//!
//! This example demonstrates:
//! 1. Importing a markdown file with Mermaid diagrams
//! 2. Rendering the graph in Bevy
//! 3. Recording events to NATS
//! 4. Replaying events from NATS when pressing 'R'

use bevy::prelude::*;
use ia::{
    application::{CommandEvent, EventNotification},
    domain::{
        commands::{
            Command, GraphCommand, ImportOptions, ImportSource, graph_commands::MergeBehavior,
        },
        events::{DomainEvent, EdgeEvent, GraphEvent, NodeEvent},
        services::ImportFormat,
        value_objects::{EdgeId, GraphId, NodeId, Position3D},
    },
    infrastructure::{
        event_store::{DistributedEventStore, EventStore},
        nats::{JetStreamConfig, NatsClient, NatsConfig},
    },
    presentation::{
        components::{GraphContainer, GraphEdge, GraphNode},
        plugins::GraphEditorPlugin,
    },
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;

/// Resource to hold NATS connection and event store
#[derive(Resource)]
struct NatsConnection {
    runtime: Arc<Runtime>,
    event_store: Arc<RwLock<Option<DistributedEventStore>>>,
    current_graph_id: Option<GraphId>,
}

/// State for tracking replay
#[derive(Resource, Default)]
struct ReplayState {
    is_replaying: bool,
    replayed_count: usize,
}

fn main() {
    println!("=== Markdown Import with NATS Replay Demo ===");
    println!();
    println!("Controls:");
    println!("  M - Import markdown file (KECO_DDD_Core_Model.md)");
    println!("  R - Replay graph from NATS");
    println!("  C - Clear current graph");
    println!("  ESC - Exit");
    println!();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GraphEditorPlugin)
        .insert_resource(ReplayState::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_import_input,
                handle_replay_input,
                handle_clear_input,
                process_nats_events,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).build())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            ..default()
        })),
    ));

    // Setup NATS connection
    let runtime = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
    let event_store = Arc::new(RwLock::new(None));

    let event_store_clone = event_store.clone();
    runtime.spawn(async move {
        match setup_nats_connection().await {
            Ok(store) => {
                println!("✅ NATS connection established");
                *event_store_clone.write().await = Some(store);
            }
            Err(e) => {
                eprintln!("❌ Failed to connect to NATS: {}", e);
            }
        }
    });

    commands.insert_resource(NatsConnection {
        runtime,
        event_store,
        current_graph_id: None,
    });
}

async fn setup_nats_connection()
-> Result<DistributedEventStore, Box<dyn std::error::Error + Send + Sync>> {
    let client = NatsClient::new(NatsConfig {
        url: "nats://localhost:4222".to_string(),
        auth: None,
        jetstream: Some(JetStreamConfig {
            domain: Some("demo".to_string()),
            prefix: Some("demo".to_string()),
            max_memory: Some(1024 * 1024 * 100), // 100MB
            max_file: Some(1024 * 1024 * 1000),  // 1GB
        }),
        connection_name: Some("markdown_demo".to_string()),
        max_reconnects: Some(5),
        reconnect_wait: Some(std::time::Duration::from_secs(1)),
    })
    .await?;

    let jetstream = client.jetstream().await?;
    let event_store = DistributedEventStore::new(jetstream).await?;

    Ok(event_store)
}

fn handle_import_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut event_writer: EventWriter<CommandEvent>,
    mut nats: ResMut<NatsConnection>,
    graph_query: Query<&GraphContainer>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        println!("\n📄 Importing markdown file...");

        // Create or get graph ID
        let graph_id = if let Ok(container) = graph_query.get_single() {
            container.graph_id
        } else {
            let new_id = GraphId::new();

            // Create graph first
            event_writer.send(CommandEvent {
                command: Command::Graph(GraphCommand::CreateGraph {
                    id: new_id,
                    name: "DDD Core Model".to_string(),
                    metadata: HashMap::new(),
                }),
            });

            new_id
        };

        nats.current_graph_id = Some(graph_id);

        // Send import command
        event_writer.send(CommandEvent {
            command: Command::Graph(GraphCommand::ImportGraph {
                graph_id,
                source: ImportSource::File {
                    path: "assets/keco/KECO_DDD_Core_Model.md".to_string(),
                },
                format: "mermaid".to_string(),
                options: ImportOptions {
                    merge_behavior: MergeBehavior::MergePreferImported,
                    id_prefix: Some("ddd".to_string()),
                    position_offset: Some(Position3D {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                    mapping: None,
                    validate: true,
                    max_nodes: Some(1000),
                },
            }),
        });

        println!("✅ Import command sent");
    }
}

fn handle_replay_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut replay_state: ResMut<ReplayState>,
    nats: Res<NatsConnection>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        if nats.current_graph_id.is_none() {
            println!("❌ No graph imported yet. Press 'M' to import first.");
            return;
        }

        println!("\n🔄 Starting NATS replay...");
        replay_state.is_replaying = true;
        replay_state.replayed_count = 0;
    }
}

fn handle_clear_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    nodes: Query<Entity, With<GraphNode>>,
    edges: Query<Entity, With<GraphEdge>>,
    containers: Query<Entity, With<GraphContainer>>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        println!("\n🗑️  Clearing graph...");

        // Remove all graph entities
        for entity in nodes.iter() {
            commands.entity(entity).despawn_recursive();
        }
        for entity in edges.iter() {
            commands.entity(entity).despawn_recursive();
        }
        for entity in containers.iter() {
            commands.entity(entity).despawn_recursive();
        }

        println!("✅ Graph cleared");
    }
}

fn process_nats_events(
    mut commands: Commands,
    mut replay_state: ResMut<ReplayState>,
    nats: Res<NatsConnection>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut event_writer: EventWriter<EventNotification>,
) {
    if !replay_state.is_replaying {
        return;
    }

    let graph_id = match nats.current_graph_id {
        Some(id) => id,
        None => return,
    };

    // Process events from NATS
    let runtime = nats.runtime.clone();
    let event_store = nats.event_store.clone();

    runtime.spawn(async move {
        if let Some(store) = event_store.read().await.as_ref() {
            match store.get_events(graph_id.to_string(), None).await {
                Ok(events) => {
                    println!("📨 Retrieved {} events from NATS", events.len());

                    // In a real implementation, we'd send these through the bridge
                    // For demo, we'll just print them
                    for (i, event) in events.iter().enumerate() {
                        match event {
                            DomainEvent::Node(NodeEvent::NodeAdded {
                                content, position, ..
                            }) => {
                                println!(
                                    "  {} Node: {} at ({:.1}, {:.1}, {:.1})",
                                    i + 1,
                                    content.label,
                                    position.x,
                                    position.y,
                                    position.z
                                );
                            }
                            DomainEvent::Edge(EdgeEvent::EdgeAdded { source, target, .. }) => {
                                println!("  {} Edge: {:?} → {:?}", i + 1, source, target);
                            }
                            DomainEvent::Graph(GraphEvent::GraphImportCompleted {
                                imported_nodes,
                                imported_edges,
                                ..
                            }) => {
                                println!(
                                    "  {} Import completed: {} nodes, {} edges",
                                    i + 1,
                                    imported_nodes,
                                    imported_edges
                                );
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to retrieve events: {}", e);
                }
            }
        }
    });

    // For demo purposes, just mark replay as complete
    replay_state.is_replaying = false;
    println!("✅ Replay complete");
}

/// Store events to NATS when they're generated
fn store_events_to_nats(mut events: EventReader<EventNotification>, nats: Res<NatsConnection>) {
    let runtime = nats.runtime.clone();
    let event_store = nats.event_store.clone();

    for notification in events.read() {
        if let Some(graph_id) = nats.current_graph_id {
            let event = notification.event.clone();

            runtime.spawn(async move {
                if let Some(store) = event_store.read().await.as_ref() {
                    match store.append_event(graph_id.to_string(), event).await {
                        Ok(_) => {
                            // Event stored successfully
                        }
                        Err(e) => {
                            eprintln!("Failed to store event: {}", e);
                        }
                    }
                }
            });
        }
    }
}
