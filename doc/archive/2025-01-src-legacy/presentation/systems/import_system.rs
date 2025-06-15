//! Import system for loading graph files

use crate::application::CommandEvent;
use crate::domain::{
    commands::{Command, GraphCommand, ImportOptions, ImportSource, graph_commands::MergeBehavior},
    services::{
        ImportFormat,
        graph_import::{ImportMapping, LayoutAlgorithm, LayoutConfig},
    },
    value_objects::{GraphId, Position3D},
};
use bevy::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use tracing::{error, info};

/// Resource to track import state
#[derive(Resource)]
pub struct ImportState {
    pub last_imported_path: Option<String>,
    pub default_format: ImportFormat,
    pub import_count: u32, // Track number of imports for positioning
}

impl Default for ImportState {
    fn default() -> Self {
        Self {
            last_imported_path: None,
            default_format: ImportFormat::ArrowsApp,
            import_count: 0,
        }
    }
}

/// Plugin for graph import functionality
pub struct ImportPlugin;

impl Plugin for ImportPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ImportState>()
            .add_systems(Update, (handle_import_shortcuts, handle_mouse_import))
            .add_systems(
                Startup,
                (log_import_instructions /* test_direct_import, */,),
            );
    }
}

/// Handle mouse-based import (as a fallback)
fn handle_mouse_import(
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: EventWriter<CommandEvent>,
    mut import_state: ResMut<ImportState>,
    graph_query: Query<&crate::presentation::components::GraphContainer>,
) {
    // Right-click to import
    if mouse.just_pressed(MouseButton::Right) {
        eprintln!("Right mouse button clicked - triggering import");
        info!("Right mouse button clicked - triggering import");
        import_file_to_graph(
            &mut commands,
            &graph_query,
            "examples/data/sample_graph.json",
            ImportFormat::ArrowsApp,
            &mut import_state,
        );
    }

    // Middle-click to import Mermaid
    if mouse.just_pressed(MouseButton::Middle) {
        eprintln!("Middle mouse button clicked - importing Mermaid");
        info!("Middle mouse button clicked - importing Mermaid");
        import_file_to_graph(
            &mut commands,
            &graph_query,
            "examples/data/workflow.mermaid",
            ImportFormat::Mermaid,
            &mut import_state,
        );
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

/// Handle keyboard shortcuts for import
pub fn handle_import_shortcuts(
    mut commands: EventWriter<CommandEvent>,
    keyboard: Res<ButtonInput<KeyCode>>,
    graph_query: Query<&crate::presentation::components::GraphContainer>,
    mut import_state: ResMut<ImportState>,
) {
    // Ctrl+J to import JSON
    if keyboard.just_pressed(KeyCode::KeyJ) && keyboard.pressed(KeyCode::ControlLeft) {
        info!("Import JSON shortcut triggered (Ctrl+J)");
        import_file_to_graph(
            &mut commands,
            &graph_query,
            "assets/models/sample_graph.json",
            ImportFormat::ArrowsApp,
            &mut import_state,
        );
    }

    // Ctrl+M to import Mermaid
    if keyboard.just_pressed(KeyCode::KeyM) && keyboard.pressed(KeyCode::ControlLeft) {
        info!("Import Mermaid shortcut triggered (Ctrl+M)");
        import_file_to_graph(
            &mut commands,
            &graph_query,
            "examples/data/workflow.mermaid",
            ImportFormat::Mermaid,
            &mut import_state,
        );
    }

    // Ctrl+D to import DDD markdown files
    if keyboard.just_pressed(KeyCode::KeyD) && keyboard.pressed(KeyCode::ControlLeft) {
        info!("Import DDD markdown shortcut triggered (Ctrl+D)");
        // Cycle through different DDD markdown files
        let ddd_files = [
            "assets/models/KECO_DDD_Core_Model.md",
            "assets/models/KECO_DDD_LoanOriginationContext.md",
            "assets/models/KECO_DDD_UnderwritingContext.md",
            "assets/models/KECO_DDD_DocumentContext.md",
            "assets/models/KECO_DDD_ClosingContext.md",
        ];

        let file_index = (import_state.import_count as usize) % ddd_files.len();
        let file_path = ddd_files[file_index];

        info!("Importing DDD markdown file: {}", file_path);
        import_file_to_graph(
            &mut commands,
            &graph_query,
            file_path,
            ImportFormat::Mermaid,
            &mut import_state,
        );
    }

    // Ctrl+C to import from clipboard
    if keyboard.just_pressed(KeyCode::KeyC) && keyboard.pressed(KeyCode::ControlLeft) {
        info!("Import from clipboard shortcut triggered (Ctrl+C)");
        import_from_clipboard_to_graph(&mut commands, &graph_query, &mut import_state);
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

    // Create a mapping that preserves original positions
    let mut mapping = ImportMapping::default();
    mapping.layout_config = LayoutConfig {
        algorithm: LayoutAlgorithm::None,
        parameters: Default::default(),
    };

    // Send import command
    commands.write(CommandEvent {
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
                mapping: Some(mapping),
                validate: true,
                max_nodes: Some(10000),
            },
        }),
    });

    eprintln!("Import command sent for graph: {graph_id:?}");
    info!("Import command sent for graph: {:?}", graph_id);
}

/// Import a file to the existing graph
pub fn import_file_to_graph(
    commands: &mut EventWriter<CommandEvent>,
    graph_query: &Query<&crate::presentation::components::GraphContainer>,
    file_path: &str,
    format: ImportFormat,
    import_state: &mut ResMut<ImportState>,
) {
    eprintln!("import_file_to_graph called with: {file_path} format: {format:?}");
    info!(
        "Importing file to existing graph: {} with format: {:?}",
        file_path, format
    );

    // Get the current graph ID, or create a new one if none exists
    let (graph_id, is_new_graph) = if let Ok(container) = graph_query.single() {
        eprintln!("Using existing graph: {:?}", container.graph_id);
        (container.graph_id, false)
    } else {
        eprintln!("No existing graph found, creating new one");
        let new_id = GraphId::new();
        // First create the graph
        commands.write(CommandEvent {
            command: Command::Graph(GraphCommand::CreateGraph {
                id: new_id,
                name: "Imported Graph".to_string(),
                metadata: HashMap::new(),
            }),
        });
        (new_id, true)
    };

    // Check if file exists
    if !Path::new(file_path).exists() {
        eprintln!("ERROR: File not found: {file_path}");
        error!("File not found: {}", file_path);
        return;
    }

    // Calculate position offset for this import
    // Each import is offset by 30 units to the right
    // If this is a new graph, don't offset the first import
    let x_offset = if is_new_graph {
        0.0
    } else {
        import_state.import_count as f32 * 50.0 // Increased offset to 50 units
    };

    eprintln!(
        "File exists, creating import command for graph: {:?} with offset: {}",
        graph_id, x_offset
    );

    // Create a mapping that preserves original positions
    let mut mapping = ImportMapping::default();
    mapping.layout_config = LayoutConfig {
        algorithm: LayoutAlgorithm::None,
        parameters: Default::default(),
    };

    // Send import command
    commands.write(CommandEvent {
        command: Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::File {
                path: file_path.to_string(),
            },
            format: format_to_string(&format),
            options: ImportOptions {
                merge_behavior: MergeBehavior::MergePreferImported, // Merge into existing graph
                id_prefix: Some(format!("import_{}", import_state.import_count)),
                position_offset: Some(Position3D {
                    x: x_offset,
                    y: 0.0,
                    z: 0.0,
                }),
                mapping: Some(mapping),
                validate: true,
                max_nodes: Some(10000),
            },
        }),
    });

    // Increment import count for next import
    import_state.import_count += 1;

    eprintln!(
        "Import command sent for graph: {graph_id:?}, import count incremented to: {}",
        import_state.import_count
    );
    info!(
        "Import command sent for graph: {:?}, import count incremented to: {}",
        graph_id, import_state.import_count
    );
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

    commands.write(CommandEvent {
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

/// Import from clipboard to existing graph
fn import_from_clipboard_to_graph(
    commands: &mut EventWriter<CommandEvent>,
    graph_query: &Query<&crate::presentation::components::GraphContainer>,
    import_state: &mut ResMut<ImportState>,
) {
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

    // Get the current graph ID, or create a new one if none exists
    let graph_id = if let Ok(container) = graph_query.single() {
        container.graph_id
    } else {
        let new_id = GraphId::new();
        // First create the graph
        commands.write(CommandEvent {
            command: Command::Graph(GraphCommand::CreateGraph {
                id: new_id,
                name: "Imported Graph".to_string(),
                metadata: HashMap::new(),
            }),
        });
        new_id
    };

    // Calculate position offset for this import
    let x_offset = import_state.import_count as f32 * 30.0;

    // For Mermaid, we don't have positions, so use default layout
    // But for other formats that might have positions, preserve them
    let mapping = ImportMapping::default(); // Mermaid will use force-directed layout

    commands.write(CommandEvent {
        command: Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::InlineContent {
                content: sample_mermaid.to_string(),
            },
            format: "mermaid".to_string(),
            options: ImportOptions {
                merge_behavior: MergeBehavior::MergePreferImported, // Merge into existing graph
                id_prefix: Some(format!("clipboard_{}", import_state.import_count)),
                position_offset: Some(Position3D {
                    x: x_offset,
                    y: 0.0,
                    z: 0.0,
                }),
                mapping: Some(mapping),
                validate: true,
                max_nodes: Some(1000),
            },
        }),
    });

    // Increment import count
    import_state.import_count += 1;

    info!(
        "Imported content from clipboard to graph: {:?}, total imports: {}",
        graph_id, import_state.import_count
    );
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

/// Display import help
pub fn display_import_help() {
    eprintln!("Import System Ready!");
    eprintln!("  Press 'I' to import sample graph");
    eprintln!("  Press Ctrl+J to import JSON");
    eprintln!("  Press Ctrl+M to import Mermaid");
    eprintln!("  Press Ctrl+D to import DDD markdown files");
    eprintln!("  Press Ctrl+C to import from clipboard");
    eprintln!("  Left-click to import JSON");
    eprintln!("  Right-click to import Cypher");
    eprintln!("  Middle-click to import Mermaid");

    info!("Import System Ready!");
    info!("  Press 'I' to import sample graph");
    info!("  Ctrl+J - Import JSON");
    info!("  Ctrl+M - Import Mermaid diagram");
    info!("  Ctrl+D - Import DDD markdown files");
    info!("  Ctrl+C - Import from clipboard");
    info!("  Left-click - Import JSON");
    info!("  Right-click - Import Cypher");
    info!("  Middle-click - Import Mermaid");
}
