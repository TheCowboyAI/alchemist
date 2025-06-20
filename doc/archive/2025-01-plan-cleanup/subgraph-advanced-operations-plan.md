# Advanced Subgraph Operations and Enhanced Visualization Plan

## Executive Summary

This plan outlines the implementation of advanced subgraph operations including collapsing/expanding, drag-and-drop, merging/splitting, and enhanced visualization features. The implementation will be modular, following DDD principles and leveraging the existing event-sourced architecture.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     Presentation Layer (Bevy)                    │
├─────────────────────────────────────────────────────────────────┤
│  Interaction     │  Visualization    │  Animation    │  UI       │
│  Systems         │  Systems          │  Systems      │  Systems  │
├─────────────────────────────────────────────────────────────────┤
│                    Application Layer (CQRS)                      │
├─────────────────────────────────────────────────────────────────┤
│  Command         │  Event            │  Query        │  Read     │
│  Handlers        │  Processors       │  Handlers     │  Models   │
├─────────────────────────────────────────────────────────────────┤
│                     Domain Layer (DDD)                           │
├─────────────────────────────────────────────────────────────────┤
│  Subgraph        │  Graph            │  Domain       │  Value    │
│  Operations      │  Aggregate        │  Events       │  Objects  │
└─────────────────────────────────────────────────────────────────┘
```

## Phase 1: Domain Model Extensions (Week 1)

### 1.1 Subgraph State and Metadata

**New Value Objects:**
```rust
// src/domain/value_objects/subgraph_state.rs
pub enum SubgraphState {
    Expanded,
    Collapsed,
    Transitioning { progress: f32 },
}

pub struct SubgraphMetadata {
    pub name: String,
    pub description: Option<String>,
    pub color: Color,
    pub icon: Option<IconType>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub statistics: SubgraphStatistics,
}

pub struct SubgraphStatistics {
    pub node_count: usize,
    pub edge_count: usize,
    pub internal_edges: usize,
    pub external_edges: usize,
    pub depth: usize,
    pub complexity_score: f32,
}

pub enum SubgraphType {
    Module,
    Cluster,
    Namespace,
    Workflow,
    ConceptualRegion,
    Custom(String),
}
```

**New Domain Events:**
```rust
// src/domain/events/subgraph_operations.rs
pub enum SubgraphOperationEvent {
    SubgraphCollapsed {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        collapsed_at: Position3D,
        contained_nodes: Vec<NodeId>,
    },
    SubgraphExpanded {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        expansion_layout: LayoutStrategy,
    },
    SubgraphMerged {
        graph_id: GraphId,
        source_subgraphs: Vec<SubgraphId>,
        target_subgraph: SubgraphId,
        merge_strategy: MergeStrategy,
    },
    SubgraphSplit {
        graph_id: GraphId,
        source_subgraph: SubgraphId,
        resulting_subgraphs: Vec<(SubgraphId, Vec<NodeId>)>,
        split_criteria: SplitCriteria,
    },
    NodeDraggedBetweenSubgraphs {
        graph_id: GraphId,
        node_id: NodeId,
        from_subgraph: SubgraphId,
        to_subgraph: SubgraphId,
        new_position: Position3D,
    },
    SubgraphTypeChanged {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        old_type: SubgraphType,
        new_type: SubgraphType,
    },
    SubgraphMetadataUpdated {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        metadata: SubgraphMetadata,
    },
}
```

**New Commands:**
```rust
// src/domain/commands/subgraph_operations.rs
pub enum SubgraphOperationCommand {
    CollapseSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        collapse_strategy: CollapseStrategy,
    },
    ExpandSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        layout_strategy: LayoutStrategy,
    },
    MergeSubgraphs {
        graph_id: GraphId,
        source_subgraphs: Vec<SubgraphId>,
        target_name: String,
        merge_strategy: MergeStrategy,
    },
    SplitSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        split_criteria: SplitCriteria,
    },
    DragNodeToSubgraph {
        graph_id: GraphId,
        node_id: NodeId,
        target_subgraph: SubgraphId,
        position: Position3D,
    },
    UpdateSubgraphMetadata {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        metadata: SubgraphMetadata,
    },
    ChangeSubgraphType {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        new_type: SubgraphType,
    },
}
```

### 1.2 Domain Services

```rust
// src/domain/services/subgraph_analyzer.rs
pub struct SubgraphAnalyzer {
    pub fn analyze_subgraph(&self, subgraph: &Subgraph) -> SubgraphAnalysis {
        // Calculate metrics
        // Identify patterns
        // Compute complexity
    }

    pub fn find_optimal_split(&self, subgraph: &Subgraph) -> Vec<SplitSuggestion> {
        // Analyze connectivity
        // Find natural boundaries
        // Suggest split points
    }

    pub fn calculate_merge_compatibility(&self, subgraphs: &[Subgraph]) -> MergeCompatibility {
        // Check type compatibility
        // Analyze connections
        // Calculate merge cost
    }
}

// src/domain/services/layout_calculator.rs
pub struct SubgraphLayoutCalculator {
    pub fn calculate_collapsed_position(&self, nodes: &[Node]) -> Position3D {
        // Calculate centroid
        // Apply constraints
    }

    pub fn calculate_expansion_layout(&self,
        subgraph: &Subgraph,
        strategy: LayoutStrategy
    ) -> HashMap<NodeId, Position3D> {
        // Apply layout algorithm
        // Respect boundaries
        // Minimize edge crossings
    }
}
```

## Phase 2: Interaction Systems (Week 2)

### 2.1 Collapse/Expand System

```rust
// src/presentation/bevy_systems/subgraph_collapse_expand.rs
pub struct CollapsedSubgraph {
    pub subgraph_id: SubgraphId,
    pub contained_nodes: Vec<NodeId>,
    pub preview_texture: Handle<Image>,
    pub state: SubgraphState,
}

pub fn handle_subgraph_collapse(
    mut commands: Commands,
    mut subgraphs: Query<(Entity, &SubgraphOrigin, &SubgraphMember)>,
    nodes: Query<(Entity, &GraphNode, &Transform)>,
    keyboard: Res<Input<KeyCode>>,
    mouse: Res<Input<MouseButton>>,
    mut collapse_events: EventWriter<SubgraphCollapseEvent>,
) {
    // Detect collapse trigger (e.g., double-click on subgraph)
    // Calculate collapsed representation
    // Hide child nodes
    // Create collapsed visual
    // Emit collapse event
}

pub fn animate_collapse_expand(
    mut commands: Commands,
    mut animations: Query<(&mut Transform, &CollapseAnimation)>,
    time: Res<Time>,
) {
    // Smooth transitions
    // Scale animations
    // Opacity fades
}

pub fn render_collapsed_subgraph(
    mut commands: Commands,
    collapsed: Query<(Entity, &CollapsedSubgraph, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create representative geometry
    // Show node count badge
    // Display subgraph icon
}
```

### 2.2 Drag and Drop System

```rust
// src/presentation/bevy_systems/subgraph_drag_drop.rs
pub struct DragState {
    pub dragged_entity: Option<Entity>,
    pub drag_offset: Vec3,
    pub original_subgraph: Option<SubgraphId>,
    pub hover_subgraph: Option<SubgraphId>,
}

pub fn handle_node_drag_start(
    mut drag_state: ResMut<DragState>,
    nodes: Query<(Entity, &GraphNode, &Transform)>,
    mouse_pos: Res<MouseWorldPosition>,
    mouse: Res<Input<MouseButton>>,
) {
    // Detect drag initiation
    // Store original position
    // Highlight dragged node
}

pub fn update_drag_position(
    mut drag_state: ResMut<DragState>,
    mut nodes: Query<&mut Transform, With<GraphNode>>,
    mouse_pos: Res<MouseWorldPosition>,
) {
    // Update node position
    // Show drop zones
    // Highlight valid targets
}

pub fn handle_node_drop(
    mut commands: Commands,
    mut drag_state: ResMut<DragState>,
    subgraphs: Query<(Entity, &SubgraphOrigin, &SubgraphBounds)>,
    mut drop_events: EventWriter<NodeDropEvent>,
) {
    // Detect drop target
    // Validate drop
    // Update node membership
    // Emit drop event
}

pub fn visualize_drop_zones(
    mut gizmos: Gizmos,
    drag_state: Res<DragState>,
    subgraphs: Query<(&SubgraphOrigin, &SubgraphBounds)>,
) {
    // Draw drop zone indicators
    // Highlight valid targets
    // Show connection previews
}
```

### 2.3 Merge and Split Systems

```rust
// src/presentation/bevy_systems/subgraph_merge_split.rs
pub struct MergePreview {
    pub source_subgraphs: Vec<SubgraphId>,
    pub preview_bounds: BoundingBox,
    pub merge_feasibility: MergeFeasibility,
}

pub fn handle_subgraph_selection(
    mut selection: ResMut<SubgraphSelection>,
    subgraphs: Query<(Entity, &SubgraphOrigin)>,
    keyboard: Res<Input<KeyCode>>,
    mouse: Res<Input<MouseButton>>,
) {
    // Multi-select with Ctrl/Shift
    // Track selected subgraphs
    // Show selection indicators
}

pub fn preview_merge_operation(
    selection: Res<SubgraphSelection>,
    subgraphs: Query<(&SubgraphOrigin, &SubgraphBounds)>,
    mut gizmos: Gizmos,
) {
    // Calculate merged bounds
    // Show preview outline
    // Display merge statistics
}

pub fn execute_merge(
    mut commands: Commands,
    selection: Res<SubgraphSelection>,
    mut merge_events: EventWriter<SubgraphMergeEvent>,
) {
    // Validate merge
    // Create new subgraph
    // Transfer nodes
    // Update connections
    // Emit merge event
}

pub fn handle_split_gesture(
    mut commands: Commands,
    subgraphs: Query<(Entity, &SubgraphOrigin, &SubgraphAnalysis)>,
    split_tool: Res<SplitToolState>,
    mut split_events: EventWriter<SubgraphSplitEvent>,
) {
    // Detect split gesture (e.g., draw line)
    // Calculate split boundary
    // Partition nodes
    // Create new subgraphs
    // Emit split event
}
```

## Phase 3: Enhanced Visualization (Week 3)

### 3.1 Visual Styles System

```rust
// src/presentation/bevy_systems/subgraph_visual_styles.rs
pub struct SubgraphVisualStyle {
    pub base_color: Color,
    pub border_style: BorderStyle,
    pub fill_pattern: FillPattern,
    pub glow_intensity: f32,
    pub icon: Option<Handle<Image>>,
}

pub enum BorderStyle {
    Solid { width: f32 },
    Dashed { width: f32, gap: f32 },
    Dotted { width: f32, spacing: f32 },
    Gradient { start_color: Color, end_color: Color },
}

pub enum FillPattern {
    Solid,
    Gradient { start: Color, end: Color, angle: f32 },
    Pattern { texture: Handle<Image>, scale: f32 },
    Transparent { opacity: f32 },
}

pub fn apply_subgraph_styles(
    mut commands: Commands,
    subgraphs: Query<(Entity, &SubgraphOrigin, &SubgraphType), Changed<SubgraphType>>,
    style_registry: Res<SubgraphStyleRegistry>,
    mut materials: ResMut<Assets<SubgraphMaterial>>,
) {
    // Map type to style
    // Create custom materials
    // Apply visual effects
}

pub fn render_subgraph_boundaries(
    mut commands: Commands,
    subgraphs: Query<(&SubgraphOrigin, &SubgraphBounds, &SubgraphVisualStyle)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SubgraphMaterial>>,
) {
    // Generate boundary mesh
    // Apply style materials
    // Add visual effects
}

pub fn animate_subgraph_styles(
    mut styles: Query<(&mut SubgraphVisualStyle, &SubgraphAnimation)>,
    time: Res<Time>,
) {
    // Pulse effects
    // Color transitions
    // Border animations
}
```

### 3.2 Connection Visualization

```rust
// src/presentation/bevy_systems/subgraph_connections.rs
pub struct SubgraphConnection {
    pub from_subgraph: SubgraphId,
    pub to_subgraph: SubgraphId,
    pub connection_type: ConnectionType,
    pub edge_count: usize,
    pub flow_direction: FlowDirection,
}

pub enum ConnectionType {
    DataFlow,
    Dependency,
    Hierarchy,
    Association,
    Temporal,
}

pub fn calculate_subgraph_connections(
    mut commands: Commands,
    edges: Query<(&GraphEdge, &EdgeEndpoints)>,
    nodes: Query<(&GraphNode, &SubgraphMember)>,
    mut connection_cache: ResMut<SubgraphConnectionCache>,
) {
    // Group edges by subgraph
    // Calculate connection strength
    // Determine connection types
    // Cache results
}

pub fn render_subgraph_connections(
    mut commands: Commands,
    connections: Query<&SubgraphConnection>,
    subgraphs: Query<(&SubgraphOrigin, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create connection geometry
    // Apply connection styles
    // Add flow animations
}

pub fn animate_connection_flow(
    mut connections: Query<(&SubgraphConnection, &mut ConnectionFlowAnimation)>,
    time: Res<Time>,
) {
    // Animate particles along connections
    // Pulse effects
    // Direction indicators
}
```

### 3.3 Metadata Display System

```rust
// src/presentation/bevy_systems/subgraph_metadata_display.rs
pub struct SubgraphInfoPanel {
    pub subgraph_id: SubgraphId,
    pub panel_type: PanelType,
    pub position_offset: Vec2,
    pub auto_hide: bool,
}

pub enum PanelType {
    Tooltip,
    DetailCard,
    Statistics,
    MiniMap,
}

pub fn spawn_info_panels(
    mut commands: Commands,
    subgraphs: Query<(Entity, &SubgraphOrigin, &SubgraphMetadata)>,
    hover_state: Res<SubgraphHoverState>,
    ui_settings: Res<UISettings>,
) {
    // Create UI panels
    // Position relative to subgraph
    // Populate with metadata
}

pub fn update_statistics_display(
    mut panels: Query<(&mut Text, &StatisticsPanel)>,
    subgraphs: Query<(&SubgraphOrigin, &SubgraphStatistics), Changed<SubgraphStatistics>>,
) {
    // Update node counts
    // Show complexity metrics
    // Display performance data
}

pub fn render_subgraph_minimap(
    mut commands: Commands,
    subgraphs: Query<(&SubgraphOrigin, &SubgraphBounds)>,
    camera: Query<&Transform, With<Camera>>,
    mut minimap_texture: ResMut<MinimapTexture>,
) {
    // Render top-down view
    // Show subgraph positions
    // Indicate camera position
}
```

## Phase 4: Analysis and Composition (Week 4)

### 4.1 Subgraph Analysis System

```rust
// src/presentation/bevy_systems/subgraph_analysis.rs
pub struct SubgraphAnalysisResult {
    pub cohesion_score: f32,
    pub coupling_score: f32,
    pub complexity_metrics: ComplexityMetrics,
    pub suggested_operations: Vec<SuggestedOperation>,
}

pub struct ComplexityMetrics {
    pub cyclomatic_complexity: f32,
    pub depth: usize,
    pub fan_in: usize,
    pub fan_out: usize,
    pub instability: f32,
}

pub fn analyze_subgraph_composition(
    subgraphs: Query<(&SubgraphOrigin, &Children)>,
    nodes: Query<&GraphNode>,
    edges: Query<&GraphEdge>,
    mut analysis_results: ResMut<AnalysisResults>,
) {
    // Calculate cohesion
    // Measure coupling
    // Compute complexity
    // Generate suggestions
}

pub fn visualize_analysis_results(
    mut commands: Commands,
    analysis_results: Res<AnalysisResults>,
    subgraphs: Query<(Entity, &SubgraphOrigin)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Color-code by metrics
    // Show problem areas
    // Highlight suggestions
}

pub fn suggest_refactoring_operations(
    analysis_results: Res<AnalysisResults>,
    mut suggestions: ResMut<RefactoringSuggestions>,
) {
    // Identify high coupling
    // Find split opportunities
    // Suggest merges
    // Recommend reorganization
}
```

### 4.2 Composition Patterns

```rust
// src/domain/services/composition_patterns.rs
pub enum CompositionPattern {
    Layered {
        layers: Vec<SubgraphId>,
        direction: LayerDirection,
    },
    Hierarchical {
        root: SubgraphId,
        children: HashMap<SubgraphId, Vec<SubgraphId>>,
    },
    Pipeline {
        stages: Vec<SubgraphId>,
        flow_direction: FlowDirection,
    },
    StarTopology {
        center: SubgraphId,
        satellites: Vec<SubgraphId>,
    },
    Mesh {
        nodes: Vec<SubgraphId>,
        connectivity: f32,
    },
}

pub struct CompositionAnalyzer {
    pub fn detect_pattern(&self, subgraphs: &[Subgraph]) -> Option<CompositionPattern> {
        // Analyze structure
        // Match patterns
        // Return best fit
    }

    pub fn suggest_pattern(&self,
        subgraphs: &[Subgraph],
        constraints: &Constraints
    ) -> CompositionPattern {
        // Consider constraints
        // Optimize for criteria
        // Return suggestion
    }
}
```

## Phase 5: Integration and Testing (Week 5)

### 5.1 Command Handler Integration

```rust
// src/application/command_handlers/subgraph_operations_handler.rs
pub struct SubgraphOperationsHandler {
    event_store: Arc<dyn EventStore>,
    analyzer: SubgraphAnalyzer,
    layout_calculator: SubgraphLayoutCalculator,
}

impl SubgraphOperationsHandler {
    pub async fn handle_collapse(&self, cmd: CollapseSubgraph) -> Result<Vec<DomainEvent>> {
        // Load aggregate
        // Validate operation
        // Calculate collapse
        // Generate events
        // Store events
    }

    pub async fn handle_merge(&self, cmd: MergeSubgraphs) -> Result<Vec<DomainEvent>> {
        // Validate compatibility
        // Create merged subgraph
        // Transfer nodes
        // Update connections
        // Generate events
    }
}
```

### 5.2 Read Model Projections

```rust
// src/application/projections/subgraph_projections.rs
pub struct SubgraphProjection {
    subgraph_states: HashMap<SubgraphId, SubgraphState>,
    connection_graph: petgraph::Graph<SubgraphId, ConnectionInfo>,
    statistics_cache: HashMap<SubgraphId, SubgraphStatistics>,
}

impl EventHandler for SubgraphProjection {
    fn handle_event(&mut self, event: DomainEvent) -> Result<()> {
        match event {
            DomainEvent::SubgraphCollapsed { .. } => {
                // Update state
                // Recalculate connections
            }
            DomainEvent::SubgraphMerged { .. } => {
                // Merge statistics
                // Update graph
            }
            // Handle other events
        }
    }
}
```

### 5.3 Test Suite

```rust
// tests/integration/subgraph_operations_tests.rs
#[test]
fn test_collapse_expand_cycle() {
    // Create subgraph with nodes
    // Collapse subgraph
    // Verify visual state
    // Expand subgraph
    // Verify restoration
}

#[test]
fn test_drag_drop_between_subgraphs() {
    // Create source and target subgraphs
    // Drag node from source
    // Drop on target
    // Verify membership change
    // Verify event generation
}

#[test]
fn test_merge_operation() {
    // Create multiple subgraphs
    // Select for merge
    // Execute merge
    // Verify combined state
    // Check connections preserved
}

#[test]
fn test_split_operation() {
    // Create large subgraph
    // Define split criteria
    // Execute split
    // Verify partitioning
    // Check new subgraphs
}
```

## Implementation Timeline

### Week 1: Domain Model
- [ ] Implement new value objects
- [ ] Add domain events
- [ ] Create commands
- [ ] Implement domain services
- [ ] Update Graph aggregate

### Week 2: Interaction Systems
- [ ] Collapse/expand functionality
- [ ] Drag and drop system
- [ ] Merge operations
- [ ] Split operations
- [ ] Gesture recognition

### Week 3: Visualization
- [ ] Visual style system
- [ ] Connection rendering
- [ ] Metadata display
- [ ] Animation systems
- [ ] UI components

### Week 4: Analysis
- [ ] Analysis algorithms
- [ ] Pattern detection
- [ ] Suggestion generation
- [ ] Visualization of results
- [ ] Refactoring tools

### Week 5: Integration
- [ ] Command handlers
- [ ] Event processors
- [ ] Read models
- [ ] Testing
- [ ] Documentation

## Technical Considerations

### Performance
- Use spatial indexing for hit detection
- Implement LOD for large subgraphs
- Cache analysis results
- Batch visual updates
- Use GPU instancing for repeated elements

### Scalability
- Limit subgraph nesting depth
- Implement pagination for large graphs
- Use progressive rendering
- Optimize connection calculations
- Implement view frustum culling

### User Experience
- Smooth animations (60 FPS)
- Clear visual feedback
- Intuitive gestures
- Helpful tooltips
- Undo/redo support

### Maintainability
- Modular system design
- Clear separation of concerns
- Comprehensive documentation
- Extensive test coverage
- Performance benchmarks

## Success Criteria

1. **Functional Requirements**
   - All operations work correctly
   - Events are properly generated
   - State is consistently maintained
   - Visual feedback is clear

2. **Performance Requirements**
   - Operations complete in <100ms
   - Animations run at 60 FPS
   - Memory usage is bounded
   - Large graphs remain responsive

3. **Quality Requirements**
   - 90%+ test coverage
   - No critical bugs
   - Clear documentation
   - Intuitive user interface

4. **Integration Requirements**
   - Works with existing systems
   - Maintains event sourcing
   - Preserves CQRS pattern
   - Compatible with NATS

## Risk Mitigation

1. **Performance Risks**
   - Profile early and often
   - Implement progressive enhancement
   - Use level-of-detail techniques
   - Cache aggressively

2. **Complexity Risks**
   - Start with simple operations
   - Build incrementally
   - Test each component
   - Document thoroughly

3. **Integration Risks**
   - Maintain backward compatibility
   - Version events carefully
   - Test with existing data
   - Plan migration strategy

## Conclusion

This plan provides a comprehensive roadmap for implementing advanced subgraph operations and enhanced visualization. The modular approach ensures that each component can be developed and tested independently while maintaining overall system coherence. Following this plan will result in a powerful, user-friendly subgraph management system that enhances the graph editor's capabilities significantly.
