# Single Responsibility Demos Plan

## Design Principles

1. **Single Responsibility**: Each demo focuses on ONE specific feature
2. **Composable**: Demos can be combined to show workflows
3. **Domain Focused**: Organized by domain capability, not technology
4. **Testable**: Each demo validates specific functionality
5. **Incremental**: Build from simple to complex

## Demo Organization by Domain

### 1. Event Infrastructure Demos

#### 1.1 `demo_nats_connection`
- **Purpose**: Verify NATS connection and health
- **Shows**: Connection status, server info, JetStream status
- **Output**: Console log of connection details

#### 1.2 `demo_event_persistence`
- **Purpose**: Store and retrieve a single event
- **Shows**: Event → NATS JetStream → Retrieval
- **Output**: Event stored with CID, retrieved by ID

#### 1.3 `demo_cid_chain`
- **Purpose**: Demonstrate CID chain integrity
- **Shows**: Create 5 chained events, verify chain
- **Output**: Visual chain representation

#### 1.4 `demo_event_replay`
- **Purpose**: Replay events from a point in time
- **Shows**: Store 10 events, replay from event 5
- **Output**: Replayed events in order

### 2. Graph Domain Demos

#### 2.1 `demo_graph_create`
- **Purpose**: Create a simple graph
- **Shows**: Graph aggregate creation with metadata
- **Output**: Graph with ID and metadata

#### 2.2 `demo_node_operations`
- **Purpose**: Add, update, remove nodes
- **Shows**: Node CRUD operations
- **Output**: Graph with 5 nodes

#### 2.3 `demo_edge_operations`
- **Purpose**: Connect and disconnect nodes
- **Shows**: Edge creation with relationships
- **Output**: Connected graph

#### 2.4 `demo_graph_validation`
- **Purpose**: Show business rule enforcement
- **Shows**: Max capacity, duplicate prevention
- **Output**: Validation errors and successes

### 3. Visualization Demos

#### 3.1 `demo_basic_visualization`
- **Purpose**: Render a simple graph
- **Shows**: 3D graph with camera controls
- **Output**: Interactive 3D view

#### 3.2 `demo_force_layout`
- **Purpose**: Apply force-directed layout
- **Shows**: Physics-based node positioning
- **Output**: Self-organizing graph

#### 3.3 `demo_animation_system`
- **Purpose**: Animate graph changes
- **Shows**: Smooth transitions
- **Output**: Animated node/edge updates

### 4. Conceptual Graph Demos

#### 4.1 `demo_concept_graph_create`
- **Purpose**: Create a ConceptGraph
- **Shows**: Graph with quality dimensions
- **Output**: ConceptGraph with 3 dimensions

#### 4.2 `demo_graph_morphism`
- **Purpose**: Apply graph transformations
- **Shows**: Structure-preserving mappings
- **Output**: Transformed graph

#### 4.3 `demo_graph_composition`
- **Purpose**: Compose two graphs
- **Shows**: Product/Coproduct operations
- **Output**: Composed graph

### 5. Conceptual Space Demos

#### 5.1 `demo_conceptual_space_create`
- **Purpose**: Create a conceptual space
- **Shows**: Space with quality dimensions
- **Output**: 3D conceptual space

#### 5.2 `demo_concept_mapping`
- **Purpose**: Map concepts to positions
- **Shows**: Concept placement in space
- **Output**: Positioned concepts

#### 5.3 `demo_similarity_calculation`
- **Purpose**: Calculate concept similarity
- **Shows**: Distance metrics
- **Output**: Similarity scores

#### 5.4 `demo_region_definition`
- **Purpose**: Define conceptual regions
- **Shows**: Convex regions for categories
- **Output**: Colored regions

### 6. Workflow Demos

#### 6.1 `demo_workflow_create`
- **Purpose**: Create a simple workflow
- **Shows**: Workflow with steps
- **Output**: Workflow structure

#### 6.2 `demo_workflow_validation`
- **Purpose**: Validate workflow structure
- **Shows**: Validation rules
- **Output**: Valid/invalid states

#### 6.3 `demo_workflow_execution`
- **Purpose**: Execute a workflow
- **Shows**: State transitions
- **Output**: Execution trace

### 7. Subgraph Demos

#### 7.1 `demo_subgraph_create`
- **Purpose**: Create subgraphs
- **Shows**: Hierarchical organization
- **Output**: Parent with 3 subgraphs

#### 7.2 `demo_subgraph_merge`
- **Purpose**: Merge two subgraphs
- **Shows**: Union operation
- **Output**: Merged subgraph

#### 7.3 `demo_subgraph_split`
- **Purpose**: Split a subgraph
- **Shows**: Division by criteria
- **Output**: Two new subgraphs

### 8. Import/Export Demos

#### 8.1 `demo_markdown_import`
- **Purpose**: Import from Mermaid
- **Shows**: Parse and visualize
- **Output**: Graph from markdown

#### 8.2 `demo_json_import`
- **Purpose**: Import from JSON
- **Shows**: Structured import
- **Output**: Graph from JSON

#### 8.3 `demo_export_graph`
- **Purpose**: Export to various formats
- **Shows**: Serialization options
- **Output**: Exported files

## Composition Examples

### Workflow: "Build a Knowledge Graph"
1. Run `demo_graph_create`
2. Run `demo_node_operations`
3. Run `demo_edge_operations`
4. Run `demo_concept_mapping`
5. Run `demo_similarity_calculation`

### Workflow: "Event Sourced Graph"
1. Run `demo_nats_connection`
2. Run `demo_graph_create`
3. Run `demo_event_persistence`
4. Run `demo_cid_chain`
5. Run `demo_event_replay`

### Workflow: "Visual Graph Design"
1. Run `demo_basic_visualization`
2. Run `demo_node_operations`
3. Run `demo_force_layout`
4. Run `demo_animation_system`

## Implementation Strategy

### Phase 1: Core Infrastructure (Week 1)
- Implement all Event Infrastructure demos
- Ensure NATS integration works perfectly
- Validate CID chains

### Phase 2: Domain Logic (Week 2)
- Implement Graph Domain demos
- Add Conceptual Space demos
- Complete Workflow demos

### Phase 3: Visualization (Week 3)
- Implement all Visualization demos
- Add interactive controls
- Polish animations

### Phase 4: Advanced Features (Week 4)
- Implement Subgraph demos
- Add Import/Export demos
- Create composition scripts

## Success Criteria

Each demo must:
1. Run independently
2. Complete in < 30 seconds
3. Produce verifiable output
4. Handle errors gracefully
5. Include usage documentation
6. Be testable via CI/CD

## Demo Execution

### Individual Demo
```bash
cargo run --bin demo_graph_create
```

### Composed Workflow
```bash
./scripts/run_workflow.sh "Build a Knowledge Graph"
```

### All Demos
```bash
./scripts/run_all_demos.sh
```

## Current Status

### Existing Demos to Refactor
- `conceptual_graph_demo.rs` → Split into 3 focused demos
- `markdown_import_demo.rs` → Already focused, keep
- `subgraph_demo.rs` → Split into create/merge/split

### New Demos Needed
- All Event Infrastructure demos
- Graph validation demo
- Conceptual Space demos
- Workflow execution demo
- Export demos

### Priority Order
1. Event Infrastructure (foundation)
2. Graph Domain (core functionality)
3. Visualization (user experience)
4. Advanced Features (completeness)
