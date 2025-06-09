//! Graph Command Tests

use ia::domain::{
    commands::graph_commands::GraphCommand,
    value_objects::GraphId,
};

#[test]
fn test_create_graph_command() {
    // Given
    let id = GraphId::new();
    let name = "Test Graph".to_string();

    // When
    let command = GraphCommand::CreateGraph {
        id,
        name: name.clone()
    };

    // Then
    assert_eq!(command.command_type(), "CreateGraph");
    assert_eq!(command.graph_id(), id);

    match command {
        GraphCommand::CreateGraph { id: cmd_id, name: cmd_name } => {
            assert_eq!(cmd_id, id);
            assert_eq!(cmd_name, name);
        }
        _ => panic!("Expected CreateGraph command"),
    }
}

#[test]
fn test_delete_graph_command() {
    // Given
    let id = GraphId::new();

    // When
    let command = GraphCommand::DeleteGraph { id };

    // Then
    assert_eq!(command.command_type(), "DeleteGraph");
    assert_eq!(command.graph_id(), id);
}

#[test]
fn test_rename_graph_command() {
    // Given
    let id = GraphId::new();
    let new_name = "Renamed Graph".to_string();

    // When
    let command = GraphCommand::RenameGraph {
        id,
        new_name: new_name.clone()
    };

    // Then
    assert_eq!(command.command_type(), "RenameGraph");
    assert_eq!(command.graph_id(), id);

    match command {
        GraphCommand::RenameGraph { id: cmd_id, new_name: cmd_name } => {
            assert_eq!(cmd_id, id);
            assert_eq!(cmd_name, new_name);
        }
        _ => panic!("Expected RenameGraph command"),
    }
}

#[test]
fn test_tag_graph_command() {
    // Given
    let id = GraphId::new();
    let tag = "important".to_string();

    // When
    let command = GraphCommand::TagGraph {
        id,
        tag: tag.clone()
    };

    // Then
    assert_eq!(command.command_type(), "TagGraph");
    assert_eq!(command.graph_id(), id);

    match command {
        GraphCommand::TagGraph { id: cmd_id, tag: cmd_tag } => {
            assert_eq!(cmd_id, id);
            assert_eq!(cmd_tag, tag);
        }
        _ => panic!("Expected TagGraph command"),
    }
}

#[test]
fn test_untag_graph_command() {
    // Given
    let id = GraphId::new();
    let tag = "obsolete".to_string();

    // When
    let command = GraphCommand::UntagGraph {
        id,
        tag: tag.clone()
    };

    // Then
    assert_eq!(command.command_type(), "UntagGraph");
    assert_eq!(command.graph_id(), id);

    match command {
        GraphCommand::UntagGraph { id: cmd_id, tag: cmd_tag } => {
            assert_eq!(cmd_id, id);
            assert_eq!(cmd_tag, tag);
        }
        _ => panic!("Expected UntagGraph command"),
    }
}

#[test]
fn test_command_serialization() {
    // Given
    let id = GraphId::new();
    let command = GraphCommand::CreateGraph {
        id,
        name: "Serializable Graph".to_string()
    };

    // When
    let serialized = serde_json::to_string(&command).unwrap();
    let deserialized: GraphCommand = serde_json::from_str(&serialized).unwrap();

    // Then
    assert_eq!(deserialized.command_type(), command.command_type());
    assert_eq!(deserialized.graph_id(), command.graph_id());
}

#[test]
fn test_all_commands_have_graph_id() {
    // Given
    let id = GraphId::new();
    let commands = vec![
        GraphCommand::CreateGraph { id, name: "Test".to_string() },
        GraphCommand::DeleteGraph { id },
        GraphCommand::RenameGraph { id, new_name: "New".to_string() },
        GraphCommand::TagGraph { id, tag: "tag".to_string() },
        GraphCommand::UntagGraph { id, tag: "tag".to_string() },
    ];

    // When/Then
    for command in commands {
        assert_eq!(command.graph_id(), id);
    }
}

#[test]
fn test_command_type_strings_are_unique() {
    // Given
    let id = GraphId::new();
    let commands = vec![
        GraphCommand::CreateGraph { id, name: "Test".to_string() },
        GraphCommand::DeleteGraph { id },
        GraphCommand::RenameGraph { id, new_name: "New".to_string() },
        GraphCommand::TagGraph { id, tag: "tag".to_string() },
        GraphCommand::UntagGraph { id, tag: "tag".to_string() },
    ];

    // When
    let command_types: Vec<&str> = commands.iter()
        .map(|cmd| cmd.command_type())
        .collect();

    // Then - all command types should be unique
    let unique_types: std::collections::HashSet<&str> = command_types.iter().cloned().collect();
    assert_eq!(command_types.len(), unique_types.len());
}
