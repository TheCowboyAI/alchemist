# Information Alchemist Domain Model

## Overview

This document defines the domain model for Information Alchemist using Domain-Driven Design (DDD) principles. The model is organized into bounded contexts that encapsulate related concepts and ensure clear boundaries between different aspects of the system.

## Bounded Contexts

### 1. Graph Management Context

The core context responsible for graph structure and topology.

#### Aggregates

##### Graph (Aggregate Root)
- **GraphId**: Unique identifier (UUID)
- **GraphMetadata**: Name, description, creation date
- **GraphConfiguration**: Settings for physics, layout preferences
- **GraphStatistics**: Node count, edge count, last modified
- **SubgraphRegistry**: Collection of contained subgraphs
- **ParentGraphId**: Optional reference to parent graph (if this is a subgraph)

##### GraphNode (Entity)
- **NodeId**: Unique identifier (UUID)
- **NodePosition**: Current 3D coordinates
- **NodeProperties**: Map of key-value pairs
- **NodeLabels**: Set of semantic tags
- **NodeConnections**: References to connected edges
- **SubgraphId**: Optional reference to containing subgraph
- **OriginalGraphId**: Reference to source graph (for imported nodes)

##### GraphEdge (Entity)
- **EdgeId**: Unique identifier (UUID)
- **SourceNodeId**: Reference to source node
- **TargetNodeId**: Reference to target node
- **EdgeRelationship**: Semantic relationship type
- **EdgeWeight**: Numeric weight for layout algorithms
- **EdgeProperties**: Map of key-value pairs
- **SubgraphId**: Optional reference to containing subgraph
- **CrossSubgraph**: Boolean indicating if edge crosses subgraph boundaries

##### Subgraph (Entity)
- **SubgraphId**: Unique identifier (UUID)
- **SourceGraphId**: Original graph ID (for imported graphs)
- **SubgraphBoundary**: Spatial boundary definition
- **SubgraphMetadata**: Name, source file, import date
- **NodeSet**: Set of contained node IDs
- **EdgeSet**: Set of contained edge IDs
- **TransformOffset**: Position offset within parent graph
- **PreserveLayout**: Boolean to maintain original layout

#### Value Objects

##### NodePosition
- **x**: Float coordinate
- **y**: Float coordinate
- **z**: Float coordinate
- Immutable, with methods for distance calculation

##### EdgeRelationship
- **relationshipType**: String identifier
- **relationshipDirection**: Enum (FORWARD, BIDIRECTIONAL)
- **relationshipStrength**: Float (0.0 - 1.0)

##### GraphProperty
- **propertyKey**: String key
- **propertyValue**: Serializable value
- **propertyType**: Type hint for validation

##### SubgraphBoundary
- **centerPoint**: 3D position of subgraph center
- **boundingBox**: Min/max coordinates
- **convexHull**: Points defining the boundary
- **visualStyle**: How to render the boundary

##### GraphCompositionStrategy
- **strategyType**: Enum (MERGE, OVERLAY, NESTED, LINKED)
- **conflictResolution**: How to handle ID conflicts
- **layoutPreservation**: How to maintain spatial relationships
- **connectionRules**: Rules for inter-subgraph connections

#### Domain Events

- **GraphCreated**: New graph initialized
- **NodeAdded**: Node added to graph
- **NodeRemoved**: Node removed from graph
- **NodeMoved**: Node position changed
- **EdgeCreated**: Edge added between nodes
- **EdgeRemoved**: Edge removed from graph
- **EdgeReversed**: Edge direction swapped
- **PropertyUpdated**: Property added/modified
- **LabelApplied**: Label added to node/edge
- **SubgraphImported**: External graph loaded as subgraph
- **SubgraphMerged**: Subgraph integrated into parent
- **SubgraphExtracted**: Subgraph exported as independent graph
- **InterSubgraphEdgeCreated**: Edge created between subgraphs
- **SubgraphBoundaryUpdated**: Subgraph boundary recalculated

### 2. Visualization Context

Manages the visual representation and rendering of graphs.

#### Aggregates

##### VisualizationSession (Aggregate Root)
- **SessionId**: Unique session identifier
- **GraphReference**: Reference to graph being visualized
- **CameraState**: Current camera position/orientation
- **RenderMode**: Enum (MODE_3D, MODE_2D)
- **SelectionSet**: Currently selected elements
- **SubgraphVisibility**: Map of subgraph visibility states
- **HighlightedSubgraphs**: Set of emphasized subgraphs

##### VisualElement (Entity)
- **ElementId**: Maps to GraphNode or GraphEdge
- **VisualStyle**: Appearance configuration
- **Transform**: Position, rotation, scale
- **InteractionState**: Hover, selected, highlighted
- **AnimationQueue**: Pending animations
- **SubgraphVisualOverride**: Optional style for subgraph membership

##### SubgraphVisual (Entity)
- **SubgraphId**: Reference to subgraph
- **BoundaryStyle**: How to render subgraph boundary
- **BackgroundStyle**: Fill or background effect
- **LabelStyle**: How to display subgraph name
- **CollapsedState**: Whether subgraph is collapsed to single node
- **OpacityLevel**: Transparency for layering

#### Value Objects

##### VisualStyle
- **material**: Material definition (color, texture)
- **geometry**: Shape type (sphere, cube, cylinder)
- **scale**: Size multiplier
- **opacity**: Transparency value

##### CameraState
- **position**: 3D position vector
- **rotation**: Quaternion orientation
- **projection**: Perspective or orthographic
- **fieldOfView**: Camera FOV angle

##### Transform
- **translation**: 3D position
- **rotation**: Quaternion
- **scale**: 3D scale vector

#### Domain Events

- **ViewModeChanged**: Switch between 2D/3D
- **ElementSelected**: User selected element
- **ElementDeselected**: User deselected element
- **CameraMoved**: Camera position changed
- **StyleApplied**: Visual style updated
- **AnimationStarted**: Animation began
- **AnimationCompleted**: Animation finished
- **SubgraphCollapsed**: Subgraph collapsed to node
- **SubgraphExpanded**: Subgraph expanded from node
- **SubgraphHighlighted**: Subgraph visually emphasized

### 3. Layout Engine Context

Handles graph layout algorithms and physics simulation.

#### Aggregates

##### LayoutSession (Aggregate Root)
- **SessionId**: Layout session identifier
- **GraphReference**: Graph being laid out
- **LayoutAlgorithm**: Active algorithm
- **PhysicsConfiguration**: Force parameters
- **LayoutConstraints**: User-defined constraints
- **SubgraphLayoutStrategy**: How to handle subgraph layouts

##### LayoutNode (Entity)
- **NodeReference**: Maps to GraphNode
- **Position**: Current position
- **Velocity**: Current velocity vector
- **Forces**: Applied force vectors
- **Pinned**: Boolean for manual positioning
- **SubgraphConstraint**: Layout rules from subgraph membership

##### LayoutEdge (Entity)
- **EdgeReference**: Maps to GraphEdge
- **SpringConstant**: Hooke's law parameter
- **RestLength**: Desired edge length
- **Damping**: Velocity damping factor
- **CrossSubgraphPenalty**: Additional force for boundary-crossing edges

##### SubgraphLayout (Entity)
- **SubgraphReference**: Maps to Subgraph
- **InternalLayout**: Layout algorithm for subgraph interior
- **BoundaryConstraint**: Keep nodes within boundary
- **InterSubgraphSpacing**: Minimum distance between subgraphs
- **LayoutPriority**: Order for hierarchical layout

#### Value Objects

##### ForceVector
- **x**: X-component of force
- **y**: Y-component of force
- **z**: Z-component of force
- **magnitude**: Calculated magnitude

##### PhysicsParameters
- **repulsionStrength**: Coulomb's law constant
- **attractionStrength**: Spring constant multiplier
- **damping**: Global damping factor
- **timeStep**: Simulation time step
- **subgraphCohesion**: Force to keep subgraphs together

##### LayoutConstraint
- **constraintType**: Enum (FIXED, ALIGNED, GROUPED, SUBGRAPH_BOUNDARY)
- **targetElements**: Set of affected elements
- **constraintParameters**: Type-specific parameters

#### Domain Events

- **LayoutStarted**: Layout algorithm initiated
- **LayoutIteration**: Single physics step completed
- **LayoutConverged**: Layout reached stable state
- **ConstraintApplied**: User constraint added
- **ConstraintRemoved**: User constraint removed
- **ForceApplied**: External force applied
- **SubgraphLayoutStarted**: Subgraph-specific layout begun
- **SubgraphBoundaryConstraintViolated**: Node escaped subgraph

### 4. Domain Integration Context

Manages domain-specific customizations and business logic.

#### Aggregates

##### DomainConfiguration (Aggregate Root)
- **DomainId**: Domain identifier
- **DomainName**: Human-readable name
- **DomainRules**: Validation and constraint rules
- **DomainMappings**: Type to visualization mappings
- **SubgraphTypeDefinitions**: Domain-specific subgraph types

##### DomainType (Entity)
- **TypeId**: Domain type identifier
- **TypeName**: Business name
- **TypeSchema**: Validation schema
- **TypeStyle**: Default visual style
- **TypeBehavior**: Interaction rules
- **AllowedSubgraphTypes**: Valid subgraph memberships

#### Value Objects

##### DomainRule
- **ruleName**: Identifier
- **ruleExpression**: Rule logic
- **ruleAction**: Action when triggered
- **rulePriority**: Execution order
- **ruleScope**: Graph-wide or subgraph-specific

##### TypeMapping
- **sourceType**: Domain type
- **visualGeometry**: 3D shape
- **defaultColor**: Base color
- **iconReference**: 2D icon

#### Domain Events

- **DomainRegistered**: New domain added
- **TypeDefined**: Domain type created
- **RuleAdded**: Validation rule added
- **MappingCreated**: Visual mapping defined
- **ValidationFailed**: Rule violation detected
- **SubgraphTypeDefined**: New subgraph type registered

### 5. Collaboration Context

Handles multi-user scenarios and change synchronization.

#### Aggregates

##### CollaborationSession (Aggregate Root)
- **SessionId**: Collaboration identifier
- **GraphReference**: Shared graph
- **Participants**: Active users
- **ChangeLog**: Recent modifications
- **SubgraphPermissions**: Access control per subgraph

##### Participant (Entity)
- **ParticipantId**: User identifier
- **DisplayName**: Visible name
- **CursorPosition**: Current focus
- **Permissions**: Access rights
- **ConnectionState**: Online/offline
- **AllowedSubgraphs**: Subgraphs user can modify

#### Value Objects

##### ChangeOperation
- **operationId**: Unique identifier
- **operationType**: Create/update/delete
- **targetElement**: Affected element
- **changeData**: Operation details
- **timestamp**: When occurred
- **subgraphContext**: Which subgraph was affected

##### Permission
- **permissionType**: Read/write/admin
- **scope**: Graph/element/subgraph level
- **expiration**: Time limit

#### Domain Events

- **ParticipantJoined**: User joined session
- **ParticipantLeft**: User left session
- **ChangeShared**: Modification broadcast
- **ConflictDetected**: Concurrent edit conflict
- **ConflictResolved**: Conflict resolution applied
- **SubgraphLocked**: Subgraph locked for editing
- **SubgraphUnlocked**: Subgraph edit lock released

## Event Flow

All domain events follow this flow:

1. **Command** issued by user or system
2. **Validation** against domain rules
3. **Event** created and appended to stream
4. **Projection** updates read models
5. **Notification** to interested contexts

## Integration Points

### Cross-Context Communication

- Events published to NATS JetStream subjects
- Each context maintains its own event store
- Eventual consistency between contexts
- Compensating events for failure scenarios

### External Systems

- **CIM Event Bus**: Integration with broader CIM ecosystem
- **AI Agents**: Subscribe to relevant domain events
- **Storage Providers**: Persist graph and event data
- **Analytics Systems**: Process event streams

## Glossary

- **Aggregate**: Cluster of domain objects treated as a unit
- **Entity**: Object with unique identity
- **Value Object**: Immutable object without identity
- **Domain Event**: Something that happened in the domain (past-tense fact)
- **Bounded Context**: Explicit boundary within which a domain model applies
- **Subgraph**: A graph contained within another graph, maintaining its own structure and identity
