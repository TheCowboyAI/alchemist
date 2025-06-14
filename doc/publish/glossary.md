# CIM Glossary

## Core Concepts

### Aggregate
A cluster of domain objects that can be treated as a single unit. An aggregate has a root entity and defines a consistency boundary.

### Aggregate Root
The main entity within an aggregate through which all external references should go. It enforces the aggregate's invariants.

### Bounded Context
A logical boundary within which a particular domain model is defined and applicable. Each bounded context has its own ubiquitous language.

### CID (Content Identifier)
A cryptographic hash that uniquely identifies content. Used in CIM for event chains and content-addressed storage.

### Command
An instruction to perform an action that changes state. Commands are handled by aggregates and may result in events.

### Component
In the ECS pattern, a component is a data structure that holds state. Entities are composed of multiple components.

### Conceptual Space
A geometric representation of knowledge where concepts are positioned based on their semantic properties and similarities.

### CQRS (Command Query Responsibility Segregation)
An architectural pattern that separates read and write operations into different models.

### Domain Event
A record of something that happened in the domain. Events are immutable and represent facts about past occurrences.

### Entity
An object that has a distinct identity that runs through time and different states.

### Event Sourcing
A pattern where state changes are stored as a sequence of events rather than updating data in place.

### Projection
A read model built from events, optimized for specific queries.

### Query
A request for data that doesn't change state. Queries read from projections or read models.

### Repository
An abstraction that provides access to aggregates, hiding the details of data storage.

### Saga
A long-running business process that coordinates activities across multiple aggregates or bounded contexts.

### Value Object
An immutable object that is defined by its attributes rather than a unique identity.

## Domain-Specific Terms

### Agent
An autonomous actor in the system with specific capabilities and tool access.

### Capability
A specific ability or function that an agent can perform.

### ConceptualPoint
A position in conceptual space representing the semantic properties of a concept.

### ContextGraph
The underlying graph structure that supports domain operations and relationships.

### EdgeRelationship
The type and properties of a connection between two nodes in a graph.

### EntityId
A type-safe identifier for entities, parameterized by a marker type for compile-time safety.

### Flake
In the Nix domain, a self-contained project with explicit dependencies and outputs.

### GraphAggregate
The aggregate root for graph operations, managing nodes and edges.

### NodeType
Classification of nodes in a graph (e.g., Entity, Concept, Process).

### PersonName
A value object representing a person's name with given, middle, and family components.

### Policy
A set of rules that govern behavior in the system, often reacting to events.

### Workflow
A sequence of steps or activities designed to accomplish a specific goal.

## Technical Terms

### Async/Sync Bridge
A component that translates between Bevy's synchronous ECS and NATS's asynchronous messaging.

### Bevy ECS
The Entity Component System game engine used for 3D visualization and interaction.

### DomainError
The error type used throughout domain operations to represent business rule violations.

### EventEnvelope
A wrapper around domain events that includes metadata like timestamps and correlation IDs.

### EventMetadata
Additional information attached to events including causation, correlation, and user context.

### EventPublisher
The infrastructure component responsible for publishing events to NATS.

### EventStore
The persistence layer for domain events, supporting append and replay operations.

### JetStream
NATS's persistence layer that provides at-least-once delivery guarantees.

### NATS
The messaging system used for event distribution and inter-service communication.

### ObjectStore
Storage for large objects referenced by events, using content-addressed storage.

### ReadModel
A denormalized data structure optimized for specific query patterns.

### Subject
In NATS, a hierarchical string that identifies the topic for message routing.

## Architectural Patterns

### Anti-Corruption Layer
A layer that translates between different domain models to prevent concept leakage.

### Choreography
Event-driven coordination where each service knows how to react to events independently.

### Customer-Supplier
A relationship between bounded contexts where one depends on the other's API.

### Eventually Consistent
A consistency model where data will become consistent over time rather than immediately.

### Hexagonal Architecture
An architectural pattern that isolates the domain from external concerns through ports and adapters.

### Open Host Service
A bounded context that provides services to multiple other contexts through a well-defined protocol.

### Orchestration
Coordination pattern where a central component directs the workflow between services.

### Published Language
A well-documented model that acts as a common language between bounded contexts.

### Shared Kernel
A small subset of the domain model that is shared between bounded contexts.

### Ubiquitous Language
The common language used by developers and domain experts within a bounded context.

## Visualization Terms

### 3D Layout
Positioning of graph nodes in three-dimensional space for visualization.

### Force-Directed Layout
A graph layout algorithm that uses physics simulation to position nodes.

### Hierarchical Layout
Arrangement of nodes in levels based on their relationships.

### Subgraph
A subset of a graph that can be collapsed, expanded, or manipulated as a unit.

### Visual Metaphor
A visual representation that maps abstract concepts to familiar visual forms.

## Process Terms

### Command Acknowledgment
A response indicating whether a command was accepted, rejected, or is pending.

### Command Handler
A component that processes commands and coordinates aggregate operations.

### Event Processor
A component that subscribes to events and updates projections or triggers side effects.

### Query Handler
A component that processes queries and returns data from read models.

### State Machine
A model of computation consisting of states and transitions, used in workflows.

### State Transition
A change from one state to another in response to an event or condition.

## Development Terms

### Bounded Context Mapping
The practice of identifying and documenting relationships between bounded contexts.

### Domain Modeling
The process of creating a conceptual model of the domain.

### Event Storming
A workshop technique for discovering domain events and processes.

### Test-Driven Development (TDD)
A development practice where tests are written before implementation.

### Type-Safe
Code that uses the type system to prevent certain classes of errors at compile time. 