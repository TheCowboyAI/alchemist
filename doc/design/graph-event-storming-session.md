# Graph Domain Event Storming Session

## Session Overview

**Purpose**: Discover domain events, commands, aggregates, and bounded contexts for the Information Alchemist Graph system.

**Participants Needed**:
- Domain Expert (you - understanding the business needs)
- Developer (me - technical implementation)
- Facilitator (shared role)

## Event Storming Process

### Step 1: Brainstorm Domain Events (Orange Stickies)

Let's identify all events that happen in our graph domain. Events are past-tense facts.

**Initial Event Discovery**:

#### Graph Lifecycle Events
- GraphCreatedEvent
- GraphDeletedEvent
- GraphArchivedEvent
- GraphRestoredEvent
- GraphDuplicatedEvent
- GraphMergedEvent

#### Node Management Events
- NodeAddedEvent
- NodeRemovedEvent
- NodeModifiedEvent
- NodeRepositionedEvent
- NodeLabelChangedEvent
- NodeCategoryChangedEvent
- NodePropertiesUpdatedEvent
- NodeMergedEvent
- NodeSplitEvent

#### Edge Management Events
- EdgeConnectedEvent
- EdgeDisconnectedEvent
- EdgeReversedEvent
- EdgeWeightChangedEvent
- EdgeCategoryChangedEvent
- EdgePropertiesUpdatedEvent

#### View & Interaction Events
- GraphViewChangedEvent (2D/3D)
- GraphZoomedEvent
- GraphPannedEvent
- NodeSelectedEvent
- NodeDeselectedEvent
- MultipleNodesSelectedEvent
- EdgeSelectedEvent
- SelectionClearedEvent

#### Analysis Events
- ShortestPathCalculatedEvent
- CyclesDetectedEvent
- ClustersIdentifiedEvent
- CentralityCalculatedEvent
- LayoutAppliedEvent

#### Import/Export Events
- GraphImportedEvent
- GraphExportedEvent
- ImportValidatedEvent
- ImportFailedEvent
- ExportCompletedEvent

#### Collaboration Events
- GraphSharedEvent
- CollaboratorJoinedEvent
- CollaboratorLeftEvent
- ChangesSynchronizedEvent
- ConflictDetectedEvent
- ConflictResolvedEvent

#### Animation/Replay Events
- ReplayStartedEvent
- ReplayPausedEvent
- ReplaySteppedEvent
- AnimationCompletedEvent
- TimelineMarkedEvent

### Step 2: Identify Commands (Blue Stickies)

Commands trigger events. Let's map commands to their events:

| Command | Resulting Events |
|---------|-----------------|
| CreateGraph | GraphCreatedEvent |
| AddNode | NodeAddedEvent |
| ConnectNodes | EdgeConnectedEvent |
| SelectNode | NodeSelectedEvent, (NodeDeselectedEvent for others) |
| ChangeView | GraphViewChangedEvent |
| ImportGraph | ImportValidatedEvent, GraphImportedEvent OR ImportFailedEvent |
| StartReplay | ReplayStartedEvent |
| CalculateShortestPath | ShortestPathCalculatedEvent |

### Step 3: Identify Aggregates (Yellow Stickies)

Aggregates are consistency boundaries. What are our aggregates?

1. **Graph** (Aggregate Root)
   - Identity: GraphIdentity
   - Contains: Nodes, Edges
   - Maintains: Consistency of node/edge relationships

2. **GraphView** (Aggregate)
   - Identity: ViewIdentity
   - For: Specific graph
   - Maintains: Camera state, selection state

3. **GraphAnalysis** (Aggregate)
   - Identity: AnalysisIdentity
   - For: Specific graph snapshot
   - Maintains: Analysis results cache

4. **GraphCollaboration** (Aggregate)
   - Identity: SessionIdentity
   - For: Specific graph
   - Maintains: Active collaborators, changes

### Step 4: Discover Bounded Contexts

Based on our events and aggregates, we can identify these bounded contexts:

#### 1. **Graph Management Context** (Core Domain)
- **Language**: Graph, Node, Edge, Connection, Relationship
- **Aggregates**: Graph
- **Events**:
  - GraphCreatedEvent, GraphDeletedEvent
  - NodeAddedEvent, NodeRemovedEvent, NodeModifiedEvent
  - EdgeConnectedEvent, EdgeDisconnectedEvent
- **Domain Services**:
  - GraphRepository
  - GraphValidationDomainService

#### 2. **Visualization Context** (Supporting Domain)
- **Language**: View, Perspective, Camera, Selection, Layout
- **Aggregates**: GraphView
- **Events**:
  - GraphViewChangedEvent, GraphZoomedEvent
  - NodeSelectedEvent, SelectionClearedEvent
  - LayoutAppliedEvent
- **Domain Services**:
  - GraphLayoutDomainService
  - SelectionDomainService

#### 3. **Analysis Context** (Supporting Domain)
- **Language**: Path, Cycle, Cluster, Centrality, Algorithm
- **Aggregates**: GraphAnalysis
- **Events**:
  - ShortestPathCalculatedEvent
  - CyclesDetectedEvent
  - ClustersIdentifiedEvent
- **Domain Services**:
  - GraphAnalysisDomainService
  - AlgorithmExecutionDomainService

#### 4. **Import/Export Context** (Supporting Domain)
- **Language**: Format, Schema, Validation, Transformation
- **Aggregates**: ImportSession, ExportSession
- **Events**:
  - GraphImportedEvent, ImportValidatedEvent
  - GraphExportedEvent, ExportCompletedEvent
- **Domain Services**:
  - GraphSerializationDomainService
  - FormatValidationDomainService

#### 5. **Collaboration Context** (Generic Subdomain)
- **Language**: Session, Collaborator, Synchronization, Conflict
- **Aggregates**: GraphCollaboration
- **Events**:
  - GraphSharedEvent, CollaboratorJoinedEvent
  - ChangesSynchronizedEvent, ConflictResolvedEvent
- **Domain Services**:
  - CollaborationDomainService
  - ConflictResolutionDomainService

#### 6. **Animation Context** (Supporting Domain)
- **Language**: Timeline, Replay, Animation, Transition
- **Aggregates**: AnimationSession
- **Events**:
  - ReplayStartedEvent, ReplayPausedEvent
  - AnimationCompletedEvent
- **Domain Services**:
  - AnimationDomainService
  - TimelineManagementDomainService

### Step 5: Define Context Maps

```
[Graph Management] <--published language--> [Visualization]
        |                                          |
        |                                          |
        v                                          v
[Import/Export] <--anticorruption layer--> [External Formats]
        |
        |
        v
[Analysis] <--shared kernel--> [Algorithm Library]
        |
        |
        v
[Collaboration] <--open host service--> [Client Applications]
        |
        |
        v
[Animation] <--conformist--> [Graph Management]
```

### Step 6: Refine Event Names by Context

Now let's ensure events follow DDD naming within their contexts:

#### Graph Management Context
- ✅ GraphCreatedEvent (not GraphWasCreated)
- ✅ NodeAddedEvent (not NodeWasAdded)
- ✅ EdgeConnectedEvent (not EdgeWasConnected)

#### Visualization Context
- ✅ ViewPerspectiveChangedEvent (not ViewModeChanged)
- ✅ NodeSelectionChangedEvent (not NodeWasSelected)
- ✅ LayoutCalculatedEvent (not LayoutWasApplied)

### Step 7: Event Correlation Patterns

For each context, define correlation patterns:

```rust
// Graph Management Context
pub struct GraphManagementEventCorrelation {
    pub graph_identity: GraphIdentity,
    pub command_id: CommandIdentity,
    pub actor: ActorIdentity,
    pub timestamp: SystemTime,
}

// Visualization Context
pub struct VisualizationEventCorrelation {
    pub view_identity: ViewIdentity,
    pub graph_identity: GraphIdentity,
    pub interaction_id: InteractionIdentity,
}
```

## Questions for Domain Expert (You)

1. **Graph Management**:
   - Should graphs support versioning/branching?
   - Can nodes exist in multiple graphs?
   - Are there graph templates or types?

2. **Visualization**:
   - What layout algorithms are most important?
   - Should view states be shareable?
   - Are there domain-specific visual rules?

3. **Analysis**:
   - Which graph algorithms are critical?
   - Should analysis results be cached?
   - Are there real-time analysis needs?

4. **Collaboration**:
   - What conflict resolution strategy?
   - Should we support offline editing?
   - Are there permission levels?

5. **Import/Export**:
   - Which formats are must-have vs nice-to-have?
   - Should we preserve format-specific metadata?
   - Validation strictness levels?

## Next Steps

1. **Review and refine** the discovered events and contexts
2. **Answer domain questions** to clarify boundaries
3. **Create event flow diagrams** for key scenarios
4. **Define context integration points**
5. **Begin implementation** with Graph Management context

## Event Flow Example: Creating a Graph with Nodes

```
Actor: User
Command: CreateGraphWithNodes
  |
  v
GraphCreatedEvent (Graph Management)
  |
  +--> NodeAddedEvent (multiple)
  |      |
  |      +--> LayoutCalculatedEvent (Visualization)
  |             |
  |             +--> ViewUpdatedEvent (Visualization)
  |
  +--> GraphSharedEvent (Collaboration)
         |
         +--> NotificationSentEvent (Notification Context)
```

This Event Storming session helps ensure our implementation aligns with the domain and maintains consistent naming across bounded contexts!
