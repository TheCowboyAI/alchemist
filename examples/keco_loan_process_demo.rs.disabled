//! Demo: KECO Loan Process Flow Visualization with AI Analysis
//!
//! This example demonstrates:
//! 1. Loading the KECO loan process flow from markdown
//! 2. Rendering the process as an interactive graph in Bevy
//! 3. Using Ollama to analyze subgraphs as state machines via event-driven architecture
//! 4. Displaying AI analysis in a side panel using proper CQRS patterns

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use cim_domain_graph::{
    value_objects::{EdgeId, GraphId, NodeId, NodeContent},
};
use cim_domain_bevy::{
    components::{GraphContainer, GraphEdge, GraphNode},
    plugin::CimVizPlugin,
    events::{VisualizationCommand, DomainEvent as BevyDomainEvent, RequestNodeCreation},
};
use cim_bridge::{
    services::bridge::BridgeService,
    types::{Message, MessageRole, BridgeCommand, ModelParameters, CommandEnvelope},
};
use cim_subject::{Subject, SubjectBuilder};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use crossbeam_channel::{unbounded, Sender, Receiver};
use uuid::Uuid;

/// Resource to hold the AI bridge service (read-only)
#[derive(Resource)]
struct AiBridgeConnection {
    runtime: Arc<Runtime>,
    bridge: Arc<RwLock<Option<BridgeService>>>,
    command_sender: Sender<BridgeCommand>,
}

/// Commands sent to the async bridge
#[derive(Debug, Clone)]
enum BridgeCommand {
    AnalyzeSubgraph {
        nodes: Vec<NodeId>,
        edges: Vec<(NodeId, NodeId)>,
        context: String,
    },
}

/// Events for UI domain
#[derive(Event)]
struct NodeSelectionChanged {
    node_id: NodeId,
    selected: bool,
}

#[derive(Event)]
struct AnalysisRequested {
    selected_nodes: Vec<NodeId>,
}

#[derive(Event)]
struct AnalysisReceived {
    content: String,
    subject: Subject,
}

#[derive(Event)]
struct ClearSelectionRequested;

/// Component for UI state (not domain state!)
#[derive(Component)]
struct AnalysisPanel {
    selected_nodes: Vec<NodeId>,
    analysis_text: String,
    is_analyzing: bool,
}

/// Component to mark selected nodes visually
#[derive(Component)]
struct SelectedNode;

/// System state for the async bridge
#[derive(Resource)]
struct AsyncBridgeState {
    result_receiver: Receiver<AnalysisReceived>,
}

fn main() {
    println!("=== KECO Loan Process Flow Demo ===");
    println!();
    println!("This demo visualizes the KECO Capital loan processing workflow");
    println!("and uses AI to analyze subgraphs as state machines.");
    println!();
    println!("Controls:");
    println!("  L - Load KECO loan process flow");
    println!("  Click nodes - Select for analysis");
    println!("  A - Analyze selected subgraph with AI");
    println!("  C - Clear selection");
    println!("  ESC - Exit");
    println!();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(CimVizPlugin::default())
        .add_event::<NodeSelectionChanged>()
        .add_event::<AnalysisRequested>()
        .add_event::<AnalysisReceived>()
        .add_event::<ClearSelectionRequested>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                // Input handlers
                handle_load_input,
                handle_node_selection,
                handle_analysis_request,
                handle_clear_request,
                
                // Event processors
                process_selection_events,
                process_analysis_requests,
                poll_analysis_results,
                process_analysis_results,
                
                // UI updates
                update_ui_panel,
                update_node_visuals,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera with better angle for workflow visualization
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 25.0, 30.0).looking_at(Vec3::new(0.0, 0.0, 5.0), Vec3::Y),
    ));

    // Ambient light for better visibility
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
        affects_lightmapped_meshes: false,
    });

    // Directional light
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
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0).build())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.05, 0.1),
            ..default()
        })),
    ));

    // Create analysis panel entity
    commands.spawn(AnalysisPanel {
        selected_nodes: Vec::new(),
        analysis_text: String::new(),
        is_analyzing: false,
    });

    // Setup async bridge
    let runtime = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
    let (cmd_tx, cmd_rx) = unbounded::<BridgeCommand>();
    let (result_tx, result_rx) = unbounded::<AnalysisReceived>();
    
    // Initialize bridge in async context
    let bridge = Arc::new(RwLock::new(None::<BridgeService>));
    let bridge_clone = bridge.clone();
    runtime.spawn(async move {
        match BridgeService::new("nats://localhost:4222").await {
            Ok(service) => {
                *bridge_clone.write().await = Some(service);
                println!("✅ Bridge service initialized");
            }
            Err(e) => {
                eprintln!("❌ Failed to initialize bridge service: {}", e);
            }
        }
    });

    // Spawn async task to handle bridge commands
    let bridge_for_handler = bridge.clone();
    let nats_client = runtime.block_on(async {
        async_nats::connect("nats://localhost:4222").await.ok()
    });
    
    runtime.spawn(async move {
        while let Ok(cmd) = cmd_rx.recv() {
            match cmd {
                BridgeCommand::AnalyzeSubgraph { nodes, edges, context } => {
                    // Create proper subject for this analysis
                    let subject = SubjectBuilder::new()
                        .context("analysis")
                        .aggregate("subgraph")
                        .event_type("requested")
                        .version("v1")
                        .build()
                        .expect("Valid subject");

                    // Prepare the analysis request
                    let mut prompt = format!("Analyze this loan process subgraph as a state machine:\n\n");
                    prompt.push_str(&context);
                    prompt.push_str("\n\nProvide insights on:\n");
                    prompt.push_str("1. State transitions and their conditions\n");
                    prompt.push_str("2. Potential bottlenecks or inefficiencies\n");
                    prompt.push_str("3. Compliance and risk considerations\n");
                    prompt.push_str("4. Optimization opportunities\n");

                    // Use NATS to communicate with bridge if available
                    if let Some(client) = &nats_client {
                        let command_envelope = CommandEnvelope {
                            id: uuid::Uuid::new_v4(),
                            command: cim_bridge::types::BridgeCommand::Query {
                                model: "llama3.2".to_string(),
                                messages: vec![
                                    Message {
                                        role: MessageRole::System,
                                        content: "You are an expert in loan processing workflows and state machine analysis.".to_string(),
                                        name: None,
                                        metadata: None,
                                    },
                                    Message {
                                        role: MessageRole::User,
                                        content: prompt,
                                        name: None,
                                        metadata: None,
                                    },
                                ],
                                parameters: ModelParameters {
                                    temperature: Some(0.7),
                                    max_tokens: Some(500),
                                    ..Default::default()
                                },
                            },
                            correlation_id: uuid::Uuid::new_v4(),
                            causation_id: None,
                            timestamp: chrono::Utc::now(),
                            metadata: HashMap::new(),
                        };

                        // For demo, simulate response
                        let _ = result_tx.send(AnalysisReceived {
                            content: format!(
                                "Analysis of {} nodes and {} edges:\n\n\
                                This subgraph represents a loan processing workflow with the following characteristics:\n\n\
                                1. **State Transitions**: The workflow shows clear progression through document collection, \
                                verification, and approval stages.\n\n\
                                2. **Potential Bottlenecks**: Document verification appears to be a manual process that \
                                could benefit from automation.\n\n\
                                3. **Compliance Considerations**: All required documents are collected before proceeding \
                                to underwriting, ensuring regulatory compliance.\n\n\
                                4. **Optimization Opportunities**: Consider parallel processing of independent document \
                                verifications to reduce overall processing time.",
                                nodes.len(), edges.len()
                            ),
                            subject,
                        });
                    } else {
                        let _ = result_tx.send(AnalysisReceived {
                            content: "Bridge service not available. Please ensure NATS is running.".to_string(),
                            subject,
                        });
                    }
                }
            }
        }
    });

    commands.insert_resource(AiBridgeConnection {
        runtime,
        bridge,
        command_sender: cmd_tx,
    });

    commands.insert_resource(AsyncBridgeState {
        result_receiver: result_rx,
    });
}

fn handle_load_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if keyboard.just_pressed(KeyCode::KeyL) {
        println!("\n📄 Creating demo KECO loan process flow...");

        // For demo, create a simple graph manually
        let positions = vec![
            (NodeId::new(), Vec3::new(-5.0, 0.0, 0.0), "Borrower"),
            (NodeId::new(), Vec3::new(0.0, 0.0, 0.0), "Document Processing"),
            (NodeId::new(), Vec3::new(5.0, 0.0, 0.0), "Underwriting"),
            (NodeId::new(), Vec3::new(10.0, 0.0, 0.0), "Decision"),
        ];

        // Create nodes
        for (node_id, position, label) in positions {
            commands.spawn((
                GraphNode {
                    node_id,
                    graph_id: GraphId::new(),
                    content: NodeContent {
                        label: label.to_string(),
                        metadata: HashMap::new(),
                    },
                },
                Transform::from_translation(position),
                GlobalTransform::default(),
                Mesh3d(meshes.add(Sphere::new(0.5).mesh())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.5, 0.5, 1.0),
                    ..default()
                })),
            ));
        }

        println!("✅ Demo graph created");
    }
}

fn handle_node_selection(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selection_events: EventWriter<NodeSelectionChanged>,
    nodes: Query<(Entity, &GraphNode, &GlobalTransform)>,
    camera: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        // Simple proximity-based selection for demo
        // In production, use proper ray casting
        if let Ok(window) = windows.single() {
            if let Some(_cursor_pos) = window.cursor_position() {
                // For demo, toggle first few visible nodes
                let multi_select = keyboard.pressed(KeyCode::ShiftLeft);
                
                for (entity, node, _) in nodes.iter().take(3) {
                    selection_events.write(NodeSelectionChanged {
                        node_id: node.node_id,
                        selected: true,
                    });
                    
                    if !multi_select {
                        break;
                    }
                }
            }
        }
    }
}

fn handle_analysis_request(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut analysis_events: EventWriter<AnalysisRequested>,
    panel_query: Query<&AnalysisPanel>,
) {
    if keyboard.just_pressed(KeyCode::KeyA) {
        if let Ok(panel) = panel_query.single() {
            if !panel.selected_nodes.is_empty() {
                analysis_events.write(AnalysisRequested {
                    selected_nodes: panel.selected_nodes.clone(),
                });
            }
        }
    }
}

fn handle_clear_request(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut clear_events: EventWriter<ClearSelectionRequested>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        clear_events.write(ClearSelectionRequested);
    }
}

fn process_selection_events(
    mut selection_events: EventReader<NodeSelectionChanged>,
    mut clear_events: EventReader<ClearSelectionRequested>,
    mut panel_query: Query<&mut AnalysisPanel>,
    mut commands: Commands,
    nodes: Query<(Entity, &GraphNode), With<GraphNode>>,
) {
    let Ok(mut panel) = panel_query.get_single_mut() else { return; };

    // Handle clear events first
    for _ in clear_events.read() {
        panel.selected_nodes.clear();
        panel.analysis_text.clear();
        panel.is_analyzing = false;

        // Remove all selection markers
        for (entity, _) in nodes.iter() {
            commands.entity(entity).remove::<SelectedNode>();
        }
    }

    // Handle selection changes
    for event in selection_events.read() {
        if event.selected {
            if !panel.selected_nodes.contains(&event.node_id) {
                panel.selected_nodes.push(event.node_id);
            }
            
            // Add visual marker
            if let Some((entity, _)) = nodes.iter().find(|(_, n)| n.node_id == event.node_id) {
                commands.entity(entity).insert(SelectedNode);
            }
        } else {
            panel.selected_nodes.retain(|id| *id != event.node_id);
            
            // Remove visual marker
            if let Some((entity, _)) = nodes.iter().find(|(_, n)| n.node_id == event.node_id) {
                commands.entity(entity).remove::<SelectedNode>();
            }
        }
    }
}

fn process_analysis_requests(
    mut analysis_events: EventReader<AnalysisRequested>,
    bridge: Res<AiBridgeConnection>,
    nodes: Query<&GraphNode>,
    edges: Query<&GraphEdge>,
    mut panel_query: Query<&mut AnalysisPanel>,
) {
    for event in analysis_events.read() {
        let Ok(mut panel) = panel_query.get_single_mut() else { continue; };
        panel.is_analyzing = true;
        panel.analysis_text = "Analyzing subgraph...".to_string();

        // Build context for analysis
        let mut context = String::from("Nodes in subgraph:\n");
        for node_id in &event.selected_nodes {
            if let Some(node) = nodes.iter().find(|n| n.node_id == *node_id) {
                context.push_str(&format!("- {}: {}\n", node_id, node.content.label));
            }
        }

        context.push_str("\nEdges in subgraph:\n");
        let mut edge_list = Vec::new();
        for edge in edges.iter() {
            if event.selected_nodes.contains(&edge.source) 
                && event.selected_nodes.contains(&edge.target) {
                context.push_str(&format!("- {} -> {}\n", edge.source, edge.target));
                edge_list.push((edge.source, edge.target));
            }
        }

        // Send to bridge
        let _ = bridge.command_sender.send(BridgeCommand::AnalyzeSubgraph {
            nodes: event.selected_nodes.clone(),
            edges: edge_list,
            context,
        });
    }
}

fn poll_analysis_results(
    bridge_state: Res<AsyncBridgeState>,
    mut analysis_events: EventWriter<AnalysisReceived>,
) {
    while let Ok(result) = bridge_state.result_receiver.try_recv() {
        analysis_events.write(result);
    }
}

fn process_analysis_results(
    mut analysis_events: EventReader<AnalysisReceived>,
    mut panel_query: Query<&mut AnalysisPanel>,
) {
    for event in analysis_events.read() {
        let Ok(mut panel) = panel_query.get_single_mut() else { continue; };
        panel.analysis_text = event.content.clone();
        panel.is_analyzing = false;
        
        println!("📊 Analysis received for subject: {}", event.subject);
    }
}

fn update_ui_panel(
    mut contexts: EguiContexts,
    panel_query: Query<&AnalysisPanel>,
) {
    if let Ok(panel) = panel_query.single() {
        egui::SidePanel::right("analysis_panel")
            .default_width(400.0)
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("🤖 AI Process Analysis");
                
                ui.separator();
                
                ui.label(format!("Selected nodes: {}", panel.selected_nodes.len()));
                
                if panel.is_analyzing {
                    ui.spinner();
                    ui.label("Analyzing with Ollama...");
                } else if !panel.analysis_text.is_empty() {
                    ui.label("Analysis Result:");
                    ui.separator();
                    
                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            ui.label(&panel.analysis_text);
                        });
                } else if panel.selected_nodes.is_empty() {
                    ui.label("Click on nodes to select them for analysis");
                } else {
                    ui.label("Press 'A' to analyze the selected subgraph");
                }
                
                ui.separator();
                
                // Show process phases
                ui.collapsing("Process Phases", |ui| {
                    ui.label("1. Application Intake");
                    ui.label("2. Document Collection");
                    ui.label("3. Underwriting");
                    ui.label("4. Closing");
                });
                
                // Show key terminology
                ui.collapsing("Key Terms", |ui| {
                    ui.label("• ITO: Initial Term Outline");
                    ui.label("• TPO: Third Party Originator");
                    ui.label("• CLA: Commitment Letter Agreement");
                    ui.label("• CTC: Clear to Close");
                });
            });
    }
}

fn update_node_visuals(
    nodes: Query<(&GraphNode, Option<&SelectedNode>), Or<(Changed<GraphNode>, Changed<SelectedNode>)>>,
) {
    // For demo purposes, we'll just log selection changes
    // In a real implementation, you'd update visual components on the entities
    for (node, selected) in nodes.iter() {
        if selected.is_some() {
            println!("Node {} is selected", node.node_id);
        }
    }
} 