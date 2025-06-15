//! Domain-level test that would have caught the missing import functionality

use ia::domain::{
    aggregates::Graph,
    commands::{Command, GraphCommand, ImportOptions, ImportSource, graph_commands::MergeBehavior},
    events::{DomainEvent, GraphEvent},
    services::{GraphImportService, ImportFormat},
    value_objects::GraphId,
};

#[test]
fn test_import_graph_command_generates_event() {
    // This test would have caught that ImportGraph commands were returning None
    let graph_id = GraphId::new();
    let mut graph = Graph::new(graph_id);

    let import_cmd = Command::Graph(GraphCommand::ImportGraph {
        graph_id,
        source: ImportSource::File {
            path: "test.json".to_string(),
        },
        format: "arrows".to_string(),
        options: ImportOptions {
            merge_behavior: MergeBehavior::AlwaysCreate,
            id_prefix: None,
            position_offset: None,
            mapping: None,
            validate: true,
            max_nodes: None,
        },
    });

    // The domain aggregate should handle the command
    let result = graph.handle_command(import_cmd);

    assert!(
        result.is_ok(),
        "ImportGraph command should be handled successfully"
    );

    if let Ok(events) = result {
        assert!(
            !events.is_empty(),
            "ImportGraph command should generate events"
        );

        let has_import_event = events.iter().any(|event| {
            matches!(
                event,
                DomainEvent::Graph(GraphEvent::GraphImportRequested { .. })
            )
        });

        assert!(
            has_import_event,
            "ImportGraph command should generate GraphImportRequested event"
        );
    }
}

#[test]
fn test_all_graph_commands_are_handled() {
    // This test ensures no command returns error unexpectedly
    let graph_id = GraphId::new();
    let mut graph = Graph::new(graph_id);

    let test_cases = vec![
        Command::Graph(GraphCommand::CreateGraph {
            id: graph_id,
            name: "Test".to_string(),
            metadata: Default::default(),
        }),
        Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::File {
                path: "test.json".to_string(),
            },
            format: "arrows".to_string(),
            options: ImportOptions {
                merge_behavior: MergeBehavior::AlwaysCreate,
                id_prefix: None,
                position_offset: None,
                mapping: None,
                validate: true,
                max_nodes: None,
            },
        }),
    ];

    for cmd in test_cases {
        let cmd_name = match &cmd {
            Command::Graph(GraphCommand::CreateGraph { .. }) => "CreateGraph",
            Command::Graph(GraphCommand::ImportGraph { .. }) => "ImportGraph",
            _ => "Other",
        };

        let result = graph.handle_command(cmd);
        assert!(
            result.is_ok(),
            "{} command should be handled successfully",
            cmd_name
        );
    }
}
