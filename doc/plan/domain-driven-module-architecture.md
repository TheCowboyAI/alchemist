# Domain-Driven Module Architecture

## Core Domain Modules

### 1. GraphPersistence Module
**Domain Responsibility**: Persist and retrieve graph structures while preserving their mathematical properties

**Implementations**:
- `GraphPersistence<Neo4j>` - Uses Neo4j as backing store
- `GraphPersistence<DGraph>` - Uses DGraph as backing store
- `GraphPersistence<TigerGraph>` - Uses TigerGraph as backing store

**Interface**:
```rust
pub trait GraphPersistence {
    async fn persist_graph(&self, graph: &GraphAggregate) -> Result<()>;
    async fn load_graph(&self, id: GraphId) -> Result<GraphAggregate>;
    async fn execute_traversal(&self, query: TraversalQuery) -> Result<TraversalResult>;
}
```

**NATS Subjects**:
- Subscribes: `graph.events.structure.>`
- Publishes: `persistence.events.graph.>`

### 2. WorkflowOrchestration Module
**Domain Responsibility**: Orchestrate business workflows and process automation

**Implementations**:
- `WorkflowOrchestration<N8n>` - Uses n8n for execution
- `WorkflowOrchestration<Temporal>` - Uses Temporal for execution
- `WorkflowOrchestration<Camunda>` - Uses Camunda for execution

**Interface**:
```rust
pub trait WorkflowOrchestration {
    async fn execute_workflow(&self, workflow: WorkflowDefinition) -> Result<WorkflowResult>;
    async fn schedule_workflow(&self, schedule: Schedule) -> Result<()>;
    async fn monitor_execution(&self, execution_id: ExecutionId) -> Result<ExecutionStatus>;
}
```

**NATS Subjects**:
- Subscribes: `graph.events.workflow.>`
- Publishes: `orchestration.events.workflow.>`

### 3. DocumentIntelligence Module
**Domain Responsibility**: Extract knowledge and relationships from documents

**Implementations**:
- `DocumentIntelligence<Paperless>` - Uses Paperless-NGx
- `DocumentIntelligence<Elasticsearch>` - Uses Elasticsearch with NLP
- `DocumentIntelligence<CustomOCR>` - Custom OCR pipeline

**Interface**:
```rust
pub trait DocumentIntelligence {
    async fn process_document(&self, document: Document) -> Result<ExtractedKnowledge>;
    async fn find_related_documents(&self, query: SemanticQuery) -> Result<Vec<Document>>;
    async fn extract_entities(&self, content: &str) -> Result<Vec<Entity>>;
}
```

**NATS Subjects**:
- Subscribes: `graph.events.document.>`
- Publishes: `intelligence.events.document.>`

### 4. KnowledgeManagement Module
**Domain Responsibility**: Organize and structure knowledge hierarchically

**Implementations**:
- `KnowledgeManagement<Trilium>` - Uses Trilium Notes
- `KnowledgeManagement<Obsidian>` - Uses Obsidian
- `KnowledgeManagement<Dendron>` - Uses Dendron

**Interface**:
```rust
pub trait KnowledgeManagement {
    async fn create_knowledge_node(&self, knowledge: Knowledge) -> Result<KnowledgeId>;
    async fn link_knowledge(&self, source: KnowledgeId, target: KnowledgeId, relation: Relation) -> Result<()>;
    async fn traverse_knowledge_graph(&self, start: KnowledgeId, depth: u32) -> Result<KnowledgeGraph>;
}
```

**NATS Subjects**:
- Subscribes: `graph.events.knowledge.>`
- Publishes: `knowledge.events.>`

### 5. VersionControl Module
**Domain Responsibility**: Track evolution and changes of information artifacts

**Implementations**:
- `VersionControl<Git>` - Uses Git
- `VersionControl<Fossil>` - Uses Fossil
- `VersionControl<Mercurial>` - Uses Mercurial

**Interface**:
```rust
pub trait VersionControl {
    async fn track_change(&self, change: Change) -> Result<ChangeId>;
    async fn get_history(&self, artifact_id: ArtifactId) -> Result<Vec<Change>>;
    async fn create_branch(&self, branch: BranchDefinition) -> Result<BranchId>;
}
```

**NATS Subjects**:
- Subscribes: `graph.events.artifact.>`
- Publishes: `version.events.>`

### 6. CredentialManagement Module
**Domain Responsibility**: Secure storage and access control for sensitive information

**Implementations**:
- `CredentialManagement<Vaultwarden>` - Uses Vaultwarden
- `CredentialManagement<HashiVault>` - Uses HashiCorp Vault
- `CredentialManagement<1Password>` - Uses 1Password Connect

**Interface**:
```rust
pub trait CredentialManagement {
    async fn store_credential(&self, credential: Credential) -> Result<CredentialId>;
    async fn retrieve_credential(&self, id: CredentialId) -> Result<Credential>;
    async fn rotate_credential(&self, id: CredentialId) -> Result<()>;
}
```

**NATS Subjects**:
- Subscribes: `graph.events.credential.>`
- Publishes: `credential.events.>`

### 7. ContentAggregation Module
**Domain Responsibility**: Aggregate and curate content from multiple sources

**Implementations**:
- `ContentAggregation<RSS>` - RSS/Atom feeds
- `ContentAggregation<WebScraper>` - Web scraping
- `ContentAggregation<APIPoller>` - API polling

**Interface**:
```rust
pub trait ContentAggregation {
    async fn add_source(&self, source: ContentSource) -> Result<SourceId>;
    async fn fetch_content(&self, source_id: SourceId) -> Result<Vec<Content>>;
    async fn filter_content(&self, criteria: FilterCriteria) -> Result<Vec<Content>>;
}
```

**NATS Subjects**:
- Subscribes: `graph.events.content.>`
- Publishes: `aggregation.events.content.>`

### 8. InfrastructureConfiguration Module
**Domain Responsibility**: Manage infrastructure as code and system configuration

**Implementations**:
- `InfrastructureConfiguration<Nix>` - Uses Nix
- `InfrastructureConfiguration<Ansible>` - Uses Ansible
- `InfrastructureConfiguration<Terraform>` - Uses Terraform

**Interface**:
```rust
pub trait InfrastructureConfiguration {
    async fn apply_configuration(&self, config: Configuration) -> Result<()>;
    async fn get_current_state(&self) -> Result<InfrastructureState>;
    async fn rollback(&self, generation: Generation) -> Result<()>;
}
```

**NATS Subjects**:
- Subscribes: `graph.events.infrastructure.>`
- Publishes: `infrastructure.events.>`

### 9. WebGateway Module
**Domain Responsibility**: Manage web traffic routing and API gateway functionality

**Implementations**:
- `WebGateway<Nginx>` - Uses Nginx
- `WebGateway<Traefik>` - Uses Traefik
- `WebGateway<Envoy>` - Uses Envoy

**Interface**:
```rust
pub trait WebGateway {
    async fn route_request(&self, request: WebRequest) -> Result<WebResponse>;
    async fn update_routing_rules(&self, rules: RoutingRules) -> Result<()>;
    async fn get_metrics(&self) -> Result<GatewayMetrics>;
}
```

**NATS Subjects**:
- Subscribes: `graph.events.gateway.>`
- Publishes: `gateway.events.>`

### 10. SearchDiscovery Module
**Domain Responsibility**: Enable discovery and search across all information

**Implementations**:
- `SearchDiscovery<SearXNG>` - Uses SearXNG
- `SearchDiscovery<Elasticsearch>` - Uses Elasticsearch
- `SearchDiscovery<MeiliSearch>` - Uses MeiliSearch

**Interface**:
```rust
pub trait SearchDiscovery {
    async fn index_content(&self, content: Content) -> Result<()>;
    async fn search(&self, query: SearchQuery) -> Result<SearchResults>;
    async fn get_recommendations(&self, context: Context) -> Result<Vec<Recommendation>>;
}
```

**NATS Subjects**:
- Subscribes: `graph.events.searchable.>`
- Publishes: `search.events.>`

### 11. Communication Module
**Domain Responsibility**: Enable multi-channel communication and notifications

**Implementations**:
- `Communication<Email>` - SMTP email communication
- `Communication<Matrix>` - Matrix protocol messaging
- `Communication<Slack>` - Slack integration

**Interface**:
```rust
pub trait Communication {
    async fn send_notification(&self, notification: Notification) -> Result<()>;
    async fn subscribe_to_updates(&self, subscription: Subscription) -> Result<SubscriptionId>;
    async fn get_communication_history(&self, filter: HistoryFilter) -> Result<Vec<Message>>;
}
```

**NATS Subjects**:
- Subscribes: `graph.events.notification.>`
- Publishes: `communication.events.>`

### 12. IdentityManagement Module
**Domain Responsibility**: Manage people, organizations, and their relationships

**Implementations**:
- `IdentityManagement<LDAP>` - LDAP/Active Directory integration
- `IdentityManagement<Keycloak>` - Keycloak identity provider
- `IdentityManagement<Custom>` - Custom identity store

**Interface**:
```rust
pub trait IdentityManagement {
    // Person management
    async fn create_person(&self, person: Person) -> Result<PersonId>;
    async fn update_person(&self, id: PersonId, updates: PersonUpdate) -> Result<()>;
    async fn find_person(&self, criteria: PersonCriteria) -> Result<Vec<Person>>;

    // Organization management
    async fn create_organization(&self, org: Organization) -> Result<OrganizationId>;
    async fn create_organizational_unit(&self, unit: OrganizationalUnit) -> Result<UnitId>;
    async fn assign_person_to_unit(&self, person: PersonId, unit: UnitId, role: Role) -> Result<()>;

    // Relationship management
    async fn establish_relationship(&self, source: EntityId, target: EntityId, rel: Relationship) -> Result<()>;
    async fn get_relationships(&self, entity: EntityId) -> Result<Vec<Relationship>>;
}
```

**NATS Subjects**:
- Subscribes: `graph.events.identity.>`
- Publishes: `identity.events.person.>`, `identity.events.organization.>`

### 13. AccessControl Module
**Domain Responsibility**: Manage operators, accounts, users, and agents with their permissions

**Implementations**:
- `AccessControl<OAuth2>` - OAuth2/OIDC provider
- `AccessControl<RBAC>` - Role-based access control
- `AccessControl<ABAC>` - Attribute-based access control

**Interface**:
```rust
pub trait AccessControl {
    // Operator/Account/User management
    async fn create_operator(&self, operator: Operator) -> Result<OperatorId>;
    async fn create_account(&self, account: Account) -> Result<AccountId>;
    async fn create_user(&self, user: User) -> Result<UserId>;

    // Agent management
    async fn register_agent(&self, agent: Agent) -> Result<AgentId>;
    async fn grant_agent_capabilities(&self, agent: AgentId, capabilities: Vec<Capability>) -> Result<()>;

    // Permission management
    async fn check_permission(&self, subject: SubjectId, resource: ResourceId, action: Action) -> Result<bool>;
    async fn grant_permission(&self, grant: PermissionGrant) -> Result<()>;
}
```

**NATS Subjects**:
- Subscribes: `graph.events.access.>`
- Publishes: `access.events.operator.>`, `access.events.agent.>`

### 14. LocationIntelligence Module
**Domain Responsibility**: Manage geographic and spatial information

**Implementations**:
- `LocationIntelligence<PostGIS>` - PostGIS spatial database
- `LocationIntelligence<Mapbox>` - Mapbox geocoding/mapping
- `LocationIntelligence<OpenStreetMap>` - OSM integration

**Interface**:
```rust
pub trait LocationIntelligence {
    // Location management
    async fn create_location(&self, location: Location) -> Result<LocationId>;
    async fn geocode_address(&self, address: Address) -> Result<GeoCoordinates>;
    async fn reverse_geocode(&self, coords: GeoCoordinates) -> Result<Address>;

    // Spatial queries
    async fn find_locations_within(&self, boundary: GeoBoundary) -> Result<Vec<Location>>;
    async fn calculate_distance(&self, from: LocationId, to: LocationId) -> Result<Distance>;
    async fn find_nearest(&self, point: GeoCoordinates, category: LocationCategory) -> Result<Vec<Location>>;

    // Geofencing
    async fn create_geofence(&self, fence: Geofence) -> Result<GeofenceId>;
    async fn check_location_in_fence(&self, location: LocationId, fence: GeofenceId) -> Result<bool>;
}
```

**NATS Subjects**:
- Subscribes: `graph.events.location.>`
- Publishes: `location.events.geocoded.>`, `location.events.movement.>`

### 15. NetworkAnalysis Module
**Domain Responsibility**: Analyze and manage network relationships and connections

**Implementations**:
- `NetworkAnalysis<NetworkX>` - NetworkX graph analysis
- `NetworkAnalysis<GraphTool>` - Graph-tool analytics
- `NetworkAnalysis<Custom>` - Custom network algorithms

**Interface**:
```rust
pub trait NetworkAnalysis {
    // Network structure
    async fn analyze_network(&self, network: NetworkId) -> Result<NetworkMetrics>;
    async fn find_communities(&self, network: NetworkId) -> Result<Vec<Community>>;
    async fn calculate_centrality(&self, network: NetworkId, node: NodeId) -> Result<CentralityScores>;

    // Relationship analysis
    async fn find_shortest_path(&self, from: EntityId, to: EntityId) -> Result<Vec<Relationship>>;
    async fn find_common_connections(&self, entities: Vec<EntityId>) -> Result<Vec<EntityId>>;
    async fn predict_relationships(&self, entity: EntityId) -> Result<Vec<PredictedRelationship>>;

    // Influence propagation
    async fn simulate_influence(&self, source: EntityId, influence: Influence) -> Result<PropagationResult>;
    async fn identify_influencers(&self, network: NetworkId) -> Result<Vec<Influencer>>;
}
```

**NATS Subjects**:
- Subscribes: `graph.events.network.>`
- Publishes: `network.events.analysis.>`, `network.events.relationship.>`

## Module Communication Pattern

```
┌─────────────────────────────────────────────────────────────────┐
│                     Domain Event Bus (NATS)                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  graph.events.{aggregate}.{event}                               │
│  ├── graph.events.structure.node_added                         │
│  ├── graph.events.workflow.execution_requested                 │
│  ├── graph.events.document.uploaded                            │
│  ├── graph.events.knowledge.concept_created                    │
│  ├── graph.events.identity.person_created                      │
│  ├── graph.events.access.agent_registered                      │
│  ├── graph.events.location.address_geocoded                    │
│  └── graph.events.network.relationship_established             │
│                                                                  │
│  {module}.events.{capability}.{event}                          │
│  ├── persistence.events.graph.persisted                        │
│  ├── orchestration.events.workflow.completed                   │
│  ├── intelligence.events.document.entities_extracted           │
│  ├── knowledge.events.hierarchy.updated                        │
│  ├── identity.events.person.synchronized                       │
│  ├── access.events.permission.granted                          │
│  ├── location.events.movement.detected                         │
│  └── network.events.community.discovered                       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Anti-Corruption Layer Pattern

Each module implementation includes an ACL to translate between:
- Domain concepts ↔ External system concepts
- Domain events ↔ External system APIs
- Domain models ↔ External system data structures

Example:
```rust
// ACL for Neo4j implementation
impl GraphPersistenceACL for Neo4jACL {
    fn translate_node_to_cypher(&self, node: &Node) -> CypherQuery {
        // Translate domain node to Neo4j Cypher
    }

    fn translate_cypher_to_node(&self, result: CypherResult) -> Node {
        // Translate Neo4j result to domain node
    }
}
```

## Benefits of Domain-Driven Modules

1. **Domain Focus**: Modules named by their business capability, not technology
2. **Substitutability**: Easy to swap implementations without changing domain
3. **Testability**: Can test with in-memory implementations
4. **Evolution**: Technology can change without affecting domain model
5. **Clarity**: Clear separation of concerns and responsibilities
