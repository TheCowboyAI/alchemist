//! Domain-level test that would have caught the missing import functionality

use ia::domain::{
    commands::{Command, GraphCommand, ImportSource, ImportFormat, ImportOptions, MergeBehavior},
    events::{DomainEvent, GraphEvent},
    value_objects::GraphId,
};
use ia::application::command_handlers::{handle_graph_command, process_commands};

#[test]
fn test_import_graph_command_generates_event() {
    // This test would have caught that ImportGraph commands were returning None
    let import_cmd = GraphCommand::ImportGraph {
        graph_id: GraphId::new(),
        source: ImportSource::File {
            path: "test.json".to_string(),
        },
        format: ImportFormat::ArrowsApp,
        options: ImportOptions {
            mapping: None,
            validate: true,
            max_nodes: None,
        },
    };

    // The original implementation returned None here
    let result = handle_graph_command(&import_cmd);

    assert!(result.is_some(), "ImportGraph command should generate an event, not return None");

    if let Some(event) = result {
        match event {
            DomainEvent::Graph(GraphEvent::GraphImportRequested { .. }) => {
                // Success - the command generated the expected event
            }
            _ => panic!("ImportGraph command should generate GraphImportRequested event"),
        }
    }
}

#[test]
fn test_all_graph_commands_are_handled() {
    // This test ensures no command returns None unexpectedly
    let test_cases = vec![
        GraphCommand::CreateGraph {
            id: GraphId::new(),
            name: "Test".to_string(),
            metadata: Default::default(),
        },
        GraphCommand::ImportGraph {
            graph_id: GraphId::new(),
            source: ImportSource::File {
                path: "test.json".to_string(),
            },
            format: ImportFormat::ArrowsApp,
            options: ImportOptions {
                mapping: None,
                validate: true,
                max_nodes: None,
            },
        },
    ];

    for cmd in test_cases {
        let cmd_name = match &cmd {
            GraphCommand::CreateGraph { .. } => "CreateGraph",
            GraphCommand::ImportGraph { .. } => "ImportGraph",
            _ => "Other",
        };

        let result = handle_graph_command(&cmd);
        assert!(
            result.is_some(),
            "{} command should be handled and generate an event, not return None",
            cmd_name
        );
    }
}
