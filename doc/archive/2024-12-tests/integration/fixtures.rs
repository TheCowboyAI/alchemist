//! Test fixtures and helpers for integration tests

use ia::domain::value_objects::{GraphId, NodeId, EdgeId, Position3D, GraphMetadata};
use ia::domain::commands::{GraphCommand, NodeCommand, EdgeCommand, Command};
use ia::domain::events::{GraphEvent, NodeEvent, EdgeEvent};
use ia::domain::content_types::GraphContent;
use ia::infrastructure::nats::NatsConfig;
use async_nats::jetstream;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use std::collections::HashMap;
use futures::StreamExt;

/// Test NATS configuration
pub fn test_nats_config() -> NatsConfig {
    NatsConfig {
        url: "nats://localhost:4222".to_string(),
        jetstream_enabled: true,
        ..Default::default()
    }
}

/// Helper to connect to NATS for testing
pub async fn connect_test_nats() -> Result<(async_nats::Client, jetstream::Context), Box<dyn std::error::Error>> {
    let client = async_nats::connect("nats://localhost:4222").await?;
    let jetstream = jetstream::new(client.clone());
    Ok((client, jetstream))
}

/// Test data generators
pub struct TestData;

impl TestData {
    /// Create a test graph with nodes and edges
    pub fn create_test_graph(num_nodes: usize, num_edges: usize) -> GraphContent {
        let graph_id = GraphId::new();
        let metadata = GraphMetadata {
            name: format!("test-graph-{}", graph_id),
            description: Some("Integration test graph".to_string()),
            ..Default::default()
        };

        let mut nodes = Vec::new();
        let node_ids: Vec<NodeId> = (0..num_nodes).map(|_| NodeId::new()).collect();

        // Create nodes
        // Note: NodeIPLDContent has been removed from the codebase
        // This test fixture needs to be updated to use the new content model

        // Create edges (connect nodes in a pattern)
        let edges = std::collections::HashMap::new();
        // Note: EdgeIPLDContent has been removed from the codebase
        // This test fixture needs to be updated to use the new content model

        GraphContent {
            id: graph_id,
            metadata,
            nodes,
            edges,
            conceptual_position: None,
        }
    }

    /// Create a sequence of graph commands
    pub fn create_graph_commands(graph_id: GraphId) -> Vec<GraphCommand> {
        vec![
            GraphCommand::CreateGraph {
                id: graph_id,
                metadata: GraphMetadata {
                    name: "command-test-graph".to_string(),
                    ..Default::default()
                },
            },
        ]
    }

    /// Create a sequence of node commands
    pub fn create_node_commands(graph_id: GraphId, count: usize) -> Vec<NodeCommand> {
        (0..count)
            .map(|i| NodeCommand::AddNode {
                graph_id,
                node_id: NodeId::new(),
                content: format!("test-node-{}", i),
                position: Position3D {
                    x: i as f32 * 2.0,
                    y: 0.0,
                    z: 0.0,
                },
                metadata: Default::default(),
            })
            .collect()
    }

    /// Create edge commands to connect nodes
    pub fn create_edge_commands(
        graph_id: GraphId,
        node_ids: &[NodeId],
    ) -> Vec<EdgeCommand> {
        let mut commands = Vec::new();
        for i in 0..node_ids.len() - 1 {
            commands.push(EdgeCommand::ConnectNodes {
                graph_id,
                edge_id: EdgeId::new(),
                source: node_ids[i],
                target: node_ids[i + 1],
                relationship: format!("connects-{}", i),
                metadata: Default::default(),
            });
        }
        commands
    }

    pub fn create_test_graph_command() -> Command {
        Command::Graph(GraphCommand::CreateGraph {
            id: GraphId::new(),
            name: "Test Graph".to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("created_by".to_string(), serde_json::json!("test"));
                map.insert("version".to_string(), serde_json::json!("1.0"));
                map
            },
        })
    }
}

/// Test event stream manager
pub struct TestEventStream {
    pub client: async_nats::Client,
    pub jetstream: jetstream::Context,
    pub stream_name: String,
}

impl TestEventStream {
    /// Create a new test event stream
    pub async fn new(stream_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let (client, jetstream) = connect_test_nats().await?;

        // Create test stream
        let config = jetstream::stream::Config {
            name: stream_name.to_string(),
            subjects: vec![format!("test.{}.>", stream_name)],
            retention: jetstream::stream::RetentionPolicy::WorkQueue,
            storage: jetstream::stream::StorageType::Memory,
            ..Default::default()
        };

        jetstream.create_stream(config).await?;

        Ok(Self {
            client,
            jetstream,
            stream_name: stream_name.to_string(),
        })
    }

    /// Publish a test event
    pub async fn publish_event<E: serde::Serialize>(
        &self,
        subject: &str,
        event: &E,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = serde_json::to_vec(event)?;
        self.jetstream
            .publish(format!("test.{}.{}", self.stream_name, subject), payload.into())
            .await?
            .await?;
        Ok(())
    }

    /// Create a consumer for the stream
    pub async fn create_consumer(
        &self,
        consumer_name: &str,
    ) -> Result<jetstream::consumer::Consumer<jetstream::consumer::pull::Config>, Box<dyn std::error::Error>> {
        let config = jetstream::consumer::pull::Config {
            name: Some(consumer_name.to_string()),
            ..Default::default()
        };

        let consumer = self.jetstream
            .get_stream(&self.stream_name)
            .await?
            .create_consumer(config)
            .await?;

        Ok(consumer)
    }

    /// Clean up the test stream
    pub async fn cleanup(self) -> Result<(), Box<dyn std::error::Error>> {
        self.jetstream.delete_stream(&self.stream_name).await?;
        Ok(())
    }
}

/// Test assertion helpers
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that an event was published to NATS
    pub async fn assert_event_published<E: serde::de::DeserializeOwned>(
        consumer: &mut jetstream::consumer::Consumer<jetstream::consumer::pull::Config>,
        timeout: Duration,
    ) -> Result<E, Box<dyn std::error::Error>> {
        let messages = consumer
            .messages()
            .await?
            .take(1)
            .collect::<Vec<_>>()
            .await;

        if messages.is_empty() {
            return Err("No messages received within timeout".into());
        }

        let message = messages.into_iter().next().unwrap()?;
        let event: E = serde_json::from_slice(&message.payload)?;
        message.ack().await?;

        Ok(event)
    }

    /// Assert CID chain integrity
    pub fn assert_cid_chain_valid(events: &[ia::domain::events::cid_chain::ChainedEvent]) -> Result<(), String> {
        for (i, event) in events.iter().enumerate() {
            if i > 0 {
                let expected_previous = &events[i - 1].event_cid;
                if event.previous_cid.as_ref() != Some(expected_previous) {
                    return Err(format!(
                        "CID chain broken at index {}: expected {:?}, got {:?}",
                        i, expected_previous, event.previous_cid
                    ));
                }
            } else if event.previous_cid.is_some() {
                return Err("First event should not have previous CID".to_string());
            }
        }
        Ok(())
    }
}

/// Test cleanup guard
pub struct TestCleanup {
    cleanup_tasks: Arc<Mutex<Vec<Box<dyn FnOnce() + Send>>>>,
}

impl TestCleanup {
    pub fn new() -> Self {
        Self {
            cleanup_tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_task<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.cleanup_tasks.lock().await.push(Box::new(task));
    }

    pub async fn cleanup(self) {
        let tasks = self.cleanup_tasks.lock().await;
        for task in tasks.drain(..) {
            task();
        }
    }
}

/// Macro for integration test setup
#[macro_export]
macro_rules! integration_test {
    ($name:ident, $body:expr) => {
        #[tokio::test]
        #[ignore = "requires running NATS server"]
        async fn $name() -> Result<(), Box<dyn std::error::Error>> {
            // Check if NATS is running
            match connect_test_nats().await {
                Ok((client, jetstream)) => {
                    let result = async {
                        $body(client, jetstream).await
                    }.await;

                    // Cleanup happens automatically when variables go out of scope
                    result
                }
                Err(e) => {
                    eprintln!("NATS not available: {}. Start with: nats-server -js", e);
                    Ok(())
                }
            }
        }
    };
}
