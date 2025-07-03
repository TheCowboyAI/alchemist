//! Enhanced Visualization Demo
//! 
//! Demonstrates the advanced visualization features of the CIM graph editor

use bevy::prelude::*;
use ia::{
    plugins::{GraphEditorPlugin, CameraControllerPlugin, EnhancedVisualizationPlugin},
    events::{NodeAdded, EdgeAdded},
    value_objects::{NodeId, EdgeId, GraphId, NodeType, EdgeRelationship, Position3D},
    components::{NodeEntity, EdgeEntity},
};
use uuid::Uuid;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Enhanced Visualization Demo".to_string(),
                resolution: (1600., 1200.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.05)))
        // Add plugins
        .add_plugins(GraphEditorPlugin)
        .add_plugins(CameraControllerPlugin)
        .add_plugins(EnhancedVisualizationPlugin)
        // Demo systems
        .add_systems(Startup, setup_demo_scene)
        .add_systems(Update, (
            generate_demo_events,
            animate_demo_nodes,
            show_instructions,
        ))
        .run();
}

#[derive(Resource)]
struct DemoState {
    event_timer: Timer,
    node_count: u32,
    last_node_id: Option<NodeId>,
}

impl Default for DemoState {
    fn default() -> Self {
        Self {
            event_timer: Timer::new(Duration::from_secs(2), TimerMode::Repeating),
            node_count: 0,
            last_node_id: None,
        }
    }
}

fn setup_demo_scene(
    mut commands: Commands,
    mut node_events: EventWriter<NodeAdded>,
    mut edge_events: EventWriter<EdgeAdded>,
) {
    commands.init_resource::<DemoState>();
    
    // Create initial graph structure
    let graph_id = GraphId(Uuid::new_v4());
    
    // Create a central hub node
    let hub_id = NodeId(Uuid::new_v4());
    node_events.send(NodeAdded {
        node_id: hub_id,
        graph_id,
        node_type: NodeType::Process,
        position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
        metadata: Default::default(),
    });
    
    // Create surrounding nodes in a circle
    let radius = 5.0;
    let node_count = 8;
    let mut previous_id = None;
    
    for i in 0..node_count {
        let angle = (i as f32 / node_count as f32) * std::f32::consts::TAU;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        
        let node_id = NodeId(Uuid::new_v4());
        
        // Create node
        node_events.send(NodeAdded {
            node_id,
            graph_id,
            node_type: match i % 4 {
                0 => NodeType::Start,
                1 => NodeType::Process,
                2 => NodeType::Decision,
                _ => NodeType::End,
            },
            position: Position3D { x, y: 0.0, z },
            metadata: Default::default(),
        });
        
        // Connect to hub
        edge_events.send(EdgeAdded {
            edge_id: EdgeId(Uuid::new_v4()),
            graph_id,
            source: hub_id,
            target: node_id,
            relationship: EdgeRelationship::Connects,
        });
        
        // Connect to previous node in circle
        if let Some(prev_id) = previous_id {
            edge_events.send(EdgeAdded {
                edge_id: EdgeId(Uuid::new_v4()),
                graph_id,
                source: prev_id,
                target: node_id,
                relationship: EdgeRelationship::Sequence,
            });
        }
        
        previous_id = Some(node_id);
    }
    
    // Connect last to first to complete the circle
    if let Some(last_id) = previous_id {
        let first_id = NodeId(Uuid::new_v4());
        edge_events.send(EdgeAdded {
            edge_id: EdgeId(Uuid::new_v4()),
            graph_id,
            source: last_id,
            target: hub_id,
            relationship: EdgeRelationship::Sequence,
        });
    }
    
    // Camera position
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(10.0, 15.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    println!("=== Enhanced Visualization Demo ===");
    println!("Controls:");
    println!("  P - Toggle particles");
    println!("  L - Toggle glow effects");
    println!("  F - Toggle edge flow animation");
    println!("  N - Create new node");
    println!("  E - Create edge mode");
    println!("  S - Select mode");
    println!("  Space - Generate random event");
    println!("  ESC - Exit");
    println!("================================");
}

fn generate_demo_events(
    time: Res<Time>,
    mut state: ResMut<DemoState>,
    mut node_events: EventWriter<NodeAdded>,
    mut edge_events: EventWriter<EdgeAdded>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Update timer
    state.event_timer.tick(time.delta());
    
    // Generate random events periodically or on spacebar
    if state.event_timer.just_finished() || keyboard.just_pressed(KeyCode::Space) {
        let graph_id = GraphId(Uuid::new_v4());
        
        // Randomly choose event type
        let event_type = state.node_count % 3;
        
        match event_type {
            0 => {
                // Create a new node at random position
                let angle = rand::random::<f32>() * std::f32::consts::TAU;
                let distance = 3.0 + rand::random::<f32>() * 5.0;
                let x = angle.cos() * distance;
                let z = angle.sin() * distance;
                let y = rand::random::<f32>() * 2.0 - 1.0;
                
                let node_id = NodeId(Uuid::new_v4());
                
                node_events.send(NodeAdded {
                    node_id,
                    graph_id,
                    node_type: NodeType::Process,
                    position: Position3D { x, y, z },
                    metadata: Default::default(),
                });
                
                state.last_node_id = Some(node_id);
                state.node_count += 1;
                
                println!("Created node {} at ({:.1}, {:.1}, {:.1})", state.node_count, x, y, z);
            }
            1 => {
                // Create edge between last two nodes if possible
                if let Some(target_id) = state.last_node_id {
                    let source_id = NodeId(Uuid::new_v4()); // Would need to track more nodes
                    
                    edge_events.send(EdgeAdded {
                        edge_id: EdgeId(Uuid::new_v4()),
                        graph_id,
                        source: source_id,
                        target: target_id,
                        relationship: EdgeRelationship::Connects,
                    });
                    
                    println!("Created edge");
                }
            }
            _ => {
                // Trigger a workflow event (visual effect only)
                println!("Workflow event triggered");
            }
        }
    }
}

fn animate_demo_nodes(
    time: Res<Time>,
    mut nodes: Query<(&mut Transform, &NodeEntity)>,
) {
    // Add subtle floating animation to nodes
    for (mut transform, node) in nodes.iter_mut() {
        let offset = node.node_id.0.as_u128() as f32 * 0.1;
        let float_height = (time.elapsed_secs() * 0.5 + offset).sin() * 0.2;
        transform.translation.y = float_height;
    }
}

fn show_instructions(
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyH) {
        println!("\n=== Help ===");
        println!("Visual Effects:");
        println!("  - Nodes have dynamic glow effects");
        println!("  - Edges show flowing particles");
        println!("  - Events create ripple effects");
        println!("  - Particle systems on special nodes");
        println!("  - LOD system for performance");
        println!("  - Frustum culling enabled");
        println!("============\n");
    }
} 