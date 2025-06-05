//! Tests for the event bridge

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::domain::events::{DomainEvent, GraphEvent};
    use crate::domain::value_objects::{GraphId, GraphMetadata};
    use std::time::Duration;
    use tokio::time::sleep;
    use crossbeam_channel::bounded;
    use tokio::sync::mpsc::unbounded_channel;

    #[test]
    fn test_event_bridge_creation() {
        let bridge = EventBridge::new();

        // Test that we can send commands
        let result = bridge.send_command(BridgeCommand::Subscribe("test.subject".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_event_bridge_receive_empty() {
        let bridge = EventBridge::new();

        // Should return empty vec when no events
        let events = bridge.receive_events();
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn test_command_channel_communication() {
        // Create channels directly without EventBridge to avoid runtime issues
        let (tx, rx) = bounded::<BridgeCommand>(1000);

        // Send multiple commands
        for i in 0..10 {
            let cmd = BridgeCommand::Subscribe(format!("test.subject.{}", i));
            assert!(tx.send(cmd).is_ok());
        }

        // Verify we can receive them
        let mut count = 0;
        while let Ok(_) = rx.try_recv() {
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
            let result = bridge.send_command(BridgeCommand::Subscribe(format!("test.{}", i)));
            assert!(result.is_ok(), "Failed to send command {}", i);
        }
    }

    #[tokio::test]
    async fn test_event_forwarding() {
        // Test event forwarding without creating EventBridge (which has its own runtime)
        let (event_tx, mut event_rx) = unbounded_channel::<BridgeEvent>();
        let (bevy_tx, bevy_rx) = bounded::<BridgeEvent>(1000);

        // Create test event
        let test_event = BridgeEvent::DomainEvent(DomainEvent::Graph(GraphEvent::GraphCreated {
            id: GraphId::new(),
            metadata: GraphMetadata::new("Forwarding Test".to_string()),
        }));

        // Send event
        event_tx.send(test_event.clone()).unwrap();

        // Simulate forwarding task
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                let _ = bevy_tx.send(event);
            }
        });

        // Give time for forwarding
        sleep(Duration::from_millis(10)).await;

        // Check if event was forwarded
        let forwarded = bevy_rx.try_recv();
        assert!(forwarded.is_ok());

        match forwarded.unwrap() {
            BridgeEvent::DomainEvent(DomainEvent::Graph(GraphEvent::GraphCreated {
                metadata,
                ..
            })) => {
                assert_eq!(metadata.name, "Forwarding Test");
            }
            _ => panic!("Unexpected event type"),
        }
    }
}
