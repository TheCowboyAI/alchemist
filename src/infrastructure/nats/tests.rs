//! Tests for NATS infrastructure

#[cfg(test)]
use super::*;
use crate::domain::events::{DomainEvent, GraphEvent};
use crate::domain::value_objects::GraphId;

#[tokio::test]
async fn test_nats_connection() {
    // This test requires NATS to be running
    // It will be skipped in CI unless NATS is available

    let config = NatsConfig::localhost();

    match NatsClient::new(config).await {
        Ok(client) => {
            // Test health check
            let health = client.health_check().await;
            assert!(health.is_ok());
        }
        Err(e) => {
            // This is expected if NATS is not running
            println!("Could not connect to NATS (expected if not running): {e}");
        }
    }
}

#[tokio::test]
async fn test_event_publishing() {
    // Skip if NATS not available
    let config = NatsConfig::localhost();
    let client = match NatsClient::new(config).await {
        Ok(c) => c,
        Err(_) => return, // Skip test if NATS not running
    };

    // Create a test event
    let graph_id = GraphId::new();
    let event = DomainEvent::Graph(GraphEvent::GraphCreated {
        id: graph_id,
        metadata: crate::domain::value_objects::GraphMetadata::new("Test Graph".to_string()),
    });

    // Serialize and publish event
    let payload = serde_json::to_vec(&event).unwrap();
    let result = client
        .publish("test.graph.created", payload)
        .await;

    assert!(result.is_ok());
}

#[test]
fn test_config_defaults() {
    let config = NatsConfig::default();
    assert_eq!(config.url, "nats://localhost:4222");
    assert_eq!(config.client_name, "cim-client");
    assert!(config.jetstream.enabled);
}

#[test]
fn test_production_config() {
    let config = NatsConfig::production("nats://prod.example.com:4222".to_string());
    assert_eq!(config.url, "nats://prod.example.com:4222");
    assert!(config.security.tls_enabled);
    assert!(config.client_name.starts_with("cim-client-"));
}
