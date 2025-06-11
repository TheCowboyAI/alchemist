# Workflow Implementation Plan

## Overview

This plan outlines the implementation of a category theory-based workflow system for CIM that supports injectable states and integrates with NATS messaging.

## Phase 1: Core Workflow Infrastructure

### 1.1 Create Workflow Traits in cim-domain

```rust
// cim-domain/src/workflow/mod.rs
pub mod category;
pub mod state;
pub mod transition;
pub mod aggregate;

// Core traits
pub trait WorkflowState: Clone + Debug + PartialEq + Send + Sync {
    fn id(&self) -> StateId;
    fn is_terminal(&self) -> bool;
}

pub trait TransitionInput: Clone + Debug + Send + Sync {}
pub trait TransitionOutput: Clone + Debug + Send + Sync {}

pub trait WorkflowTransition<S, I, O>
where
    S: WorkflowState,
    I: TransitionInput,
    O: TransitionOutput,
{
    fn source(&self) -> &S;
    fn target(&self) -> &S;
    fn input(&self) -> &I;
    fn output(&self) -> &O;
    fn guard(&self, context: &WorkflowContext) -> bool;
}
```

### 1.2 Implement Category Operations

```rust
// cim-domain/src/workflow/category.rs
pub struct WorkflowCategory<S, T> {
    phantom: PhantomData<(S, T)>,
}

impl<S, T> Category for WorkflowCategory<S, T>
where
    S: WorkflowState,
    T: WorkflowTransition<S, _, _>,
{
    type Object = S;
    type Morphism = T;

    fn compose(&self, f: &T, g: &T) -> Result<T, CategoryError> {
        // Verify f.target() == g.source()
        // Return composed transition
    }

    fn identity(&self, state: &S) -> T {
        // Return identity transition for state
    }
}
```

## Phase 2: WorkflowGraph in cim-contextgraph

### 2.1 Enhance GraphModel Enum

```rust
// cim-contextgraph/src/lib.rs
pub enum GraphModel {
    // ... existing variants ...

    WorkflowGraph {
        workflow_type: WorkflowType,
        state_type: TypeId,  // For runtime type checking
        enrichment_type: EnrichmentType,
    },
}

pub enum WorkflowType {
    Sequential,
    Parallel,
    Hierarchical,
    StateMachine(StateMachineType),
}

pub enum EnrichmentType {
    BusinessValue,
    ExecutionTime,
    ResourceCost,
    Custom(String),
}
```

### 2.2 Create Generic WorkflowGraph

```rust
// cim-contextgraph/src/workflow_graph.rs
pub struct WorkflowGraph<S, I, O, V>
where
    S: WorkflowState,
    I: TransitionInput,
    O: TransitionOutput,
    V: EnrichmentValue,
{
    pub id: GraphId,
    pub graph: StableGraph<S, Box<dyn WorkflowTransition<S, I, O>>>,
    pub enrichment: HashMap<EdgeIndex, V>,
    pub metadata: WorkflowMetadata,
}

impl<S, I, O, V> WorkflowGraph<S, I, O, V> {
    pub fn new(workflow_type: WorkflowType) -> Self {
        // Initialize empty workflow graph
    }

    pub fn add_state(&mut self, state: S) -> NodeIndex {
        // Add state as node
    }

    pub fn add_transition(
        &mut self,
        source: S,
        target: S,
        input: I,
        output: O,
        guard: impl Fn(&WorkflowContext) -> bool + 'static,
    ) -> EdgeIndex {
        // Add transition as edge
    }

    pub fn find_transition(&self, state: &S, input: &I) -> Option<&dyn WorkflowTransition<S, I, O>> {
        // Find applicable transition from current state with given input
    }
}
```

## Phase 3: Workflow Aggregate

### 3.1 Create Workflow Aggregate

```rust
// cim-domain/src/workflow/aggregate.rs
pub struct WorkflowAggregate<S, I, O>
where
    S: WorkflowState,
    I: TransitionInput,
    O: TransitionOutput,
{
    pub id: WorkflowId,
    pub definition_id: GraphId,  // Reference to WorkflowGraph
    pub current_state: S,
    pub context: WorkflowContext,
    pub history: Vec<TransitionEvent>,
    components: ComponentStore,
}

impl<S, I, O> AggregateRoot for WorkflowAggregate<S, I, O> {
    type Id = WorkflowId;
    type Marker = WorkflowMarker;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn version(&self) -> u64 {
        self.history.len() as u64
    }
}
```

### 3.2 Workflow Commands and Events

```rust
// cim-domain/src/workflow/commands.rs
pub enum WorkflowCommand<I: TransitionInput> {
    StartWorkflow {
        definition_id: GraphId,
        initial_context: WorkflowContext,
    },
    ExecuteTransition {
        input: I,
    },
    SuspendWorkflow {
        reason: String,
    },
    ResumeWorkflow,
    CancelWorkflow {
        reason: String,
    },
}

// cim-domain/src/workflow/events.rs
pub enum WorkflowEvent<S, I, O>
where
    S: WorkflowState,
    I: TransitionInput,
    O: TransitionOutput,
{
    WorkflowStarted {
        workflow_id: WorkflowId,
        definition_id: GraphId,
        initial_state: S,
    },
    TransitionExecuted {
        from_state: S,
        to_state: S,
        input: I,
        output: O,
        timestamp: SystemTime,
    },
    WorkflowCompleted {
        final_state: S,
        duration: Duration,
    },
    WorkflowSuspended {
        current_state: S,
        reason: String,
    },
    WorkflowCancelled {
        current_state: S,
        reason: String,
    },
}
```

## Phase 4: NATS Integration

### 4.1 Subject Mapping

```rust
// src/infrastructure/nats/workflow_subjects.rs
pub struct WorkflowSubjectMapper<S, I, O> {
    patterns: Vec<SubjectPattern>,
    phantom: PhantomData<(S, I, O)>,
}

impl<S, I, O> WorkflowSubjectMapper<S, I, O>
where
    S: WorkflowState,
    I: TransitionInput + DeserializeOwned,
    O: TransitionOutput + Serialize,
{
    pub fn map_subject_to_input(&self, subject: &str, payload: &[u8]) -> Result<I> {
        // Parse subject and deserialize input
    }

    pub fn map_output_to_subject(&self, output: &O, workflow_id: &WorkflowId) -> String {
        // Generate subject for output event
    }
}
```

### 4.2 Workflow Execution Service

```rust
// src/application/services/workflow_execution.rs
pub struct WorkflowExecutionService<S, I, O> {
    workflows: HashMap<WorkflowId, WorkflowAggregate<S, I, O>>,
    definitions: HashMap<GraphId, WorkflowGraph<S, I, O, BusinessValue>>,
    subject_mapper: WorkflowSubjectMapper<S, I, O>,
}

impl<S, I, O> WorkflowExecutionService<S, I, O> {
    pub async fn handle_message(&mut self, msg: NatsMessage) -> Result<()> {
        // Map subject to input
        let input = self.subject_mapper.map_subject_to_input(&msg.subject, &msg.payload)?;

        // Extract workflow ID from subject or payload
        let workflow_id = extract_workflow_id(&msg)?;

        // Execute transition
        if let Some(workflow) = self.workflows.get_mut(&workflow_id) {
            let result = self.execute_transition(workflow, input).await?;

            // Publish output events
            self.publish_output(workflow_id, result).await?;
        }

        Ok(())
    }
}
```

## Phase 5: Bevy Visualization

### 5.1 Workflow Visualization Components

```rust
// src/presentation/components/workflow_graph_visual.rs
#[derive(Component)]
pub struct WorkflowGraphVisual {
    pub graph_id: GraphId,
    pub layout: LayoutAlgorithm,
}

#[derive(Component)]
pub struct WorkflowStateVisual {
    pub state_id: StateId,
    pub is_current: bool,
    pub is_terminal: bool,
}

#[derive(Component)]
pub struct WorkflowTransitionVisual {
    pub from_state: StateId,
    pub to_state: StateId,
    pub is_enabled: bool,
}
```

### 5.2 Workflow Visualization System

```rust
// src/presentation/systems/workflow_visualization.rs
pub fn visualize_workflow_graph(
    mut commands: Commands,
    workflows: Query<&WorkflowGraphVisual>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for workflow in workflows.iter() {
        // Create visual representation of workflow graph
        // - States as nodes
        // - Transitions as edges
        // - Current state highlighted
        // - Enabled transitions highlighted
    }
}
```

## Phase 6: Example Implementation

### 6.1 Document Processing Workflow

```rust
// examples/document_workflow.rs
#[derive(Clone, Debug, PartialEq)]
enum DocumentWorkflowState {
    Created,
    ContentAdded,
    UnderReview,
    Approved,
    Published,
    Archived,
    Rejected,
}

impl WorkflowState for DocumentWorkflowState {
    fn id(&self) -> StateId {
        StateId::from(format!("{:?}", self))
    }

    fn is_terminal(&self) -> bool {
        matches!(self, DocumentWorkflowState::Archived | DocumentWorkflowState::Rejected)
    }
}

// Create and execute workflow
let mut workflow_graph = WorkflowGraph::new(WorkflowType::Sequential);

// Add states
let created = workflow_graph.add_state(DocumentWorkflowState::Created);
let content_added = workflow_graph.add_state(DocumentWorkflowState::ContentAdded);

// Add transitions
workflow_graph.add_transition(
    DocumentWorkflowState::Created,
    DocumentWorkflowState::ContentAdded,
    DocumentInput::AddContent,
    DocumentOutput::ContentAdded,
    |ctx| ctx.get::<Author>("author").is_some(),
);

// Execute workflow
let workflow = WorkflowAggregate::new(workflow_graph.id);
workflow.execute_transition(DocumentInput::AddContent)?;
```

## Testing Strategy

1. **Unit Tests**: Test individual components (states, transitions, category operations)
2. **Integration Tests**: Test workflow execution with mock NATS
3. **Property Tests**: Verify category laws hold for all workflows
4. **Visual Tests**: Test Bevy visualization components

## Timeline

- **Week 1**: Core traits and category implementation
- **Week 2**: WorkflowGraph in cim-contextgraph
- **Week 3**: Workflow aggregate and commands/events
- **Week 4**: NATS integration
- **Week 5**: Bevy visualization
- **Week 6**: Testing and examples

## Success Criteria

1. Workflows support fully injectable states
2. Category theory principles are correctly implemented
3. NATS integration works seamlessly
4. Workflows can be visualized in Bevy
5. All tests pass with >90% coverage
6. Examples demonstrate real-world usage
