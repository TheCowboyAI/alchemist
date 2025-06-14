# CIM Vocabulary

[← Back to Index](index.md)

## Core Concepts
*For detailed implementation, see [Architecture Overview](architecture.md)*

### Architecture Terms
- **CIM (Composable Information Machine)**: A framework for building distributed systems that transform scattered information into organized, actionable knowledge
- **Leaf Node**: A CIM Node that hosts many Containers providing services, capabilities, AI Agents, and read models, used by the CIM
- **Domain**: A unique set of ideas and concepts that cannot be further reduced, representing a specific area of knowledge or business function
- **Agent**: A self-contained functional unit within CIM that provides specific services (e.g., AI, communications, documentation)
- **Event**: A record of something that happened within the CIM system
- **Command**: An instruction that modifies CIM state
- **Query**: An instruction that observes CIM state
- **Message**: Any transmission within the CIM system
- **Observation**: A snapshot of state at a particular time

### Event Sourcing Terms
*For implementation details, see [Event-Sourced Architecture](event-sourced-graph-architecture.md)*
- **Domain Event**: A persistent business fact that represents something that happened in the domain
- **Presentation Event**: A UI interaction that stays within the presentation layer (e.g., animations, drag operations)
- **Event Store**: System for storing and managing event streams with append-only semantics
- **Event Envelope**: Container for an event with metadata including timestamp, sequence, and correlation IDs
- **CID Chain**: Cryptographic chain of Content IDs ensuring event integrity and immutability
- **Aggregate**: A cluster of domain objects treated as a single unit for data changes
- **Aggregate Root**: The entry point to an aggregate that ensures consistency
- **Command Handler**: Component that processes commands and generates domain events
- **Projection**: A read model built from domain events for query optimization
- **Event Sourcing**: Architectural pattern where state changes are stored as a sequence of events
- **CQRS (Command Query Responsibility Segregation)**: Pattern separating read and write models
- **Event Aggregator**: Component that collects multiple presentation events into domain commands

### Technical Terms
*For implementation details, see [Technical Infrastructure](technical.md)*
- **DDD (Domain-Driven Design)**: A software design approach focusing on modeling software to match a domain according to expert input
- **EDA (Event-Driven Architecture)**: An architectural pattern where components communicate through events
- **ECS (Entity Component System)**: Data-oriented architecture pattern used by Bevy for high-performance visualization
- **NATS**: Message broker system used for internal CIM communication
- **JetStream**: NATS persistence layer for reliable event storage
- **Content-Addressing**: Method of storing and retrieving information based on its content rather than location
- **IPLD**: InterPlanetary Linked Data format for content-addressed data structures
- **Bevy**: Rust game engine using ECS for our visualization layer

### Implementation Components
- **Event Store**: Distributed event storage using NATS JetStream
- **Object Store**: System for storing immutable data with content addressing
- **Event Bridge**: Async/sync bridge between NATS and Bevy ECS
- **Projection Handler**: System that updates read models from events
- **External Projection**: Projection that syncs with external systems bidirectionally
- **Repository**: Pattern for aggregate persistence and retrieval
- **Read Model**: Optimized data structure for queries, built from events

## Domain Categories
*For full domain documentation, see [Domain Categorization](domain_categorization.md)*

### Graph Domain (Event-Sourced)

#### Term: Graph
- **Category**: Domain Object
- **Type**: Aggregate Root
- **Taxonomy**: Graph Taxonomy
- **Definition**: An event-sourced aggregate representing a collection of nodes and edges with full history tracking
- **Relationships**:
  * Contains: Nodes, Edges
  * Has: GraphId, GraphMetadata, Version
  * Emits: GraphCreated, GraphRenamed, GraphDeleted
- **Usage Context**: Primary aggregate for organizing and visualizing relationships
- **Code Reference**: `src/domain/aggregates/graph.rs`

#### Term: Node
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: An immutable point in a graph representing a single concept
- **Relationships**:
  * Part-Of: Graph Aggregate
  * Has: NodeId, NodeContent, Position3D
  * Connected-By: Edges
  * Created-By: NodeAdded event
- **Usage Context**: Fundamental unit of information within a graph
- **Code Reference**: `src/domain/value_objects.rs`

#### Term: Edge
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: An immutable connection between nodes representing a relationship
- **Relationships**:
  * Part-Of: Graph Aggregate
  * Connects: Source NodeId, Target NodeId
  * Has: EdgeId, EdgeRelationship
  * Created-By: EdgeAdded event
- **Usage Context**: Defining relationships between nodes
- **Code Reference**: `src/domain/value_objects.rs`

#### Term: GraphModel
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: Recognition of standard graph patterns (K7, C5, State Machines, etc.)
- **Relationships**:
  * Describes: Graph structure
  * Enables: Structure-preserving morphisms
  * Used-By: HUD system
- **Usage Context**: Pattern recognition and graph transformations
- **Code Reference**: `src/domain/value_objects.rs`

### Identifiers

#### Term: GraphId
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: An immutable UUID-based identifier for a graph aggregate
- **Relationships**:
  * Identifies: Graph Aggregate
  * Used-In: All graph commands and events
- **Usage Context**: Ensuring unique identification across event streams
- **Code Reference**: `src/domain/value_objects.rs`

#### Term: NodeId
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: An immutable UUID-based identifier for nodes
- **Relationships**:
  * Identifies: Node
  * Referenced-By: Edges, Commands, Events
- **Usage Context**: Unique identification in event-sourced operations
- **Code Reference**: `src/domain/value_objects.rs`

#### Term: EdgeId
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: An immutable UUID-based identifier for edges
- **Relationships**:
  * Identifies: Edge
  * Links: Source and Target NodeIds
- **Usage Context**: Tracking edge lifecycle through events
- **Code Reference**: `src/domain/value_objects.rs`

### Spatial and Metadata

#### Term: Position3D
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: Immutable 3D spatial coordinates for node positioning
- **Relationships**:
  * Positions: Node
  * Updated-By: NodeMoved event
- **Usage Context**: Spatial positioning for visualization
- **Code Reference**: `src/domain/value_objects.rs`

#### Term: GraphMetadata
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: Immutable descriptive information about a graph
- **Relationships**:
  * Describes: Graph
  * Contains: Name, BoundedContext, CreatedAt, UpdatedAt, Tags
- **Usage Context**: Graph identification and search
- **Code Reference**: `src/domain/value_objects.rs`

#### Term: NodeContent
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: Immutable payload of a node including label and properties
- **Relationships**:
  * Contained-In: Node
  * Has: Label, NodeType, Properties
- **Usage Context**: Storing node information
- **Code Reference**: `src/domain/value_objects.rs`

#### Term: EdgeRelationship
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: Immutable description of edge semantics
- **Relationships**:
  * Defines: Edge meaning
  * Contains: RelationType, Strength
- **Usage Context**: Qualifying relationships
- **Code Reference**: `src/domain/value_objects.rs`

## Domain Events

### Graph Events

#### Term: GraphCreated
- **Category**: Domain Event
- **Type**: Event
- **Taxonomy**: Graph Events
- **Definition**: Immutable record that a new graph aggregate was created
- **Relationships**:
  * Contains: GraphId, GraphMetadata
  * Stored-In: EventStore
  * Projected-To: GraphSummaryProjection
- **Usage Context**: Graph lifecycle tracking
- **Code Reference**: `src/domain/events/mod.rs`

#### Term: NodeAdded
- **Category**: Domain Event
- **Type**: Event
- **Taxonomy**: Graph Events
- **Definition**: Immutable record that a node was added to a graph
- **Relationships**:
  * Contains: GraphId, NodeId, NodeContent, Position
  * Updates: Graph Aggregate
  * Triggers: Visual entity creation
- **Usage Context**: Node creation tracking
- **Code Reference**: `src/domain/events/mod.rs`

#### Term: EdgeAdded
- **Category**: Domain Event
- **Type**: Event
- **Taxonomy**: Graph Events
- **Definition**: Immutable record that an edge was established
- **Relationships**:
  * Contains: GraphId, EdgeId, Source, Target, Relationship
  * Updates: Graph structure
  * Triggers: Edge visualization
- **Usage Context**: Relationship establishment
- **Code Reference**: `src/domain/events/mod.rs`

#### Term: NodeRemoved
- **Category**: Domain Event
- **Type**: Event
- **Taxonomy**: Graph Events
- **Definition**: Immutable record that a node was removed
- **Relationships**:
  * Contains: GraphId, NodeId
  * Cascades: Edge removal
- **Usage Context**: Node deletion tracking
- **Code Reference**: `src/domain/events/mod.rs`

#### Term: EdgeRemoved
- **Category**: Domain Event
- **Type**: Event
- **Taxonomy**: Graph Events
- **Definition**: Immutable record that an edge was removed
- **Relationships**:
  * Contains: GraphId, EdgeId
  * Updates: Graph structure
- **Usage Context**: Relationship removal
- **Code Reference**: `src/domain/events/mod.rs`

### CID Chain Events

#### Term: ChainedEvent
- **Category**: Infrastructure
- **Type**: Event Wrapper
- **Taxonomy**: Event Infrastructure
- **Definition**: Event wrapper that includes CID chain information
- **Relationships**:
  * Contains: Event, CID, PreviousCID
  * Ensures: Cryptographic integrity
  * Part-Of: EventChain
- **Usage Context**: Tamper-proof event history
- **Code Reference**: `src/domain/events/cid_chain.rs`

## Commands

### Graph Commands

#### Term: CreateGraph
- **Category**: Command
- **Type**: Command
- **Taxonomy**: Graph Commands
- **Definition**: Command to create a new graph aggregate
- **Relationships**:
  * Contains: GraphMetadata
  * Produces: GraphCreated event
  * Handled-By: GraphCommandHandler
- **Usage Context**: Graph initialization
- **Code Reference**: `src/domain/commands/mod.rs`

#### Term: AddNode
- **Category**: Command
- **Type**: Command
- **Taxonomy**: Graph Commands
- **Definition**: Command to add a node to a graph
- **Relationships**:
  * Contains: GraphId, NodeContent, Position
  * Produces: NodeAdded event
  * Validates: Graph exists
- **Usage Context**: Node creation
- **Code Reference**: `src/domain/commands/mod.rs`

#### Term: AddEdge
- **Category**: Command
- **Type**: Command
- **Taxonomy**: Graph Commands
- **Definition**: Command to create an edge between nodes
- **Relationships**:
  * Contains: GraphId, Source, Target, Relationship
  * Produces: EdgeAdded event
  * Validates: Nodes exist
- **Usage Context**: Relationship creation
- **Code Reference**: `src/domain/commands/mod.rs`

### Aggregated Commands

#### Term: MoveNodes
- **Category**: Command
- **Type**: Aggregated Command
- **Taxonomy**: Graph Commands
- **Definition**: Command aggregated from multiple drag events
- **Relationships**:
  * Aggregated-From: DragStarted, DragUpdated, DragEnded
  * Contains: NodePositions map
  * Produces: Multiple NodeMoved events
- **Usage Context**: Batch position updates from UI interactions
- **Code Reference**: `src/domain/commands/aggregated_commands.rs`

## Infrastructure Components

### Event Infrastructure

#### Term: DistributedEventStore
- **Category**: Infrastructure
- **Type**: Service
- **Taxonomy**: Event Sourcing Infrastructure
- **Definition**: NATS JetStream-based distributed event store
- **Relationships**:
  * Uses: NATS JetStream
  * Stores: ChainedEvents
  * Indexes: By aggregate, sequence
- **Usage Context**: Distributed event persistence
- **Code Reference**: `src/infrastructure/event_store/distributed_impl.rs`

#### Term: EventBridge
- **Category**: Infrastructure
- **Type**: Service
- **Taxonomy**: Integration Infrastructure
- **Definition**: Bidirectional bridge between async NATS and sync Bevy
- **Relationships**:
  * Connects: NATS client, Bevy ECS
  * Uses: Crossbeam channels
  * Manages: Event flow
- **Usage Context**: Async/sync integration
- **Code Reference**: `src/infrastructure/event_bridge/mod.rs`

### Projections

#### Term: GraphSummaryProjection
- **Category**: Application
- **Type**: Read Model
- **Taxonomy**: CQRS Infrastructure
- **Definition**: Optimized read model for graph summaries
- **Relationships**:
  * Projects-From: Graph events
  * Contains: GraphSummary data
  * Serves: Summary queries
- **Usage Context**: Fast graph overview queries
- **Code Reference**: `src/application/projections/graph_summary.rs`

#### Term: ExternalProjection
- **Category**: Application
- **Type**: Integration Pattern
- **Taxonomy**: CQRS Infrastructure
- **Definition**: Projection that syncs with external systems
- **Relationships**:
  * Implements: Bidirectional sync
  * Uses: IngestHandler
  * Manages: External state
- **Usage Context**: External system integration
- **Code Reference**: `src/application/projections/external/mod.rs`

### Command Handlers

#### Term: GraphCommandHandler
- **Category**: Application
- **Type**: Service
- **Taxonomy**: CQRS Infrastructure
- **Definition**: Handles graph commands and generates events
- **Relationships**:
  * Processes: GraphCommands
  * Uses: GraphAggregate
  * Generates: DomainEvents
- **Usage Context**: Command processing
- **Code Reference**: `src/application/command_handlers/graph_handler.rs`

## Presentation Layer

### Components

#### Term: GraphNode (Component)
- **Category**: Presentation
- **Type**: Component
- **Taxonomy**: Bevy ECS
- **Definition**: ECS component linking visual entities to domain nodes
- **Relationships**:
  * References: NodeId, GraphId
  * Attached-To: Visual entities
  * Created-By: Event systems
- **Usage Context**: Visual representation
- **Code Reference**: `src/presentation/components/mod.rs`

#### Term: GraphEdge (Component)
- **Category**: Presentation
- **Type**: Component
- **Taxonomy**: Bevy ECS
- **Definition**: ECS component for edge visualization
- **Relationships**:
  * References: EdgeId, Source, Target
  * Attached-To: Edge visuals
- **Usage Context**: Edge rendering
- **Code Reference**: `src/presentation/components/mod.rs`

### Presentation Events

#### Term: DragStarted
- **Category**: Presentation Event
- **Type**: UI Event
- **Taxonomy**: Bevy Events
- **Definition**: Event when user starts dragging a node
- **Relationships**:
  * Contains: NodeId, StartPosition
  * Part-Of: Drag sequence
  * Aggregated-To: MoveNodes command
- **Usage Context**: UI interaction tracking
- **Code Reference**: `src/presentation/events/interaction.rs`

#### Term: AnimationFrame
- **Category**: Presentation Event
- **Type**: Animation Event
- **Taxonomy**: Bevy Events
- **Definition**: Event for animation progress updates
- **Relationships**:
  * Contains: Progress, Target
  * Stays-In: Presentation layer
  * Never-Becomes: Domain event
- **Usage Context**: Smooth visual transitions
- **Code Reference**: `src/presentation/events/animation.rs`

### Aggregators

#### Term: DragAggregator
- **Category**: Presentation
- **Type**: Aggregator
- **Taxonomy**: Event Aggregation
- **Definition**: Collects drag events into move commands
- **Relationships**:
  * Processes: DragStarted, DragUpdated, DragEnded
  * Produces: MoveNodes command
  * Manages: Drag state
- **Usage Context**: UI event aggregation
- **Code Reference**: `src/presentation/aggregators/drag.rs`

#### Term: SelectionAggregator
- **Category**: Presentation
- **Type**: Aggregator
- **Taxonomy**: Event Aggregation
- **Definition**: Manages node selection state
- **Relationships**:
  * Processes: NodeClicked, SelectionCleared
  * Maintains: Selected set
  * Emits: SelectionChanged
- **Usage Context**: Multi-selection handling
- **Code Reference**: `src/presentation/aggregators/selection.rs`

## Integration Patterns

### Term: Bidirectional Event Flow
- **Category**: Architecture Pattern
- **Type**: Integration Pattern
- **Taxonomy**: System Integration
- **Definition**: Pattern for two-way event synchronization with external systems
- **Relationships**:
  * Uses: ExternalProjection, IngestHandler
  * Enables: System integration
  * Maintains: Consistency
- **Usage Context**: CRM, ERP, Analytics integration
- **Code Reference**: Design document

### Term: Event Correlation
- **Category**: Integration
- **Type**: Pattern
- **Taxonomy**: System Integration
- **Definition**: Matching external events to internal domain events
- **Relationships**:
  * Maps: External to internal events
  * Uses: Correlation rules
  * Part-Of: Bidirectional flow
- **Usage Context**: External event processing
- **Code Reference**: `src/application/projections/external/mod.rs`

## Conceptual Spaces (Planned)

### Term: ConceptualPoint
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Conceptual Space
- **Definition**: Position in semantic space representing meaning
- **Relationships**:
  * Positions: Concepts
  * Enables: Similarity calculation
  * Maps-To: Visual position
- **Usage Context**: Semantic positioning
- **Code Reference**: TBD

### Term: ConceptualSpace
- **Category**: Domain Object
- **Type**: Aggregate
- **Taxonomy**: Conceptual Space
- **Definition**: Multi-dimensional space for semantic representation
- **Relationships**:
  * Contains: ConceptualPoints
  * Defines: Dimensions
  * Enables: AI reasoning
- **Usage Context**: Knowledge representation
- **Code Reference**: TBD

---

*This vocabulary is continuously updated as the system evolves. For the latest implementation details, refer to the source code and documentation.*
