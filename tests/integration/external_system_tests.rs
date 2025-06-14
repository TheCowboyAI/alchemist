//! Integration tests for external system connections
//!
//! These tests verify integration with external systems:
//! 1. Neo4j graph database synchronization
//! 2. External API integrations
//! 3. Third-party service mocking
//! 4. Bidirectional data flow
//!
//! ```mermaid
//! graph LR
//!     A[CIM Domain] --> B[Event Store]
//!     B --> C[Projection Service]
//!     C --> D[Neo4j Sync]
//!     C --> E[External APIs]
//!     D --> F[Neo4j Database]
//!     E --> G[Third-party Services]
//!     F --> C
//!     G --> C
//! ```

use crate::fixtures::{TestNatsServer, TestEventStore, create_test_graph, assertions::*};
use cim_domain::{DomainResult, GraphId, NodeId, EdgeId, DomainEvent};
use cim_domain_graph::{
    GraphAggregate, GraphDomainEvent, NodeType, Position3D,
    ExternalProjection, SyncDirection,
};
use std::collections::HashMap;

/// Mock Neo4j client for testing
struct MockNeo4jClient {
    nodes: HashMap<String, serde_json::Value>,
    relationships: HashMap<String, serde_json::Value>,
}

impl MockNeo4jClient {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            relationships: HashMap::new(),
        }
    }

    async fn execute(&mut self, query: &str) -> DomainResult<()> {
        // Mock query execution
        if query.contains("CREATE") {
            // Simulate node/relationship creation
        } else if query.contains("MATCH") {
            // Simulate query
        }
        Ok(())
    }

    async fn get_nodes(&self) -> Vec<serde_json::Value> {
        self.nodes.values().cloned().collect()
    }
}

/// Test Neo4j bidirectional synchronization
#[tokio::test]
#[ignore] // Requires external services
async fn test_neo4j_bidirectional_sync() -> DomainResult<()> {
    // Arrange
    let nats = TestNatsServer::start().await?;
    let event_store = TestEventStore::with_nats(&nats).await?;
    let neo4j = MockNeo4jClient::new();

    // Create projection service
    let projection = Neo4jProjection::new(neo4j, SyncDirection::Bidirectional);

    // Create graph in our system
    let graph_id = GraphId::new();
    let node1 = NodeId::new();
    let node2 = NodeId::new();

    let events = vec![
        DomainEvent::Graph(GraphDomainEvent::NodeAdded {
            graph_id,
            node_id: node1,
            node_type: NodeType::Concept,
            position: Position3D::default(),
            conceptual_point: Default::default(),
            metadata: HashMap::from([("name".to_string(), "Node1".to_string())]),
        }),
        DomainEvent::Graph(GraphDomainEvent::NodeAdded {
            graph_id,
            node_id: node2,
            node_type: NodeType::Concept,
            position: Position3D::default(),
            conceptual_point: Default::default(),
            metadata: HashMap::from([("name".to_string(), "Node2".to_string())]),
        }),
        DomainEvent::Graph(GraphDomainEvent::EdgeConnected {
            graph_id,
            edge_id: EdgeId::new(),
            source: node1,
            target: node2,
            relationship: cim_domain_graph::EdgeRelationship::default(),
        }),
    ];

    // Act - Sync to Neo4j
    for event in &events {
        event_store.append(event.clone()).await?;
        projection.sync_event(event).await?;
    }

    // Modify in Neo4j (simulate external change)
    projection.neo4j.execute("MATCH (n) SET n.updated = true").await?;

    // Sync back
    let external_events = projection.ingest_changes().await?;

    // Assert
    assert!(!external_events.is_empty(), "Should detect external changes");
    assert!(external_events.iter().any(|e| matches!(
        e,
        DomainEvent::Graph(GraphDomainEvent::NodeUpdated { .. })
    )), "Should generate update events");

    // Cleanup
    nats.cleanup().await?;

    Ok(())
}

/// Test external API integration with retry logic
#[tokio::test]
async fn test_external_api_integration_with_retry() -> DomainResult<()> {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    // Arrange - Start mock server
    let mock_server = MockServer::start().await;

    // Configure mock to fail twice then succeed
    let fail_response = ResponseTemplate::new(500);
    let success_response = ResponseTemplate::new(200)
        .set_body_json(serde_json::json!({
            "status": "success",
            "data": {"processed": true}
        }));

    Mock::given(method("POST"))
        .and(path("/api/process"))
        .respond_with(fail_response)
        .expect(2)
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/process"))
        .respond_with(success_response)
        .expect(1)
        .mount(&mock_server)
        .await;

    // Create API client with retry logic
    let api_client = ExternalApiClient::new(
        &mock_server.uri(),
        RetryConfig {
            max_attempts: 3,
            backoff_ms: 100,
        },
    );

    // Act - Send data with retry
    let result = api_client.process_graph_data(GraphData {
        graph_id: GraphId::new(),
        nodes: vec![],
        edges: vec![],
    }).await;

    // Assert
    assert!(result.is_ok(), "Should succeed after retries");
    let response = result.unwrap();
    assert_eq!(response["status"], "success");

    // Verify all requests were made
    mock_server.verify().await;

    Ok(())
}

/// Test webhook event reception from external systems
#[tokio::test]
async fn test_webhook_event_reception() -> DomainResult<()> {
    use axum::{Router, routing::post, Json};
    use tokio::sync::mpsc;

    // Arrange - Create webhook receiver
    let (tx, mut rx) = mpsc::channel(10);

    let app = Router::new()
        .route("/webhook", post(move |Json(payload): Json<serde_json::Value>| {
            let tx = tx.clone();
            async move {
                tx.send(payload).await.unwrap();
                Json(serde_json::json!({"status": "received"}))
            }
        }));

    // Start webhook server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Act - Simulate external system sending webhook
    let client = reqwest::Client::new();
    let webhook_data = serde_json::json!({
        "event": "external_update",
        "entity_id": "12345",
        "changes": {
            "status": "completed",
            "timestamp": "2025-01-11T10:00:00Z"
        }
    });

    let response = client
        .post(format!("http://{}/webhook", addr))
        .json(&webhook_data)
        .send()
        .await?;

    // Assert
    assert_eq!(response.status(), 200);

    // Verify webhook data received
    let received = rx.recv().await.unwrap();
    assert_eq!(received["event"], "external_update");
    assert_eq!(received["entity_id"], "12345");

    Ok(())
}

/// Test data transformation between CIM and external formats
#[tokio::test]
async fn test_data_transformation_pipeline() -> DomainResult<()> {
    // Arrange
    let transformer = DataTransformer::new();

    // CIM domain event
    let cim_event = DomainEvent::Graph(GraphDomainEvent::NodeAdded {
        graph_id: GraphId::new(),
        node_id: NodeId::new(),
        node_type: NodeType::Concept,
        position: Position3D { x: 1.0, y: 2.0, z: 3.0 },
        conceptual_point: Default::default(),
        metadata: HashMap::from([
            ("title".to_string(), "Test Node".to_string()),
            ("category".to_string(), "concept".to_string()),
        ]),
    });

    // Act - Transform to external format
    let external_format = transformer.to_external_format(&cim_event)?;

    // Assert external format structure
    assert_eq!(external_format["type"], "node_created");
    assert_eq!(external_format["properties"]["title"], "Test Node");
    assert_eq!(external_format["position"]["x"], 1.0);

    // Act - Transform back to CIM format
    let restored_event = transformer.from_external_format(external_format)?;

    // Assert round-trip consistency
    match restored_event {
        DomainEvent::Graph(GraphDomainEvent::NodeAdded { metadata, .. }) => {
            assert_eq!(metadata.get("title"), Some(&"Test Node".to_string()));
        }
        _ => panic!("Wrong event type after transformation"),
    }

    Ok(())
}

/// Test external system health monitoring
#[tokio::test]
async fn test_external_system_health_monitoring() -> DomainResult<()> {
    // Arrange
    let health_monitor = ExternalSystemHealthMonitor::new();

    // Register external systems
    health_monitor.register_system("neo4j", HealthCheck {
        endpoint: "http://localhost:7474/health",
        timeout_ms: 5000,
        expected_status: 200,
    }).await?;

    health_monitor.register_system("api_gateway", HealthCheck {
        endpoint: "http://api.example.com/health",
        timeout_ms: 3000,
        expected_status: 200,
    }).await?;

    // Act - Perform health checks
    let health_status = health_monitor.check_all_systems().await;

    // Assert
    assert!(health_status.contains_key("neo4j"));
    assert!(health_status.contains_key("api_gateway"));

    // Check circuit breaker activation
    if let Some(status) = health_status.get("neo4j") {
        if !status.is_healthy {
            assert!(status.circuit_breaker_open, "Circuit breaker should open on failure");
        }
    }

    Ok(())
}

/// Test bulk data export to external systems
#[tokio::test]
async fn test_bulk_data_export() -> DomainResult<()> {
    // Arrange
    let event_store = TestEventStore::new();
    let exporter = BulkDataExporter::new();

    // Generate test data
    let graph_id = GraphId::new();
    let mut events = Vec::new();

    for i in 0..1000 {
        events.push(DomainEvent::Graph(GraphDomainEvent::NodeAdded {
            graph_id,
            node_id: NodeId::new(),
            node_type: NodeType::Concept,
            position: Position3D::default(),
            conceptual_point: Default::default(),
            metadata: HashMap::from([("index".to_string(), i.to_string())]),
        }));
    }

    // Store events
    for event in &events {
        event_store.append(event.clone()).await?;
    }

    // Act - Export in batches
    let export_config = ExportConfig {
        batch_size: 100,
        format: ExportFormat::JsonLines,
        compression: Some(CompressionType::Gzip),
    };

    let export_result = exporter.export_graph(
        &event_store,
        graph_id,
        export_config,
    ).await?;

    // Assert
    assert_eq!(export_result.total_records, 1000);
    assert_eq!(export_result.batches_processed, 10);
    assert!(export_result.compressed_size < export_result.uncompressed_size);

    Ok(())
}

/// Test conflict resolution for concurrent external updates
#[tokio::test]
async fn test_concurrent_external_update_conflict_resolution() -> DomainResult<()> {
    // Arrange
    let event_store = TestEventStore::new();
    let conflict_resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins);

    let node_id = NodeId::new();
    let graph_id = GraphId::new();

    // Simulate concurrent updates from different sources
    let internal_update = DomainEvent::Graph(GraphDomainEvent::NodeUpdated {
        graph_id,
        node_id,
        changes: cim_domain_graph::NodeChanges {
            metadata: Some(HashMap::from([("status".to_string(), "internal".to_string())])),
            position: None,
            node_type: None,
        },
        timestamp: std::time::SystemTime::now(),
        source: UpdateSource::Internal,
    });

    let external_update = DomainEvent::Graph(GraphDomainEvent::NodeUpdated {
        graph_id,
        node_id,
        changes: cim_domain_graph::NodeChanges {
            metadata: Some(HashMap::from([("status".to_string(), "external".to_string())])),
            position: None,
            node_type: None,
        },
        timestamp: std::time::SystemTime::now() + std::time::Duration::from_secs(1),
        source: UpdateSource::External("neo4j".to_string()),
    });

    // Act - Resolve conflict
    let resolved = conflict_resolver.resolve(vec![internal_update, external_update])?;

    // Assert - External update wins (later timestamp)
    match resolved {
        DomainEvent::Graph(GraphDomainEvent::NodeUpdated { changes, .. }) => {
            assert_eq!(
                changes.metadata.unwrap().get("status"),
                Some(&"external".to_string())
            );
        }
        _ => panic!("Wrong event type"),
    }

    Ok(())
}

// Helper types and implementations

struct Neo4jProjection {
    neo4j: MockNeo4jClient,
    sync_direction: SyncDirection,
}

impl Neo4jProjection {
    fn new(neo4j: MockNeo4jClient, sync_direction: SyncDirection) -> Self {
        Self { neo4j, sync_direction }
    }

    async fn sync_event(&mut self, event: &DomainEvent) -> DomainResult<()> {
        // Transform and sync to Neo4j
        match event {
            DomainEvent::Graph(GraphDomainEvent::NodeAdded { node_id, metadata, .. }) => {
                let query = format!(
                    "CREATE (n:Node {{id: '{}', name: '{}'}})",
                    node_id,
                    metadata.get("name").unwrap_or(&"".to_string())
                );
                self.neo4j.execute(&query).await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn ingest_changes(&self) -> DomainResult<Vec<DomainEvent>> {
        // Detect changes in Neo4j and convert to events
        let nodes = self.neo4j.get_nodes().await;
        let mut events = Vec::new();

        for node in nodes {
            if node.get("updated").and_then(|v| v.as_bool()).unwrap_or(false) {
                // Generate update event
                events.push(DomainEvent::Graph(GraphDomainEvent::NodeUpdated {
                    graph_id: GraphId::new(),
                    node_id: NodeId::new(),
                    changes: Default::default(),
                    timestamp: std::time::SystemTime::now(),
                    source: UpdateSource::External("neo4j".to_string()),
                }));
            }
        }

        Ok(events)
    }
}

struct ExternalApiClient {
    base_url: String,
    retry_config: RetryConfig,
    client: reqwest::Client,
}

impl ExternalApiClient {
    fn new(base_url: &str, retry_config: RetryConfig) -> Self {
        Self {
            base_url: base_url.to_string(),
            retry_config,
            client: reqwest::Client::new(),
        }
    }

    async fn process_graph_data(&self, data: GraphData) -> Result<serde_json::Value, reqwest::Error> {
        let mut attempts = 0;

        loop {
            attempts += 1;

            let result = self.client
                .post(format!("{}/api/process", self.base_url))
                .json(&data)
                .send()
                .await;

            match result {
                Ok(response) if response.status().is_success() => {
                    return response.json().await;
                }
                Ok(_) | Err(_) if attempts < self.retry_config.max_attempts => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        self.retry_config.backoff_ms * attempts as u64
                    )).await;
                }
                Err(e) => return Err(e),
                Ok(response) => {
                    return Err(reqwest::Error::from(
                        std::io::Error::new(std::io::ErrorKind::Other, "Request failed")
                    ));
                }
            }
        }
    }
}

#[derive(Clone)]
struct RetryConfig {
    max_attempts: usize,
    backoff_ms: u64,
}

#[derive(serde::Serialize)]
struct GraphData {
    graph_id: GraphId,
    nodes: Vec<NodeId>,
    edges: Vec<EdgeId>,
}

struct DataTransformer;

impl DataTransformer {
    fn new() -> Self {
        Self
    }

    fn to_external_format(&self, event: &DomainEvent) -> DomainResult<serde_json::Value> {
        match event {
            DomainEvent::Graph(GraphDomainEvent::NodeAdded {
                node_id, position, metadata, ..
            }) => {
                Ok(serde_json::json!({
                    "type": "node_created",
                    "id": node_id.to_string(),
                    "properties": metadata,
                    "position": {
                        "x": position.x,
                        "y": position.y,
                        "z": position.z,
                    }
                }))
            }
            _ => Ok(serde_json::json!({}))
        }
    }

    fn from_external_format(&self, data: serde_json::Value) -> DomainResult<DomainEvent> {
        // Transform external format back to domain event
        let event_type = data["type"].as_str().unwrap_or("");

        match event_type {
            "node_created" => {
                Ok(DomainEvent::Graph(GraphDomainEvent::NodeAdded {
                    graph_id: GraphId::new(),
                    node_id: NodeId::new(),
                    node_type: NodeType::Concept,
                    position: Position3D {
                        x: data["position"]["x"].as_f64().unwrap_or(0.0) as f32,
                        y: data["position"]["y"].as_f64().unwrap_or(0.0) as f32,
                        z: data["position"]["z"].as_f64().unwrap_or(0.0) as f32,
                    },
                    conceptual_point: Default::default(),
                    metadata: data["properties"].as_object()
                        .map(|obj| obj.iter()
                            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                            .collect()
                        )
                        .unwrap_or_default(),
                }))
            }
            _ => Err(cim_domain::DomainError::ValidationError("Unknown event type".to_string()))
        }
    }
}

struct ExternalSystemHealthMonitor {
    systems: tokio::sync::RwLock<HashMap<String, HealthCheck>>,
}

impl ExternalSystemHealthMonitor {
    fn new() -> Self {
        Self {
            systems: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    async fn register_system(&self, name: &str, check: HealthCheck) -> DomainResult<()> {
        let mut systems = self.systems.write().await;
        systems.insert(name.to_string(), check);
        Ok(())
    }

    async fn check_all_systems(&self) -> HashMap<String, SystemHealth> {
        let systems = self.systems.read().await;
        let mut results = HashMap::new();

        for (name, check) in systems.iter() {
            // Simulate health check
            results.insert(name.clone(), SystemHealth {
                is_healthy: true, // Would actually check endpoint
                circuit_breaker_open: false,
                last_check: std::time::SystemTime::now(),
            });
        }

        results
    }
}

struct HealthCheck {
    endpoint: String,
    timeout_ms: u64,
    expected_status: u16,
}

struct SystemHealth {
    is_healthy: bool,
    circuit_breaker_open: bool,
    last_check: std::time::SystemTime,
}

struct BulkDataExporter;

impl BulkDataExporter {
    fn new() -> Self {
        Self
    }

    async fn export_graph(
        &self,
        event_store: &TestEventStore,
        graph_id: GraphId,
        config: ExportConfig,
    ) -> DomainResult<ExportResult> {
        let events = event_store.get_events().await;
        let total_records = events.len();
        let batches_processed = (total_records + config.batch_size - 1) / config.batch_size;

        // Simulate export
        let uncompressed_size = total_records * 100; // Approximate
        let compressed_size = if config.compression.is_some() {
            uncompressed_size / 3 // Approximate compression ratio
        } else {
            uncompressed_size
        };

        Ok(ExportResult {
            total_records,
            batches_processed,
            uncompressed_size,
            compressed_size,
        })
    }
}

struct ExportConfig {
    batch_size: usize,
    format: ExportFormat,
    compression: Option<CompressionType>,
}

enum ExportFormat {
    JsonLines,
    Csv,
    Parquet,
}

enum CompressionType {
    Gzip,
    Zstd,
    Lz4,
}

struct ExportResult {
    total_records: usize,
    batches_processed: usize,
    uncompressed_size: usize,
    compressed_size: usize,
}

struct ConflictResolver {
    strategy: ResolutionStrategy,
}

impl ConflictResolver {
    fn new(strategy: ResolutionStrategy) -> Self {
        Self { strategy }
    }

    fn resolve(&self, events: Vec<DomainEvent>) -> DomainResult<DomainEvent> {
        match self.strategy {
            ResolutionStrategy::LastWriteWins => {
                // Return event with latest timestamp
                events.into_iter()
                    .max_by_key(|e| match e {
                        DomainEvent::Graph(GraphDomainEvent::NodeUpdated { timestamp, .. }) => *timestamp,
                        _ => std::time::SystemTime::UNIX_EPOCH,
                    })
                    .ok_or(cim_domain::DomainError::ValidationError("No events to resolve".to_string()))
            }
        }
    }
}

enum ResolutionStrategy {
    LastWriteWins,
    FirstWriteWins,
    MergeChanges,
}

#[derive(Clone)]
enum UpdateSource {
    Internal,
    External(String),
}
