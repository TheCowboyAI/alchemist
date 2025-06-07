//! Tests for the event bridge

#[cfg(test)]
use super::*;
use crate::domain::events::{DomainEvent, GraphEvent};
use crate::domain::value_objects::{GraphId, GraphMetadata};
use crate::domain::commands::Command;
use crossbeam_channel::bounded;

#[test]
fn test_event_bridge_creation() {
    let bridge = EventBridge::new();
    // Bridge should be created successfully
    assert!(bridge.send_command(BridgeCommand::Subscribe("test.subject".to_string())).is_ok());
}

#[test]
fn test_command_handling() {
    let bridge = EventBridge::new();

    // Test various commands
    assert!(bridge.send_command(BridgeCommand::Subscribe("test.subject".to_string())).is_ok());
    assert!(bridge.send_command(BridgeCommand::Unsubscribe("test.subject".to_string())).is_ok());
}

#[test]
fn test_event_receiving() {
    let bridge = EventBridge::new();

    // Send multiple commands
    for i in 0..5 {
        let cmd = BridgeCommand::Subscribe(format!("test.subject.{i}"));
        bridge.send_command(cmd).unwrap();
    }

    // Should return empty vec when no events
    let events = bridge.receive_events();
    assert!(events.is_empty());
}

#[test]
fn test_event_bridge_receive_empty() {
    let bridge = EventBridge::new();

    // Should return empty vec when no events
    let events = bridge.receive_events();
    assert!(events.is_empty());
}

#[test]
fn test_command_channel_communication() {
    // Create channels directly without EventBridge to avoid runtime issues
    let (tx, rx) = bounded::<BridgeCommand>(1000);

    // Send multiple commands
    for i in 0..10 {
        let cmd = BridgeCommand::Subscribe(format!("test.subject.{i}"));
        assert!(tx.send(cmd).is_ok());
    }

    // Verify we can receive them
    let mut count = 0;
    while rx.try_recv().is_ok() {
        count += 1;
    }
    assert_eq!(count, 10);
}

#[test]
fn test_bridge_event_serialization() {
    // Test that bridge events can be serialized/deserialized
    let event = BridgeEvent::DomainEvent(DomainEvent::Graph(GraphEvent::GraphCreated {
        id: GraphId::new(),
        metadata: GraphMetadata::new("Test Graph".to_string()),
    }));

    let serialized = serde_json::to_string(&event).unwrap();
    let deserialized: BridgeEvent = serde_json::from_str(&serialized).unwrap();

    match deserialized {
        BridgeEvent::DomainEvent(DomainEvent::Graph(GraphEvent::GraphCreated {
            metadata,
            ..
        })) => {
            assert_eq!(metadata.name, "Test Graph");
        }
        _ => panic!("Unexpected event type"),
    }
}

#[test]
fn test_connection_status_events() {
    let event = BridgeEvent::ConnectionStatus(ConnectionStatus::Connected);
    let serialized = serde_json::to_string(&event).unwrap();
    assert!(serialized.contains("Connected"));

    let event = BridgeEvent::ConnectionStatus(ConnectionStatus::Disconnected);
    let serialized = serde_json::to_string(&event).unwrap();
    assert!(serialized.contains("Disconnected"));
}

#[test]
fn test_error_events() {
    let event = BridgeEvent::Error("Test error message".to_string());
    let serialized = serde_json::to_string(&event).unwrap();
    assert!(serialized.contains("Test error message"));
}

#[test]
fn test_bridge_command_types() {
    // Test all command types can be created and serialized
    let commands = vec![
        BridgeCommand::PublishEvent(DomainEvent::Graph(GraphEvent::GraphCreated {
            id: GraphId::new(),
            metadata: GraphMetadata::new("Test".to_string()),
        })),
        BridgeCommand::ExecuteCommand(Command::Graph(
            crate::domain::commands::GraphCommand::CreateGraph {
                id: GraphId::new(),
                name: "Test".to_string(),
                metadata: std::collections::HashMap::new(),
            },
        )),
        BridgeCommand::Subscribe("test.subject".to_string()),
        BridgeCommand::Unsubscribe("test.subject".to_string()),
    ];

    for cmd in commands {
        let serialized = serde_json::to_string(&cmd).unwrap();
        let deserialized: BridgeCommand = serde_json::from_str(&serialized).unwrap();

        // Verify roundtrip serialization works
        let reserialized = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(serialized, reserialized);
    }
}

#[test]
fn test_channel_capacity() {
    let bridge = EventBridge::new();

    // Test that we can send up to channel capacity without blocking
    for i in 0..1000 {
        let result = bridge.send_command(BridgeCommand::Subscribe(format!("test.{i}")));
        assert!(result.is_ok(), "Failed to send command {i}");
    }
}
