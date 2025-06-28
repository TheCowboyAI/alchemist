# CIM-Integrated Event-Sourced Graph Architecture

## Overview

Information Alchemist operates as a specialized leaf node in the Composable Information Machine (CIM) cluster, providing a sophisticated user interface for designing, creating, manipulating, and analyzing graphs of Domain-Driven Design components. This architecture integrates event sourcing, NATS messaging, conceptual spaces theory, and prepares for AI agent integration.

## Core Principles

1. **CIM Leaf Node**: Information Alchemist is a UI-focused leaf node in the CIM distributed system
2. **NATS-First Communication**: All backend communication occurs through NATS subjects
3. **Dual-Store Architecture**: Event Store for state transitions, Object Store for content
4. **Conceptual Spaces**: Spatial knowledge representation based on Gärdenfors' theory
5. **AI-Ready**: Prepared for intelligent agent integration
6. **Modular "Lego Blocks"**: Composable architecture with clear interfaces

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    CIM Cluster                               │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ Backend Nodes│  │ Event Store  │  │  Object Store   │  │
│  │ (NATS-based) │  │    Nodes     │  │     Nodes       │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬────────┘  │
└─────────┼──────────────────┼──────────────────┼────────────┘
          │                  │                  │
          └──────────────────┴──────────────────┘
                             │
                        NATS Messaging
                             │
┌─────────────────────────────▼───────────────────────────────┐
│              Information Alchemist (CIM Leaf Node)          │
├─────────────────────────────────────────────────────────────┤
│                    Presentation Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ Bevy Systems │  │  Components  │  │   UI Events     │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬────────┘  │
├─────────┼──────────────────┼──────────────────┼────────────┤
│                    Application Layer                         │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │   Command   │  │    Query     │  │   Event         │  │
│  │  Handlers   │  │  Handlers    │  │  Projections    │  │
│  └──────┬──────┘  └──────┬───────┘  └────────┬────────┘  │
├─────────┼──────────────────┼──────────────────┼────────────┤
│                     Domain Layer                            │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ Aggregates  │  │   Commands   │  │    Events       │  │
│  │   (Graph)   │  │              │  │                 │  │
│  └──────┬──────┘  └──────┬───────┘  └────────┬────────┘  │
├─────────┼──────────────────┼──────────────────┼────────────┤
│                  Infrastructure Layer                       │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ NATS Client │  │  Event Bridge│  │  Local Cache    │  │
│  │             │  │              │  │   (Petgraph)    │  │
│  └─────────────┘  └──────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Domain Model

### Aggregates

#### Graph (Aggregate Root)
```rust
pub struct Graph {
    pub id: GraphId,
    pub metadata: GraphMetadata,
    pub version: u64,

    // Petgraph for efficient structure
    graph: StableGraph<NodeId, EdgeId>,

    // Component storage
    nodes: HashMap<NodeId, Node>,
    edges: HashMap<EdgeId, Edge>,

    // Conceptual space positioning
    conceptual_space: ConceptualSpace,

    // Game theory components
    strategies: HashMap<NodeId, StrategyComponent>,
    coalitions: Vec<Coalition>,

    // Indices for fast queries
    indices: GraphIndices,
}

pub struct GraphMetadata {
    pub name: String,
    pub bounded_context: String,  // DDD context
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub tags: Vec<String>,
    pub cid: Option<ContentId>,   // Object store reference
}
```

### Entities

#### Node with Conceptual Positioning
```rust
pub struct Node {
    pub id: NodeId,
    pub content: NodeContent,
    pub position: Position3D,
    pub conceptual_position: ConceptualPosition,
    pub components: HashSet<ComponentId>,
}

#[derive(Component)]
pub struct ConceptualPosition {
    // Primary dimensions (visible)
    pub spatial: Vec3,

    // Semantic dimensions (computed)
    pub properties: HashMap<String, f32>,

    // Similarity metrics
    pub centroid_distance: f32,
    pub category_membership: f32,
}

pub struct NodeContent {
    pub label: String,
    pub node_type: NodeType,
    pub properties: HashMap<String, Value>,
    pub content_cid: Option<ContentId>,  // Large content in object store
}
```

#### Edge with Semantic Relationships
```rust
pub struct Edge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub relationship: EdgeRelationship,
    pub weight: f32,
    pub semantic_similarity: f32,
}

pub struct EdgeRelationship {
    pub relationship_type: RelationshipType,
    pub properties: HashMap<String, Value>,
    pub bidirectional: bool,
}
```

### Game Theory Components

```rust
#[derive(Component)]
pub struct StrategyComponent {
    pub strategy_type: StrategyType,
    pub utility_function: UtilityFunction,
    pub decision_history: Vec<Decision>,
}

#[derive(Component)]
pub struct CoalitionComponent {
    pub coalition_id: CoalitionId,
    pub members: HashSet<NodeId>,
    pub shared_utility: f32,
    pub formation_time: SystemTime,
}

#[derive(Component)]
pub struct UtilityComponent {
    pub current_utility: f32,
    pub utility_history: Vec<(SystemTime, f32)>,
    pub optimization_target: OptimizationTarget,
}
```

## NATS Integration

### Subject Naming Convention

Following CIM standards:

```rust
// Commands
pub const GRAPH_CREATE: &str = "graph.commands.create";
pub const GRAPH_DELETE: &str = "graph.commands.delete";
pub const NODE_ADD: &str = "node.commands.add";
pub const NODE_REMOVE: &str = "node.commands.remove";
pub const EDGE_CONNECT: &str = "edge.commands.connect";
pub const EDGE_DISCONNECT: &str = "edge.commands.disconnect";

// Events
pub const GRAPH_CREATED: &str = "graph.events.created";
pub const GRAPH_DELETED: &str = "graph.events.deleted";
pub const NODE_ADDED: &str = "node.events.added";
pub const NODE_REMOVED: &str = "node.events.removed";
pub const EDGE_CONNECTED: &str = "edge.events.connected";
pub const EDGE_DISCONNECTED: &str = "edge.events.disconnected";

// Queries
pub const GRAPH_GET: &str = "graph.queries.get";
pub const GRAPH_LIST: &str = "graph.queries.list";
pub const NODE_FIND: &str = "node.queries.find";

// AI Agent subjects
pub const AGENT_ANALYZE: &str = "agent.commands.analyze";
pub const AGENT_SUGGEST: &str = "agent.commands.suggest";
pub const AGENT_RESULTS: &str = "agent.events.results";
```

### NATS Client Integration

```rust
pub struct NatsIntegration {
    client: async_nats::Client,
    jetstream: async_nats::jetstream::Context,
    subscriptions: HashMap<String, Subscription>,
}

impl NatsIntegration {
    pub async fn publish_command(&self, command: GraphCommand) -> Result<()> {
        let subject = match &command {
            GraphCommand::CreateGraph { .. } => GRAPH_CREATE,
            GraphCommand::AddNode { .. } => NODE_ADD,
            // ... other mappings
        };

        let payload = serde_json::to_vec(&command)?;
        self.client.publish(subject, payload.into()).await?;
        Ok(())
    }

    pub async fn subscribe_to_events(&mut self) -> Result<()> {
        // Subscribe to all graph events
        let sub = self.client.subscribe("graph.events.>").await?;
        self.subscriptions.insert("graph_events".to_string(), sub);

        // Subscribe to agent results
        let agent_sub = self.client.subscribe(AGENT_RESULTS).await?;
        self.subscriptions.insert("agent_results".to_string(), agent_sub);

        Ok(())
    }
}
```

## Distributed Storage Architecture

### Event Store (via NATS JetStream)

```rust
pub struct DistributedEventStore {
    jetstream: async_nats::jetstream::Context,
    stream_name: String,
    local_cache: Arc<RwLock<Vec<EventEnvelope>>>,
}

impl DistributedEventStore {
    pub async fn append(&self, event: DomainEvent) -> Result<EventEnvelope> {
        let envelope = EventEnvelope {
            event_id: EventId(Uuid::new_v4()),
            event,
            timestamp: SystemTime::now(),
            sequence: 0, // JetStream will assign
            correlation_id: None,
            causation_id: None,
        };

        // Publish to JetStream
        let subject = format!("events.{}", envelope.event.event_type());
        let payload = serde_json::to_vec(&envelope)?;

        let ack = self.jetstream
            .publish(subject, payload.into())
            .await?;

        // Update local cache
        let mut cache = self.local_cache.write().await;
        cache.push(envelope.clone());

        Ok(envelope)
    }
}
```

### Object Store Integration

```rust
pub struct ObjectStoreClient {
    nats_client: async_nats::Client,
    cache: Arc<DashMap<ContentId, Vec<u8>>>,
}

impl ObjectStoreClient {
    pub async fn store(&self, content: &[u8]) -> Result<ContentId> {
        // Calculate content ID (CID)
        let cid = ContentId::from_content(content);

        // Store via NATS request
        let response = self.nats_client
            .request("objectstore.put", content.into())
            .await?;

        // Cache locally
        self.cache.insert(cid.clone(), content.to_vec());

        Ok(cid)
    }

    pub async fn retrieve(&self, cid: &ContentId) -> Result<Vec<u8>> {
        // Check cache first
        if let Some(content) = self.cache.get(cid) {
            return Ok(content.clone());
        }

        // Fetch from object store
        let response = self.nats_client
            .request("objectstore.get", cid.to_bytes().into())
            .await?;

        let content = response.payload.to_vec();
        self.cache.insert(cid.clone(), content.clone());

        Ok(content)
    }
}
```

## Conceptual Spaces Implementation

### Spatial Knowledge Representation

```rust
pub struct ConceptualSpace {
    dimensions: Vec<Dimension>,
    regions: HashMap<CategoryId, ConvexRegion>,
    similarity_metric: SimilarityMetric,
}

impl ConceptualSpace {
    pub fn calculate_position(&self, node: &Node) -> ConceptualPosition {
        let mut position = ConceptualPosition {
            spatial: Vec3::ZERO,
            properties: HashMap::new(),
            centroid_distance: 0.0,
            category_membership: 0.0,
        };

        // Map properties to dimensions
        for dimension in &self.dimensions {
            let value = dimension.extract_value(node);
            position.properties.insert(dimension.name.clone(), value);

            // Update spatial position
            match dimension.axis {
                Axis::X => position.spatial.x = value,
                Axis::Y => position.spatial.y = value,
                Axis::Z => position.spatial.z = value,
            }
        }

        // Calculate distances
        position.centroid_distance = self.distance_to_centroid(&position);
        position.category_membership = self.category_membership_score(&position);

        position
    }

    pub fn find_similar_nodes(&self, node: &Node, threshold: f32) -> Vec<NodeId> {
        // Use spatial indexing for efficient similarity search
        self.similarity_metric.find_similar(node, threshold)
    }
}
```

### Force-Directed Layout with Conceptual Forces

```rust
fn calculate_conceptual_forces(
    node_a: &Node,
    node_b: &Node,
    space: &ConceptualSpace,
) -> Vec3 {
    // Semantic similarity attracts
    let semantic_force = space.semantic_similarity(node_a, node_b)
        * ATTRACTION_CONSTANT;

    // Different categories repel
    let category_force = if node_a.category() != node_b.category() {
        REPULSION_CONSTANT
    } else {
        0.0
    };

    // Temporal proximity attracts
    let temporal_force = calculate_temporal_proximity(
        node_a.created_at,
        node_b.created_at
    ) * TEMPORAL_CONSTANT;

    // Game theory influence
    let strategic_force = calculate_strategic_alignment(
        &node_a.strategy,
        &node_b.strategy
    ) * STRATEGY_CONSTANT;

    combine_forces(semantic_force, category_force, temporal_force, strategic_force)
}
```

## AI Agent Integration

### Agent Communication Interface

```rust
pub struct AgentInterface {
    nats_client: async_nats::Client,
    agent_registry: HashMap<AgentId, AgentCapabilities>,
}

impl AgentInterface {
    pub async fn request_analysis(
        &self,
        graph_id: GraphId,
        analysis_type: AnalysisType,
    ) -> Result<AnalysisRequestId> {
        let request = AgentRequest::Analyze {
            graph_id,
            analysis_type,
            parameters: Default::default(),
        };

        let response = self.nats_client
            .request(AGENT_ANALYZE, serde_json::to_vec(&request)?.into())
            .await?;

        let request_id: AnalysisRequestId = serde_json::from_slice(&response.payload)?;
        Ok(request_id)
    }

    pub async fn subscribe_to_results(&self) -> Result<Subscription> {
        self.nats_client.subscribe(AGENT_RESULTS).await
    }
}
```

### Agent-Ready Components

```rust
#[derive(Component)]
pub struct AgentSuggestion {
    pub agent_id: AgentId,
    pub suggestion_type: SuggestionType,
    pub confidence: f32,
    pub reasoning: String,
}

#[derive(Component)]
pub struct AgentAnalysis {
    pub analysis_id: AnalysisRequestId,
    pub results: AnalysisResults,
    pub timestamp: SystemTime,
}
```

## Modular Plugin Architecture

### Plugin System

```rust
pub trait GraphPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn build(&self, app: &mut App);
    fn dependencies(&self) -> Vec<&str>;
}

// Core plugins
pub struct ConceptualSpacePlugin;
pub struct GameTheoryPlugin;
pub struct NatsIntegrationPlugin;
pub struct EventSourcingPlugin;

// Compose plugins
pub fn build_app() -> App {
    let mut app = App::new();

    // Core Bevy plugins
    app.add_plugins(DefaultPlugins);

    // CIM integration plugins
    app.add_plugin(NatsIntegrationPlugin)
        .add_plugin(EventSourcingPlugin)
        .add_plugin(ConceptualSpacePlugin)
        .add_plugin(GameTheoryPlugin);

    // Domain plugins
    app.add_plugin(GraphVisualizationPlugin)
        .add_plugin(InteractionPlugin)
        .add_plugin(LayoutPlugin);

    // AI plugins (optional)
    if cfg!(feature = "ai-agents") {
        app.add_plugin(AgentInterfacePlugin);
    }

    app
}
```

## Performance Optimizations

### Distributed Performance
- NATS subject filtering for relevant events only
- Local caching of frequently accessed data
- Lazy loading from object store
- Event batching for bulk operations

### Conceptual Space Performance
- Spatial indexing for similarity queries
- Approximate nearest neighbor algorithms
- Dimension reduction for visualization
- Progressive detail loading

### Game Theory Performance
- Cached utility calculations
- Parallel strategy evaluation
- Coalition formation heuristics
- Incremental updates

## Security Architecture

### NATS Security
```rust
pub struct SecurityConfig {
    pub jwt_auth: bool,
    pub tls_required: bool,
    pub user_credentials: Option<PathBuf>,
    pub nkey_seed: Option<String>,
}

impl NatsIntegration {
    pub async fn connect_secure(config: SecurityConfig) -> Result<Self> {
        let mut options = async_nats::ConnectOptions::new();

        if let Some(creds_path) = config.user_credentials {
            options = options.credentials_file(creds_path).await?;
        }

        if config.tls_required {
            options = options.require_tls(true);
        }

        let client = options.connect("nats://cim-cluster:4222").await?;
        // ... rest of initialization
    }
}
```

## Migration Path

1. **Phase 0**: NATS Integration Setup
2. **Phase 1**: Event Infrastructure with NATS
3. **Phase 2**: Distributed Storage Integration
4. **Phase 3**: Conceptual Spaces Implementation
5. **Phase 4**: Game Theory Components
6. **Phase 5**: AI Agent Interface
7. **Phase 6**: Full CIM Integration
8. **Phase 7**: Performance Optimization

## Key Differences from Standalone Architecture

1. **Distributed First**: All communication via NATS
2. **Dual Storage**: Event Store + Object Store
3. **Conceptual Spaces**: Spatial knowledge representation
4. **AI Ready**: Agent communication built-in
5. **Game Theory**: Strategic components included
6. **Modular Plugins**: True "Lego block" architecture
7. **CIM Integration**: Part of larger distributed system
