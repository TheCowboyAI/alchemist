//! Example: Importing graphs from various sources

use bevy::prelude::*;
use ia::application::CommandEvent;
use ia::domain::{
    commands::graph_commands::MergeBehavior,
    commands::{Command, GraphCommand, ImportOptions, ImportSource},
    services::{GraphImportService, ImportFormat},
    value_objects::GraphId,
};
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, import_examples)
        .run();
}

fn import_examples(mut commands: EventWriter<CommandEvent>) {
    // Example 1: Import from a local file
    import_from_file(&mut commands, "examples/data/sample_graph.json");

    // Example 2: Import from inline content
    import_from_inline(&mut commands);

    // Example 3: Import from URL
    import_from_url(&mut commands, "https://example.com/graph.json");

    // Example 4: Import from Git repository
    import_from_git(&mut commands);
}

fn import_from_file(commands: &mut EventWriter<CommandEvent>, file_path: &str) {
    println!("Importing graph from file: {}", file_path);

    let graph_id = GraphId::new();

    commands.send(CommandEvent {
        command: Command::Graph(GraphCommand::ImportFromFile {
            graph_id,
            file_path: file_path.to_string(),
            format: "arrows_app".to_string(), // or "cypher", "mermaid", "dot", etc.
        }),
    });
}

fn import_from_inline(commands: &mut EventWriter<CommandEvent>) {
    println!("Importing graph from inline content");

    let graph_id = GraphId::new();

    // Example Mermaid diagram
    let mermaid_content = r#"
        graph TD
            A[Start] --> B{Decision}
            B -->|Yes| C[Process]
            B -->|No| D[End]
            C --> D
    "#;

    commands.send(CommandEvent {
        command: Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::InlineContent {
                content: mermaid_content.to_string(),
            },
            format: "mermaid".to_string(),
            options: ImportOptions {
                merge_behavior: MergeBehavior::AlwaysCreate,
                id_prefix: Some("imported".to_string()),
                position_offset: None,
                mapping: None,
                validate: true,
                max_nodes: Some(1000),
            },
        }),
    });
}

fn import_from_url(commands: &mut EventWriter<CommandEvent>, url: &str) {
    println!("Importing graph from URL: {}", url);

    let graph_id = GraphId::new();

    commands.send(CommandEvent {
        command: Command::Graph(GraphCommand::ImportFromUrl {
            graph_id,
            url: url.to_string(),
            format: "arrows_app".to_string(),
        }),
    });
}

fn import_from_git(commands: &mut EventWriter<CommandEvent>) {
    println!("Importing graph from Git repository");

    let graph_id = GraphId::new();

    commands.send(CommandEvent {
        command: Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::GitRepository {
                url: "https://github.com/example/graphs.git".to_string(),
                branch: Some("main".to_string()),
                path: "examples/workflow.mermaid".to_string(),
            },
            format: "mermaid".to_string(),
            options: ImportOptions {
                merge_behavior: MergeBehavior::MergePreferImported,
                id_prefix: None,
                position_offset: Some(ia::domain::value_objects::Position3D {
                    x: 100.0,
                    y: 100.0,
                    z: 0.0,
                }),
                mapping: None,
                validate: true,
                max_nodes: None,
            },
        }),
    });
}
