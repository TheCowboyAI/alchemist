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

### Technical Terms
*For implementation details, see [Technical Infrastructure](technical.md)*
- **DDD (Domain-Driven Design)**: A software design approach focusing on modeling software to match a domain according to expert input
- **EDA (Event-Driven Architecture)**: An architectural pattern where components communicate through events
- **FRP (Functional Reactive Programming)**: Programming paradigm for reactive programming using functional programming concepts
- **NATS**: Message broker system used for internal CIM communication
- **Content-Addressing**: Method of storing and retrieving information based on its content rather than location

### Implementation Components
- **Event Store**: System for storing and managing event streams
- **Object Store**: System for storing immutable data with content addressing
- **Ontology**: System for managing relationships and classifications
- **Bundle**: Collection of reusable components and resources

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

## Graph Domain

### Term: Graph
- **Category**: Domain Object
- **Type**: Aggregate Root
- **Taxonomy**: Graph Taxonomy
- **Definition**: A collection of nodes and edges representing relationships between entities, serving as the primary organizing structure for knowledge
- **Relationships**:
  * Contains: Nodes, Edges
  * Has: GraphIdentity, GraphMetadata, GraphJourney
  * Emits: GraphCreated, GraphDeleted
- **Usage Context**: Primary structure for organizing and visualizing relationships between domain entities
- **Code Reference**: `src/contexts/graph_management/domain.rs`

### Term: Node
- **Category**: Domain Object
- **Type**: Entity
- **Taxonomy**: Graph Taxonomy
- **Definition**: A discrete point in a graph representing a single concept, entity, or piece of information
- **Relationships**:
  * Part-Of: Graph
  * Has: NodeIdentity, NodeContent, SpatialPosition
  * Connected-By: Edges
  * Emits: NodeAdded, NodeRemoved, NodeMoved
- **Usage Context**: Fundamental unit of information within a graph structure
- **Code Reference**: `src/contexts/graph_management/domain.rs`

### Term: Edge
- **Category**: Domain Object
- **Type**: Entity
- **Taxonomy**: Graph Taxonomy
- **Definition**: A connection between two nodes representing a relationship, dependency, or interaction
- **Relationships**:
  * Part-Of: Graph
  * Connects: Source Node, Target Node
  * Has: EdgeIdentity, EdgeRelationship
  * Emits: EdgeConnected, EdgeDisconnected
- **Usage Context**: Defining relationships and interactions between nodes
- **Code Reference**: `src/contexts/graph_management/domain.rs`

### Term: GraphIdentity
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: An immutable unique identifier for a graph instance
- **Relationships**:
  * Identifies: Graph
  * Used-By: All graph operations
- **Usage Context**: Ensuring unique identification of graphs across the system
- **Code Reference**: `src/contexts/graph_management/domain.rs`

### Term: NodeIdentity
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: An immutable unique identifier for a node within the graph system
- **Relationships**:
  * Identifies: Node
  * Referenced-By: Edges
- **Usage Context**: Unique identification of nodes for edge connections and operations
- **Code Reference**: `src/contexts/graph_management/domain.rs`

### Term: EdgeIdentity
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: An immutable unique identifier for an edge connection
- **Relationships**:
  * Identifies: Edge
  * Links: Source and Target nodes
- **Usage Context**: Tracking and managing edge relationships
- **Code Reference**: `src/contexts/graph_management/domain.rs`

### Term: GraphMetadata
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: Descriptive information about a graph including name, description, domain, and tags
- **Relationships**:
  * Describes: Graph
  * Contains: Name, Description, Domain, Tags
- **Usage Context**: Providing context and searchability for graphs
- **Code Reference**: `src/contexts/graph_management/domain.rs`

### Term: NodeContent
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: The informational payload of a node including label, category, and properties
- **Relationships**:
  * Contained-In: Node
  * Has: Label, Category, Properties
- **Usage Context**: Storing the actual information represented by a node
- **Code Reference**: `src/contexts/graph_management/domain.rs`

### Term: EdgeRelationship
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: The nature and properties of a connection between nodes
- **Relationships**:
  * Defines: Edge semantics
  * Contains: Source, Target, Category, Strength
- **Usage Context**: Qualifying the type and strength of relationships
- **Code Reference**: `src/contexts/graph_management/domain.rs`

### Term: SpatialPosition
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: The spatial coordinates of a node in both 2D and 3D space
- **Relationships**:
  * Positions: Node
  * Contains: 3D coordinates, 2D coordinates
- **Usage Context**: Positioning nodes for visualization and spatial algorithms
- **Code Reference**: `src/contexts/graph_management/domain.rs`

### Term: GraphJourney
- **Category**: Domain Object
- **Type**: Value Object
- **Taxonomy**: Graph Taxonomy
- **Definition**: The evolutionary history and version tracking of a graph
- **Relationships**:
  * Tracks: Graph evolution
  * Contains: Version, Event count, Last event
- **Usage Context**: Event sourcing and graph history tracking
- **Code Reference**: `src/contexts/graph_management/domain.rs`

### Term: GraphMotion
- **Category**: Domain Object
- **Type**: Component
- **Taxonomy**: Graph Visualization
- **Definition**: Dynamics controlling the motion of an entire graph including rotation, oscillation, and scaling
- **Relationships**:
  * Animates: Graph
  * Contains: Rotation speed, Oscillation parameters, Scale factor
- **Usage Context**: Creating dynamic graph visualizations
- **Code Reference**: `src/contexts/visualization/services.rs`

### Term: SubgraphOrbit
- **Category**: Domain Object
- **Type**: Component
- **Taxonomy**: Graph Visualization
- **Definition**: Orbital dynamics for subgraphs within a larger graph structure
- **Relationships**:
  * Animates: Subgraph
  * Contains: Local rotation, Orbit radius, Orbit speed
- **Usage Context**: Visualizing hierarchical graph relationships
- **Code Reference**: `src/contexts/visualization/services.rs`

### Term: NodePulse
- **Category**: Domain Object
- **Type**: Component
- **Taxonomy**: Graph Visualization
- **Definition**: Pulse dynamics for individual nodes including bouncing and scaling effects
- **Relationships**:
  * Animates: Node
  * Contains: Bounce parameters, Pulse parameters
- **Usage Context**: Highlighting or emphasizing specific nodes
- **Code Reference**: `src/contexts/visualization/services.rs`

## Graph Domain Events

### Term: GraphCreated
- **Category**: Domain Event
- **Type**: Event
- **Taxonomy**: Graph Events
- **Definition**: A fact recording that a new graph has been created in the system
- **Relationships**:
  * Emitted-By: CreateGraph service
  * Contains: GraphIdentity, GraphMetadata, Timestamp
- **Usage Context**: Event sourcing for graph lifecycle
- **Code Reference**: `src/contexts/graph_management/events.rs`

### Term: NodeAdded
- **Category**: Domain Event
- **Type**: Event
- **Taxonomy**: Graph Events
- **Definition**: A fact recording that a node has been added to a graph
- **Relationships**:
  * Emitted-By: AddNodeToGraph service
  * Contains: GraphIdentity, NodeIdentity, NodeContent, Position
- **Usage Context**: Tracking graph composition changes
- **Code Reference**: `src/contexts/graph_management/events.rs`

### Term: EdgeConnected
- **Category**: Domain Event
- **Type**: Event
- **Taxonomy**: Graph Events
- **Definition**: A fact recording that an edge has been established between nodes
- **Relationships**:
  * Emitted-By: ConnectGraphNodes service
  * Contains: GraphIdentity, EdgeIdentity, EdgeRelationship
- **Usage Context**: Tracking relationship establishment
- **Code Reference**: `src/contexts/graph_management/events.rs`

### Term: NodeRemoved
- **Category**: Domain Event
- **Type**: Event
- **Taxonomy**: Graph Events
- **Definition**: A fact recording that a node has been removed from a graph
- **Relationships**:
  * Contains: GraphIdentity, NodeIdentity
- **Usage Context**: Tracking graph composition changes
- **Code Reference**: `src/contexts/graph_management/events.rs`

### Term: EdgeDisconnected
- **Category**: Domain Event
- **Type**: Event
- **Taxonomy**: Graph Events
- **Definition**: A fact recording that an edge has been removed between nodes
- **Relationships**:
  * Contains: GraphIdentity, EdgeIdentity
- **Usage Context**: Tracking relationship removal
- **Code Reference**: `src/contexts/graph_management/events.rs`

### Term: NodeMoved
- **Category**: Domain Event
- **Type**: Event
- **Taxonomy**: Graph Events
- **Definition**: A fact recording that a node's spatial position has changed
- **Relationships**:
  * Contains: GraphIdentity, NodeIdentity, From Position, To Position
- **Usage Context**: Tracking spatial changes for visualization
- **Code Reference**: `src/contexts/graph_management/events.rs`

## Graph Domain Services

### Term: CreateGraph
- **Category**: Domain Service
- **Type**: Service
- **Taxonomy**: Graph Services
- **Definition**: Service responsible for creating new graph instances with proper initialization
- **Relationships**:
  * Creates: Graph
  * Emits: GraphCreated
  * Uses: GraphIdentity, GraphMetadata
- **Usage Context**: Graph lifecycle management
- **Code Reference**: `src/contexts/graph_management/services.rs`

### Term: AddNodeToGraph
- **Category**: Domain Service
- **Type**: Service
- **Taxonomy**: Graph Services
- **Definition**: Service responsible for adding nodes to existing graphs
- **Relationships**:
  * Modifies: Graph
  * Creates: Node
  * Emits: NodeAdded
- **Usage Context**: Graph composition management
- **Code Reference**: `src/contexts/graph_management/services.rs`

### Term: ConnectGraphNodes
- **Category**: Domain Service
- **Type**: Service
- **Taxonomy**: Graph Services
- **Definition**: Service responsible for establishing edges between nodes
- **Relationships**:
  * Creates: Edge
  * Connects: Nodes
  * Emits: EdgeConnected
- **Usage Context**: Relationship management
- **Code Reference**: `src/contexts/graph_management/services.rs`

### Term: ValidateGraph
- **Category**: Domain Service
- **Type**: Service
- **Taxonomy**: Graph Services
- **Definition**: Service responsible for validating graph operations against domain rules
- **Relationships**:
  * Validates: Graph operations
  * Returns: GraphConstraintViolation
- **Usage Context**: Ensuring graph integrity
- **Code Reference**: `src/contexts/graph_management/services.rs`

### Term: EstablishGraphHierarchy
- **Category**: Domain Service
- **Type**: Service
- **Taxonomy**: Graph Services
- **Definition**: Service responsible for establishing parent-child relationships in the scene graph
- **Relationships**:
  * Organizes: Graph-Node hierarchy
  * Creates: Parent-child relationships
- **Usage Context**: Hierarchical graph visualization
- **Code Reference**: `src/contexts/graph_management/services.rs`

### Term: RenderGraphElements
- **Category**: Visualization Service
- **Type**: Service
- **Taxonomy**: Graph Visualization
- **Definition**: Service responsible for creating visual representations of graph elements
- **Relationships**:
  * Visualizes: Nodes, Edges
  * Responds-To: NodeAdded events
- **Usage Context**: 3D graph visualization
- **Code Reference**: `src/contexts/visualization/services.rs`

### Term: HandleUserInput
- **Category**: Visualization Service
- **Type**: Service
- **Taxonomy**: Graph Visualization
- **Definition**: Service responsible for processing user interactions with the graph
- **Relationships**:
  * Processes: Mouse clicks, Keyboard input
  * Modifies: Selection state
- **Usage Context**: Interactive graph manipulation
- **Code Reference**: `src/contexts/visualization/services.rs`

### Term: AnimateGraphElements
- **Category**: Visualization Service
- **Type**: Service
- **Taxonomy**: Graph Visualization
- **Definition**: Service responsible for animating graph elements at all hierarchy levels
- **Relationships**:
  * Animates: Graphs, Subgraphs, Nodes
  * Uses: GraphMotion, SubgraphOrbit, NodePulse
- **Usage Context**: Dynamic graph visualization
- **Code Reference**: `src/contexts/visualization/services.rs`

### Term: GraphConstraintViolation
- **Category**: Domain Object
- **Type**: Error Type
- **Taxonomy**: Graph Validation
- **Definition**: Domain-specific violations of graph integrity rules
- **Relationships**:
  * Returned-By: ValidateGraph
  * Contains: Specific violation details
- **Usage Context**: Graph integrity enforcement
- **Code Reference**: `src/contexts/graph_management/services.rs`

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
