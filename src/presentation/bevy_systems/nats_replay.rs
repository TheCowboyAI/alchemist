//! NATS-based event replay system for testing connectivity

use bevy::prelude::*;
use tracing::{info, error, debug};
use async_nats::Client;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::presentation::components::{GraphNode, GraphEdge, AnimationProgress};
use crate::domain::value_objects::{NodeId, EdgeId, GraphId};
use super::force_layout::ForceNode;

/// NATS event replay subject
const REPLAY_SUBJECT: &str = "graph.replay.events";

/// Serializable graph event for NATS transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NatsGraphEvent {
    NodeAdded {
        node_id: String,
        position: [f32; 3],
        label: String,
    },
    EdgeAdded {
        edge_id: String,
        source_id: String,
        target_id: String,
    },
    AnimationProgress {
        entity_type: String,
        entity_id: String,
        progress: f32,
    },
}

/// Resource for NATS client
#[derive(Resource, Clone)]
pub struct NatsClient {
    client: Arc<Mutex<Client>>,
}

impl NatsClient {
    pub async fn new(url: &str) -> Result<Self, async_nats::Error> {
        let client = async_nats::connect(url).await?;
        Ok(Self {
            client: Arc::new(Mutex::new(client)),
        })
    }

    pub async fn publish(&self, subject: &str, event: &NatsGraphEvent) -> Result<(), Box<dyn std::error::Error>> {
        let payload = serde_json::to_vec(event)?;
        let client = self.client.lock().await;
        client.publish(subject.to_string(), payload.into()).await?;
        Ok(())
    }

    pub async fn subscribe(&self, subject: &str) -> Result<async_nats::Subscriber, Box<dyn std::error::Error>> {
        let client = self.client.lock().await;
        Ok(client.subscribe(subject.to_string()).await?)
    }
}

/// Component to track NATS replay state
#[derive(Component)]
pub struct NatsReplayState {
    pub is_replaying: bool,
    pub events_sent: usize,
    pub events_received: usize,
}

/// Event to trigger NATS replay
#[derive(Event)]
pub struct StartNatsReplay;

/// Event received from NATS
#[derive(Event)]
pub struct NatsEventReceived(pub NatsGraphEvent);

/// System to publish recorded events to NATS
pub fn publish_events_to_nats(
    mut commands: Commands,
    nats_client: Res<NatsClient>,
    mut replay_events: EventReader<StartNatsReplay>,
    recorded_events: Query<&RecordedEvent>,
    runtime: Res<TokioRuntime>,
) {
    for _ in replay_events.read() {
        info!("Starting NATS event replay");

        // Collect all recorded events
        let events: Vec<_> = recorded_events.iter()
            .map(|e| e.0.clone())
            .collect();

        let events_count = events.len();
        let nats = nats_client.clone();

        // Spawn async task to publish events
        runtime.0.spawn(async move {
            for (i, event) in events.iter().enumerate() {
                if let Err(e) = nats.publish(REPLAY_SUBJECT, event).await {
                    error!("Failed to publish event {}: {:?}", i, e);
                } else {
                    debug!("Published event {} to NATS", i);
                }

                // Small delay between events for visual effect
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            info!("Finished publishing {} events to NATS", events_count);
        });

        // Track replay state
        commands.spawn(NatsReplayState {
            is_replaying: true,
            events_sent: events_count,
            events_received: 0,
        });
    }
}

/// System to subscribe to NATS events and update graph
pub fn subscribe_to_nats_events(
    mut commands: Commands,
    nats_client: Res<NatsClient>,
    runtime: Res<TokioRuntime>,
) {
    // This should run once at startup
    static SUBSCRIBED: std::sync::Once = std::sync::Once::new();

    SUBSCRIBED.call_once(|| {
        let nats = nats_client.clone();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // Spawn subscription task
        runtime.0.spawn(async move {
            let subscriber = match nats.subscribe(REPLAY_SUBJECT).await {
                Ok(sub) => sub,
                Err(e) => {
                    error!("Failed to subscribe to NATS: {:?}", e);
                    return;
                }
            };

            let mut subscriber = subscriber;
            info!("Subscribed to NATS subject: {}", REPLAY_SUBJECT);

            while let Some(msg) = subscriber.next().await {
                match serde_json::from_slice::<NatsGraphEvent>(&msg.payload) {
                    Ok(event) => {
                        debug!("Received event from NATS: {:?}", event);
                        if tx.send(event).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to deserialize NATS message: {:?}", e);
                    }
                }
            }
        });

        // Store receiver in a resource
        commands.insert_resource(NatsEventReceiver(Arc::new(Mutex::new(rx))));
    });
}

/// Resource to hold the event receiver
#[derive(Resource)]
pub struct NatsEventReceiver(Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<NatsGraphEvent>>>);

/// System to poll NATS events and send to Bevy
pub fn poll_nats_events(
    receiver: Option<Res<NatsEventReceiver>>,
    mut event_writer: EventWriter<NatsEventReceived>,
    runtime: Res<TokioRuntime>,
) {
    if let Some(receiver) = receiver {
        let rx = receiver.0.clone();

        // Try to receive events without blocking
        runtime.0.block_on(async {
            let mut rx = rx.lock().await;
            while let Ok(event) = rx.try_recv() {
                event_writer.write(NatsEventReceived(event));
            }
        });
    }
}

/// System to process NATS events and update the graph
pub fn process_nats_events(
    mut commands: Commands,
    mut events: EventReader<NatsEventReceived>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut replay_state: Query<&mut NatsReplayState>,
    existing_nodes: Query<(Entity, &GraphNode)>,
) {
    for event in events.read() {
        match &event.0 {
            NatsGraphEvent::NodeAdded { node_id, position, label } => {
                info!("Processing NodeAdded from NATS: {}", node_id);

                // Check if node already exists
                let exists = existing_nodes.iter()
                    .any(|(_, node)| node.node_id.to_string() == *node_id);

                if !exists {
                    // Create new node
                    let node_id = NodeId::new();
                    commands.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.5))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(0.3, 0.5, 0.8),
                            metallic: 0.8,
                            perceptual_roughness: 0.2,
                            ..default()
                        })),
                        Transform::from_translation(Vec3::from_array(*position)),
                        GraphNode {
                            node_id,
                            graph_id: GraphId::new(),
                        },
                        AnimationProgress(1.0),
                        Name::new(label.clone()),
                        ForceNode::default(),
                    ));
                }
            }

            NatsGraphEvent::EdgeAdded { edge_id, source_id, target_id } => {
                info!("Processing EdgeAdded from NATS: {}", edge_id);

                // Find source and target entities
                let mut source_entity = None;
                let mut target_entity = None;

                for (entity, node) in existing_nodes.iter() {
                    if node.node_id.to_string() == *source_id {
                        source_entity = Some(entity);
                    }
                    if node.node_id.to_string() == *target_id {
                        target_entity = Some(entity);
                    }
                }

                if let (Some(source), Some(target)) = (source_entity, target_entity) {
                    // Create edge (simplified - just spawn a marker)
                    commands.spawn((
                        GraphEdge {
                            edge_id: EdgeId::new(),
                            graph_id: GraphId::new(),
                            source,
                            target,
                        },
                        AnimationProgress(1.0),
                        Name::new(format!("Edge_{edge_id}")),
                    ));
                }
            }

            NatsGraphEvent::AnimationProgress { entity_type, entity_id, progress } => {
                debug!("Animation progress from NATS: {} {} = {}", entity_type, entity_id, progress);
                // Could update animation progress here if needed
            }
        }

        // Update replay state
        for mut state in replay_state.iter_mut() {
            state.events_received += 1;
            if state.events_received >= state.events_sent {
                state.is_replaying = false;
                info!("NATS replay complete: {} events processed", state.events_received);
            }
        }
    }
}

/// Component to store recorded events
#[derive(Component)]
pub struct RecordedEvent(pub NatsGraphEvent);

/// Resource for Tokio runtime
#[derive(Resource)]
pub struct TokioRuntime(pub tokio::runtime::Runtime);

impl Default for TokioRuntime {
    fn default() -> Self {
        Self(tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"))
    }
}

/// Plugin to add NATS replay functionality
pub struct NatsReplayPlugin;

impl Plugin for NatsReplayPlugin {
    fn build(&self, app: &mut App) {
        // Create NATS client
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        let nats_client = runtime.block_on(async {
            NatsClient::new("nats://localhost:4222").await
                .expect("Failed to connect to NATS")
        });

        app.insert_resource(nats_client)
            .insert_resource(TokioRuntime(runtime))
            .add_event::<StartNatsReplay>()
            .add_event::<NatsEventReceived>()
            .add_systems(Startup, subscribe_to_nats_events)
            .add_systems(Update, (
                publish_events_to_nats,
                poll_nats_events,
                process_nats_events,
            ).chain());
    }
}
