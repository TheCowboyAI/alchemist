# Multi-System Projections Implementation Plan

## Overview

Our graph system needs to project events to multiple domain modules, each serving different business capabilities in the overall information management ecosystem.

## Domain Modules

### 1. GraphPersistence Module
- **Purpose**: Store and analyze graph structures
- **Implementations**: Neo4j for graph queries, JSON for portable representations
- **Capabilities**: Complex queries, path finding, graph algorithms, export/import

### 2. WorkflowOrchestration Module
- **Purpose**: Automate processes based on graph changes
- **Implementations**: n8n, Node-RED, custom workflow engines
- **Capabilities**: Event-triggered workflows, process automation, data flows

### 3. DocumentIntelligence Module
- **Purpose**: Extract knowledge from documents and link to graph
- **Implementations**: Paperless-NGx for OCR/organization, Nextcloud for storage
- **Capabilities**: Document linking, automatic tagging, content extraction

### 4. SearchDiscovery Module
- **Purpose**: Enable discovery of information across the graph
- **Implementations**: SearXNG for federated search, Elasticsearch for full-text
- **Capabilities**: Unified search, content indexing, semantic search

### 5. Communication Module
- **Purpose**: Notify and collaborate on graph changes
- **Implementations**: Email (SMTP), Matrix/Slack, RSS/Atom feeds
- **Capabilities**: Notifications, real-time updates, subscription feeds

### 6. Analytics Module
- **Purpose**: Monitor and analyze graph operations
- **Implementations**: Grafana for visualization, Prometheus for metrics
- **Capabilities**: Performance monitoring, usage analytics, health checks

### 7. IdentityManagement Module
- **Purpose**: Manage people, organizations, and their relationships
- **Implementations**: LDAP/AD, Keycloak, custom identity stores
- **Capabilities**: Person/org management, relationship tracking, directory sync

### 8. AccessControl Module
- **Purpose**: Manage operators, accounts, users, and agents
- **Implementations**: OAuth2/OIDC, RBAC, ABAC systems
- **Capabilities**: Authentication, authorization, agent registration, permissions

### 9. LocationIntelligence Module
- **Purpose**: Manage geographic and spatial information
- **Implementations**: PostGIS, Mapbox, OpenStreetMap
- **Capabilities**: Geocoding, spatial queries, geofencing, location tracking

### 10. NetworkAnalysis Module
- **Purpose**: Analyze and manage network relationships
- **Implementations**: NetworkX, Graph-tool, custom algorithms
- **Capabilities**: Community detection, centrality analysis, influence propagation

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Domain Events                            │
│  (NodeAdded, PersonCreated, LocationGeocoded, AgentRegistered)  │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Event Bridge (NATS)                         │
└─────────────────────────────┬───────────────────────────────────┘
                              │
    ┌─────┬─────┬─────┬─────┬─┴─┬─────┬─────┬─────┬─────┬───────┐
    ▼     ▼     ▼     ▼     ▼   ▼     ▼     ▼     ▼     ▼       ▼
┌──────┐┌────┐┌────┐┌────┐┌────┐┌────┐┌────┐┌────┐┌────┐┌────────┐
│Graph ││Work││Doc ││Know││Srch││Comm││Iden││Accs││Loc ││Network │
│Perst ││flow││Int ││Mgmt││Disc││unic││tity││Ctrl││Int ││Analysis│
└──────┘└────┘└────┘└────┘└────┘└────┘└────┘└────┘└────┘└────────┘

Legend:
- GraphPerst: GraphPersistence Module
- Workflow: WorkflowOrchestration Module
- DocInt: DocumentIntelligence Module
- KnowMgmt: KnowledgeManagement Module
- SrchDisc: SearchDiscovery Module
- Communic: Communication Module
- Identity: IdentityManagement Module
- AccsCtrl: AccessControl Module
- LocInt: LocationIntelligence Module
- Network: NetworkAnalysis Module
```

## Implementation Strategy

### Phase 1: Core Domain Modules (Week 1)
1. **GraphPersistence Module**
   - Event-to-storage transformation
   - Batch operations for performance
   - Transaction management
   - Multiple storage backend support (Neo4j, JSON, etc.)

2. **Analytics Module**
   - Real-time metrics collection
   - Performance monitoring
   - Health check endpoints
   - Dashboard integration

### Phase 2: Process Automation (Week 2)
3. **WorkflowOrchestration Module**
   - Event-triggered workflow execution
   - Custom workflow definitions
   - Integration with multiple engines (n8n, Node-RED)
   - Workflow templates for common patterns

4. **DocumentIntelligence Module**
   - Document-to-graph linking
   - Knowledge extraction pipelines
   - OCR and NLP integration
   - Metadata enrichment

### Phase 3: Discovery & Communication (Week 3)
5. **SearchDiscovery Module**
   - Multi-source search aggregation
   - Real-time index updates
   - Semantic search capabilities
   - Federated search across modules

6. **Communication Module**
   - Multi-channel notifications
   - Subscription management
   - Template-based formatting
   - Digest and real-time options

## Projection Patterns

### 1. Event-Driven Projection
```rust
#[async_trait]
pub trait ExternalProjection: Send + Sync {
    type Config;
    type Connection;

    async fn connect(config: Self::Config) -> Result<Self::Connection>;
    async fn project_event(&self, event: DomainEvent, conn: &mut Self::Connection) -> Result<()>;
    async fn handle_error(&self, error: ProjectionError) -> ErrorRecovery;
}
```

### 2. Batch Projection
```rust
pub struct BatchProjection<T: ExternalProjection> {
    buffer: Vec<DomainEvent>,
    flush_interval: Duration,
    max_batch_size: usize,
}
```

### 3. Resilient Projection
```rust
pub struct ResilientProjection<T: ExternalProjection> {
    projection: T,
    retry_policy: RetryPolicy,
    dead_letter_queue: DeadLetterQueue,
    circuit_breaker: CircuitBreaker,
}
```

## Configuration Schema

```yaml
modules:
  graph_persistence:
    enabled: true
    implementations:
      neo4j:
        uri: "bolt://localhost:7687"
        batch_size: 1000
        flush_interval: "5s"
      json:
        output_dir: "/data/graph-exports"
        formats: ["graphml", "gexf", "cytoscape"]
        compression: "zstd"

  workflow_orchestration:
    enabled: true
    implementations:
      n8n:
        webhook_url: "https://n8n.local/webhook/graph-events"
        api_key: "${N8N_API_KEY}"
      node_red:
        api_url: "https://nodered.local/api"

  document_intelligence:
    enabled: true
    implementations:
      paperless:
        api_url: "https://paperless.local/api"
        auto_tag: true
        link_documents: true
      nextcloud:
        webdav_url: "https://cloud.local/dav"

  search_discovery:
    enabled: true
    implementations:
      searxng:
        instance_url: "https://search.local"
        index_content: true
      elasticsearch:
        cluster_url: "https://elastic.local:9200"

  communication:
    enabled: true
    implementations:
      email:
        smtp_host: "mail.local"
        from: "graph@system.local"
      matrix:
        homeserver: "https://matrix.local"
      rss:
        feed_url: "/feeds/graph-events.xml"

  analytics:
    enabled: true
    implementations:
      prometheus:
        pushgateway: "https://prometheus.local:9091"
      grafana:
        api_url: "https://grafana.local/api"
```

## Error Handling

### Projection Failures
1. **Transient Failures**: Retry with exponential backoff
2. **Permanent Failures**: Send to dead letter queue
3. **System Unavailable**: Circuit breaker pattern
4. **Data Conflicts**: Conflict resolution strategies

### Consistency Guarantees
- **At-least-once delivery**: Events may be projected multiple times
- **Idempotent operations**: Projections must handle duplicates
- **Eventual consistency**: External systems may lag behind
- **Compensation**: Ability to undo projections if needed

## Monitoring & Observability

### Metrics
- Projection lag (time behind event stream)
- Success/failure rates per projection
- Batch sizes and processing times
- External system availability

### Health Checks
- Connection status to each system
- Queue depths and backlogs
- Error rates and types
- Performance degradation alerts

## Security Considerations

### Authentication
- API keys stored in secure vault
- OAuth2 for supported systems
- Certificate-based auth for databases
- Rotation policies for credentials

### Data Privacy
- PII handling in projections
- Encryption in transit and at rest
- Audit logging for compliance
- Data retention policies

## Testing Strategy

### Unit Tests
- Mock external systems
- Test event transformation logic
- Verify error handling

### Integration Tests
- Docker containers for external systems
- End-to-end projection flows
- Performance benchmarks

### Chaos Testing
- Network failures
- System unavailability
- Data corruption scenarios
- Recovery procedures

## Next Steps

1. Implement base `ExternalProjection` trait
2. Create Neo4j projection as reference implementation
3. Add configuration management system
4. Build monitoring dashboard
5. Create projection for each external system
6. Document deployment procedures
