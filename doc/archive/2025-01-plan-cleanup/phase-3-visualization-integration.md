# Phase 3: Visualization and UI Integration

## Overview

This phase focuses on creating visual representations of conceptual graphs and integrating them with the Bevy UI system. We'll build interactive tools for graph composition, domain model import, and workflow visualization.

## Timeline: 2 Weeks (January 9-23, 2025)

## Week 1: Visual Representation and Interaction

### Day 1-2: ConceptGraph Visualization Components

**Goal**: Create Bevy components for visualizing conceptual graphs with their quality dimensions and relationships.

#### Visual Components

```rust
// src/presentation/components/conceptual_visualization.rs

#[derive(Component)]
pub struct ConceptualNodeVisual {
    pub concept_id: ConceptId,
    pub node_type: ConceptNodeType,
    pub quality_position: ConceptualPoint,
    pub visual_style: NodeVisualStyle,
}

#[derive(Component)]
pub struct ConceptualEdgeVisual {
    pub edge_id: EdgeId,
    pub relationship: ConceptRelationship,
    pub visual_style: EdgeVisualStyle,
}

#[derive(Component)]
pub struct QualityDimensionAxis {
    pub dimension: QualityDimension,
    pub axis_direction: Vec3,
    pub scale: f32,
}

#[derive(Component)]
pub struct ConceptualSpaceVisual {
    pub space_id: ConceptualSpaceId,
    pub dimensions: Vec<QualityDimensionAxis>,
    pub origin: Vec3,
}
```

#### Visualization Systems

```rust
// src/presentation/systems/conceptual_visualization.rs

fn visualize_conceptual_nodes(
    mut commands: Commands,
    concepts: Query<&ConceptNode, Added<ConceptNode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for concept in concepts.iter() {
        // Map conceptual position to 3D space
        let visual_position = map_to_visual_space(&concept.quality_position);

        // Create visual representation based on node type
        let (mesh, material) = create_concept_visual(&concept.node_type);

        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(visual_position),
            ConceptualNodeVisual { ... },
        ));
    }
}

fn animate_quality_dimensions(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &QualityDimensionAxis)>,
) {
    // Animate dimension axes for clarity
}
```

### Day 3-4: Interactive Graph Manipulation

**Goal**: Enable direct manipulation of conceptual graphs through UI interactions.

#### Interaction Components

```rust
#[derive(Component)]
pub struct DraggableNode {
    pub constraints: DragConstraints,
    pub snap_to_grid: bool,
}

#[derive(Component)]
pub struct ConnectableNode {
    pub allowed_connections: Vec<ConceptRelationship>,
    pub max_connections: Option<usize>,
}

#[derive(Component)]
pub struct SelectableGraph {
    pub graph_id: ConceptGraphId,
    pub selection_mode: SelectionMode,
}
```

#### Interaction Systems

```rust
fn handle_node_dragging(
    mut events: EventReader<PointerDrag>,
    mut nodes: Query<(&mut Transform, &DraggableNode, &ConceptualNodeVisual)>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    // Convert screen coordinates to world space
    // Update node position while respecting constraints
    // Emit events for domain model updates
}

fn handle_edge_creation(
    mut events: EventReader<NodeConnectionRequest>,
    nodes: Query<(&ConceptualNodeVisual, &ConnectableNode)>,
    mut commands: EventWriter<CreateConceptEdgeCommand>,
) {
    // Validate connection is allowed
    // Create edge command for domain layer
}
```

### Day 5: Context Bridge Visualization

**Goal**: Visualize relationships between different bounded contexts.

```rust
#[derive(Component)]
pub struct ContextBridgeVisual {
    pub bridge_id: ContextBridgeId,
    pub source_position: Vec3,
    pub target_position: Vec3,
    pub mapping_type: ContextMappingType,
}

fn visualize_context_bridges(
    bridges: Query<&ContextBridge, Added<ContextBridge>>,
    contexts: Query<(&ConceptualSpaceVisual, &Transform)>,
    mut gizmos: Gizmos,
) {
    for bridge in bridges.iter() {
        // Draw bridge visualization between contexts
        // Use different styles for different mapping types
    }
}
```

## Week 2: Domain Import and Workflow Engine

### Day 6-7: Domain Model Importers

**Goal**: Import external domain models into conceptual graphs.

#### Import Infrastructure

```rust
// src/domain/services/domain_import.rs

pub trait DomainImporter {
    type Input;
    type Error;

    fn import(&self, input: Self::Input) -> Result<ConceptGraph, Self::Error>;
}

pub struct DDDImporter {
    pub context_mapper: ContextMapper,
    pub aggregate_mapper: AggregateMapper,
}

impl DomainImporter for DDDImporter {
    type Input = DDDModel;
    type Error = DDDImportError;

    fn import(&self, model: DDDModel) -> Result<ConceptGraph> {
        // Map DDD concepts to conceptual graph
        let mut graph = ConceptGraph::new();

        // Import bounded contexts
        for context in model.bounded_contexts {
            let context_node = self.context_mapper.map_context(&context);
            graph.add_node(context_node);
        }

        // Import aggregates
        for aggregate in model.aggregates {
            let aggregate_graph = self.aggregate_mapper.map_aggregate(&aggregate);
            graph.embed(aggregate_graph);
        }

        // Import relationships
        for relationship in model.relationships {
            let edge = self.map_relationship(&relationship);
            graph.add_edge(edge);
        }

        Ok(graph)
    }
}
```

#### Import UI

```rust
#[derive(Component)]
pub struct ImportPanel {
    pub import_type: ImportType,
    pub file_path: Option<PathBuf>,
    pub preview: Option<ConceptGraph>,
}

fn handle_import_request(
    mut events: EventReader<ImportRequest>,
    mut commands: EventWriter<ImportDomainModelCommand>,
    importers: Res<ImporterRegistry>,
) {
    for request in events.read() {
        let importer = importers.get(&request.import_type);
        match importer.import(&request.data) {
            Ok(graph) => {
                commands.send(ImportDomainModelCommand {
                    graph,
                    merge_strategy: request.merge_strategy,
                });
            }
            Err(e) => {
                // Show error in UI
            }
        }
    }
}
```

### Day 8-9: Workflow Engine Visualization

**Goal**: Create visual workflow editor using conceptual graphs.

```rust
#[derive(Component)]
pub struct WorkflowNode {
    pub node_type: WorkflowNodeType,
    pub inputs: Vec<PortId>,
    pub outputs: Vec<PortId>,
    pub execution_state: ExecutionState,
}

#[derive(Component)]
pub struct WorkflowEdge {
    pub source_port: PortId,
    pub target_port: PortId,
    pub data_type: DataType,
}

fn execute_workflow_visually(
    mut workflows: Query<(&WorkflowGraph, &mut WorkflowState)>,
    mut nodes: Query<(&WorkflowNode, &mut NodeAppearance)>,
    time: Res<Time>,
) {
    // Update visual state based on execution
    // Animate data flow through edges
    // Show execution progress
}
```

### Day 10: Integration and Polish

**Goal**: Complete integration of all visualization features.

- Create unified graph editor UI
- Add toolbar for graph operations
- Implement save/load functionality
- Performance optimization
- Create example workflows

## Deliverables

1. **ConceptGraph Visualization**
   - 3D visualization of conceptual spaces
   - Quality dimension representation
   - Interactive node manipulation

2. **Context Bridge Visualization**
   - Visual representation of context relationships
   - Translation flow visualization
   - Mapping type indicators

3. **Domain Model Importers**
   - DDD model importer
   - UML importer (stretch goal)
   - Import preview and validation

4. **Workflow Engine**
   - Visual workflow editor
   - Execution visualization
   - Data flow animation

5. **Integrated Graph Editor**
   - Unified UI for all graph types
   - Tool palette
   - Property inspector
   - Save/load functionality

## Success Criteria

- [ ] Conceptual graphs render with quality dimensions
- [ ] Users can drag nodes and create edges
- [ ] Context bridges show relationships clearly
- [ ] DDD models import successfully
- [ ] Workflows can be created and executed visually
- [ ] Performance: 60 FPS with 1000+ nodes
- [ ] All features integrated in cohesive UI

## Technical Considerations

1. **Performance**
   - Use instanced rendering for large graphs
   - LOD system for distant nodes
   - Spatial indexing for interaction

2. **UI/UX**
   - Consistent visual language
   - Clear affordances for interactions
   - Responsive to different screen sizes

3. **Integration**
   - Events flow properly between UI and domain
   - State synchronization is robust
   - Undo/redo support

## Next Phase Preview

Phase 4 will focus on:
- AI agent integration
- Semantic search and reasoning
- Advanced composition patterns
- Collaborative features
