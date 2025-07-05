//! Test fixtures and helpers for integration tests

use async_nats::jetstream;
use cim_domain::{DomainError, DomainEvent, DomainResult, GraphId, NodeId, EdgeId};
use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;

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
#[derive(Clone)]
pub struct TestEventStore {
    events: Arc<Mutex<Vec<Box<dyn DomainEvent>>>>,
    nats_server: Option<Arc<TestNatsServer>>,
}

impl TestEventStore {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            nats_server: None,
        }
    }

    /// Create event store with NATS backend
    pub async fn with_nats(nats: &TestNatsServer) -> DomainResult<Self> {
        Ok(Self {
            events: Arc::new(Mutex::new(Vec::new())),
            nats_server: Some(Arc::new(TestNatsServer::start().await?)),
        })
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

    GraphAggregate::new(
        GraphId::new(),
        "Test Graph".to_string(),
        GraphType::WorkflowGraph,
    )
}

/// Create test node
pub fn create_test_node() -> cim_domain_graph::Node {
    use cim_domain_graph::{Node, NodeType};

    Node::new(
        NodeId::new(),
        NodeType::WorkflowStep {
            step_type: cim_domain_graph::StepType::Process,
        },
    )
}

/// Create a test app for Bevy integration tests
pub fn create_test_app() -> bevy::app::App {
    use bevy::prelude::*;
    
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}

/// Create a large graph for performance testing
pub fn create_large_graph() -> cim_domain_graph::GraphAggregate {
    use cim_domain_graph::{GraphAggregate, GraphType};

    GraphAggregate::new(
        GraphId::new(),
        "Large Test Graph".to_string(),
        GraphType::ConceptualGraph,
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

/// Mock implementation of projection sync (simplified)
pub struct ProjectionSync {
    event_store: TestEventStore,
}

impl ProjectionSync {
    pub fn new(event_store: TestEventStore) -> Self {
        Self { event_store }
    }

    pub async fn subscribe_projection(
        &self,
        _name: &str,
        _projection: Arc<RwLock<dyn Projection>>,
    ) -> DomainResult<()> {
        // Mock implementation
        Ok(())
    }
}

/// Trait for projections
#[async_trait::async_trait]
pub trait Projection: Send + Sync {
    async fn apply_event(&mut self, event: &dyn DomainEvent) -> DomainResult<()>;
}

/// Mock graph summary projection
pub struct GraphSummaryProjection {
    summaries: HashMap<GraphId, GraphSummary>,
}

#[derive(Clone)]
pub struct GraphSummary {
    pub node_count: usize,
    pub edge_count: usize,
}

impl GraphSummaryProjection {
    pub fn new() -> Self {
        Self {
            summaries: HashMap::new(),
        }
    }

    pub fn get_summary(&self, graph_id: &GraphId) -> DomainResult<GraphSummary> {
        self.summaries
            .get(graph_id)
            .cloned()
            .ok_or_else(|| DomainError::NotFound("Graph not found".to_string()))
    }
}

#[async_trait::async_trait]
impl Projection for GraphSummaryProjection {
    async fn apply_event(&mut self, event: &dyn DomainEvent) -> DomainResult<()> {
        // Mock implementation
        Ok(())
    }
}

/// Mock node list projection
pub struct NodeListProjection {
    nodes: HashMap<GraphId, Vec<NodeId>>,
}

impl NodeListProjection {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn get_nodes(&self, graph_id: &GraphId) -> DomainResult<Vec<NodeId>> {
        Ok(self.nodes.get(graph_id).cloned().unwrap_or_default())
    }
}

#[async_trait::async_trait]
impl Projection for NodeListProjection {
    async fn apply_event(&mut self, event: &dyn DomainEvent) -> DomainResult<()> {
        // Mock implementation
        Ok(())
    }
}

/// Mock edge list projection
pub struct EdgeListProjection {
    edges: HashMap<GraphId, Vec<EdgeId>>,
}

impl EdgeListProjection {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    pub fn get_edges(&self, graph_id: &GraphId) -> DomainResult<Vec<EdgeId>> {
        Ok(self.edges.get(graph_id).cloned().unwrap_or_default())
    }
}

#[async_trait::async_trait]
impl Projection for EdgeListProjection {
    async fn apply_event(&mut self, event: &dyn DomainEvent) -> DomainResult<()> {
        // Mock implementation
        Ok(())
    }
}
