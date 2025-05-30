# Requirements of Alchemist

Alchemist is a Graph Modeling Expert implementing a dual-layer architecture for maximum performance and flexibility.

## Core Architecture Requirements

Completely modular system STRICTLY adhering to ECS and DDD principles

### Dual-Layer Design
1. **Computational Layer** (Petgraph/Daggy)
   - Graph topology and relationships
   - Node/edge data storage
   - Graph algorithms (shortest path, topological sort, cycle detection)
   - Serialization/deserialization
   - Must support 250k+ elements (CIM requirement)

2. **Visualization Layer** (Bevy ECS)
   - Spatial positioning and transforms
   - Visual properties (colors, sizes, styles)
   - User interaction (selection, dragging, hovering)
   - Animation and transitions
   - Dual-mode rendering (3D/2D)

### Graph Data Model
- **Nodes** are entities with:
  - Unique UUID identifier
  - Position in 3D space
  - Properties (key-value pairs)
  - Labels (semantic tags)
  - Visual style (color, size, shape)

- **Edges** have:
  - A Triple: (SourceNodeID, TargetNodeID, Relationship)
  - Edges do not have save position, they are a direct connection between two nodes, source->target denotes direction
  - Variable weights for layout algorithms
  - Properties (key-value pairs)
  - Labels (semantic tags)
  - Visual style (material, color, thickness, line style)

### Physics-Based Layout
- Edge weights controlled by:
  - **Hooke's Law**: Spring forces for connected nodes
  - **Coulomb's Law**: Repulsion between all nodes
  - Force-directed layout for automatic organization
  - Support for manual positioning override

## Functional Requirements

### Graph Operations
1. **Event-Driven Operations**
   - Create New Graph
   - Create nodes/edges
   - reverse edge direction (swap source and target)
   - Update properties and positions
   - Delete with cascade handling
   - Batch operations for performance

2. **Graph Algorithms**
   - Shortest path finding
   - Topological sorting for DAGs
   - Cycle detection
   - Subgraph extraction
   - Connected component analysis

3. **Layout Algorithms**
   - Force-directed (spring layout)
   - Hierarchical (for DAGs)
   - Circular
   - Geometric
   - Star
   - Grid-based
   - Custom domain-specific layouts

### Visualization Requirements
1. **3D Mode**
   - Orbital camera controls
   - Perspective projection
   - 3D node meshes (spheres, cubes)
   - Cylindrical edges with proper alignment

2. **2D Mode**
   - Top-down orthographic view
   - 2D shapes (circles, rectangles)
   - Simplified edge rendering
   - Optimized for large graph overview

3. **Smooth Transitions**
   - Animated camera mode switching
   - Interpolated position changes
   - LOD system for performance

### User Interaction
1. **Selection**
   - Click to select nodes/edges
   - Box selection in 2D mode
   - Multi-select with modifier keys
   - Selection highlighting

2. **Manipulation**
   - Drag nodes to reposition
   - Create edges by dragging between nodes
   - Delete with keyboard shortcuts
   - Undo/redo support via event sourcing

3. **Navigation**
   - Pan, zoom, orbit controls
   - Focus on selection
   - Minimap for large graphs
   - Search and filter capabilities

## Performance Requirements

### Scalability
- Render 60 FPS with 1,000 nodes
- Support 250,000+ elements total
- Incremental updates (only changed elements)
- Spatial indexing for quick lookups

### Optimization
- Frustum culling for off-screen elements
- Level-of-detail (LOD) system
- Batched rendering operations
- Change detection to minimize updates

## Integration Requirements

### Event Sourcing
- All modifications through events
- Append-only event log
- Event replay for reconstruction
- Integration with NATS JetStream
- Tiered escalation of Events: App, Local, Leaf, Cluster, SuperCluster, Domain

### Serialization
- JSON format for graph data
- Compatible with common graph formats (native, arrows.app, cyper, mermaid)
- Preserves all properties and metadata in native format
- Support for incremental updates

### Domain Integration
- Pluggable domain types
- Custom validation rules
- Domain-specific visual styles
- Business workflow mapping

## Technical Constraints

### Platform
- NixOS development environment
- Bevy 0.16.0 game engine
- Rust programming language
- WebGPU/Vulkan rendering

### Dependencies
- petgraph 0.8+ for graph algorithms
- daggy 0.9+ for DAG operations
- bevy_egui 0.34+ for UI panels
- egui-snarl 0.7.1+ for workflow nodes
- serde for serialization

### Architecture Principles
- ECS (Entity Component System) design
- Event-driven architecture
- Composable modules ("Lego blocks")
- Clean separation of concerns
- DDD Naming Conventions:
  - Events are Past Tense
  - Commands are Verbs
  - Names reflect Known Hierarchies

