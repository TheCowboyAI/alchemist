//! Graph Aggregate Tests - Testing domain logic without infrastructure

use ia::domain::{
    aggregates::graph::Graph,
    events::{DomainEvent, GraphEvent},
    value_objects::{GraphId, GraphMetadata},
};

#[test]
fn test_graph_creation_with_command() {
    // Given
    let id = GraphId::new();
    let name = "Test Graph".to_string();

    // When
    let graph = Graph::new(id, name.clone(), None);

    // Then
    assert_eq!(graph.id, id);
    assert_eq!(graph.metadata.name, name);
    assert_eq!(graph.version, 0);

    // Verify event was generated
    let events = graph.get_uncommitted_events();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Graph(GraphEvent::GraphCreated { id: event_id, metadata }) => {
            assert_eq!(*event_id, id);
            assert_eq!(metadata.name, name);
        }
        _ => panic!("Expected GraphCreated event"),
    }
}

#[test]
fn test_graph_rename_generates_correct_event() {
    // Given
    let id = GraphId::new();
    let mut graph = Graph::new(id, "Original".to_string(), None);
    graph.mark_events_as_committed();

    // When
    graph.update_metadata(Some("New Name".to_string()), None);

    // Then
    assert_eq!(graph.metadata.name, "New Name");

    let events = graph.get_uncommitted_events();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Graph(GraphEvent::GraphRenamed { id: event_id, old_name, new_name }) => {
            assert_eq!(*event_id, id);
            assert_eq!(old_name, "Original");
            assert_eq!(new_name, "New Name");
        }
        _ => panic!("Expected GraphRenamed event"),
    }
}

#[test]
fn test_graph_tagging_operations() {
    // Given
    let id = GraphId::new();
    let mut graph = Graph::new(id, "Tagged Graph".to_string(), Some("initial-tag".to_string()));
    graph.mark_events_as_committed();

    // When - add tags
    graph.update_metadata(None, Some("tag1".to_string()));
    graph.update_metadata(None, Some("tag2".to_string()));

    // Then
    assert_eq!(graph.metadata.tags.len(), 3); // initial-tag + tag1 + tag2
    assert!(graph.metadata.tags.contains(&"initial-tag".to_string()));
    assert!(graph.metadata.tags.contains(&"tag1".to_string()));
    assert!(graph.metadata.tags.contains(&"tag2".to_string()));

    let events = graph.get_uncommitted_events();
    assert_eq!(events.len(), 2);

    for event in &events {
        match event {
            DomainEvent::Graph(GraphEvent::GraphTagged { .. }) => {},
            _ => panic!("Expected GraphTagged events"),
        }
    }
}

#[test]
fn test_graph_event_sourcing_rebuild() {
    // Given - a series of events
    let id = GraphId::new();
    let events = vec![
        DomainEvent::Graph(GraphEvent::GraphCreated {
            id,
            metadata: GraphMetadata::new("Initial".to_string()),
        }),
        DomainEvent::Graph(GraphEvent::GraphRenamed {
            id,
            old_name: "Initial".to_string(),
            new_name: "Renamed".to_string(),
        }),
        DomainEvent::Graph(GraphEvent::GraphTagged {
            id,
            tag: "important".to_string(),
        }),
        DomainEvent::Graph(GraphEvent::GraphTagged {
            id,
            tag: "production".to_string(),
        }),
    ];

    // When - rebuild from events
    let graph = Graph::from_events(id, events);

    // Then
    assert_eq!(graph.id, id);
    assert_eq!(graph.metadata.name, "Renamed");
    assert_eq!(graph.metadata.tags.len(), 2);
    assert!(graph.metadata.tags.contains(&"important".to_string()));
    assert!(graph.metadata.tags.contains(&"production".to_string()));
    assert_eq!(graph.get_uncommitted_events().len(), 0);
}

#[test]
fn test_graph_untag_removes_specific_tag() {
    // Given
    let id = GraphId::new();
    let events = vec![
        DomainEvent::Graph(GraphEvent::GraphCreated {
            id,
            metadata: GraphMetadata::new("Graph".to_string()),
        }),
        DomainEvent::Graph(GraphEvent::GraphTagged {
            id,
            tag: "keep-me".to_string(),
        }),
        DomainEvent::Graph(GraphEvent::GraphTagged {
            id,
            tag: "remove-me".to_string(),
        }),
        DomainEvent::Graph(GraphEvent::GraphUntagged {
            id,
            tag: "remove-me".to_string(),
        }),
    ];

    // When
    let graph = Graph::from_events(id, events);

    // Then
    assert_eq!(graph.metadata.tags.len(), 1);
    assert!(graph.metadata.tags.contains(&"keep-me".to_string()));
    assert!(!graph.metadata.tags.contains(&"remove-me".to_string()));
}

#[test]
fn test_graph_event_ordering_matters() {
    // Given - same events in different order
    let id = GraphId::new();

    // Scenario 1: Rename then tag
    let events1 = vec![
        DomainEvent::Graph(GraphEvent::GraphCreated {
            id,
            metadata: GraphMetadata::new("Original".to_string()),
        }),
        DomainEvent::Graph(GraphEvent::GraphRenamed {
            id,
            old_name: "Original".to_string(),
            new_name: "Final".to_string(),
        }),
        DomainEvent::Graph(GraphEvent::GraphTagged {
            id,
            tag: "after-rename".to_string(),
        }),
    ];

    // Scenario 2: Tag then rename
    let events2 = vec![
        DomainEvent::Graph(GraphEvent::GraphCreated {
            id,
            metadata: GraphMetadata::new("Original".to_string()),
        }),
        DomainEvent::Graph(GraphEvent::GraphTagged {
            id,
            tag: "before-rename".to_string(),
        }),
        DomainEvent::Graph(GraphEvent::GraphRenamed {
            id,
            old_name: "Original".to_string(),
            new_name: "Final".to_string(),
        }),
    ];

    // When
    let graph1 = Graph::from_events(id, events1);
    let graph2 = Graph::from_events(id, events2);

    // Then - both end up with same name but different tags
    assert_eq!(graph1.metadata.name, "Final");
    assert_eq!(graph2.metadata.name, "Final");
    assert!(graph1.metadata.tags.contains(&"after-rename".to_string()));
    assert!(graph2.metadata.tags.contains(&"before-rename".to_string()));
}

#[test]
fn test_graph_idempotent_tag_operations() {
    // Given
    let id = GraphId::new();
    let mut graph = Graph::new(id, "Graph".to_string(), None);
    graph.mark_events_as_committed();

    // When - add same tag multiple times
    graph.update_metadata(None, Some("duplicate".to_string()));
    graph.update_metadata(None, Some("duplicate".to_string()));
    graph.update_metadata(None, Some("duplicate".to_string()));

    // Then - tag appears multiple times (not idempotent by design)
    assert_eq!(graph.metadata.tags.len(), 3);
    assert_eq!(graph.metadata.tags.iter().filter(|t| *t == &"duplicate".to_string()).count(), 3);

    // This shows that the aggregate doesn't enforce idempotency
    // This would need to be handled at the command handler level
}

#[test]
fn test_graph_empty_name_allowed() {
    // Given
    let id = GraphId::new();

    // When
    let graph = Graph::new(id, String::new(), None);

    // Then
    assert_eq!(graph.metadata.name, "");
    assert_eq!(graph.id, id);
}

#[test]
fn test_graph_with_description_becomes_tag() {
    // Given
    let id = GraphId::new();
    let description = "This is a test graph".to_string();

    // When
    let graph = Graph::new(id, "Test".to_string(), Some(description.clone()));

    // Then
    assert_eq!(graph.metadata.tags.len(), 1);
    assert_eq!(graph.metadata.tags[0], description);
}
