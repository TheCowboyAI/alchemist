# Legacy System Feature Set

## Core Features Implemented

### 1. Graph Management
- Create new graphs with metadata
- Add nodes to graphs
- Connect nodes with edges
- Remove nodes and edges
- Graph persistence using Petgraph OR Daggy

### 2. Visualization (Bevy 3D)
- 3D node rendering (blue spheres)
- Edge rendering as lines
- Camera controls (arrow keys, mouse)
- Graph rotation animation
- Real-time updates from events

### 3. Selection System
- Single node selection (click)
- Multi-select with Shift/Ctrl
- Select all (Ctrl+A)
- Clear selection (Esc)
- Visual feedback (highlight colors)

### 4. Layout Algorithms
- Force-directed layout
- Physics-based positioning
- Manual layout trigger (L key)
- Smooth animation transitions
- Configurable physics parameters

### 5. Import/Export
- JSON format support
- MD format support (mermaid)
- File dialog integration (rfd)
- Import graphs (Ctrl+O)
- Export graphs (Ctrl+S)
- Round-trip data preservation

### 6. Storage Layer
- Daggy-based graph storage
- Node and edge indices
- Event synchronization
- Error handling and validation

### 7. Event System
- Domain events (GraphCreated, NodeAdded, etc.)
- Event-driven architecture
- Bevy event integration
- DDD-compliant naming

### 8. User Interface
- Keyboard shortcuts
- Mouse interactions
- Status feedback
- Error messages

## Technical Capabilities

### Performance
- Handles up to ~10K nodes
- 60 FPS rendering
- Real-time updates

### Architecture
- Clean bounded contexts
- Plugin-based modularity
- Event-driven communication
- DDD principles throughout

### Testing
- 106/114 tests passing
- Unit and integration tests
- Domain isolation tests
- Performance benchmarks

## Key Bindings

| Key | Action |
|-----|--------|
| Click | Select node |
| Shift+Click | Add to selection |
| Ctrl+Click | Toggle selection |
| Ctrl+A | Select all |
| Esc | Clear selection |
| L | Apply layout |
| Ctrl+O | Import graph |
| Ctrl+S | Export graph |
| Arrow Keys | Move camera |

## Data Formats

### Graph JSON Structure
```json
{
  "version": "1.0",
  "metadata": {
    "name": "Graph Name",
    "created_at": "timestamp"
  },
  "nodes": [
    {
      "id": "uuid",
      "content": {},
      "position": [x, y, z]
    }
  ],
  "edges": [
    {
      "id": "uuid",
      "source": "node_uuid",
      "target": "node_uuid",
      "relationship": "type"
    }
  ]
}
```

## Bounded Contexts

1. **graph_management** - Core domain logic
2. **visualization** - 3D rendering
3. **selection** - User interactions
4. **layout** - Graph algorithms
5. **import_export** - File I/O
6. **event_store** - Event persistence (minimal)
