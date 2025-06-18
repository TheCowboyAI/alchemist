//! Simplified Demo: KECO Loan Process Flow Visualization with AI Analysis
//!
//! This example demonstrates:
//! 1. Creating a simple loan process graph
//! 2. Simulating AI analysis of the workflow
//! 3. Displaying results in a UI panel

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use std::collections::HashMap;

/// Component to mark selected nodes
#[derive(Component)]
struct SelectedNode;

/// Component for graph nodes
#[derive(Component)]
struct LoanProcessNode {
    id: String,
    label: String,
    node_type: NodeType,
}

#[derive(Debug, Clone, PartialEq)]
enum NodeType {
    Start,
    Process,
    Decision,
    End,
}

/// Resource to track analysis state
#[derive(Resource, Default)]
struct AnalysisState {
    selected_nodes: Vec<String>,
    analysis_text: String,
    is_analyzing: bool,
}

fn main() {
    println!("=== KECO Loan Process Flow Demo (Simplified) ===");
    println!();
    println!("This demo visualizes the KECO Capital loan processing workflow");
    println!("and simulates AI analysis of the process.");
    println!();
    println!("Controls:");
    println!("  L - Load demo loan process");
    println!("  Click nodes - Select for analysis");
    println!("  A - Analyze selected nodes");
    println!("  C - Clear selection");
    println!("  ESC - Exit");
    println!();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .insert_resource(AnalysisState::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_load_input,
                handle_node_selection,
                handle_analysis_input,
                handle_clear_input,
                update_ui_panel,
                update_node_colors,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
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
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
    });
}

fn handle_load_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_nodes: Query<Entity, With<LoanProcessNode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyL) {
        println!("\n📄 Loading KECO loan process flow...");

        // Clear existing nodes
        for entity in existing_nodes.iter() {
            commands.entity(entity).despawn();
        }

        // Define the loan process nodes
        let nodes = vec![
            ("borrower", "Borrower Application", NodeType::Start, Vec3::new(-8.0, 0.0, 0.0)),
            ("documents", "Document Collection", NodeType::Process, Vec3::new(-4.0, 0.0, 0.0)),
            ("verification", "Document Verification", NodeType::Process, Vec3::new(0.0, 0.0, 0.0)),
            ("underwriting", "Underwriting Review", NodeType::Process, Vec3::new(4.0, 0.0, 0.0)),
            ("decision", "Approval Decision", NodeType::Decision, Vec3::new(8.0, 0.0, 0.0)),
            ("approved", "Loan Approved", NodeType::End, Vec3::new(12.0, 2.0, 0.0)),
            ("rejected", "Loan Rejected", NodeType::End, Vec3::new(12.0, -2.0, 0.0)),
        ];

        // Create nodes
        for (id, label, node_type, position) in nodes {
            let color = match node_type {
                NodeType::Start => Color::srgb(0.2, 0.6, 1.0),
                NodeType::Process => Color::srgb(0.6, 0.8, 0.3),
                NodeType::Decision => Color::srgb(1.0, 0.6, 0.2),
                NodeType::End => Color::srgb(0.7, 0.7, 0.7),
            };

            commands.spawn((
                LoanProcessNode {
                    id: id.to_string(),
                    label: label.to_string(),
                    node_type,
                },
                Mesh3d(meshes.add(Sphere::new(0.5).mesh())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    ..default()
                })),
                Transform::from_translation(position),
            ));
        }

        // Create edges (simplified - just visual lines)
        let edges = vec![
            (Vec3::new(-8.0, 0.0, 0.0), Vec3::new(-4.0, 0.0, 0.0)),
            (Vec3::new(-4.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0)),
            (Vec3::new(0.0, 0.0, 0.0), Vec3::new(4.0, 0.0, 0.0)),
            (Vec3::new(4.0, 0.0, 0.0), Vec3::new(8.0, 0.0, 0.0)),
            (Vec3::new(8.0, 0.0, 0.0), Vec3::new(12.0, 2.0, 0.0)),
            (Vec3::new(8.0, 0.0, 0.0), Vec3::new(12.0, -2.0, 0.0)),
        ];

        for (start, end) in edges {
            let midpoint = (start + end) / 2.0;
            let direction = (end - start).normalize();
            let length = start.distance(end);
            
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(length, 0.1, 0.1).mesh())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.5, 0.5, 0.5),
                    ..default()
                })),
                Transform::from_translation(midpoint)
                    .looking_to(direction, Vec3::Y),
            ));
        }

        println!("✅ Loan process flow loaded");
    }
}

fn handle_node_selection(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    nodes: Query<(Entity, &LoanProcessNode, &Transform)>,
    camera: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut analysis: ResMut<AnalysisState>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Ok(window) = windows.single() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok((camera, camera_transform)) = camera.get_single() {
                    // Simple ray casting
                    let ray = camera.viewport_to_world(camera_transform, cursor_pos);
                    
                    if let Ok(ray) = ray {
                        let multi_select = keyboard.pressed(KeyCode::ShiftLeft);
                        
                        // Check intersection with nodes
                        for (entity, node, transform) in nodes.iter() {
                            let distance = ray.intersects_sphere(
                                transform.translation,
                                0.5, // sphere radius
                            );
                            
                            if distance.is_some() {
                                if !multi_select {
                                    // Clear previous selection
                                    analysis.selected_nodes.clear();
                                    for (e, _, _) in nodes.iter() {
                                        commands.entity(e).remove::<SelectedNode>();
                                    }
                                }
                                
                                if !analysis.selected_nodes.contains(&node.id) {
                                    analysis.selected_nodes.push(node.id.clone());
                                    commands.entity(entity).insert(SelectedNode);
                                    println!("Selected: {}", node.label);
                                }
                                
                                if !multi_select {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn handle_analysis_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut analysis: ResMut<AnalysisState>,
    nodes: Query<&LoanProcessNode>,
) {
    if keyboard.just_pressed(KeyCode::KeyA) && !analysis.selected_nodes.is_empty() {
        println!("\n🤖 Analyzing selected nodes...");
        analysis.is_analyzing = true;
        
        // Simulate AI analysis
        let mut analysis_text = String::from("AI Analysis of Selected Loan Process Steps:\n\n");
        
        // Find selected nodes
        let selected: Vec<_> = nodes.iter()
            .filter(|n| analysis.selected_nodes.contains(&n.id))
            .collect();
        
        analysis_text.push_str(&format!("Analyzing {} selected steps:\n", selected.len()));
        for node in &selected {
            analysis_text.push_str(&format!("- {}\n", node.label));
        }
        
        analysis_text.push_str("\n📊 State Machine Analysis:\n\n");
        
        // Provide specific analysis based on selection
        if selected.iter().any(|n| n.id == "documents") {
            analysis_text.push_str("1. **Document Collection State**:\n");
            analysis_text.push_str("   - Entry condition: Application submitted\n");
            analysis_text.push_str("   - Required documents: Tax returns, bank statements, property info\n");
            analysis_text.push_str("   - Exit condition: All documents received\n");
            analysis_text.push_str("   - Timeout: 30 days\n\n");
        }
        
        if selected.iter().any(|n| n.id == "underwriting") {
            analysis_text.push_str("2. **Underwriting State**:\n");
            analysis_text.push_str("   - Entry condition: Documents verified\n");
            analysis_text.push_str("   - Process: Risk assessment, DTI calculation, collateral evaluation\n");
            analysis_text.push_str("   - Decision criteria: Credit score >650, DTI <43%, LTV <80%\n");
            analysis_text.push_str("   - Exit: Approval/Rejection decision\n\n");
        }
        
        if selected.iter().any(|n| n.id == "decision") {
            analysis_text.push_str("3. **Decision Point**:\n");
            analysis_text.push_str("   - Branching logic based on underwriting results\n");
            analysis_text.push_str("   - Approval path: Generate loan documents, schedule closing\n");
            analysis_text.push_str("   - Rejection path: Send adverse action notice\n");
            analysis_text.push_str("   - Conditional approval: List additional requirements\n\n");
        }
        
        analysis_text.push_str("💡 **Optimization Opportunities**:\n");
        analysis_text.push_str("- Implement parallel document verification\n");
        analysis_text.push_str("- Add automated pre-qualification checks\n");
        analysis_text.push_str("- Use ML for faster underwriting decisions\n");
        
        analysis.analysis_text = analysis_text;
        analysis.is_analyzing = false;
        println!("✅ Analysis complete");
    }
}

fn handle_clear_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut analysis: ResMut<AnalysisState>,
    mut commands: Commands,
    nodes: Query<Entity, With<SelectedNode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        println!("\n🗑️ Clearing selection...");
        analysis.selected_nodes.clear();
        analysis.analysis_text.clear();
        analysis.is_analyzing = false;
        
        for entity in nodes.iter() {
            commands.entity(entity).remove::<SelectedNode>();
        }
    }
}

fn update_ui_panel(
    mut contexts: EguiContexts,
    analysis: Res<AnalysisState>,
) {
    egui::SidePanel::right("analysis_panel")
        .default_width(400.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("🤖 AI Process Analysis");
            
            ui.separator();
            
            ui.label(format!("Selected nodes: {}", analysis.selected_nodes.len()));
            
            if analysis.is_analyzing {
                ui.spinner();
                ui.label("Analyzing workflow...");
            } else if !analysis.analysis_text.is_empty() {
                egui::ScrollArea::vertical()
                    .max_height(600.0)
                    .show(ui, |ui| {
                        ui.label(&analysis.analysis_text);
                    });
            } else if analysis.selected_nodes.is_empty() {
                ui.label("Click on nodes to select them for analysis");
            } else {
                ui.label("Press 'A' to analyze the selected nodes");
            }
            
            ui.separator();
            
            // Show instructions
            ui.collapsing("Instructions", |ui| {
                ui.label("• L - Load loan process");
                ui.label("• Click - Select node");
                ui.label("• Shift+Click - Multi-select");
                ui.label("• A - Analyze selection");
                ui.label("• C - Clear selection");
            });
        });
}

fn update_node_colors(
    mut materials: ResMut<Assets<StandardMaterial>>,
    nodes: Query<(&LoanProcessNode, &MeshMaterial3d<StandardMaterial>, Option<&SelectedNode>), Changed<SelectedNode>>,
) {
    for (node, material_handle, selected) in nodes.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let base_color = match node.node_type {
                NodeType::Start => Color::srgb(0.2, 0.6, 1.0),
                NodeType::Process => Color::srgb(0.6, 0.8, 0.3),
                NodeType::Decision => Color::srgb(1.0, 0.6, 0.2),
                NodeType::End => Color::srgb(0.7, 0.7, 0.7),
            };
            
            if selected.is_some() {
                material.base_color = base_color.lighter(0.3);
                material.emissive = base_color.into();
            } else {
                material.base_color = base_color;
                material.emissive = LinearRgba::BLACK;
            }
        }
    }
} 