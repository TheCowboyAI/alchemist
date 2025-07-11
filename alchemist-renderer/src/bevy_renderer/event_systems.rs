//! Event-driven systems for Bevy renderer
//!
//! This module provides ECS systems that handle renderer events,
//! integrating naturally with Bevy's event-driven architecture.

use bevy::prelude::*;
use alchemist::renderer_events::{
    ShellToRendererEvent, RendererToShellEvent, EventBuilder,
    bevy_integration::{RendererEventHandler, RendererEventQueue}
};
use alchemist::renderer::{GraphNode, GraphEdge};
use tokio::sync::mpsc;
use std::sync::Arc;
use parking_lot::Mutex;

/// Plugin for handling renderer events
pub struct RendererEventPlugin {
    pub renderer_id: String,
    pub event_sender: mpsc::Sender<RendererToShellEvent>,
}

impl Plugin for RendererEventPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RendererInfo {
            renderer_id: self.renderer_id.clone(),
        })
        .insert_resource(EventSender(Arc::new(Mutex::new(self.event_sender.clone()))))
        .insert_resource(RendererEventQueue::default())
        .add_event::<GraphNodeClickedEvent>()
        .add_event::<GraphEdgeClickedEvent>()
        .add_systems(Update, (
            process_shell_events,
            handle_graph_interactions,
            send_events_to_shell,
        ));
    }
}

/// Resource containing renderer information
#[derive(Resource)]
struct RendererInfo {
    renderer_id: String,
}

/// Resource for sending events to shell
#[derive(Resource)]
struct EventSender(Arc<Mutex<mpsc::Sender<RendererToShellEvent>>>);

/// Internal event for graph node clicks
#[derive(Event)]
struct GraphNodeClickedEvent {
    node_id: String,
}

/// Internal event for graph edge clicks
#[derive(Event)]
struct GraphEdgeClickedEvent {
    edge_id: String,
}

/// Component for graph nodes
#[derive(Component)]
pub struct GraphNodeComponent {
    pub node_id: String,
    pub label: String,
    pub metadata: serde_json::Value,
}

/// Component for graph edges
#[derive(Component)]
pub struct GraphEdgeComponent {
    pub edge_id: String,
    pub source: String,
    pub target: String,
    pub label: Option<String>,
}

/// System to process events from the shell
fn process_shell_events(
    mut commands: Commands,
    mut event_queue: ResMut<RendererEventQueue>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &GraphNodeComponent)>,
) {
    for event in event_queue.events.drain(..) {
        match event {
            ShellToRendererEvent::GraphDataUpdated { nodes, edges, .. } => {
                // Clear existing graph
                for (entity, _) in query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                
                // Create new nodes
                for node in nodes {
                    spawn_graph_node(&mut commands, &mut meshes, &mut materials, node);
                }
                
                // Create edges
                for edge in edges {
                    spawn_graph_edge(&mut commands, &mut meshes, &mut materials, edge);
                }
            }
            
            ShellToRendererEvent::CloseRequested { .. } => {
                // Handle window close
                info!("Close requested via event");
            }
            
            _ => {}
        }
    }
}

/// System to handle graph interactions
fn handle_graph_interactions(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    nodes: Query<(&GraphNodeComponent, &Transform, &Aabb)>,
    edges: Query<(&GraphEdgeComponent, &Transform, &Aabb)>,
    mut node_clicked: EventWriter<GraphNodeClickedEvent>,
    mut edge_clicked: EventWriter<GraphEdgeClickedEvent>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Ok(window) = windows.get_single() {
            if let Some(cursor_pos) = window.cursor_position() {
                // Get ray from camera
                for (camera, camera_transform) in cameras.iter() {
                    if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
                        // Check node intersections
                        for (node, transform, aabb) in nodes.iter() {
                            if ray_intersects_aabb(&ray, transform, aabb) {
                                node_clicked.send(GraphNodeClickedEvent {
                                    node_id: node.node_id.clone(),
                                });
                                return;
                            }
                        }
                        
                        // Check edge intersections
                        for (edge, transform, aabb) in edges.iter() {
                            if ray_intersects_aabb(&ray, transform, aabb) {
                                edge_clicked.send(GraphEdgeClickedEvent {
                                    edge_id: edge.edge_id.clone(),
                                });
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// System to send events back to shell
fn send_events_to_shell(
    renderer_info: Res<RendererInfo>,
    event_sender: Res<EventSender>,
    mut node_clicked: EventReader<GraphNodeClickedEvent>,
    mut edge_clicked: EventReader<GraphEdgeClickedEvent>,
) {
    let sender = event_sender.0.lock();
    
    for event in node_clicked.read() {
        let shell_event = RendererToShellEvent::GraphNodeClicked {
            event_id: EventBuilder::new_id(),
            timestamp: EventBuilder::now(),
            renderer_id: renderer_info.renderer_id.clone(),
            node_id: event.node_id.clone(),
        };
        
        // Send asynchronously
        let sender_clone = sender.clone();
        bevy::tasks::AsyncComputeTaskPool::get().spawn(async move {
            let _ = sender_clone.send(shell_event).await;
        }).detach();
    }
    
    for event in edge_clicked.read() {
        let shell_event = RendererToShellEvent::GraphEdgeClicked {
            event_id: EventBuilder::new_id(),
            timestamp: EventBuilder::now(),
            renderer_id: renderer_info.renderer_id.clone(),
            edge_id: event.edge_id.clone(),
        };
        
        // Send asynchronously
        let sender_clone = sender.clone();
        bevy::tasks::AsyncComputeTaskPool::get().spawn(async move {
            let _ = sender_clone.send(shell_event).await;
        }).detach();
    }
}

/// Helper to spawn a graph node
fn spawn_graph_node(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    node: GraphNode,
) {
    let position = node.position.unwrap_or([0.0, 0.0, 0.0]);
    let color = node.color.unwrap_or([0.5, 0.5, 1.0, 1.0]);
    let size = node.size.unwrap_or(1.0);
    
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(color[0], color[1], color[2], color[3]),
            ..default()
        })),
        Transform::from_xyz(position[0], position[1], position[2]),
        GraphNodeComponent {
            node_id: node.id,
            label: node.label,
            metadata: node.metadata,
        },
    ));
}

/// Helper to spawn a graph edge
fn spawn_graph_edge(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    edge: GraphEdge,
) {
    // For now, just create a simple cylinder between nodes
    // In a real implementation, you'd look up node positions
    let color = edge.color.unwrap_or([0.7, 0.7, 0.7, 1.0]);
    
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.1, 5.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(color[0], color[1], color[2], color[3]),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GraphEdgeComponent {
            edge_id: format!("{}-{}", edge.source, edge.target),
            source: edge.source,
            target: edge.target,
            label: edge.label,
        },
    ));
}

/// Helper to check ray-AABB intersection
fn ray_intersects_aabb(ray: &Ray3d, transform: &Transform, aabb: &Aabb) -> bool {
    // Transform AABB to world space
    let world_min = transform.transform_point(aabb.min());
    let world_max = transform.transform_point(aabb.max());
    
    // Simple ray-AABB intersection test
    let inv_dir = ray.direction.recip();
    let t1 = (world_min - ray.origin) * inv_dir;
    let t2 = (world_max - ray.origin) * inv_dir;
    
    let t_min = t1.min(t2);
    let t_max = t1.max(t2);
    
    let t_enter = t_min.max_element();
    let t_exit = t_max.min_element();
    
    t_enter <= t_exit && t_exit >= 0.0
}

/// Helper to connect event stream from NATS
pub async fn setup_event_stream(
    renderer_id: String,
    nats_client: async_nats::Client,
) -> mpsc::Receiver<ShellToRendererEvent> {
    let (tx, rx) = mpsc::channel(100);
    
    // Subscribe to events for this renderer
    let subject = format!("alchemist.renderer.cmd.{}", renderer_id);
    let mut subscriber = nats_client.subscribe(subject).await.unwrap();
    
    tokio::spawn(async move {
        while let Some(msg) = subscriber.next().await {
            if let Ok(event) = serde_json::from_slice::<ShellToRendererEvent>(&msg.payload) {
                let _ = tx.send(event).await;
            }
        }
    });
    
    rx
}