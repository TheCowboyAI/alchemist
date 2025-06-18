//! Markdown Import with NATS Replay Demo
//!
//! This demo shows:
//! 1. Importing markdown files with Mermaid diagrams
//! 2. Recording all events to NATS
//! 3. Replaying the events from NATS
//!
//! Controls:
//!   M - Import markdown file
//!   R - Replay events from NATS
//!   C - Clear current graph
//!   ESC - Exit

use bevy::prelude::*;
use ia::{
    application::{CommandEvent, EventNotification},
    domain::{
        commands::{
            Command, GraphCommand, ImportOptions, ImportSource, graph_commands::MergeBehavior,
        },
        events::{DomainEvent, EdgeEvent, GraphEvent, NodeEvent},
        value_objects::{GraphId, Position3D},
    },
    infrastructure::{
        event_store::{DistributedEventStore, EventStore},
        nats::{NatsClient, NatsConfig, config::JetStreamConfig},
    },
    presentation::{
        components::{GraphContainer, GraphEdge, GraphNode},
        plugins::GraphEditorPlugin,
    },
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::{error, info};

#[derive(Resource)]
struct NatsRuntime {
    runtime: Arc<Runtime>,
    event_store: Arc<DistributedEventStore>,
}

#[derive(Resource)]
struct EventRecorder {
    events: Vec<DomainEvent>,
    recording: bool,
}

fn main() {
    println!("=== Markdown Import with NATS Replay Demo ===");
    println!();
    println!("Controls:");
    println!("  M - Import markdown file with Mermaid diagrams");
    println!("  R - Replay events from NATS");
    println!("  C - Clear current graph");
    println!("  ESC - Exit");
    println!();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GraphEditorPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_markdown_import,
                handle_replay,
                handle_clear,
                record_events,
                display_stats,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, mut event_writer: EventWriter<CommandEvent>) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 20.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
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

    // Create initial graph
    let graph_id = GraphId::new();
    event_writer.write(CommandEvent {
        command: Command::Graph(GraphCommand::CreateGraph {
            id: graph_id,
            name: "NATS Replay Demo".to_string(),
            metadata: HashMap::new(),
        }),
    });

    // Initialize NATS runtime
    let runtime = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));

    // Initialize event store (will connect on first use)
    let event_store = runtime.block_on(async {
        match create_event_store().await {
            Ok(store) => Arc::new(store),
            Err(e) => {
                error!("Failed to create event store: {}", e);
                panic!("Cannot continue without NATS connection");
            }
        }
    });

    commands.insert_resource(NatsRuntime {
        runtime: runtime.clone(),
        event_store,
    });

    commands.insert_resource(EventRecorder {
        events: Vec::new(),
        recording: true,
    });

    println!("✅ Setup complete. Graph ID: {:?}", graph_id);
}

async fn create_event_store() -> Result<DistributedEventStore, Box<dyn std::error::Error>> {
    println!("Connecting to NATS at localhost:4222...");

    let config = NatsConfig {
        url: "nats://localhost:4222".to_string(),
        client_name: "markdown_nats_demo".to_string(),
        connection_timeout: std::time::Duration::from_secs(5),
        security: ia::infrastructure::nats::config::SecurityConfig::default(),
        jetstream: JetStreamConfig {
            enabled: true,
            default_stream: ia::infrastructure::nats::config::StreamConfig {
                name_prefix: "DEMO-EVENTS".to_string(),
                retention: ia::infrastructure::nats::config::RetentionPolicy::Limits,
                max_age: std::time::Duration::from_secs(24 * 60 * 60), // 1 day
                max_messages: None,
                max_bytes: Some(1024 * 1024 * 1000), // 1GB
                duplicate_window: std::time::Duration::from_secs(120),
            },
        },
        max_reconnects: Some(5),
    };

    let client = match NatsClient::new(config).await {
        Ok(client) => {
            println!("✅ Connected to NATS successfully");
            client
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to NATS: {}", e);
            eprintln!("   Make sure NATS is running with: nats-server -js");
            return Err(e.into());
        }
    };

    let jetstream = match client.jetstream() {
        Ok(js) => {
            println!("✅ JetStream context created");
            js
        }
        Err(e) => {
            eprintln!("❌ Failed to create JetStream context: {}", e);
            return Err(e.into());
        }
    };

    // Clean up any existing demo streams
    match jetstream.delete_stream("DEMO-EVENTS").await {
        Ok(_) => println!("✅ Cleaned up existing DEMO-EVENTS stream"),
        Err(_) => println!("ℹ️  No existing DEMO-EVENTS stream to clean up"),
    }

    match DistributedEventStore::new(jetstream.clone()).await {
        Ok(store) => {
            println!("✅ Event store initialized successfully");
            Ok(store)
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize event store: {}", e);
            Err(e.into())
        }
    }
}

fn handle_markdown_import(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut event_writer: EventWriter<CommandEvent>,
    graph_query: Query<&GraphContainer>,
    mut file_index: Local<usize>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        if let Ok(container) = graph_query.single() {
            let files = [
                ("assets/keco/KECO_DDD_Core_Model.md", "core"),
                ("assets/keco/KECO_DDD_LoanOriginationContext.md", "loan"),
                (
                    "assets/keco/KECO_DDD_UnderwritingContext.md",
                    "underwriting",
                ),
                ("assets/keco/KECO_DDD_DocumentContext.md", "document"),
                ("assets/keco/KECO_DDD_ClosingContext.md", "closing"),
            ];

            let (file_path, prefix) = files[*file_index % files.len()];
            *file_index += 1;

            println!("\n📄 Importing {}...", file_path);

            event_writer.write(CommandEvent {
                command: Command::Graph(GraphCommand::ImportGraph {
                    graph_id: container.graph_id,
                    source: ImportSource::File {
                        path: file_path.to_string(),
                    },
                    format: "mermaid".to_string(),
                    options: ImportOptions {
                        merge_behavior: MergeBehavior::MergePreferImported,
                        id_prefix: Some(prefix.to_string()),
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
}

fn handle_replay(
    keyboard: Res<ButtonInput<KeyCode>>,
    nats: Res<NatsRuntime>,
    graph_query: Query<&GraphContainer>,
    mut event_writer: EventWriter<CommandEvent>,
    mut recorder: ResMut<EventRecorder>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        if let Ok(container) = graph_query.single() {
            println!("\n🔄 Replaying events from NATS...");

            // Stop recording during replay
            recorder.recording = false;

            let graph_id = container.graph_id;
            let event_store = nats.event_store.clone();

            let events = nats
                .runtime
                .block_on(async { event_store.get_events(graph_id.to_string()).await });

            match events {
                Ok(events) => {
                    println!("📊 Found {} events to replay", events.len());

                    // Clear current graph first
                    event_writer.write(CommandEvent {
                        command: Command::Graph(GraphCommand::ClearGraph { graph_id }),
                    });

                    // Replay each event
                    for (i, event) in events.iter().enumerate() {
                        println!(
                            "  Replaying event {}/{}: {:?}",
                            i + 1,
                            events.len(),
                            event_type_name(event)
                        );

                        // Convert domain events back to commands
                        if let Some(command) = event_to_command(event, graph_id) {
                            event_writer.write(CommandEvent { command });
                        }
                    }

                    println!("✅ Replay complete!");

                    // Resume recording
                    recorder.recording = true;
                }
                Err(e) => {
                    error!("Failed to replay events: {}", e);
                }
            }
        }
    }
}

fn handle_clear(
    keyboard: Res<ButtonInput<KeyCode>>,
    graph_query: Query<&GraphContainer>,
    mut event_writer: EventWriter<CommandEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        if let Ok(container) = graph_query.single() {
            println!("\n🗑️  Clearing graph...");

            event_writer.write(CommandEvent {
                command: Command::Graph(GraphCommand::ClearGraph {
                    graph_id: container.graph_id,
                }),
            });

            println!("✅ Graph cleared");
        }
    }
}

fn record_events(
    mut event_reader: EventReader<EventNotification>,
    mut recorder: ResMut<EventRecorder>,
    nats: Res<NatsRuntime>,
    graph_query: Query<&GraphContainer>,
) {
    if !recorder.recording {
        // Clear events during replay
        event_reader.clear();
        return;
    }

    if let Ok(container) = graph_query.single() {
        let graph_id = container.graph_id;

        for notification in event_reader.read() {
            let event = notification.event.clone();
            recorder.events.push(event.clone());

            // Store in NATS
            let event_store = nats.event_store.clone();
            let result = nats.runtime.block_on(async {
                event_store
                    .append_events(graph_id.to_string(), vec![event.clone()])
                    .await
            });

            match result {
                Ok(_) => {
                    info!("Stored event in NATS: {:?}", event_type_name(&event));
                }
                Err(e) => {
                    error!("Failed to store event: {}", e);
                }
            }
        }
    }
}

fn display_stats(
    nodes: Query<&GraphNode>,
    edges: Query<&GraphEdge>,
    recorder: Res<EventRecorder>,
    time: Res<Time>,
) {
    if time.elapsed_secs() as u32 % 5 == 0 && time.delta_secs() > 0.0 {
        let node_count = nodes.iter().count();
        let edge_count = edges.iter().count();
        let event_count = recorder.events.len();

        println!(
            "\n📊 Stats: {} nodes, {} edges, {} events recorded",
            node_count, edge_count, event_count
        );
    }
}

fn event_type_name(event: &DomainEvent) -> &'static str {
    match event {
        DomainEvent::Graph(GraphEvent::GraphCreated { .. }) => "GraphCreated",
        DomainEvent::Graph(GraphEvent::GraphImportRequested { .. }) => "GraphImportRequested",
        DomainEvent::Graph(GraphEvent::GraphImportCompleted { .. }) => "GraphImportCompleted",
        DomainEvent::Node(NodeEvent::NodeAdded { .. }) => "NodeAdded",
        DomainEvent::Node(NodeEvent::NodeRemoved { .. }) => "NodeRemoved",
        DomainEvent::Edge(EdgeEvent::EdgeConnected { .. }) => "EdgeConnected",
        DomainEvent::Edge(EdgeEvent::EdgeRemoved { .. }) => "EdgeRemoved",
        _ => "Other",
    }
}

fn event_to_command(event: &DomainEvent, graph_id: GraphId) -> Option<Command> {
    match event {
        DomainEvent::Node(NodeEvent::NodeAdded {
            node_id,
            metadata,
            position,
            ..
        }) => {
            // Extract node_type from metadata
            let node_type = metadata
                .get("node_type")
                .and_then(|v| v.as_str())
                .unwrap_or("default")
                .to_string();

            // Convert metadata to content
            let content = serde_json::to_value(metadata)
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

            Some(Command::Graph(GraphCommand::AddNode {
                graph_id,
                node_id: *node_id,
                node_type,
                position: position.clone(),
                content,
            }))
        }
        DomainEvent::Edge(EdgeEvent::EdgeConnected {
            edge_id,
            source,
            target,
            relationship,
            ..
        }) => Some(Command::Graph(GraphCommand::ConnectNodes {
            graph_id,
            edge_id: *edge_id,
            source_id: *source,
            target_id: *target,
            edge_type: relationship.clone(),
            properties: HashMap::new(),
        })),
        _ => None,
    }
}
