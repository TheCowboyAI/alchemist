# CIM Vocabulary

[← Back to Index](index.md)

## Core Concepts
*For detailed implementation, see [Architecture Overview](architecture.md)*

### Architecture Terms
- **CIM (Composable Information Machine)**: A framework for building distributed systems that transform scattered information into organized, actionable knowledge
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
- **Event Store**: System for storing and managing event streams with append-only semantics
- **Event Envelope**: Container for an event with metadata including timestamp, sequence, and correlation IDs
- **Aggregate**: A cluster of domain objects treated as a single unit for data changes
- **Aggregate Root**: The entry point to an aggregate that ensures consistency
- **Command Handler**: Component that processes commands and generates domain events
- **Projection**: A read model built from domain events for query optimization
- **Event Sourcing**: Architectural pattern where state changes are stored as a sequence of events
- **CQRS (Command Query Responsibility Segregation)**: Pattern separating read and write models

### Technical Terms
*For implementation details, see [Technical Infrastructure](technical.md)*
- **DDD (Domain-Driven Design)**: A software design approach focusing on modeling software to match a domain according to expert input
- **EDA (Event-Driven Architecture)**: An architectural pattern where components communicate through events
- **FRP (Functional Reactive Programming)**: Programming paradigm for reactive programming using functional programming concepts
- **NATS**: Message broker system used for internal CIM communication
- **Content-Addressing**: Method of storing and retrieving information based on its content rather than location
- **Petgraph**: Rust graph data structure library used for efficient graph storage
- **Bevy ECS**: Entity Component System framework used for visualization and UI

### Implementation Components
- **Event Store**: System for storing and managing event streams
- **Object Store**: System for storing immutable data with content addressing
- **Ontology**: System for managing relationships and classifications
- **Bundle**: Collection of reusable components and resources
- **Read Model**: Optimized data structure for queries, built from events
- **Repository**: Pattern for aggregate persistence and retrieval

## Domain Categories
*For full domain documentation, see [Domain Categorization](domain_categorization.md)*

### Knowledge Management
*Detailed in [Knowledge Management](knowledge_management.md)*
- **Fact**: A proven claim with verifiable evidence
- **Claim**: An idea with repeatable construction
- **Theory**: A belief with supporting context and sources
- **Idea**: A preliminary thought without formal theory
- **Argument**: Support or opposition for a claim

### Organizational
- **Goal**: A defined achievement target
- **Organization**: A structural entity
- **Operator**: A system controller
- **Account**: A managed group or entity
- **User**: A managed person

### Business
- **Business Model**: Operational framework for value creation
- **Value Proposition**: Benefit offering to stakeholders
- **Solution**: Resolution to a defined problem
- **Proposal**: Formal suggestion for action

### Governance
- **Policy**: Operational guideline
- **Law**: Regulatory framework
- **Ethics**: Moral principles
- **Politics**: Power dynamics and relationships

### Technical
- **Model**: System representation
- **Equipment**: Physical resource
- **Environment**: Contextual setting
- **Location**: Spatial information
- **Secret**: Protected information

### Infrastructure
- **App**: The local Application
- **Local**: The local Container
- **Domain Implementation**: Specific instance of CIM for a particular organization or purpose
- **Leaf**: A Deterministic Host Node
- **Cluster**: A Managed Cluster of Leaf Nodes
- **SuperCluster**: A Managed Cluster of Clusters

### Security
*Detailed in [Security Model](security.md)*
- **mTLS**: Mutual Transport Layer Security authentication
- **YubiKey**: Hardware authentication device
- **OpenPGP**: Encryption standard
- **OpenSSL**: Cryptographic software library

---

## Graph Domain (Event-Sourced)

### Term: Graph
- **Category**: Domain Object
- **Type**: Aggregate Root
- **Taxonomy**: Graph Taxonomy
- **Definition**: An event-sourced aggregate representing a collection of nodes and edges with full history tracking
- **Relationships**:
  * Contains: Nodes, Edges (via Petgraph)
  * Has: GraphId, GraphMetadata, Version
  * Emits: GraphCreated, GraphRenamed, GraphDeleted
- **Usage Context**: Primary aggregate for organizing and visualizing relationships
- **Code Reference**: `src/domain/aggregates/graph.rs`

### Term: Node
- **Category**: Domain Object
- **Type**: Entity
- **Taxonomy**: Graph Taxonomy
- **Definition**: A discrete point in a graph representing a single concept, tracked through events
- **Relationships**:
  * Part-Of: Graph Aggregate
  * Has: NodeId, NodeContent, Position3D
  * Connected-By: Edges
  * Created-By: NodeAdded event
- **Usage Context**: Fundamental unit of information within a graph
- **Code Reference**: `src/domain/aggregates/node.rs`

### Term: Edge
- **Category**: Domain Object
- **Type**: Entity
- **Taxonomy**: Graph Taxonomy
- **Definition**: An event-sourced connection between nodes representing a relationship
- **Relationships**:
  * Part-Of: Graph Aggregate
  * Connects: Source NodeId, Target NodeId
  * Has: EdgeId, EdgeRelationship, Weight
  * Created-By: EdgeConnected event
- **Usage Context**: Defining relationships between nodes
- **Code Reference**: `src/domain/aggregates/edge.rs`

### Term: GraphId
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: An immutable UUID-based identifier for a graph aggregate
- **Relationships**:
  * Identifies: Graph Aggregate
  * Used-In: All graph commands and events
- **Usage Context**: Ensuring unique identification across event streams
- **Code Reference**: `src/domain/values/identifiers.rs`

### Term: NodeId
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: An immutable UUID-based identifier for nodes
- **Relationships**:
  * Identifies: Node Entity
  * Referenced-By: Edges, Commands, Events
- **Usage Context**: Unique identification in event-sourced operations
- **Code Reference**: `src/domain/values/identifiers.rs`

### Term: EdgeId
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: An immutable UUID-based identifier for edges
- **Relationships**:
  * Identifies: Edge Entity
  * Links: Source and Target NodeIds
- **Usage Context**: Tracking edge lifecycle through events
- **Code Reference**: `src/domain/values/identifiers.rs`

### Term: Position3D
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: Immutable 3D spatial coordinates for node positioning
- **Relationships**:
  * Positions: Node
  * Updated-By: NodeMoved event
- **Usage Context**: Spatial positioning for visualization
- **Code Reference**: `src/domain/values/position.rs`

### Term: GraphMetadata
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: Immutable descriptive information about a graph
- **Relationships**:
  * Describes: Graph
  * Contains: Name, CreatedAt, UpdatedAt, Tags
- **Usage Context**: Graph identification and search
- **Code Reference**: `src/domain/values/metadata.rs`

### Term: NodeContent
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: Immutable payload of a node including label and properties
- **Relationships**:
  * Contained-In: Node
  * Has: Label, NodeType, Properties
- **Usage Context**: Storing node information
- **Code Reference**: `src/domain/values/content.rs`

### Term: EdgeRelationship
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: Immutable description of edge semantics
- **Relationships**:
  * Defines: Edge meaning
  * Contains: RelationType, Strength
- **Usage Context**: Qualifying relationships
- **Code Reference**: `src/domain/values/relationship.rs`

## Domain Events

### Term: GraphCreated
- **Category**: Domain Event
- **Type**: Event
- **Taxonomy**: Graph Events
- **Definition**: Immutable record that a new graph aggregate was created
- **Relationships**:
  * Contains: GraphId, GraphMetadata
  * Stored-In: EventStore
  * Projected-To: GraphReadModel
- **Usage Context**: Graph lifecycle tracking
- **Code Reference**: `src/domain/events/graph_events.rs`

### Term: NodeAdded
- **Category**: Domain Event
- **Type**: Event
- **Taxonomy**: Graph Events
- **Definition**: Immutable record that a node was added to a graph
- **Relationships**:
  * Contains: GraphId, Node
  * Updates: Graph Aggregate
  * Triggers: Visual entity creation
- **Usage Context**: Node creation tracking
- **Code Reference**: `src/domain/events/graph_events.rs`

### Term: EdgeConnected
- **Category**: Domain Event
- **Type**: Event
- **Taxonomy**: Graph Events
- **Definition**: Immutable record that an edge was established
- **Relationships**:
  * Contains: GraphId, Edge
  * Updates: Petgraph structure
  * Triggers: Edge visualization
- **Usage Context**: Relationship establishment
- **Code Reference**: `src/domain/events/graph_events.rs`

### Term: NodeRemoved
- **Category**: Domain Event
- **Type**: Event
- **Taxonomy**: Graph Events
- **Definition**: Immutable record that a node was removed
- **Relationships**:
  * Contains: GraphId, NodeId
  * Cascades: Edge removal
- **Usage Context**: Node deletion tracking
- **Code Reference**: `src/domain/events/graph_events.rs`

### Term: EdgeDisconnected
- **Category**: Domain Event
- **Type**: Event
- **Taxonomy**: Graph Events
- **Definition**: Immutable record that an edge was removed
- **Relationships**:
  * Contains: GraphId, EdgeId
  * Updates: Petgraph structure
- **Usage Context**: Relationship removal
- **Code Reference**: `src/domain/events/graph_events.rs`

### Term: LayoutApplied
- **Category**: Domain Event
- **Type**: Event
- **Taxonomy**: Graph Events
- **Definition**: Immutable record that a layout algorithm was applied
- **Relationships**:
  * Contains: GraphId, LayoutType, Positions
  * Updates: Node positions
- **Usage Context**: Layout change tracking
- **Code Reference**: `src/domain/events/graph_events.rs`

## Commands

### Term: CreateGraph
- **Category**: Command
- **Type**: Command
- **Taxonomy**: Graph Commands
- **Definition**: Command to create a new graph aggregate
- **Relationships**:
  * Contains: GraphMetadata
  * Produces: GraphCreated event
  * Handled-By: GraphCommandHandler
- **Usage Context**: Graph initialization
- **Code Reference**: `src/domain/commands/graph_commands.rs`

### Term: AddNode
- **Category**: Command
- **Type**: Command
- **Taxonomy**: Graph Commands
- **Definition**: Command to add a node to a graph
- **Relationships**:
  * Contains: GraphId, NodeContent, Position
  * Produces: NodeAdded event
  * Validates: Graph exists
- **Usage Context**: Node creation
- **Code Reference**: `src/domain/commands/graph_commands.rs`

### Term: ConnectNodes
- **Category**: Command
- **Type**: Command
- **Taxonomy**: Graph Commands
- **Definition**: Command to create an edge between nodes
- **Relationships**:
  * Contains: GraphId, Source, Target, Relationship
  * Produces: EdgeConnected event
  * Validates: Nodes exist
- **Usage Context**: Relationship creation
- **Code Reference**: `src/domain/commands/graph_commands.rs`

## Infrastructure Components

### Term: EventStore
- **Category**: Infrastructure
- **Type**: Service
- **Taxonomy**: Event Sourcing Infrastructure
- **Definition**: Append-only store for domain events with indexing
- **Relationships**:
  * Stores: EventEnvelopes
  * Indexes: By aggregate, sequence, timestamp
  * Persists-To: JsonFilePersistence
- **Usage Context**: Event persistence and retrieval
- **Code Reference**: `src/infrastructure/event_store/mod.rs`

### Term: GraphRepository
- **Category**: Infrastructure
- **Type**: Repository
- **Taxonomy**: Event Sourcing Infrastructure
- **Definition**: Repository for loading and saving graph aggregates via events
- **Relationships**:
  * Uses: EventStore
  * Rebuilds: Aggregates from events
  * Caches: Loaded aggregates
- **Usage Context**: Aggregate persistence
- **Code Reference**: `src/infrastructure/repositories/graph_repository.rs`

### Term: GraphReadModel
- **Category**: Infrastructure
- **Type**: Read Model
- **Taxonomy**: CQRS Infrastructure
- **Definition**: Optimized read model for graph queries using Petgraph
- **Relationships**:
  * Projects-From: Domain events
  * Contains: StableGraph, Indices, Caches
  * Serves: GraphQueries
- **Usage Context**: Query optimization
- **Code Reference**: `src/application/projections/graph_read_model.rs`

### Term: GraphCommandHandler
- **Category**: Application
- **Type**: Service
- **Taxonomy**: CQRS Infrastructure
- **Definition**: Handles graph commands and generates events
- **Relationships**:
  * Processes: GraphCommands
  * Generates: DomainEvents
  * Uses: EventStore, GraphRepository
- **Usage Context**: Command processing
- **Code Reference**: `src/application/command_handlers/graph_command_handler.rs`

## Bevy Integration Components

### Term: GraphNode
- **Category**: Presentation
- **Type**: Component
- **Taxonomy**: Bevy ECS
- **Definition**: ECS component linking visual entities to domain nodes
- **Relationships**:
  * References: NodeId, GraphId
  * Attached-To: Visual entities
  * Created-By: Event bridge
- **Usage Context**: Visual representation
- **Code Reference**: `src/presentation/components/graph_components.rs`

### Term: GraphEdge
- **Category**: Presentation
- **Type**: Component
- **Taxonomy**: Bevy ECS
- **Definition**: ECS component for edge visualization
- **Relationships**:
  * References: EdgeId, Source Entity, Target Entity
  * Attached-To: Edge visuals
- **Usage Context**: Edge rendering
- **Code Reference**: `src/presentation/components/graph_components.rs`

### Term: DomainEventOccurred
- **Category**: Presentation
- **Type**: Event
- **Taxonomy**: Bevy ECS
- **Definition**: Bevy event wrapping domain events for ECS processing
- **Relationships**:
  * Contains: EventEnvelope
  * Processed-By: Event bridge systems
  * Triggers: Visual updates
- **Usage Context**: Domain-ECS bridge
- **Code Reference**: `src/presentation/bevy_systems/event_bridge.rs`

### Term: EventBridge
- **Category**: Presentation
- **Type**: System
- **Taxonomy**: Bevy ECS
- **Definition**: System that polls EventStore and converts to Bevy events
- **Relationships**:
  * Polls: EventStore
  * Emits: DomainEventOccurred
  * Manages: Backpressure
- **Usage Context**: Event synchronization
- **Code Reference**: `src/presentation/bevy_systems/event_bridge.rs`

---

## Knowledge Domain

### Term: Research
- **Category**: Domain Object
- **Type**: Aggregate
- **Taxonomy**: Knowledge Taxonomy
- **Definition**: A systematic investigation to establish facts, test theories, and develop new understanding
- **Relationships**:
  * Contains: Evidence, Methods, Findings
  * Validates: Claims
  * Produces: Knowledge
- **Usage Context**: Foundation for knowledge creation and validation
- **Code Reference**: TBD

### Term: Evidence
- **Category**: Domain Object
- **Type**: Entity
- **Taxonomy**: Knowledge Taxonomy
- **Definition**: Observable data or information that supports or contradicts a claim
- **Relationships**:
  * Supports: Facts, Claims
  * Part-Of: Research
  * Has: Classification
- **Usage Context**: Basis for fact validation and theory building
- **Code Reference**: TBD

### Term: Method
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Knowledge Taxonomy
- **Definition**: A systematic procedure for collecting and analyzing evidence
- **Relationships**:
  * Part-Of: Research
  * Produces: Evidence
  * Follows: Standards
- **Usage Context**: Ensures reproducibility and validity of research
- **Code Reference**: TBD

### Term: Finding
- **Category**: Domain Object
- **Type**: Entity
- **Taxonomy**: Knowledge Taxonomy
- **Definition**: A specific result or insight derived from research
- **Relationships**:
  * Based-On: Evidence
  * Supports: Claims
  * Part-Of: Research
- **Usage Context**: Building blocks for knowledge construction
- **Code Reference**: TBD

### Term: Citation
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Knowledge Taxonomy
- **Definition**: A reference to a source of information or evidence
- **Relationships**:
  * References: Source
  * Validates: Claims
  * Supports: Research
- **Usage Context**: Tracking and validating information sources
- **Code Reference**: TBD

### Term: Knowledge Graph
- **Category**: Technical Concept
- **Type**: Aggregate
- **Taxonomy**: Knowledge Taxonomy
- **Definition**: A structured representation of relationships between knowledge entities
- **Relationships**:
  * Contains: Nodes, Edges
  * Represents: Relationships
  * Enables: Navigation
- **Usage Context**: Visual and programmatic knowledge exploration
- **Code Reference**: TBD

### Term: Fact
- **Category**: Domain Object
- **Type**: Entity
- **Taxonomy**: Storage Taxonomy
- **Definition**: A proven claim with verifiable evidence and reproducible validation
- **Relationships**:
  * Validates: Claims
  * Supports: Theories
  * Contains: Proofs
- **Usage Context**: Foundation for building reliable knowledge base
- **Code Reference**: TBD

### Term: Claim
- **Category**: Domain Object
- **Type**: Entity
- **Taxonomy**: Storage Taxonomy
- **Definition**: An assertion that has a repeatable construction method
- **Relationships**:
  * Depends-On: Facts
  * Supports: Theories
  * Contains: Arguments
- **Usage Context**: Building blocks for theories and knowledge construction
- **Code Reference**: TBD

### Term: Theory
- **Category**: Domain Object
- **Type**: Aggregate
- **Taxonomy**: Storage Taxonomy
- **Definition**: A structured belief system with context, explanation, and sources
- **Relationships**:
  * Contains: Claims
  * Uses: Facts
  * Supports: Models
- **Usage Context**: Framework for understanding complex systems
- **Code Reference**: TBD

## Organization Domain

### Term: Operator
- **Category**: Business Concept
- **Type**: Entity
- **Taxonomy**: Configuration Taxonomy
- **Definition**: Organization responsible for operating a CIM instance
- **Relationships**:
  * Manages: Tenants
  * Configures: Policies
  * Contains: Accounts
- **Usage Context**: Primary administrative entity for CIM operations
- **Code Reference**: TBD

### Term: Account
- **Category**: Business Concept
- **Type**: Entity
- **Taxonomy**: Configuration Taxonomy
- **Definition**: A group or individual identity within the CIM system
- **Relationships**:
  * Part-Of: Operator
  * Contains: Users
  * Has: Permissions
- **Usage Context**: Access control and resource management
- **Code Reference**: TBD

## Agent Domain

### Term: Agent
- **Category**: Technical Concept
- **Type**: Service
- **Taxonomy**: Processing Rules
- **Definition**: An autonomous entity capable of performing tasks within the CIM
- **Relationships**:
  * Uses: AI Tools
  * Processes: Information Entities
  * Has: Behaviors
- **Usage Context**: Automated processing and decision making
- **Code Reference**: TBD

### Term: Behavior
- **Category**: Technical Concept
- **Type**: Value Object
- **Taxonomy**: Processing Rules
- **Definition**: Defined patterns of action and response for agents
- **Relationships**:
  * Configures: Agent
  * Follows: Policies
  * Uses: Models
- **Usage Context**: Defining how agents interact with the system
- **Code Reference**: TBD

## Business Domain

### Term: Value Proposition
- **Category**: Business Concept
- **Type**: Aggregate
- **Taxonomy**: Business Rules
- **Definition**: The unique value offered by a solution or service
- **Relationships**:
  * Supports: Business Model
  * Contains: Solutions
  * Targets: Goals
- **Usage Context**: Defining business value and market positioning
- **Code Reference**: TBD

### Term: Solution
- **Category**: Business Concept
- **Type**: Entity
- **Taxonomy**: Business Rules
- **Definition**: A specific implementation addressing business needs
- **Relationships**:
  * Part-Of: Value Proposition
  * Uses: Models
  * Achieves: Goals
- **Usage Context**: Concrete implementations of business value
- **Code Reference**: TBD

## Environment Domain

### Term: Equipment
- **Category**: Technical Concept
- **Type**: Entity
- **Taxonomy**: Configuration Taxonomy
- **Definition**: Physical or virtual resources used by the CIM
- **Relationships**:
  * Located-In: Environment
  * Supports: Solutions
  * Has: Preferences
- **Usage Context**: Resource management and deployment
- **Code Reference**: TBD

### Term: Location
- **Category**: Business Concept
- **Type**: Value Object
- **Taxonomy**: Configuration Taxonomy
- **Definition**: Physical or logical placement of CIM components
- **Relationships**:
  * Contains: Equipment
  * Follows: Policies
  * Has: Environment
- **Usage Context**: Geographic and logical resource organization
- **Code Reference**: TBD

## Governance Domain

### Term: Policy
- **Category**: Business Concept
- **Type**: Service
- **Taxonomy**: Configuration Taxonomy
- **Definition**: Rules and guidelines governing CIM operation
- **Relationships**:
  * Governs: Behaviors
  * Enforces: Ethics
  * Follows: Laws
- **Usage Context**: System governance and compliance
- **Code Reference**: TBD

### Term: Ethics
- **Category**: Cross-Cutting
- **Type**: Service
- **Taxonomy**: Business Rules
- **Definition**: Moral principles and values guiding CIM operation
- **Relationships**:
  * Guides: Policies
  * Influences: Decisions
  * Aligns-With: Laws
- **Usage Context**: Ethical decision making and governance
- **Code Reference**: TBD
