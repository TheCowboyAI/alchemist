# CIM Vocabulary

[← Back to Index](index.md)

## Core Concepts
*For detailed implementation, see [Architecture Overview](architecture.md)*

### Architecture Terms
- **CIM (Composable Information Machine)**: A framework for building distributed systems that transform scattered information into organized, actionable knowledge
- **Domain**: A unique set of ideas and concepts that cannot be further reduced, representing a specific area of knowledge or business function
- **Pod**: A self-contained functional unit within CIM that provides specific services (e.g., AI, communications, documentation)
- **Event**: A record of something that happened within the CIM system
- **Command**: An instruction that modifies CIM state
- **Message**: Any transmission within the CIM system
- **Observation**: A snapshot of state at a particular time

### Technical Terms
*For implementation details, see [Technical Infrastructure](technical.md)*
- **DDD (Domain-Driven Design)**: A software design approach focusing on modeling software to match a domain according to expert input
- **EDA (Event-Driven Architecture)**: An architectural pattern where components communicate through events
- **FRP (Functional Reactive Programming)**: Programming paradigm for reactive programming using functional programming concepts
- **MCP (Model Context Protocol)**: Protocol for AI systems to communicate with data sources and tools
- **NATS**: Message broker system used for internal CIM communication
- **Content-Addressing**: Method of storing and retrieving information based on its content rather than location

### Implementation Components
*For detailed examples, see [Implementation Guide](implementation.md)*
- **Event Store**: System for storing and managing event streams
- **Object Store**: System for storing immutable data with content addressing
- **BMC (Board Management Controller)**: Component for hardware management
- **Fabricator**: Component for system construction and deployment
- **Ontology**: System for managing relationships and classifications
- **Library**: Collection of reusable components and resources

## Domain Categories
*For full domain documentation, see [Domain Structure](domain_structure.md)*

### Knowledge Management
*Detailed in [Knowledge Management](knowledge_management.md)*
- **Fact**: A proven claim with verifiable evidence
- **Claim**: An idea with repeatable construction
- **Theory**: A belief with supporting context and sources
- **Idea**: A preliminary thought without formal theory
- **Argument**: Support or opposition for a claim

### Organizational
*Detailed in [Organizational Structure](organizational.md)*
- **Goal**: A defined achievement target
- **Organization**: A structural entity
- **Operator**: A system controller
- **Account**: A managed group or entity

### Business
*Detailed in [Business and Value](business.md)*
- **Business Model**: Operational framework for value creation
- **Value Proposition**: Benefit offering to stakeholders
- **Solution**: Resolution to a defined problem
- **Proposal**: Formal suggestion for action

### Governance
*Detailed in [Governance](governance.md)*
- **Policy**: Operational guideline
- **Law**: Regulatory framework
- **Ethics**: Moral principles
- **Politics**: Power dynamics and relationships

### Technical
*Detailed in [Technical Infrastructure](technical.md)*
- **Model**: System representation
- **Equipment**: Physical resource
- **Environment**: Contextual setting
- **Location**: Spatial information
- **Secret**: Protected information

## Implementation Types
*For specific implementations, see [Implementation Guide](implementation.md)*

### Infrastructure
*See specific implementations: [STB](stb_implementation.md), [Portal](portal_implementation.md)*
- **STB (Set-Top Box)**: Hardware implementation for media streaming and control
- **Portal**: Web-based interface implementation
- **Domain Implementation**: Specific instance of CIM for a particular organization or purpose

### Security
*Detailed in [Security Model](security.md)*
- **mTLS**: Mutual Transport Layer Security authentication
- **YubiKey**: Hardware authentication device
- **OpenPGP**: Encryption standard
- **OpenSSL**: Cryptographic software library

---
*For contribution guidelines, see [Contributing](index.md#contributing)*

# CIM Domain Vocabulary - Notes

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