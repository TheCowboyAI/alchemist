# Bidirectional Event Flow Architecture

## Overview

The CIM graph system acts as a central nervous system that both:
1. **Projects** graph changes TO external systems
2. **Ingests** event streams FROM external systems

This creates a living, breathing information ecosystem where all systems contribute to and benefit from the collective knowledge graph.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        CIM Graph System                          │
│                                                                  │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐      │
│  │   Domain    │     │   Event     │     │  Event      │      │
│  │  Aggregates │────▶│   Store     │────▶│  Publisher  │      │
│  └─────────────┘     └──────┬──────┘     └──────┬──────┘      │
│                             │                     │              │
│                             ▼                     ▼              │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐      │
│  │  Command    │     │   Event     │     │   NATS      │      │
│  │  Handlers   │◀────│  Correlator │◀────│  Subscriber │      │
│  └─────────────┘     └─────────────┘     └─────────────┘      │
└────────────────────────────┬─────────────────────┬──────────────┘
                             │                     │
                             ▼                     ▼
                    ┌────────────────────────────────────┐
                    │         NATS JetStream             │
                    │                                    │
                    │  Subjects:                        │
                    │  - graph.events.>                 │
                    │  - persistence.events.>           │
                    │  - orchestration.events.>         │
                    │  - intelligence.events.>          │
                    │  - knowledge.events.>             │
                    │  - version.events.>               │
                    │  - credential.events.>            │
                    │  - aggregation.events.>           │
                    │  - infrastructure.events.>        │
                    │  - gateway.events.>               │
                    │  - search.events.>                │
                    └────────────┬───────────────────────┘
                                 │
        ┌────────────────────────┴────────────────────────┐
        │                                                  │
        ▼                                                  ▼
┌──────────────────────┐                          ┌──────────────────────┐
│   Domain Modules     │                          │   Domain Modules     │
├──────────────────────┤                          ├──────────────────────┤
│ • GraphPersistence   │                          │ • VersionControl     │
│ • WorkflowOrchest.   │                          │ • KnowledgeMgmt      │
│ • DocumentIntel.     │                          │ • CredentialMgmt     │
│ • SearchDiscovery    │                          │ • ContentAggregation │
│ • Communication      │                          │ • InfrastructureConf │
└──────────────────────┘                          │ • WebGateway         │
                                                  └──────────────────────┘

Each module:
- Subscribes to graph.events.> for relevant updates
- Publishes to {system}.events.> for system-specific events
- Completely decoupled from core system
- Communicates only through NATS
```

## NATS-Based Module Architecture

Each external system integration is implemented as an independent module that:

1. **Runs as a separate process** - Can be written in any language
2. **Communicates only through NATS** - No direct API dependencies
3. **Subscribes to relevant events** - Filters graph.events.> for what it needs
4. **Publishes its own events** - To {system}.events.> subjects
5. **Maintains its own state** - Local cache/database as needed
6. **Handles its own failures** - Retry logic, circuit breakers, etc.

### Module Communication Pattern

```
┌─────────────────┐         NATS          ┌──────────────────────┐
│   Graph Core    │ ──────────────────────▶│ GraphPersistence     │
│                 │  graph.events.node.*   │ Module               │
│                 │◀────────────────────── │                      │
└─────────────────┘ persistence.events.*  └──────────────────────┘

Subject Naming Convention:
- Outbound: graph.events.{aggregate}.{event}
  - graph.events.node.created
  - graph.events.edge.connected
  - graph.events.graph.updated

- Inbound: {capability}.events.{entity}.{action}
  - persistence.events.path.discovered
  - orchestration.events.workflow.completed
  - intelligence.events.entities.extracted
  - version.events.commit.pushed
  - aggregation.events.content.received
```

## Bidirectional Event Patterns

### 1. Outbound Events (Graph → External Systems)

```rust
pub enum OutboundEvent {
    // Graph mutations to sync
    NodeCreated { node_id: NodeId, properties: NodeProperties },
    EdgeAdded { source: NodeId, target: NodeId, relationship: EdgeType },
    PropertyUpdated { entity_id: EntityId, changes: PropertyChanges },

    // Workflow triggers
    WorkflowTriggered { workflow_id: WorkflowId, context: TriggerContext },

    // Search index updates
    ContentIndexed { content_id: ContentId, searchable_text: String },
}
```

### 2. Inbound Events (External Systems → Graph)

```rust
pub enum InboundEvent {
    // From GraphPersistence - Graph analysis results
    PathDiscovered { path: Vec<NodeId>, algorithm: String },
    CommunityDetected { nodes: Vec<NodeId>, community_id: CommunityId },
    GraphPersisted { graph_id: GraphId, node_count: usize, edge_count: usize },

    // From WorkflowOrchestration - Workflow execution results
    WorkflowCompleted { workflow_id: WorkflowId, outputs: WorkflowOutputs },
    WorkflowScheduled { workflow_id: WorkflowId, schedule: Schedule },
    ExternalDataFetched { source: String, data: JsonValue },

    // From DocumentIntelligence - Document processing
    DocumentProcessed { document_id: DocumentId, extracted_data: ExtractedData },
    EntitiesExtracted { document_id: DocumentId, entities: Vec<Entity> },
    RelationshipsIdentified { document_id: DocumentId, relationships: Vec<Relationship> },

    // From SearchDiscovery - Search and discovery results
    SearchResultsFound { query: String, results: Vec<SearchResult> },
    RelatedContentDiscovered { node_id: NodeId, related: Vec<ContentRef> },
    RecommendationsGenerated { context: Context, recommendations: Vec<Recommendation> },

    // From Communication - Message events
    MessageReceived { channel: Channel, from: Address, content: String },
    NotificationDelivered { notification_id: NotificationId, status: DeliveryStatus },

    // From VersionControl - Change tracking events
    ChangeCommitted { artifact_id: ArtifactId, change_id: ChangeId, author: String },
    BranchCreated { artifact_id: ArtifactId, branch: BranchId, base: ChangeId },
    MergeRequested { artifact_id: ArtifactId, source: BranchId, target: BranchId },

    // From KnowledgeManagement - Knowledge organization events
    KnowledgeNodeCreated { knowledge_id: KnowledgeId, title: String, content: String },
    KnowledgeLinked { source: KnowledgeId, target: KnowledgeId, relation: Relation },
    HierarchyUpdated { parent: KnowledgeId, children: Vec<KnowledgeId> },

    // From CredentialManagement - Security events
    CredentialStored { credential_id: CredentialId, service: String },
    CredentialRotated { credential_id: CredentialId, reason: RotationReason },
    AccessAudited { credential_id: CredentialId, accessor: String, timestamp: DateTime<Utc> },

    // From ContentAggregation - Content ingestion
    ContentReceived { source_id: SourceId, content_id: ContentId, content: Content },
    SourceUpdated { source_id: SourceId, new_items: Vec<Content> },

    // From InfrastructureConfiguration - System state events
    ConfigurationApplied { config_id: ConfigId, changes: Vec<Change> },
    StateChanged { component: Component, old_state: State, new_state: State },
    RollbackPerformed { generation: Generation, reason: String },

    // From WebGateway - Traffic and routing events
    RequestRouted { request_id: RequestId, route: Route, latency_ms: u32 },
    RateLimitApplied { client: ClientId, limit: RateLimit },
    HealthCheckPerformed { service: ServiceId, status: HealthStatus },
}
```

## Event Correlation and Enrichment

### Event Correlator

```rust
pub struct EventCorrelator {
    correlation_rules: Vec<CorrelationRule>,
    enrichment_pipeline: EnrichmentPipeline,
    deduplication_cache: DeduplicationCache,
}

impl EventCorrelator {
    pub async fn correlate_inbound_event(
        &self,
        event: InboundEvent,
        source: EventSource,
    ) -> Result<Vec<DomainCommand>, CorrelationError> {
        // Deduplicate
        if self.deduplication_cache.has_seen(&event)? {
            return Ok(vec![]);
        }

        // Enrich with context
        let enriched = self.enrichment_pipeline.enrich(event, source).await?;

        // Apply correlation rules to generate commands
        let mut commands = Vec::new();
        for rule in &self.correlation_rules {
            if rule.matches(&enriched) {
                commands.extend(rule.generate_commands(&enriched)?);
            }
        }

        Ok(commands)
    }
}
```

### Correlation Rules

```rust
pub struct CorrelationRule {
    pub name: String,
    pub source_pattern: EventPattern,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
}

// Example: Document processed → Update graph
let document_correlation = CorrelationRule {
    name: "document_to_graph".to_string(),
    source_pattern: EventPattern::DocumentProcessed,
    conditions: vec![
        Condition::HasExtractedEntities,
        Condition::ConfidenceAbove(0.8),
    ],
    actions: vec![
        Action::CreateNodes { node_type: NodeType::ExtractedEntity },
        Action::ConnectToDocument { edge_type: EdgeType::ExtractedFrom },
        Action::UpdateConceptualSpace { recalibrate: true },
    ],
};

// Example: Workflow completed → Trigger next workflow
let workflow_chain = CorrelationRule {
    name: "workflow_chaining".to_string(),
    source_pattern: EventPattern::WorkflowCompleted,
    conditions: vec![
        Condition::OutputContains("next_workflow_id"),
    ],
    actions: vec![
        Action::TriggerWorkflow { use_output_as_input: true },
        Action::CreateEdge { edge_type: EdgeType::TriggeredBy },
    ],
};
```

## Ingest Handlers

### Base Ingest Handler

```rust
#[async_trait]
pub trait IngestHandler: Send + Sync {
    type Event: InboundEvent;
    type Config: Send + Sync;

    /// Initialize the handler with configuration
    fn new(config: Self::Config) -> Self;

    /// Subscribe to external system events
    async fn subscribe(&self) -> Result<EventStream<Self::Event>, IngestError>;

    /// Transform external event to domain commands
    async fn transform_event(
        &self,
        event: Self::Event,
    ) -> Result<Vec<DomainCommand>, TransformError>;

    /// Handle subscription errors
    async fn handle_error(&self, error: IngestError) -> ErrorRecovery;
}
```

### Example: GraphPersistence Handler

```rust
pub struct GraphPersistenceHandler<T: GraphStore> {
    config: PersistenceConfig,
    store: T,
    change_stream: ChangeStream,
}

#[async_trait]
impl<T: GraphStore> IngestHandler for GraphPersistenceHandler<T> {
    type Event = PersistenceEvent;
    type Config = PersistenceConfig;

    async fn subscribe(&self) -> Result<EventStream<Self::Event>, IngestError> {
        // Subscribe to Neo4j change data capture
        self.change_stream
            .subscribe(vec![
                "MATCH (n) WHERE n.source = 'external' RETURN n",
                "MATCH ()-[r]->() WHERE r.discovered = true RETURN r",
            ])
            .await
    }

    async fn transform_event(
        &self,
        event: Self::Event,
    ) -> Result<Vec<DomainCommand>, TransformError> {
        match event {
            PersistenceEvent::PathAnalyzed { path, algorithm } => {
                // Transform to domain command
                Ok(vec![DomainCommand::RecordPath {
                    nodes: path,
                    discovery_method: algorithm,
                    source: EventSource::Persistence,
                }])
            }
            PersistenceEvent::CommunityDetected { nodes, community_id } => {
                Ok(vec![DomainCommand::GroupNodes {
                    node_ids: nodes,
                    group_id: community_id,
                    group_type: GroupType::Community,
                }])
            }
            // ... other event types
        }
    }
}
```

### Example: WorkflowOrchestration Handler

```rust
pub struct WorkflowOrchestrationHandler<T: WorkflowEngine> {
    webhook_server: WebhookServer,
    workflow_engine: T,
}

#[async_trait]
impl<T: WorkflowEngine> IngestHandler for WorkflowOrchestrationHandler<T> {
    type Event = WorkflowEvent;
    type Config = OrchestrationConfig;

    async fn subscribe(&self) -> Result<EventStream<Self::Event>, IngestError> {
        // Start webhook server
        self.webhook_server
            .listen_on(self.config.webhook_port)
            .await?;

        // Return event stream
        Ok(self.webhook_server.event_stream())
    }

    async fn transform_event(
        &self,
        event: Self::Event,
    ) -> Result<Vec<DomainCommand>, TransformError> {
        match event {
            WorkflowEvent::ExecutionCompleted { workflow_id, results } => {
                let mut commands = vec![];

                // Record workflow execution
                commands.push(DomainCommand::RecordExecution {
                    workflow_id,
                    execution_time: Utc::now(),
                    results: results.clone(),
                    source: EventSource::Orchestration,
                });

                // Process extracted data
                if let Some(extracted_data) = results.get("extracted_data") {
                    commands.push(DomainCommand::ProcessExtractedData {
                        data: extracted_data.clone(),
                        source_workflow: workflow_id,
                    });
                }

                Ok(commands)
            }
            // ... other event types
        }
    }
}
```

## Example Module Implementations

### VersionControl Module (Rust)
```rust
// Standalone service that tracks changes across artifacts
pub struct VersionControlModule<T: VersionControlSystem> {
    nats_client: async_nats::Client,
    vcs: T,
    webhook_server: WebhookServer,
}

impl GitModule {
    pub async fn run(&mut self) -> Result<()> {
        // Subscribe to graph events
        let mut graph_sub = self.nats_client
            .subscribe("graph.events.>")
            .await?;

        // Start webhook server for Git events
        self.webhook_server.start().await?;

        // Process events
        loop {
            tokio::select! {
                // Handle graph events
                Some(msg) = graph_sub.next() => {
                    match msg.subject.as_str() {
                        "graph.events.node.created" => {
                            // Maybe create issue or PR
                            self.handle_node_created(msg).await?;
                        }
                        _ => {}
                    }
                }

                // Handle Git webhooks
                Some(webhook) = self.webhook_server.next() => {
                    // Publish to NATS
                    self.nats_client.publish(
                        "version.events.change.committed",
                        webhook.to_bytes()
                    ).await?;
                }
            }
        }
    }
}
```

### ContentAggregation Module (Python)
```python
# Standalone service for content aggregation from multiple sources
import asyncio
import feedparser
from nats.aio.client import Client as NATS

class ContentAggregationModule:
    def __init__(self, nats_url: str, feeds: list[str]):
        self.nc = NATS()
        self.feeds = feeds

    async def run(self):
        await self.nc.connect(self.nats_url)

        # Subscribe to graph events
        await self.nc.subscribe("graph.events.>", cb=self.handle_graph_event)

        # Poll RSS feeds periodically
        while True:
            for feed_url in self.feeds:
                items = await self.fetch_feed(feed_url)
                for item in items:
                    # Publish to NATS
                    await self.nc.publish(
                        "aggregation.events.content.received",
                        json.dumps({
                            "source_id": feed_url,
                            "title": item.title,
                            "content": item.description,
                            "link": item.link,
                            "content_type": "rss_item"
                        }).encode()
                    )

            await asyncio.sleep(900)  # 15 minutes
```

### KnowledgeManagement Module (Node.js)
```javascript
// Standalone service for hierarchical knowledge organization
const { connect } = require('nats');

class KnowledgeManagementModule {
    constructor(knowledgeStore) {
        this.knowledgeStore = knowledgeStore; // Could be Trilium, Obsidian, etc.
    }
    async run() {
        this.nc = await connect({ servers: process.env.NATS_URL });
        // Subscribe to graph events
        const sub = this.nc.subscribe('graph.events.>');

        for await (const msg of sub) {
            const event = JSON.parse(msg.data);

            if (msg.subject === 'graph.events.node.created') {
                // Create corresponding knowledge node
                const knowledge = await this.knowledgeStore.createNode({
                    title: event.node.title,
                    content: event.node.description,
                    parentId: 'root'
                });

                // Publish knowledge created event
                this.nc.publish('knowledge.events.node.created', JSON.stringify({
                    knowledge_id: knowledge.id,
                    graph_node_id: event.node.id,
                    title: knowledge.title
                }));
            }
        }
    }
}
```

## Event Flow Orchestration

### Bidirectional Event Manager

```rust
pub struct BidirectionalEventManager {
    // Outbound
    projections: HashMap<SystemId, Box<dyn ExternalProjection>>,
    projection_dispatcher: ProjectionDispatcher,

    // Inbound
    ingest_handlers: HashMap<SystemId, Box<dyn IngestHandler>>,
    event_correlator: EventCorrelator,

    // Coordination
    event_store: EventStore,
    command_bus: CommandBus,
    metrics: EventFlowMetrics,
}

impl BidirectionalEventManager {
    pub async fn start(&mut self) -> Result<(), Error> {
        // Start projection dispatching
        self.start_projections().await?;

        // Start ingest handlers
        self.start_ingestion().await?;

        // Start correlation engine
        self.start_correlation().await?;

        Ok(())
    }

    async fn start_projections(&mut self) -> Result<(), Error> {
        // Subscribe to domain events
        let mut event_stream = self.event_store.subscribe_all().await?;

        tokio::spawn(async move {
            while let Some(event) = event_stream.next().await {
                // Dispatch to all projections
                self.projection_dispatcher.dispatch(event).await?;
            }
        });

        Ok(())
    }

    async fn start_ingestion(&mut self) -> Result<(), Error> {
        for (system_id, handler) in &self.ingest_handlers {
            let event_stream = handler.subscribe().await?;
            let correlator = self.event_correlator.clone();
            let command_bus = self.command_bus.clone();

            tokio::spawn(async move {
                while let Some(event) = event_stream.next().await {
                    // Correlate and transform
                    match correlator.correlate_inbound_event(event, system_id).await {
                        Ok(commands) => {
                            for command in commands {
                                command_bus.send(command).await?;
                            }
                        }
                        Err(e) => {
                            // Handle correlation error
                            log::error!("Correlation failed: {}", e);
                        }
                    }
                }
            });
        }

        Ok(())
    }
}
```

## Feedback Loops

### 1. Knowledge Enhancement Loop
```
Graph → GraphPersistence → Path Analysis → Community Detection → Graph (new relationships)
```

### 2. Workflow Automation Loop
```
Graph → WorkflowOrchestration → External Integration → Process Results → Graph (enrichment)
```

### 3. Document Intelligence Loop
```
Graph → DocumentIntelligence → Entity Extraction → Relationship Discovery → Graph (knowledge)
```

### 4. Search Discovery Loop
```
Graph → SearchDiscovery → User Queries → Recommendations → Graph (connections)
```

### 5. Version Control Loop
```
Graph → VersionControl → Change Tracking → Impact Analysis → Graph (evolution insights)
```

### 6. Knowledge Management Loop
```
Graph → KnowledgeManagement → Hierarchical Organization → Concept Linking → Graph (concepts)
```

### 7. Security Loop
```
Graph → CredentialManagement → Access Auditing → Security Analysis → Graph (access patterns)
```

### 8. Content Aggregation Loop
```
External Sources → ContentAggregation → Content Analysis → Topic Extraction → Graph (connections)
```

### 9. Infrastructure Loop
```
Graph → InfrastructureConfiguration → State Management → Change Detection → Graph (system knowledge)
```

### 10. Web Analytics Loop
```
WebGateway → Graph → Traffic Analysis → Performance Metrics → Graph (optimization insights)
```

## Configuration

```yaml
bidirectional_flow:
  # Outbound projections
  projections:
    graph_persistence:
      enabled: true
      batch_size: 1000
      flush_interval: 5s
      implementation: "neo4j"

    workflow_orchestration:
      enabled: true
      webhook_url: "https://orchestration.local/webhook/graph-events"
      implementation: "n8n"

    document_intelligence:
      enabled: true
      auto_extract: true
      implementation: "paperless"

    version_control:
      enabled: true
      artifacts:
        - "/git/thecowboyai/alchemist"
        - "/git/thecowboyai/cim-ipld"
      implementation: "git"

    knowledge_management:
      enabled: true
      api_url: "https://knowledge.local/api"
      sync_interval: 300s
      implementation: "trilium"

    credential_management:
      enabled: true
      api_url: "https://credentials.local/api"
      audit_access: true
      implementation: "vaultwarden"

    content_aggregation:
      enabled: true
      output_format: "json"
      sources:
        - url: "https://news.ycombinator.com/rss"
          type: "rss"
        - url: "https://lobste.rs/rss"
          type: "rss"
      implementation: "feedparser"

    web_gateway:
      enabled: true
      log_format: "json"
      metrics_endpoint: "/metrics"
      implementation: "nginx"

  # Inbound ingestion
  ingestion:
    graph_persistence:
      enabled: true
      change_detection: true
      poll_interval: 10s
      filters:
        - "source = 'external'"
        - "auto_discovered = true"

    workflow_orchestration:
      enabled: true
      webhook_port: 8080
      allowed_workflows:
        - "data-enrichment"
        - "entity-extraction"

    document_intelligence:
      enabled: true
      watch_folders:
        - "/documents/processed"
      event_types:
        - "entities.extracted"
        - "relationships.identified"

    version_control:
      enabled: true
      webhook_secret: "${VERSION_WEBHOOK_SECRET}"
      watch_artifacts:
        - "/git/thecowboyai/alchemist"
        - "/git/thecowboyai/cim-ipld"
      event_types:
        - "change.committed"
        - "merge.requested"
        - "issue.created"

    knowledge_management:
      enabled: true
      api_token: "${KNOWLEDGE_API_TOKEN}"
      sync_interval: 300s
      watch_hierarchies:
        - "root"
        - "projects"

    credential_management:
      enabled: true
      api_key: "${CREDENTIAL_API_KEY}"
      audit_events: true
      sync_interval: 600s

    content_aggregation:
      enabled: true
      poll_interval: 900s
      sources:
        - url: "https://news.ycombinator.com/rss"
          category: "tech-news"
        - url: "https://lobste.rs/rss"
          category: "programming"

    infrastructure_configuration:
      enabled: true
      watch_paths:
        - "/etc/nixos"
        - "/home/*/nixos"
      event_types:
        - "configuration.applied"
        - "state.changed"

    web_gateway:
      enabled: true
      log_path: "/var/log/nginx/access.log"
      error_log_path: "/var/log/nginx/error.log"
      parse_interval: 60s

  # Correlation rules
  correlation:
    deduplication_window: 60s
    enrichment_timeout: 5s
    max_commands_per_event: 10
```

## Benefits

1. **Continuous Learning**: The graph learns from all connected systems
2. **Automated Workflows**: Events trigger cross-system automations
3. **Knowledge Discovery**: External systems contribute new insights
4. **Feedback Loops**: Systems enhance each other's capabilities
5. **Unified View**: All information flows through the central graph
6. **Language Agnostic**: Modules can be written in any language with NATS support
7. **Fault Isolation**: Module failures don't affect core system
8. **Independent Scaling**: Each module can scale based on its load
9. **Easy Integration**: New systems just need to speak NATS
10. **Decoupled Development**: Teams can work on modules independently

## Next Steps

1. Define NATS subject hierarchy and event schemas
2. Create base module template in multiple languages (Rust, Python, Node.js)
3. Implement core graph event publisher
4. Build first module (Git) as reference implementation
5. Create module deployment configurations (systemd, Docker, Nix)
6. Set up NATS monitoring for all event flows
7. Document module development guide
8. Create integration test framework for modules
