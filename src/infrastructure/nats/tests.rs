//! Tests for NATS integration

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use super::super::*;
    use tokio;
    use crate::domain::events::{DomainEvent, GraphEvent};
    use crate::domain::value_objects::GraphId;

    #[tokio::test]
    async fn test_connect_to_localhost() {
        // This test assumes NATS is running on localhost:4222
        let config = NatsConfig::localhost();

        match NatsClient::new(config).await {
            Ok(client) => {
                println!("Successfully connected to NATS");

                // Test health check
                assert!(client.health_check().await.is_ok());
            }
            Err(e) => {
                // Don't fail the test if NATS isn't running
                println!("Could not connect to NATS (expected if not running): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_publish_subscribe() {
        let config = NatsConfig::localhost();

        if let Ok(client) = NatsClient::new(config).await {
            // Subscribe to a test subject
            let mut subscriber = client.subscribe("test.subject").await.unwrap();

            // Publish a message
            let payload = b"Hello, NATS!".to_vec();
            client.publish("test.subject", payload.clone()).await.unwrap();

            // Receive the message
            if let Some(msg) = subscriber.next().await {
                assert_eq!(msg.payload.to_vec(), payload);
            } else {
                panic!("Did not receive message");
            }
        } else {
            println!("Skipping test - NATS not available");
        }
    }

    #[test]
    fn test_config_defaults() {
        let config = NatsConfig::default();
        assert_eq!(config.url, "nats://localhost:4222");
        assert_eq!(config.client_name, "cim-client");
        assert!(config.jetstream.enabled);
        assert!(!config.security.tls_enabled);
    }

    #[tokio::test]
    async fn test_domain_event_serialization() {
        let graph_id = GraphId::new();
        let event = DomainEvent::Graph(GraphEvent::GraphCreated {
            id: graph_id,
            metadata: Default::default(),
        });

        // Serialize event
        let json = serde_json::to_string(&event).expect("Failed to serialize");

        // Deserialize event
        let deserialized: DomainEvent = serde_json::from_str(&json)
            .expect("Failed to deserialize");

        match deserialized {
            DomainEvent::Graph(GraphEvent::GraphCreated { id, .. }) => {
                assert_eq!(id, graph_id);
            }
            _ => panic!("Wrong event type"),
        }
    }
}
