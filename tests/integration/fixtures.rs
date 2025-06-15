//! Test fixtures and helpers for integration tests

use async_nats::jetstream;
use cim_domain::{DomainResult, DomainError, DomainEvent};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Test NATS server manager
pub struct TestNatsServer {
    client: async_nats::Client,
    jetstream: jetstream::Context,
}

impl TestNatsServer {
    /// Start a test NATS server (assumes nats-server is running)
    pub async fn start() -> DomainResult<Self> {
        let client = async_nats::connect("nats://localhost:4222")
            .await
            .map_err(|e| DomainError::generic(format!("Failed to connect to NATS: {}", e)))?;

        let jetstream = jetstream::new(client.clone());

        // Create test streams
        let _ = jetstream
            .create_stream(jetstream::stream::Config {
                name: "TEST-EVENTS".to_string(),
                subjects: vec!["test.events.>".to_string()],
                ..Default::default()
            })
            .await;

        Ok(Self { client, jetstream })
    }

    /// Get NATS client
    pub fn client(&self) -> &async_nats::Client {
        &self.client
    }

    /// Get JetStream context
    pub fn jetstream(&self) -> &jetstream::Context {
        &self.jetstream
    }

    /// Cleanup test data
    pub async fn cleanup(&self) -> DomainResult<()> {
        // Delete test stream
        let _ = self.jetstream.delete_stream("TEST-EVENTS").await;
        Ok(())
    }
}

/// Test event store
pub struct TestEventStore {
    events: Arc<Mutex<Vec<Box<dyn DomainEvent>>>>,
}

impl TestEventStore {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn append(&self, event: Box<dyn DomainEvent>) -> DomainResult<()> {
        let mut events = self.events.lock().await;
        events.push(event);
        Ok(())
    }

    pub async fn get_events(&self) -> Vec<Box<dyn DomainEvent>> {
        let events = self.events.lock().await;
        events.iter().map(|e| e.boxed_clone()).collect()
    }

    pub async fn clear(&self) {
        let mut events = self.events.lock().await;
        events.clear();
    }
}

/// Create a test graph aggregate
pub fn create_test_graph() -> cim_domain_graph::GraphAggregate {
    use cim_domain_graph::{GraphAggregate, GraphType};
    use cim_domain::GraphId;

    GraphAggregate::new(
        GraphId::new(),
        "Test Graph".to_string(),
        GraphType::WorkflowGraph,
    )
}

/// Create test node
pub fn create_test_node() -> cim_domain_graph::Node {
    use cim_domain_graph::{Node, NodeType};
    use cim_domain::NodeId;

    Node::new(
        NodeId::new(),
        NodeType::WorkflowStep {
            step_type: cim_domain_graph::StepType::Process,
        },
    )
}

/// Test assertion helpers
pub mod assertions {
    use cim_domain::DomainEvent;

    /// Assert that an event was published
    pub fn assert_event_published(events: &[Box<dyn DomainEvent>], expected_type: &str) {
        assert!(
            events.iter().any(|e| e.event_type() == expected_type),
            "Expected event type '{}' was not published. Found: {:?}",
            expected_type,
            events.iter().map(|e| e.event_type()).collect::<Vec<_>>()
        );
    }

    /// Assert event count
    pub fn assert_event_count(events: &[Box<dyn DomainEvent>], expected: usize) {
        assert_eq!(
            events.len(),
            expected,
            "Expected {} events, but found {}",
            expected,
            events.len()
        );
    }
}
