//! Import system for loading graph files

use bevy::prelude::*;
use tracing::{info, error};
use crate::application::CommandEvent;
use crate::domain::{
    commands::{Command, GraphCommand, ImportSource, ImportOptions, graph_commands::MergeBehavior},
    services::ImportFormat,
    value_objects::{GraphId, Position3D},
};
use std::path::Path;
use std::collections::HashMap;

/// Resource to track import state
#[derive(Resource)]
pub struct ImportState {
    pub last_imported_path: Option<String>,
    pub default_format: ImportFormat,
}

impl Default for ImportState {
    fn default() -> Self {
        Self {
            last_imported_path: None,
            default_format: ImportFormat::ArrowsApp,
        }
    }
}

/// Plugin for graph import functionality
pub struct ImportPlugin;

impl Plugin for ImportPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ImportState>()
            .add_systems(Update, (handle_import_shortcuts, handle_mouse_import))
            .add_systems(Startup, (log_import_instructions, test_direct_import, create_test_graph_on_startup));
    }
}

/// Handle mouse-based import (as a fallback)
fn handle_mouse_import(
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: EventWriter<CommandEvent>,
    import_state: ResMut<ImportState>,
) {
    // Right-click to import
    if mouse.just_pressed(MouseButton::Right) {
        eprintln!("Right mouse button clicked - triggering import");
        info!("Right mouse button clicked - triggering import");
        import_file(&mut commands, "examples/data/sample_graph.json", ImportFormat::ArrowsApp);
    }

    // Middle-click to import Mermaid
    if mouse.just_pressed(MouseButton::Middle) {
        eprintln!("Middle mouse button clicked - importing Mermaid");
        info!("Middle mouse button clicked - importing Mermaid");
        import_file(&mut commands, "examples/data/workflow.mermaid", ImportFormat::Mermaid);
    }
}

/// Test direct import by creating nodes directly
fn test_direct_import(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    eprintln!("Testing direct import - creating sample nodes with visuals");

    let graph_id = crate::domain::value_objects::GraphId::new();

    // Create node mesh and material
    let node_mesh = meshes.add(Sphere::new(0.5));
    let node_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.3, 0.3),
        metallic: 0.3,
        perceptual_roughness: 0.6,
        ..default()
    });

        // Create a few test nodes directly with proper visuals
    let node1_id = crate::domain::value_objects::NodeId::new();
    commands.spawn((
        crate::presentation::components::GraphNode {
            node_id: node1_id,
            graph_id,
        },
        crate::presentation::components::NodeLabel {
            text: "Test Node 1".to_string(),
        },
        Mesh3d(node_mesh.clone()),
        MeshMaterial3d(node_material.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(1.0)), // Full scale, no animation
        Visibility::Visible,
    ));

    let node2_id = crate::domain::value_objects::NodeId::new();
    commands.spawn((
        crate::presentation::components::GraphNode {
            node_id: node2_id,
            graph_id,
        },
        crate::presentation::components::NodeLabel {
            text: "Test Node 2".to_string(),
        },
        Mesh3d(node_mesh.clone()),
        MeshMaterial3d(node_material.clone()),
        Transform::from_xyz(3.0, 0.0, 0.0).with_scale(Vec3::splat(1.0)), // Full scale, no animation
        Visibility::Visible,
    ));

    eprintln!("Created 2 test nodes with visual components");

    // Also create a simple cube to verify rendering works
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let cube_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.8, 0.3),
        ..default()
    });

    commands.spawn((
        Mesh3d(cube_mesh),
        MeshMaterial3d(cube_material),
        Transform::from_xyz(0.0, 2.0, 0.0),
    ));

    eprintln!("Created test cube at (0, 2, 0)");
}

/// Log import instructions on startup
fn log_import_instructions() {
    eprintln!("=== Import System Ready ===");
    eprintln!("KEYBOARD SHORTCUTS:");
    eprintln!("  Press 'I' to import sample_graph.json");
    eprintln!("  Press Space to test keyboard input");
    eprintln!("  Press Ctrl+I for original import shortcut");
    eprintln!("  Press Ctrl+M to import Mermaid");
    eprintln!("  Press Ctrl+D to import DOT");
    eprintln!();
    eprintln!("MOUSE SHORTCUTS (if keyboard doesn't work):");
    eprintln!("  Right-click to import sample_graph.json");
    eprintln!("  Middle-click to import Mermaid");
    eprintln!("==========================");

    info!("=== Import System Ready ===");
    info!("Press 'I' to import sample_graph.json");
    info!("Press Space to test keyboard input");
    info!("Press Ctrl+I for original import shortcut");
    info!("Press Ctrl+M to import Mermaid");
    info!("Press Ctrl+D to import DOT");
    info!("Right-click to import sample_graph.json");
    info!("Middle-click to import Mermaid");
    info!("==========================");
}

/// Handle keyboard shortcuts for importing
fn handle_import_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: EventWriter<CommandEvent>,
    _import_state: ResMut<ImportState>,
) {
    // Debug: Print to stderr to ensure we see output
    if keyboard.get_just_pressed().len() > 0 {
        eprintln!("Keys pressed: {:?}", keyboard.get_just_pressed().collect::<Vec<_>>());
    }

    // Debug: Log any key press
    for key in keyboard.get_just_pressed() {
        info!("Key pressed: {:?}", key);
        eprintln!("Key pressed: {key:?}");
    }

    // Debug: Check if Ctrl is pressed
    if keyboard.pressed(KeyCode::ControlLeft) {
        info!("Ctrl is pressed");
        eprintln!("Ctrl is pressed");
    }

        // Test with simple key press (no modifiers)
    if keyboard.just_pressed(KeyCode::KeyI) && !keyboard.pressed(KeyCode::ControlLeft) {
        eprintln!("'I' key pressed - triggering import");
        info!("'I' key pressed - triggering import");
        import_file(&mut commands, "examples/data/sample_graph.json", ImportFormat::ArrowsApp);
    }

    // Test with Space key
    if keyboard.just_pressed(KeyCode::Space) {
        eprintln!("Space key pressed - testing keyboard input");
        info!("Space key pressed - testing keyboard input");
    }

    // Ctrl+I to import from file
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyI) {
        eprintln!("Import shortcut triggered (Ctrl+I)");
        info!("Import shortcut triggered (Ctrl+I)");

        // For now, import a predefined file. In a real app, you'd open a file dialog
        import_file(&mut commands, "examples/data/sample_graph.json", ImportFormat::ArrowsApp);
    }

    // Ctrl+M to import Mermaid
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyM) {
        info!("Import Mermaid shortcut triggered (Ctrl+M)");
        import_file(&mut commands, "examples/data/workflow.mermaid", ImportFormat::Mermaid);
    }

    // Ctrl+D to import DOT
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyD) {
        info!("Import DOT shortcut triggered (Ctrl+D)");
        import_file(&mut commands, "examples/data/network.dot", ImportFormat::Dot);
    }

    // Ctrl+Shift+I to import from clipboard (inline content)
    if keyboard.pressed(KeyCode::ControlLeft)
        && keyboard.pressed(KeyCode::ShiftLeft)
        && keyboard.just_pressed(KeyCode::KeyI) {
        info!("Import from clipboard triggered (Ctrl+Shift+I)");
        import_from_clipboard(&mut commands);
    }
}

/// Import a file with the specified format
pub fn import_file(
    commands: &mut EventWriter<CommandEvent>,
    file_path: &str,
    format: ImportFormat,
) {
    eprintln!("import_file called with: {file_path} format: {format:?}");
    info!("Importing file: {} with format: {:?}", file_path, format);

    let graph_id = GraphId::new();

    // Check if file exists
    if !Path::new(file_path).exists() {
        eprintln!("ERROR: File not found: {file_path}");
        error!("File not found: {}", file_path);
        return;
    }

    eprintln!("File exists, creating import command...");

    // Send import command
    commands.send(CommandEvent {
        command: Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::File {
                path: file_path.to_string(),
            },
            format: format_to_string(&format),
            options: ImportOptions {
                merge_behavior: MergeBehavior::AlwaysCreate,
                id_prefix: Some("imported".to_string()),
                position_offset: Some(Position3D {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
                mapping: None,
                validate: true,
                max_nodes: Some(10000),
            },
        }),
    });

    eprintln!("Import command sent for graph: {graph_id:?}");
    info!("Import command sent for graph: {:?}", graph_id);
}

/// Import from clipboard (simulated with inline content)
fn import_from_clipboard(commands: &mut EventWriter<CommandEvent>) {
    // In a real application, you would read from the system clipboard
    // For now, we'll use a sample inline content
    let sample_mermaid = r#"
graph LR
    A[Import System] --> B[Parse Content]
    B --> C{Valid?}
    C -->|Yes| D[Create Nodes]
    C -->|No| E[Show Error]
    D --> F[Create Edges]
    F --> G[Update View]
"#;

    let graph_id = GraphId::new();

    commands.send(CommandEvent {
        command: Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::InlineContent {
                content: sample_mermaid.to_string(),
            },
            format: "mermaid".to_string(),
            options: ImportOptions {
                merge_behavior: MergeBehavior::AlwaysCreate,
                id_prefix: Some("clipboard".to_string()),
                position_offset: Some(Position3D {
                    x: 100.0,
                    y: 100.0,
                    z: 0.0,
                }),
                mapping: None,
                validate: true,
                max_nodes: Some(1000),
            },
        }),
    });

    info!("Imported content from clipboard");
}

/// Convert ImportFormat enum to string
fn format_to_string(format: &ImportFormat) -> String {
    match format {
        ImportFormat::ArrowsApp => "arrows_app".to_string(),
        ImportFormat::Cypher => "cypher".to_string(),
        ImportFormat::Mermaid => "mermaid".to_string(),
        ImportFormat::Dot => "dot".to_string(),
        ImportFormat::ProgressJson => "progress_json".to_string(),
        ImportFormat::VocabularyJson => "vocabulary_json".to_string(),
        ImportFormat::RssAtom => "rss_atom".to_string(),
    }
}

/// System to display import instructions
pub fn display_import_help(commands: Commands) {
    info!("Graph Import Shortcuts:");
    info!("  Ctrl+I - Import from file (sample_graph.json)");
    info!("  Ctrl+M - Import Mermaid diagram");
    info!("  Ctrl+D - Import DOT graph");
    info!("  Ctrl+Shift+I - Import from clipboard");
}

/// Create a test graph on startup for testing imports
pub fn create_test_graph_on_startup(
    mut commands: EventWriter<CommandEvent>,
) {
    use crate::domain::commands::{Command, GraphCommand};
    use crate::domain::value_objects::GraphId;

    // Create a test graph
    let graph_id = GraphId::new();
    commands.write(CommandEvent {
        command: Command::Graph(GraphCommand::CreateGraph {
            id: graph_id,
            name: "Test Import Graph".to_string(),
            metadata: HashMap::new(),
        }),
    });

    info!("Created test graph with ID: {:?}", graph_id);
}
