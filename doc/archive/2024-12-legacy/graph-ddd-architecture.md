# Graph Domain - DDD Architecture

## Domain Model

Following Domain-Driven Design principles, our Graph domain is structured as:

### Aggregate: Domain<Graph>

```rust
/// The Graph Aggregate - our consistency boundary
pub struct GraphAggregate {
    /// Aggregate Root
    pub root: Graph,
    /// Entities within this aggregate
    pub subgraphs: HashMap<SubgraphId, Subgraph>,
    /// Invariants and business rules
    pub policies: GraphPolicies,
}

/// The Aggregate Root - maintains consistency
pub struct Graph {
    pub identity: GraphId,
    pub metadata: GraphMetadata,
    pub journey: GraphJourney,
    /// References to all subgraphs in this domain
    pub subgraph_registry: HashSet<SubgraphId>,
}
```

### Entities: Subgraphs

```rust
/// Subgraph Entity - has identity and lifecycle
pub struct Subgraph {
    pub identity: SubgraphId,
    pub parent_graph: GraphId,
    /// Components (Nodes) in this subgraph
    pub nodes: HashSet<NodeId>,
    /// Systems (Edges) connecting nodes
    pub edges: HashSet<EdgeId>,
    /// Subgraph-specific invariants
    pub constraints: SubgraphConstraints,
}
```

### Value Objects

```rust
/// Node as Component (Value Object in DDD terms)
#[derive(Component)]
pub struct Node {
    pub content: NodeContent,
    pub position: SpatialPosition,
    pub properties: NodeProperties,
}

/// Edge as System (represents behavior/communication)
#[derive(Component)]
pub struct Edge {
    pub source: NodeId,
    pub target: NodeId,
    pub behavior: EdgeBehavior,
    pub communication_type: CommunicationType,
}

/// Edge behavior defines how nodes communicate
pub enum EdgeBehavior {
    /// Synchronous system call
    SystemCall { latency: Duration },
    /// Asynchronous event emission
    EventEmission { event_type: String },
    /// Data transformation
    Transform { function: TransformFunction },
    /// Query relationship
    Query { predicate: QueryPredicate },
}
```

## Event-Driven Communication

### Domain Events

```rust
/// Events that Entities use to communicate
pub enum GraphDomainEvent {
    // Aggregate-level events
    GraphCreated { graph_id: GraphId, metadata: GraphMetadata },

    // Entity-level events
    SubgraphAdded { graph_id: GraphId, subgraph: Subgraph },
    SubgraphRemoved { graph_id: GraphId, subgraph_id: SubgraphId },

    // Component events (within entities)
    NodeAddedToSubgraph { subgraph_id: SubgraphId, node: Node },
    NodeRemovedFromSubgraph { subgraph_id: SubgraphId, node_id: NodeId },

    // System events (edges)
    SystemConnected {
        subgraph_id: SubgraphId,
        edge: Edge,
        behavior: EdgeBehavior,
    },
    SystemDisconnected {
        subgraph_id: SubgraphId,
        edge_id: EdgeId,
    },

    // Inter-entity communication
    InterSubgraphMessage {
        from_subgraph: SubgraphId,
        to_subgraph: SubgraphId,
        message: DomainMessage,
    },
}
```

### Entity Communication Pattern

```rust
impl Subgraph {
    /// Entities communicate through events
    pub fn send_message_to(
        &self,
        target: SubgraphId,
        message: DomainMessage,
        event_writer: &mut EventWriter<GraphDomainEvent>,
    ) {
        event_writer.send(GraphDomainEvent::InterSubgraphMessage {
            from_subgraph: self.identity,
            to_subgraph: target,
            message,
        });
    }

    /// Process incoming messages
    pub fn handle_message(
        &mut self,
        message: DomainMessage,
        commands: &mut Commands,
    ) -> Result<(), DomainError> {
        match message {
            DomainMessage::RequestNode { node_id } => {
                // Handle node request
            }
            DomainMessage::PropagateChange { change } => {
                // Handle change propagation
            }
            // ... other message types
        }
        Ok(())
    }
}
```

## Petgraph Integration with DDD

### Graph Aggregate in Petgraph

```rust
/// Petgraph representation of our domain model
pub struct GraphModel {
    /// The aggregate root graph
    pub root: petgraph::Graph<SubgraphNode, SubgraphEdge>,
    /// Individual subgraph structures
    pub subgraphs: HashMap<SubgraphId, petgraph::Graph<Node, Edge>>,
    /// Aggregate identity
    pub aggregate_id: GraphId,
}

/// Subgraph as a node in the root graph
pub struct SubgraphNode {
    pub subgraph_id: SubgraphId,
    pub entity_data: Subgraph,
}

/// Inter-subgraph relationships
pub struct SubgraphEdge {
    pub relationship_type: SubgraphRelationship,
    pub communication_channel: CommunicationChannel,
}
```

### Repository Pattern

```rust
/// Repository for Graph Aggregates
pub trait GraphRepository {
    /// Load entire aggregate
    fn load_aggregate(&self, id: GraphId) -> Result<GraphAggregate, RepositoryError>;

    /// Save aggregate maintaining consistency
    fn save_aggregate(&mut self, aggregate: &GraphAggregate) -> Result<(), RepositoryError>;

    /// Query subgraphs within aggregate
    fn find_subgraphs(&self, graph_id: GraphId, criteria: SubgraphCriteria) -> Vec<Subgraph>;
}

/// Event-sourced implementation
pub struct EventSourcedGraphRepository {
    event_store: EventStore,
    snapshot_store: SnapshotStore,
}

impl GraphRepository for EventSourcedGraphRepository {
    fn load_aggregate(&self, id: GraphId) -> Result<GraphAggregate, RepositoryError> {
        // Try snapshot first
        if let Some(snapshot) = self.snapshot_store.get_latest(id)? {
            // Apply events since snapshot
            let events = self.event_store.get_events_since(id, snapshot.version)?;
            return Ok(replay_from_snapshot(snapshot, events)?);
        }

        // Full replay from event store
        let events = self.event_store.get_events_for_aggregate(id)?;
        Ok(replay_from_events(events)?)
    }

    fn save_aggregate(&mut self, aggregate: &GraphAggregate) -> Result<(), RepositoryError> {
        // Extract uncommitted events
        let events = aggregate.get_uncommitted_events();

        // Persist to event store
        for event in events {
            self.event_store.append(event)?;
        }

        // Create snapshot if needed
        if should_snapshot(aggregate) {
            self.snapshot_store.save(create_snapshot(aggregate))?;
        }

        Ok(())
    }
}
```

## Domain Services

### Graph Construction Service

```rust
pub struct GraphConstructionService;

impl GraphConstructionService {
    /// Create a new graph domain
    pub fn create_domain(
        &self,
        name: String,
        event_store: &mut EventStore,
    ) -> Result<GraphAggregate, DomainError> {
        let graph_id = GraphId::new();

        // Create aggregate root
        let root = Graph {
            identity: graph_id,
            metadata: GraphMetadata { name, ..Default::default() },
            journey: GraphJourney::new(),
            subgraph_registry: HashSet::new(),
        };

        // Record creation event
        let event = GraphDomainEvent::GraphCreated {
            graph_id,
            metadata: root.metadata.clone(),
        };

        event_store.append_domain_event(graph_id, event)?;

        Ok(GraphAggregate {
            root,
            subgraphs: HashMap::new(),
            policies: GraphPolicies::default(),
        })
    }

    /// Add a subgraph entity to the domain
    pub fn add_subgraph(
        &self,
        aggregate: &mut GraphAggregate,
        name: String,
    ) -> Result<SubgraphId, DomainError> {
        // Enforce aggregate invariants
        aggregate.policies.validate_subgraph_addition(&name)?;

        let subgraph_id = SubgraphId::new();
        let subgraph = Subgraph {
            identity: subgraph_id,
            parent_graph: aggregate.root.identity,
            nodes: HashSet::new(),
            edges: HashSet::new(),
            constraints: SubgraphConstraints::default(),
        };

        // Update aggregate
        aggregate.root.subgraph_registry.insert(subgraph_id);
        aggregate.subgraphs.insert(subgraph_id, subgraph.clone());

        // Emit event
        aggregate.add_event(GraphDomainEvent::SubgraphAdded {
            graph_id: aggregate.root.identity,
            subgraph,
        });

        Ok(subgraph_id)
    }
}
```

### System Connection Service

```rust
pub struct SystemConnectionService;

impl SystemConnectionService {
    /// Connect nodes with a system (edge)
    pub fn connect_nodes(
        &self,
        aggregate: &mut GraphAggregate,
        subgraph_id: SubgraphId,
        source: NodeId,
        target: NodeId,
        behavior: EdgeBehavior,
    ) -> Result<EdgeId, DomainError> {
        // Get subgraph entity
        let subgraph = aggregate.subgraphs.get_mut(&subgraph_id)
            .ok_or(DomainError::SubgraphNotFound)?;

        // Validate connection
        subgraph.constraints.validate_connection(&source, &target, &behavior)?;

        let edge_id = EdgeId::new();
        let edge = Edge {
            source,
            target,
            behavior: behavior.clone(),
            communication_type: behavior.communication_type(),
        };

        // Update subgraph
        subgraph.edges.insert(edge_id);

        // Emit event
        aggregate.add_event(GraphDomainEvent::SystemConnected {
            subgraph_id,
            edge,
            behavior,
        });

        Ok(edge_id)
    }
}
```

## Usage Example

```rust
pub fn create_workflow_graph(
    construction_service: &GraphConstructionService,
    connection_service: &SystemConnectionService,
    event_store: &mut EventStore,
) -> Result<GraphAggregate, DomainError> {
    // Create domain
    let mut aggregate = construction_service.create_domain(
        "Order Processing Workflow".to_string(),
        event_store,
    )?;

    // Add subgraph entities
    let order_validation = construction_service.add_subgraph(
        &mut aggregate,
        "Order Validation".to_string(),
    )?;

    let payment_processing = construction_service.add_subgraph(
        &mut aggregate,
        "Payment Processing".to_string(),
    )?;

    let fulfillment = construction_service.add_subgraph(
        &mut aggregate,
        "Order Fulfillment".to_string(),
    )?;

    // Add nodes (components) to subgraphs
    // ... node creation code ...

    // Connect with systems (edges)
    connection_service.connect_nodes(
        &mut aggregate,
        order_validation,
        validate_node,
        payment_node,
        EdgeBehavior::EventEmission {
            event_type: "OrderValidated".to_string(),
        },
    )?;

    // Entities communicate through events
    // The edge behavior defines HOW they communicate

    Ok(aggregate)
}
```

## Key DDD Principles Applied

1. **Aggregate Boundary**: The Graph is our consistency boundary
2. **Entity Identity**: Subgraphs have unique identities and lifecycles
3. **Value Objects**: Nodes and their properties are immutable values
4. **Domain Events**: Entities communicate through well-defined events
5. **Repository Pattern**: Abstracts persistence, supports event sourcing
6. **Domain Services**: Encapsulate domain logic that doesn't belong to entities
7. **Ubiquitous Language**: Graph, Subgraph, Node, Edge, System - all domain terms

This architecture ensures that our graph system follows DDD principles while leveraging petgraph for efficient operations and maintaining event sourcing for complete history.

## Overview

This document describes the Domain-Driven Design (DDD) architecture for our graph system, where:
- **Domain** = Graph (the bounded context)
- **Aggregate Root** = Graph (maintains consistency)
- **Entities** = Subgraphs and Nodes (have identity and lifecycle)
- **Value Objects** = Components within Nodes, and Edges (relationships between nodes)
- **Domain Services** = Graph operations and algorithms
- **Events** = Graph mutations delivered via NATS EventStream transactions

Key principles:
- **Nodes** are entities with their own identity and lifecycle
- **Components** are value objects that compose nodes
- **Edges** are value objects representing relationships - they have no independent existence
- **Subgraphs** are entities that group related nodes

## Core Architecture

### 1. Graph Aggregate

```rust
use petgraph::graph::{Graph, NodeIndex, EdgeIndex};
use async_nats::jetstream;
use cid::Cid;

/// The Graph Aggregate - maintains consistency boundaries
pub struct GraphAggregate {
    /// Unique identifier for this graph
    pub id: GraphId,

    /// The petgraph structure for algorithms
    graph: Graph<NodeEntity, EdgeData>,

    /// Subgraph entities within this aggregate
    subgraphs: HashMap<SubgraphId, SubgraphEntity>,

    /// Mapping from domain IDs to petgraph indices
    node_id_to_index: HashMap<NodeId, NodeIndex>,

    /// Metadata about the graph
    metadata: GraphMetadata,

    /// Version for optimistic concurrency
    version: u64,

    /// Last processed event sequence from NATS
    last_sequence: u64,
}

impl GraphAggregate {
    /// Find a node's petgraph index by its domain ID
    pub fn find_node_index(&self, node_id: &NodeId) -> Result<NodeIndex, GraphError> {
        self.node_id_to_index
            .get(node_id)
            .copied()
            .ok_or(GraphError::NodeNotFound)
    }

    /// Apply domain events from EventStream transaction
    pub fn apply_transaction(&mut self, transaction: &EventStreamTransaction) -> Result<Vec<DomainEvent>, GraphError> {
        let mut new_events = Vec::new();

        for event in &transaction.events {
            match self.apply_event(event) {
                Ok(resulting_events) => new_events.extend(resulting_events),
                Err(e) => return Err(GraphError::EventApplicationFailed(e)),
            }
        }

        self.last_sequence = transaction.sequence_range.end;
        self.version += 1;

        Ok(new_events)
    }

    /// Command: Add a node entity to the graph
    pub fn add_node(&mut self, node_entity: NodeEntity) -> Result<NodeAddedEvent, GraphError> {
        // Validate business rules
        self.validate_node_addition(&node_entity)?;

        // Add to graph
        let index = self.graph.add_node(node_entity.clone());

        // Update ID mapping
        self.node_id_to_index.insert(node_entity.id.clone(), index);

        // Update subgraph if node belongs to one
        if let Some(subgraph_id) = &node_entity.subgraph_id {
            if let Some(subgraph) = self.subgraphs.get_mut(subgraph_id) {
                subgraph.add_node(index)?;
            }
        }

        // Create event
        Ok(NodeAddedEvent {
            graph_id: self.id.clone(),
            node_id: node_entity.id.clone(),
            node_index: index,
            node_entity,
            timestamp: SystemTime::now(),
        })
    }

    /// Command: Connect two node entities
    pub fn connect_nodes(
        &mut self,
        source: NodeIndex,
        target: NodeIndex,
        edge_type: EdgeType,
    ) -> Result<EdgeConnectedEvent, GraphError> {
        // Get the actual nodes to validate connection
        let source_node = self.graph.node_weight(source)
            .ok_or(GraphError::NodeNotFound)?;
        let target_node = self.graph.node_weight(target)
            .ok_or(GraphError::NodeNotFound)?;

        // Create edge data (value object)
        let edge_data = EdgeData::new(
            source_node.id.clone(),
            target_node.id.clone(),
            edge_type,
        );

        // Validate the edge can exist between these nodes
        edge_data.validate_connection(source_node, target_node)?;

        // Validate aggregate-level connection rules
        self.validate_edge_connection(source, target, &edge_data)?;

        // Add edge to graph (edge only exists as relationship between nodes)
        let edge_index = self.graph.add_edge(source, target, edge_data.clone());

        // Create event
        Ok(EdgeConnectedEvent {
            graph_id: self.id.clone(),
            edge_index,
            source_node_id: source_node.id.clone(),
            target_node_id: target_node.id.clone(),
            edge_data,
            timestamp: SystemTime::now(),
        })
    }

    /// Remove a node and all its dependent edges
    pub fn remove_node(&mut self, node_id: NodeId) -> Result<NodeRemovedEvent, GraphError> {
        // Find the node
        let node_index = self.find_node_index(&node_id)?;

        // Get the node data before removal
        let node_entity = self.graph.node_weight(node_index)
            .ok_or(GraphError::NodeNotFound)?
            .clone();

        // Remove from subgraph if it belongs to one
        if let Some(subgraph_id) = &node_entity.subgraph_id {
            if let Some(subgraph) = self.subgraphs.get_mut(subgraph_id) {
                subgraph.remove_node(node_index)?;
            }
        }

        // Get all edges connected to this node (they will be removed as they depend on the node)
        let mut removed_edges = Vec::new();

        // Collect incoming edges
        let incoming: Vec<_> = self.graph
            .edges_directed(node_index, petgraph::Direction::Incoming)
            .map(|edge| (edge.source(), edge.target(), edge.weight().clone()))
            .collect();

        // Collect outgoing edges
        let outgoing: Vec<_> = self.graph
            .edges_directed(node_index, petgraph::Direction::Outgoing)
            .map(|edge| (edge.source(), edge.target(), edge.weight().clone()))
            .collect();

        // Remove the node (this automatically removes all connected edges in petgraph)
        let removed_node = self.graph.remove_node(node_index)
            .ok_or(GraphError::NodeNotFound)?;

        // Remove from ID mapping
        self.node_id_to_index.remove(&node_id);

        // Combine all removed edges
        removed_edges.extend(incoming);
        removed_edges.extend(outgoing);

        Ok(NodeRemovedEvent {
            graph_id: self.id.clone(),
            node_id,
            removed_node,
            removed_edges, // Edges that ceased to exist when node was removed
            timestamp: SystemTime::now(),
        })
    }
}
```

### 2. Node Entities

```rust
/// Node Entity - has identity and lifecycle
#[derive(Debug, Clone)]
pub struct NodeEntity {
    /// Unique identifier
    pub id: NodeId,

    /// Node type defines its behavior
    pub node_type: NodeType,

    /// Components (value objects) that make up this node
    pub components: HashMap<ComponentType, Component>,

    /// Lifecycle state
    pub state: NodeState,

    /// Which subgraph this node belongs to
    pub subgraph_id: Option<SubgraphId>,

    /// External domain references
    pub external_refs: Vec<ExternalEntityRef>,
}

#[derive(Debug, Clone)]
pub enum NodeState {
    Active,
    Processing,
    Suspended,
    Completed,
    Failed(String),
}

impl NodeEntity {
    /// Add a component to this node
    pub fn add_component(&mut self, component: Component) -> Result<(), NodeError> {
        // Validate component addition
        self.validate_component(&component)?;

        self.components.insert(component.component_type(), component);
        Ok(())
    }

    /// Update node state based on business rules
    pub fn transition_state(&mut self, new_state: NodeState) -> Result<(), NodeError> {
        self.validate_state_transition(&self.state, &new_state)?;
        self.state = new_state;
        Ok(())
    }

    /// Process an event that affects this node
    pub fn apply_event(&mut self, event: &NodeEvent) -> Result<(), NodeError> {
        match event {
            NodeEvent::ComponentAdded { component } => {
                self.add_component(component.clone())?;
            }
            NodeEvent::ComponentUpdated { component_type, updates } => {
                if let Some(component) = self.components.get_mut(component_type) {
                    component.apply_updates(updates)?;
                }
            }
            NodeEvent::StateChanged { new_state } => {
                self.transition_state(new_state.clone())?;
            }
        }
        Ok(())
    }
}
```

### 3. Components (Value Objects)

```rust
/// Component - immutable value object within a Node entity
#[derive(Debug, Clone, PartialEq)]
pub enum Component {
    /// Visual representation component
    Visual(VisualComponent),

    /// Data processing component
    DataProcessor(DataProcessorComponent),

    /// External system integration
    Integration(IntegrationComponent),

    /// Business logic component
    BusinessLogic(BusinessLogicComponent),

    /// Metadata component
    Metadata(MetadataComponent),
}

/// Visual component for rendering
#[derive(Debug, Clone, PartialEq)]
pub struct VisualComponent {
    pub shape: NodeShape,
    pub color: Color,
    pub size: f32,
    pub label: Option<String>,
    pub icon: Option<IconType>,
}

/// Data processor component
#[derive(Debug, Clone, PartialEq)]
pub struct DataProcessorComponent {
    pub input_schema: DataSchema,
    pub output_schema: DataSchema,
    pub transform_function: TransformFunction,
    pub validation_rules: Vec<ValidationRule>,
}

/// Integration component for external systems
#[derive(Debug, Clone, PartialEq)]
pub struct IntegrationComponent {
    pub system_type: ExternalSystemType,
    pub connection_config: ConnectionConfig,
    pub retry_policy: RetryPolicy,
    pub timeout: Duration,
}

/// Business logic component
#[derive(Debug, Clone, PartialEq)]
pub struct BusinessLogicComponent {
    pub rules: Vec<BusinessRule>,
    pub constraints: Vec<Constraint>,
    pub calculations: Vec<Calculation>,
}

impl Component {
    /// Components are immutable - create new instance with updates
    pub fn with_updates(&self, updates: &ComponentUpdates) -> Result<Component, ComponentError> {
        match (self, updates) {
            (Component::Visual(visual), ComponentUpdates::Visual(updates)) => {
                Ok(Component::Visual(visual.apply_updates(updates)?))
            }
            (Component::DataProcessor(processor), ComponentUpdates::DataProcessor(updates)) => {
                Ok(Component::DataProcessor(processor.apply_updates(updates)?))
            }
            // ... other component types
            _ => Err(ComponentError::IncompatibleUpdate),
        }
    }
}
```

### 4. Subgraph Entities

```rust
/// Subgraph Entity - groups related nodes
#[derive(Debug, Clone)]
pub struct SubgraphEntity {
    /// Unique identifier
    pub id: SubgraphId,

    /// Name of the subgraph
    pub name: String,

    /// Node indices that belong to this subgraph (petgraph indices)
    pub nodes: HashSet<NodeIndex>,

    /// Edge indices within this subgraph
    pub edges: HashSet<EdgeIndex>,

    /// Subgraph-specific constraints
    pub constraints: SubgraphConstraints,

    /// Lifecycle state
    pub state: SubgraphState,
}

impl SubgraphEntity {
    /// Add a node to this subgraph by its petgraph index
    pub fn add_node(&mut self, node_index: NodeIndex) -> Result<(), SubgraphError> {
        // Validate node addition
        self.constraints.validate_node_addition(node_index)?;

        self.nodes.insert(node_index);
        Ok(())
    }

    /// Remove a node from this subgraph
    pub fn remove_node(&mut self, node_index: NodeIndex) -> Result<(), SubgraphError> {
        if !self.nodes.contains(&node_index) {
            return Err(SubgraphError::NodeNotInSubgraph);
        }

        self.nodes.remove(&node_index);
        Ok(())
    }

    /// Check if this subgraph contains a node
    pub fn contains_node(&self, node_index: NodeIndex) -> bool {
        self.nodes.contains(&node_index)
    }

    /// Add an edge to this subgraph
    pub fn add_edge(&mut self, edge_index: EdgeIndex) -> Result<(), SubgraphError> {
        self.edges.insert(edge_index);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SubgraphConstraints {
    pub max_nodes: Option<usize>,
    pub allowed_node_types: Vec<NodeType>,
    pub required_components: Vec<ComponentType>,
}

impl SubgraphConstraints {
    /// Validate that a node can be added to this subgraph
    pub fn validate_node_addition(&self, node_index: NodeIndex) -> Result<(), SubgraphError> {
        // Note: actual validation would need access to the graph to check node properties
        // This is typically done at the aggregate level
        Ok(())
    }
}
```

### 5. Edge Data

```rust
/// Edge - value object representing relationships/communication between nodes
/// Edges have no independent existence - they only exist between nodes
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeData {
    /// Source and target nodes this edge connects
    pub source_node_id: NodeId,
    pub target_node_id: NodeId,

    /// Type of communication/relationship
    pub edge_type: EdgeType,

    /// Communication properties
    pub properties: EdgeProperties,

    /// Constraints on this edge
    pub constraints: EdgeConstraints,
}

impl EdgeData {
    /// Create a new edge between nodes
    pub fn new(
        source: NodeId,
        target: NodeId,
        edge_type: EdgeType,
    ) -> Self {
        EdgeData {
            source_node_id: source,
            target_node_id: target,
            edge_type,
            properties: EdgeProperties::default(),
            constraints: EdgeConstraints::default(),
        }
    }

    /// Edges are immutable - create new instance with updates
    pub fn with_properties(mut self, properties: EdgeProperties) -> Self {
        self.properties = properties;
        self
    }

    /// Validate that this edge can exist between the given nodes
    pub fn validate_connection(
        &self,
        source_node: &NodeEntity,
        target_node: &NodeEntity,
    ) -> Result<(), EdgeError> {
        // Validate node types can be connected
        self.edge_type.validate_node_types(
            &source_node.node_type,
            &target_node.node_type,
        )?;

        // Validate edge constraints
        self.constraints.validate(source_node, target_node)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    /// Synchronous call with request/response
    SyncCall {
        timeout: Duration,
        retry_policy: RetryPolicy,
    },

    /// Asynchronous event emission
    AsyncEvent {
        event_types: Vec<String>,
        delivery_guarantee: DeliveryGuarantee,
    },

    /// Data transformation pipeline
    Transform {
        transform_type: TransformType,
        batch_size: Option<usize>,
    },

    /// Hierarchical relationship
    Hierarchy {
        relationship: HierarchyType,
        cascade_behavior: CascadeBehavior,
    },

    /// Conditional flow
    Conditional {
        condition: FlowCondition,
        priority: u8,
    },
}

impl EdgeType {
    /// Validate that this edge type can connect the given node types
    fn validate_node_types(
        &self,
        source_type: &NodeType,
        target_type: &NodeType,
    ) -> Result<(), EdgeError> {
        match self {
            EdgeType::SyncCall { .. } => {
                // Validate sync calls are between compatible node types
                if !source_type.can_call(target_type) {
                    return Err(EdgeError::IncompatibleNodeTypes);
                }
            }
            EdgeType::Hierarchy { relationship, .. } => {
                // Validate hierarchical relationships
                match relationship {
                    HierarchyType::Parent => {
                        if !source_type.can_parent(target_type) {
                            return Err(EdgeError::InvalidHierarchy);
                        }
                    }
                    HierarchyType::Contains => {
                        if !source_type.can_contain(target_type) {
                            return Err(EdgeError::InvalidContainment);
                        }
                    }
                }
            }
            // ... other edge types
        }
        Ok(())
    }
}

/// Edge properties - additional data for the relationship
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EdgeProperties {
    pub weight: Option<f32>,
    pub metadata: HashMap<String, Value>,
    pub created_at: SystemTime,
    pub last_used: Option<SystemTime>,
}

/// Constraints that must be satisfied for an edge to exist
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EdgeConstraints {
    pub max_edges_from_source: Option<usize>,
    pub max_edges_to_target: Option<usize>,
    pub required_source_components: Vec<ComponentType>,
    pub required_target_components: Vec<ComponentType>,
    pub custom_validators: Vec<EdgeValidator>,
}
```

### 6. Repository Pattern with EventStream

```rust
/// Repository for Graph aggregates using NATS EventStream
pub struct GraphRepository {
    /// NATS JetStream context
    jetstream: jetstream::Context,

    /// Event stream service for transactions
    event_stream: Arc<EventStreamService>,

    /// In-memory cache of active graphs
    cache: Arc<RwLock<HashMap<GraphId, GraphAggregate>>>,
}

impl GraphRepository {
    /// Load a graph by replaying its event history
    pub async fn load(&self, id: GraphId) -> Result<GraphAggregate, RepositoryError> {
        // Check cache first
        if let Some(graph) = self.cache.read().await.get(&id) {
            return Ok(graph.clone());
        }

        // Fetch complete event history as transaction
        let transaction = self.event_stream
            .fetch_transaction(
                id.clone().into(),
                TransactionOptions {
                    replay_policy: ReplayPolicy::FromBeginning,
                    ..Default::default()
                },
            )
            .await?;

        // Rebuild graph from events
        let mut graph = GraphAggregate::new(id.clone());
        graph.apply_transaction(&transaction)?;

        // Cache the loaded graph
        self.cache.write().await.insert(id, graph.clone());

        Ok(graph)
    }

    /// Find nodes by component criteria
    pub async fn find_nodes_with_component(
        &self,
        graph_id: GraphId,
        component_type: ComponentType,
        criteria: ComponentCriteria,
    ) -> Result<Vec<NodeEntity>, RepositoryError> {
        let graph = self.load(graph_id).await?;

        let matching_nodes = graph.graph
            .node_weights()
            .filter(|node| {
                node.components.get(&component_type)
                    .map(|comp| criteria.matches(comp))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        Ok(matching_nodes)
    }
}
```

### 7. Domain Services

```rust
/// Domain service for node operations
pub struct NodeManagementService {
    repository: Arc<GraphRepository>,
}

impl NodeManagementService {
    /// Create a node with initial components
    pub fn create_node(
        &self,
        node_type: NodeType,
        initial_components: Vec<Component>,
    ) -> Result<NodeEntity, ServiceError> {
        // Create node entity
        let mut node = NodeEntity {
            id: NodeId::new(),
            node_type,
            components: HashMap::new(),
            state: NodeState::Active,
            subgraph_id: None,
            external_refs: Vec::new(),
        };

        // Add initial components
        for component in initial_components {
            node.add_component(component)?;
        }

        // Validate node is complete
        self.validate_node_completeness(&node)?;

        Ok(node)
    }

    /// Transform a node by updating its components
    pub async fn transform_node(
        &self,
        graph_id: GraphId,
        node_id: NodeId,
        transformations: Vec<ComponentTransformation>,
    ) -> Result<Vec<DomainEvent>, ServiceError> {
        let mut graph = self.repository.load(graph_id).await?;
        let mut events = Vec::new();

        // Find node in graph
        let node_idx = graph.find_node_index(&node_id)?;
        if let Some(node) = graph.graph.node_weight_mut(node_idx) {
            // Apply transformations
            for transformation in transformations {
                match transformation {
                    ComponentTransformation::Add(component) => {
                        node.add_component(component.clone())?;
                        events.push(DomainEvent::ComponentAdded {
                            node_id: node_id.clone(),
                            component,
                        });
                    }
                    ComponentTransformation::Update { component_type, updates } => {
                        if let Some(existing) = node.components.get(&component_type) {
                            let updated = existing.with_updates(&updates)?;
                            node.components.insert(component_type, updated);
                            events.push(DomainEvent::ComponentUpdated {
                                node_id: node_id.clone(),
                                component_type,
                                updates,
                            });
                        }
                    }
                    ComponentTransformation::Remove(component_type) => {
                        node.components.remove(&component_type);
                        events.push(DomainEvent::ComponentRemoved {
                            node_id: node_id.clone(),
                            component_type,
                        });
                    }
                }
            }
        }

        // Save changes
        self.repository.save(&graph, events.clone()).await?;

        Ok(events)
    }
}

/// Domain service for graph analysis
pub struct GraphAnalysisService {
    event_stream: Arc<EventStreamService>,
}

impl GraphAnalysisService {
    /// Analyze component distribution across nodes
    pub async fn analyze_component_distribution(
        &self,
        graph_id: GraphId,
    ) -> Result<ComponentDistribution, ServiceError> {
        let graph = self.repository.load(graph_id).await?;
        let mut distribution = HashMap::new();

        for node in graph.graph.node_weights() {
            for (component_type, _) in &node.components {
                *distribution.entry(component_type.clone()).or_insert(0) += 1;
            }
        }

        Ok(ComponentDistribution {
            graph_id,
            total_nodes: graph.graph.node_count(),
            component_counts: distribution,
            timestamp: SystemTime::now(),
        })
    }

    /// Find paths between nodes with specific components
    pub async fn find_component_paths(
        &self,
        graph_id: GraphId,
        source_component: ComponentType,
        target_component: ComponentType,
    ) -> Result<Vec<ComponentPath>, ServiceError> {
        let graph = self.repository.load(graph_id).await?;

        // Find nodes with source component
        let source_nodes: Vec<_> = graph.graph
            .node_indices()
            .filter(|&idx| {
                graph.graph[idx].components.contains_key(&source_component)
            })
            .collect();

        // Find nodes with target component
        let target_nodes: Vec<_> = graph.graph
            .node_indices()
            .filter(|&idx| {
                graph.graph[idx].components.contains_key(&target_component)
            })
            .collect();

        // Find paths between them
        let mut paths = Vec::new();
        for &source in &source_nodes {
            for &target in &target_nodes {
                if let Some(path) = petgraph::algo::astar(
                    &graph.graph,
                    source,
                    |n| n == target,
                    |e| e.weight().cost(),
                    |_| 0.0,
                ) {
                    paths.push(ComponentPath {
                        source_node: graph.graph[source].id.clone(),
                        target_node: graph.graph[target].id.clone(),
                        path_nodes: path.1.into_iter()
                            .map(|idx| graph.graph[idx].id.clone())
                            .collect(),
                        total_cost: path.0,
                    });
                }
            }
        }

        Ok(paths)
    }
}
```

### 8. Integration with Bevy

```rust
/// Plugin that integrates DDD graph with Bevy
pub struct GraphDomainPlugin;

impl Plugin for GraphDomainPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(GraphRepository::new())
            .insert_resource(NodeManagementService::new())
            .insert_resource(GraphAnalysisService::new())

            // Events
            .add_event::<GraphMutationEvent>()
            .add_event::<NodeComponentEvent>()

            // Systems
            .add_systems(Update, (
                // Process EventStream transactions
                process_event_transactions_system,
                // Apply graph mutations to ECS
                apply_graph_mutations_system,
                // Update node components
                update_node_components_system,
                // Handle real-time updates
                handle_realtime_updates_system,
            ).chain());
    }
}

/// System that updates node components in ECS
fn update_node_components_system(
    mut commands: Commands,
    mut node_events: EventReader<NodeComponentEvent>,
    mut query: Query<(Entity, &mut NodeEntity)>,
) {
    for event in node_events.read() {
        match event {
            NodeComponentEvent::ComponentAdded { node_id, component } => {
                // Find entity with this node
                for (entity, mut node) in query.iter_mut() {
                    if node.id == *node_id {
                        // Add component to ECS entity based on type
                        match component {
                            Component::Visual(visual) => {
                                commands.entity(entity).insert(visual.clone());
                            }
                            Component::DataProcessor(processor) => {
                                commands.entity(entity).insert(processor.clone());
                            }
                            // ... other component types
                        }

                        // Update node entity
                        node.add_component(component.clone()).ok();
                    }
                }
            }
            NodeComponentEvent::ComponentRemoved { node_id, component_type } => {
                // Remove component from ECS entity
                for (entity, mut node) in query.iter_mut() {
                    if node.id == *node_id {
                        match component_type {
                            ComponentType::Visual => {
                                commands.entity(entity).remove::<VisualComponent>();
                            }
                            ComponentType::DataProcessor => {
                                commands.entity(entity).remove::<DataProcessorComponent>();
                            }
                            // ... other component types
                        }

                        node.components.remove(component_type);
                    }
                }
            }
        }
    }
}
```

## Event Flow

1. **Commands** trigger domain methods on aggregates and entities
2. **Node Entities** manage their own components and state
3. **Domain Events** are generated and sent to NATS JetStream
4. **EventStream Transactions** batch related events together
5. **Repositories** load aggregates by replaying transactions
6. **Bevy Systems** process transactions and update visualizations
7. **Components** are mapped to ECS components for rendering

## Benefits

- **Proper Entity Modeling**: Nodes are entities with identity and lifecycle
- **Component Flexibility**: Nodes can have any combination of components
- **Event Sourcing**: Complete history in NATS JetStream
- **Transactional Consistency**: Events processed as atomic units
- **Real-time Updates**: Direct NATS to Bevy pipeline
- **Domain Isolation**: Business logic separate from infrastructure
- **ECS Integration**: Components map naturally to Bevy's ECS
