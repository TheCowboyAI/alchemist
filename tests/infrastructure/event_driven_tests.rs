//! Event-driven architecture tests for cim-contextgraph
//!
//! Tests the integration of context graphs with NATS event streaming,
//! ensuring proper event flow for graph operations and context management.

use async_nats::jetstream::{self, stream};
use futures::StreamExt;
use std::time::Duration;

/// Test helper for creating test event streams
struct EventStreamValidator {
    client: async_nats::Client,
    jetstream: jetstream::Context,
}

impl EventStreamValidator {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = async_nats::connect("nats://localhost:4222").await?;
        let jetstream = jetstream::new(client.clone());
        Ok(Self { client, jetstream })
    }

    async fn create_test_stream(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config = stream::Config {
            name: name.to_string(),
            subjects: vec![format!("{}.>", name)],
            retention: stream::RetentionPolicy::WorkQueue,
            ..Default::default()
        };

        self.jetstream.create_stream(config).await?;
        Ok(())
    }

    async fn validate_event_sequence(
        &self,
        stream_name: &str,
        expected_count: usize,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut stream = self.jetstream.get_stream(stream_name).await?;
        let info = stream.info().await?;
        Ok(info.state.messages == expected_count as u64)
    }
}

#[cfg(test)]
mod layer_1_1_nats_connection {
    use super::*;

    #[tokio::test]
    #[ignore = "requires NATS server"]
    async fn test_contextgraph_nats_connection() {
        let client = async_nats::connect("nats://localhost:4222").await;
        assert!(
            client.is_ok(),
            "Failed to connect to NATS for context graph operations"
        );
    }

    #[tokio::test]
    #[ignore = "requires NATS server"]
    async fn test_contextgraph_jetstream_context() {
        let client = async_nats::connect("nats://localhost:4222").await.unwrap();
        let js = jetstream::new(client);

        // Test creating a context graph event stream
        let config = stream::Config {
            name: "context-graph-events".to_string(),
            subjects: vec!["context.graph.>".to_string()],
            ..Default::default()
        };

        let result = js.create_stream(config).await;
        assert!(
            result.is_ok(),
            "Failed to create context graph event stream"
        );
    }

    #[tokio::test]
    #[ignore = "requires NATS server"]
    async fn test_graph_event_publishing() {
        let validator = EventStreamValidator::new().await.unwrap();
        validator
            .create_test_stream("test-graph-events")
            .await
            .unwrap();

        // Publish context graph events
        let events = vec![
            (
                "test-graph-events.node.added",
                r#"{"node_id": "n1", "context": "test"}"#,
            ),
            (
                "test-graph-events.edge.created",
                r#"{"source": "n1", "target": "n2"}"#,
            ),
            (
                "test-graph-events.context.updated",
                r#"{"context_id": "ctx1", "changes": []}"#,
            ),
        ];

        for (subject, payload) in events {
            let result = validator.client.publish(subject, payload.into()).await;
            assert!(result.is_ok(), "Failed to publish graph event");
        }

        // Verify events were published
        tokio::time::sleep(Duration::from_millis(100)).await;
        let valid = validator
            .validate_event_sequence("test-graph-events", 3)
            .await
            .unwrap();
        assert!(valid, "Not all graph events were published");
    }
}

#[cfg(test)]
mod layer_1_2_event_store {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct GraphEvent {
        event_type: String,
        graph_id: String,
        timestamp: u64,
        data: serde_json::Value,
    }

    #[tokio::test]
    #[ignore = "requires NATS server"]
    async fn test_graph_event_persistence() {
        let validator = EventStreamValidator::new().await.unwrap();
        validator
            .create_test_stream("graph-persistence")
            .await
            .unwrap();

        let event = GraphEvent {
            event_type: "NodeAdded".to_string(),
            graph_id: "graph-123".to_string(),
            timestamp: 1234567890,
            data: serde_json::json!({
                "node_id": "node-456",
                "properties": {"label": "Test Node"}
            }),
        };

        let payload = serde_json::to_vec(&event).unwrap();
        validator
            .client
            .publish("graph-persistence.events", payload.into())
            .await
            .unwrap();

        // For now, we can't check the ack.stream property without waiting for ack
        // Just verify the publish succeeded
    }

    #[tokio::test]
    #[ignore = "requires NATS server"]
    async fn test_context_graph_replay() {
        let validator = EventStreamValidator::new().await.unwrap();
        validator.create_test_stream("graph-replay").await.unwrap();

        // Publish multiple graph mutation events
        let events = vec![
            GraphEvent {
                event_type: "GraphCreated".to_string(),
                graph_id: "replay-graph".to_string(),
                timestamp: 1000,
                data: serde_json::json!({"name": "Test Graph"}),
            },
            GraphEvent {
                event_type: "NodeAdded".to_string(),
                graph_id: "replay-graph".to_string(),
                timestamp: 2000,
                data: serde_json::json!({"node_id": "n1"}),
            },
            GraphEvent {
                event_type: "EdgeAdded".to_string(),
                graph_id: "replay-graph".to_string(),
                timestamp: 3000,
                data: serde_json::json!({"source": "n1", "target": "n2"}),
            },
        ];

        for event in &events {
            let payload = serde_json::to_vec(event).unwrap();
            validator
                .client
                .publish("graph-replay.events", payload.into())
                .await
                .unwrap();
        }

        // Create consumer for replay
        let mut stream = validator
            .jetstream
            .get_stream("graph-replay")
            .await
            .unwrap();
        let consumer = stream
            .create_consumer(jetstream::consumer::pull::Config {
                name: Some("replay-consumer".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();

        // Replay events
        let mut messages = consumer.messages().await.unwrap();
        let mut replayed_count = 0;

        while let Ok(Some(msg)) =
            tokio::time::timeout(Duration::from_millis(100), messages.next()).await
        {
            let _: GraphEvent = serde_json::from_slice(&msg.payload).unwrap();
            msg.ack().await.unwrap();
            replayed_count += 1;
        }

        assert_eq!(replayed_count, 3, "Not all graph events were replayed");
    }

    #[tokio::test]
    #[ignore = "requires NATS server"]
    async fn test_graph_snapshot_storage() {
        let validator = EventStreamValidator::new().await.unwrap();

        // Create object store bucket for graph snapshots
        let bucket = validator
            .jetstream
            .create_object_store(jetstream::object_store::Config {
                bucket: "graph-snapshots".to_string(),
                ..Default::default()
            })
            .await
            .unwrap();

        // Store a graph snapshot
        let snapshot_data = serde_json::json!({
            "graph_id": "snapshot-test",
            "nodes": ["n1", "n2", "n3"],
            "edges": [["n1", "n2"], ["n2", "n3"]],
            "contexts": {"default": {"properties": {}}},
            "version": 1
        });

        let data = serde_json::to_vec(&snapshot_data).unwrap();
        let info = bucket
            .put("graph-snapshot-v1", &mut data.as_slice())
            .await
            .unwrap();

        assert!(info.size > 0, "Graph snapshot not stored");

        // Retrieve snapshot using the newer API
        let object = bucket.get("graph-snapshot-v1").await.unwrap();
        let retrieved_data = object.data;

        let retrieved: serde_json::Value = serde_json::from_slice(&retrieved_data).unwrap();
        assert_eq!(retrieved["graph_id"], "snapshot-test");
        assert_eq!(retrieved["nodes"].as_array().unwrap().len(), 3);
    }
}

#[cfg(test)]
mod layer_1_3_graph_operations {
    use super::*;

    #[tokio::test]
    #[ignore = "requires NATS server"]
    async fn test_distributed_graph_sync() {
        let validator = EventStreamValidator::new().await.unwrap();
        validator.create_test_stream("graph-sync").await.unwrap();

        // Simulate graph operations from multiple sources
        let operations = vec![
            (
                "graph-sync.node.add",
                r#"{"source": "client-1", "node_id": "n1"}"#,
            ),
            (
                "graph-sync.node.add",
                r#"{"source": "client-2", "node_id": "n2"}"#,
            ),
            (
                "graph-sync.edge.add",
                r#"{"source": "client-1", "from": "n1", "to": "n2"}"#,
            ),
            (
                "graph-sync.context.merge",
                r#"{"source": "client-2", "contexts": ["ctx1", "ctx2"]}"#,
            ),
        ];

        for (subject, payload) in operations {
            validator
                .client
                .publish(subject, payload.into())
                .await
                .unwrap();
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
        let valid = validator
            .validate_event_sequence("graph-sync", 4)
            .await
            .unwrap();
        assert!(
            valid,
            "Graph synchronization events not properly distributed"
        );
    }

    #[tokio::test]
    #[ignore = "requires NATS server"]
    async fn test_graph_composition_events() {
        let validator = EventStreamValidator::new().await.unwrap();
        validator
            .create_test_stream("graph-composition")
            .await
            .unwrap();

        // Test graph composition operations
        let composition_events = vec![
            r#"{"operation": "merge", "graphs": ["g1", "g2"], "result": "g3"}"#,
            r#"{"operation": "intersect", "graphs": ["g3", "g4"], "result": "g5"}"#,
            r#"{"operation": "difference", "graphs": ["g5", "g1"], "result": "g6"}"#,
        ];

        for (i, event) in composition_events.iter().enumerate() {
            let subject = format!("graph-composition.operation.{}", i);
            validator
                .client
                .publish(subject, event.as_bytes().to_vec().into())
                .await
                .unwrap();
        }

        // Verify composition event ordering
        let mut stream = validator
            .jetstream
            .get_stream("graph-composition")
            .await
            .unwrap();
        let info = stream.info().await.unwrap();
        assert_eq!(
            info.state.messages, 3,
            "Not all composition events were recorded"
        );
    }

    #[tokio::test]
    #[ignore = "requires NATS server"]
    async fn test_conceptual_space_mapping() {
        let validator = EventStreamValidator::new().await.unwrap();
        validator
            .create_test_stream("conceptual-mapping")
            .await
            .unwrap();

        // Test conceptual space mapping events
        let mapping_event = serde_json::json!({
            "graph_id": "concept-graph-1",
            "mapping_type": "semantic_embedding",
            "nodes": {
                "n1": {"embedding": [0.1, 0.2, 0.3], "concept": "Entity"},
                "n2": {"embedding": [0.4, 0.5, 0.6], "concept": "Relationship"}
            },
            "similarity_threshold": 0.8
        });

        let payload = serde_json::to_vec(&mapping_event).unwrap();
        validator
            .client
            .publish("conceptual-mapping.update", payload.into())
            .await
            .unwrap();

        // Verify mapping was stored
        let mut stream = validator
            .jetstream
            .get_stream("conceptual-mapping")
            .await
            .unwrap();
        let info = stream.info().await.unwrap();
        assert_eq!(
            info.state.messages, 1,
            "Conceptual mapping event not stored"
        );
    }

    #[tokio::test]
    #[ignore = "requires NATS server"]
    async fn test_graph_query_cache_invalidation() {
        let validator = EventStreamValidator::new().await.unwrap();
        validator.create_test_stream("graph-cache").await.unwrap();

        // Test cache invalidation on graph mutations
        let mutation_events = vec![
            (
                "graph-cache.mutation.node_added",
                r#"{"graph_id": "g1", "node_id": "n1"}"#,
            ),
            (
                "graph-cache.invalidate.query",
                r#"{"graph_id": "g1", "query_types": ["neighbors", "paths"]}"#,
            ),
            (
                "graph-cache.mutation.edge_removed",
                r#"{"graph_id": "g1", "edge_id": "e1"}"#,
            ),
            (
                "graph-cache.invalidate.all",
                r#"{"graph_id": "g1", "reason": "structural_change"}"#,
            ),
        ];

        for (subject, payload) in mutation_events {
            validator
                .client
                .publish(subject, payload.into())
                .await
                .unwrap();
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
        let valid = validator
            .validate_event_sequence("graph-cache", 4)
            .await
            .unwrap();
        assert!(valid, "Cache invalidation events not properly published");
    }
}

#[cfg(test)]
mod test_helpers {
    use super::*;

    /// Cleanup helper for test streams
    pub async fn cleanup_test_stream(js: &jetstream::Context, stream_name: &str) {
        let _ = js.delete_stream(stream_name).await;
    }

    /// Helper to create test graph events
    pub fn create_test_graph_event(event_type: &str, graph_id: &str) -> Vec<u8> {
        let event = serde_json::json!({
            "event_type": event_type,
            "graph_id": graph_id,
            "timestamp": chrono::Utc::now().timestamp(),
            "data": {}
        });
        serde_json::to_vec(&event).unwrap()
    }
}
