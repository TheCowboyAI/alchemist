//! Graph Event Tests

use ia::domain::{
    events::{DomainEvent, GraphEvent},
    value_objects::{GraphId, GraphMetadata},
};

#[test]
fn test_graph_created_event() {
    // Given
    let id = GraphId::new();
    let metadata = GraphMetadata::new("Test Graph".to_string());

    // When
    let event = GraphEvent::GraphCreated {
        id,
        metadata: metadata.clone(),
    };

    // Then
    match event {
        GraphEvent::GraphCreated {
            id: event_id,
            metadata: event_metadata,
        } => {
            assert_eq!(event_id, id);
            assert_eq!(event_metadata.name, metadata.name);
        }
        _ => panic!("Expected GraphCreated event"),
    }
}

#[test]
fn test_graph_renamed_event() {
    // Given
    let id = GraphId::new();
    let old_name = "Old Name".to_string();
    let new_name = "New Name".to_string();

    // When
    let event = GraphEvent::GraphRenamed {
        id,
        old_name: old_name.clone(),
        new_name: new_name.clone(),
    };

    // Then
    match event {
        GraphEvent::GraphRenamed {
            id: event_id,
            old_name: event_old,
            new_name: event_new,
        } => {
            assert_eq!(event_id, id);
            assert_eq!(event_old, old_name);
            assert_eq!(event_new, new_name);
        }
        _ => panic!("Expected GraphRenamed event"),
    }
}

#[test]
fn test_graph_tagged_event() {
    // Given
    let id = GraphId::new();
    let tag = "important".to_string();

    // When
    let event = GraphEvent::GraphTagged {
        id,
        tag: tag.clone(),
    };

    // Then
    match event {
        GraphEvent::GraphTagged {
            id: event_id,
            tag: event_tag,
        } => {
            assert_eq!(event_id, id);
            assert_eq!(event_tag, tag);
        }
        _ => panic!("Expected GraphTagged event"),
    }
}

#[test]
fn test_graph_untagged_event() {
    // Given
    let id = GraphId::new();
    let tag = "obsolete".to_string();

    // When
    let event = GraphEvent::GraphUntagged {
        id,
        tag: tag.clone(),
    };

    // Then
    match event {
        GraphEvent::GraphUntagged {
            id: event_id,
            tag: event_tag,
        } => {
            assert_eq!(event_id, id);
            assert_eq!(event_tag, tag);
        }
        _ => panic!("Expected GraphUntagged event"),
    }
}

#[test]
fn test_graph_deleted_event() {
    // Given
    let id = GraphId::new();

    // When
    let event = GraphEvent::GraphDeleted { id };

    // Then
    match event {
        GraphEvent::GraphDeleted { id: event_id } => {
            assert_eq!(event_id, id);
        }
        _ => panic!("Expected GraphDeleted event"),
    }
}

#[test]
fn test_domain_event_wrapping() {
    // Given
    let id = GraphId::new();
    let graph_event = GraphEvent::GraphCreated {
        id,
        metadata: GraphMetadata::new("Test".to_string()),
    };

    // When
    let domain_event = DomainEvent::Graph(graph_event);

    // Then
    match domain_event {
        DomainEvent::Graph(GraphEvent::GraphCreated { .. }) => {
            // Success - event is properly wrapped
        }
        _ => panic!("Expected wrapped GraphCreated event"),
    }
}

#[test]
fn test_event_serialization() {
    // Given
    let id = GraphId::new();
    let event = GraphEvent::GraphCreated {
        id,
        metadata: GraphMetadata::new("Serializable".to_string()),
    };
    let domain_event = DomainEvent::Graph(event);

    // When
    let serialized = serde_json::to_string(&domain_event).unwrap();
    let deserialized: DomainEvent = serde_json::from_str(&serialized).unwrap();

    // Then
    match deserialized {
        DomainEvent::Graph(GraphEvent::GraphCreated {
            id: event_id,
            metadata,
        }) => {
            assert_eq!(event_id, id);
            assert_eq!(metadata.name, "Serializable");
        }
        _ => panic!("Expected deserialized GraphCreated event"),
    }
}

#[test]
fn test_graph_metadata_creation() {
    // Given
    let name = "Test Metadata".to_string();

    // When
    let metadata = GraphMetadata::new(name.clone());

    // Then
    assert_eq!(metadata.name, name);
    assert!(metadata.tags.is_empty());
}

#[test]
fn test_graph_metadata_with_tags() {
    // Given
    let name = "Tagged Graph".to_string();
    let mut metadata = GraphMetadata::new(name.clone());

    // When
    metadata.tags.push("tag1".to_string());
    metadata.tags.push("tag2".to_string());

    // Then
    assert_eq!(metadata.name, name);
    assert_eq!(metadata.tags.len(), 2);
    assert!(metadata.tags.contains(&"tag1".to_string()));
    assert!(metadata.tags.contains(&"tag2".to_string()));
}

#[test]
fn test_event_equality() {
    // Given
    let id = GraphId::new();
    let metadata = GraphMetadata::new("Test".to_string());

    // When
    let event1 = GraphEvent::GraphCreated {
        id,
        metadata: metadata.clone(),
    };
    let event2 = GraphEvent::GraphCreated {
        id,
        metadata: metadata.clone(),
    };

    // Then - events with same data should be equal
    // Note: This assumes GraphEvent derives PartialEq
    let json1 = serde_json::to_string(&event1).unwrap();
    let json2 = serde_json::to_string(&event2).unwrap();
    assert_eq!(json1, json2);
}
