# Presentation Events vs Domain Events

## Core Architectural Principle

**NOT EVERY EVENT IS A DOMAIN EVENT**

In our CIM architecture, we maintain a strict separation between:
- **Presentation Events**: Local, ephemeral, UI-specific events that occur within Bevy
- **Domain Events**: Business-meaningful state changes that are persisted to NATS/Event Store

## Event Categories

### Presentation Events (Bevy-Only)
These events remain entirely within the Bevy presentation layer and are NEVER sent to NATS:

1. **Animation Events**
   - Frame updates
   - Interpolation states
   - Visual transitions
   - Particle effects
   - Camera movements

2. **Interaction Events**
   - Mouse hover states
   - Drag operations in progress
   - Selection highlights
   - Preview states
   - Temporary visual feedback

3. **Layout Events**
   - Force-directed layout iterations
   - Temporary positioning during drag
   - Visual clustering animations
   - Zoom/pan operations
   - Grid snapping previews

4. **Ephemeral State**
   - Undo/redo preview states
   - Temporary subgraph manipulations
   - Visual-only transformations
   - UI state changes (menus, panels)

### Domain Events (Persisted to NATS)
These events represent actual business state changes and are sent to the Event Store:

1. **Graph Structure Events**
   - `GraphCreated` - A new graph is initialized
   - `GraphDeleted` - A graph is removed
   - `GraphModelRecognized` - A graph is identified as a known model (K7, C5, etc.)

2. **Node Events**
   - `NodeAdded` - A node is permanently added to the graph
   - `NodeRemoved` - A node is permanently removed
   - `NodeContentUpdated` - Node's business data changes

3. **Edge Events**
   - `EdgeConnected` - An edge is permanently established
   - `EdgeDisconnected` - An edge is permanently removed
   - `EdgeRelationshipChanged` - Edge type/properties change

4. **Model Events**
   - `ModelTransformed` - A graph morphism is applied
   - `ModelValidated` - A graph conforms to a known structure
   - `ModelExported` - A graph is serialized for external use

## The Aggregation Pattern

```rust
// Presentation Layer: Many small changes
fn handle_drag_operation(
    mut events: EventReader<MouseDrag>,
    mut nodes: Query<&mut Transform, With<NodeEntity>>,
    mut drag_state: ResMut<DragState>,
) {
    for event in events.read() {
        // Update visual position many times per second
        if let Ok(mut transform) = nodes.get_mut(event.entity) {
            transform.translation = event.world_position;
            drag_state.add_movement(event.entity, event.world_position);
        }
    }
}

// Domain Boundary: Aggregate into meaningful change
fn complete_drag_operation(
    mut events: EventReader<DragComplete>,
    drag_state: Res<DragState>,
    mut domain_events: EventWriter<DomainCommand>,
) {
    for event in events.read() {
        // Only send final position to domain
        if let Some(final_position) = drag_state.get_final_position(event.entity) {
            domain_events.send(DomainCommand::UpdateNodePosition {
                node_id: event.node_id,
                position: final_position,
            });
        }
    }
}
```

## Graph Model Recognition

### Known Graph Models
Our system recognizes and works with well-defined graph structures:

1. **Complete Graphs (Kn)**
   - K3: Triangle
   - K4: Tetrahedron structure
   - K5: Complete graph on 5 vertices
   - K7: Complete graph on 7 vertices

2. **Cycle Graphs (Cn)**
   - C3: Triangle cycle
   - C4: Square cycle
   - C5: Pentagon cycle

3. **State Machines**
   - Mealy Machine: Output depends on state and input
   - Moore Machine: Output depends only on state
   - Finite State Automaton: Basic state transitions

4. **Domain-Specific Models**
   - Address Graph: Represents address value objects
   - Workflow Graph: Business process representation
   - Concept Graph: Knowledge representation

### Model-Based Operations

```rust
pub enum GraphModel {
    Complete(usize),      // Kn
    Cycle(usize),        // Cn
    MealyMachine,
    MooreMachine,
    DomainModel(String), // Custom domain models
}

impl GraphAggregate {
    pub fn recognize_model(&self) -> Option<GraphModel> {
        // Analyze structure to identify known patterns
        if self.is_complete_graph() {
            Some(GraphModel::Complete(self.node_count()))
        } else if self.is_cycle() {
            Some(GraphModel::Cycle(self.node_count()))
        } else if self.matches_domain_pattern() {
            Some(GraphModel::DomainModel(self.pattern_name()))
        } else {
            None
        }
    }

    pub fn apply_morphism(&mut self, morphism: GraphMorphism) -> Result<()> {
        // Apply structure-preserving transformations
        match morphism {
            GraphMorphism::ToComplete => self.make_complete(),
            GraphMorphism::ToCycle => self.make_cycle(),
            GraphMorphism::Subdivide => self.subdivide_edges(),
            // ... other morphisms
        }
    }
}
```

## Development Workflow

### Initial State Options

1. **Empty Model**
   ```rust
   pub fn create_empty_graph() -> GraphAggregate {
       GraphAggregate::new(GraphId::new(), GraphMetadata::default())
   }
   ```

2. **Load from JSON**
   ```rust
   pub fn load_graph_from_json(json: &str) -> Result<GraphAggregate> {
       let model: GraphModel = serde_json::from_str(json)?;
       GraphAggregate::from_model(model)
   }
   ```

3. **Create Known Model**
   ```rust
   pub fn create_k7() -> GraphAggregate {
       GraphAggregate::from_complete_graph(7)
   }
   ```

### Persistence Strategy

```rust
// Work with a known model
let mut k7 = create_k7();

// Apply many visual operations in Bevy
animate_graph_layout(&mut k7);
apply_force_directed_layout(&mut k7);
highlight_subgraphs(&mut k7);

// Only persist meaningful changes
if user_confirms_save() {
    // Aggregate all changes into domain events
    let events = vec![
        DomainEvent::GraphModelRecognized {
            graph_id: k7.id,
            model: GraphModel::Complete(7),
        },
        DomainEvent::GraphLayoutApplied {
            graph_id: k7.id,
            layout_type: LayoutType::ForceDirected,
            final_positions: collect_final_positions(&k7),
        },
    ];

    // Send to domain
    event_store.append_events(events).await?;
}
```

## Benefits of This Approach

1. **Performance**: Thousands of animation frames don't flood the event store
2. **Clarity**: Domain events represent business-meaningful changes only
3. **Flexibility**: UI can experiment without committing every change
4. **Recognition**: Known models enable powerful transformations
5. **Reusability**: Standard graph models can be instantiated quickly

## Implementation Guidelines

### DO:
- ✅ Aggregate multiple UI operations into single domain commands
- ✅ Recognize and name graph models using domain terminology
- ✅ Keep animation and layout calculations in Bevy
- ✅ Send only final, user-confirmed changes to domain
- ✅ Use known graph models as templates

### DON'T:
- ❌ Send every mouse movement to NATS
- ❌ Persist animation keyframes as domain events
- ❌ Create inheritance hierarchies for graph models
- ❌ Send force-directed layout iterations to event store
- ❌ Treat UI state changes as domain events

## Example: Complete Workflow

```rust
// 1. User creates a new K7 graph
let k7_template = GraphModel::Complete(7);
let graph = GraphAggregate::from_model(k7_template);

// 2. Bevy renders and allows manipulation
// - User drags nodes around (100s of position updates)
// - Force-directed layout runs (1000s of iterations)
// - User highlights subgraphs (temporary visual state)
// - Animations play (60fps updates)

// 3. User clicks "Save"
// Only NOW do we create domain events:
let domain_events = vec![
    DomainEvent::GraphCreated {
        graph_id: graph.id,
        model: GraphModel::Complete(7),
        metadata: graph.metadata,
    },
    DomainEvent::NodesPositioned {
        graph_id: graph.id,
        positions: final_node_positions,
    },
];

// 4. Send to NATS/Event Store
event_store.append_events(domain_events).await?;
```

This architecture ensures that our domain model remains clean and focused on business-meaningful state changes, while our presentation layer has the freedom to provide rich, interactive experiences without polluting the event stream.
