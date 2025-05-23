# 3D Graph Editor Implementation Roadmap

## Quick Start Guide

This roadmap provides actionable steps to begin implementing the 3D graph editor following CIM principles and Bevy ECS architecture.

## Prerequisites Checklist

- [ ] Bevy 0.16.0 configured in Cargo.toml
- [ ] NixOS development environment with direnv
- [ ] NATS JetStream for event persistence
- [ ] Egui integration for UI panels

## Week 1: Foundation

### Day 1-2: Project Structure
```bash
# Create modular structure
src/
├── graph/
│   ├── mod.rs
│   ├── components.rs
│   ├── events.rs
│   └── plugin.rs
├── camera/
│   ├── mod.rs
│   ├── components.rs
│   ├── systems.rs
│   └── plugin.rs
└── main.rs
```

### Day 3-4: Basic Components
1. Implement core graph components (GraphNode, GraphEdge)
2. Create camera mode components (ViewMode, GraphViewCamera)
3. Define initial events (GraphEvent enum)

### Day 5: Minimal Working Example
```rust
// Goal: Display a single node that can be viewed in 3D/2D
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            GraphPlugin,
            CameraPlugin,
        ))
        .run();
}
```

## Week 2: Core Functionality

### Day 1-2: Camera Systems
- [ ] Implement orbit camera controls (mouse drag to rotate)
- [ ] Implement 2D pan controls (mouse drag to pan)
- [ ] Add smooth transition between modes (press 'V' to switch)

### Day 3-4: Graph Rendering
- [ ] Basic node rendering (3D spheres/2D circles)
- [ ] Simple edge rendering (lines between nodes)
- [ ] Grid/reference plane rendering

### Day 5: Interaction
- [ ] Mouse picking/selection in both modes
- [ ] Basic node dragging in 2D mode
- [ ] Node selection highlighting

## Week 3: Tools and UI

### Day 1-2: Egui Integration
- [ ] Tools panel on left side of viewport
- [ ] Mode indicator (2D/3D)
- [ ] Basic graph statistics display

### Day 3-4: Creation Tools
- [ ] Node creation tool (click to place)
- [ ] Edge creation tool (drag between nodes)
- [ ] Delete tool (select and press Delete)

### Day 5: Property Editing
- [ ] Node properties panel
- [ ] Edge properties panel
- [ ] Basic validation feedback

## Week 4: Domain Integration

### Day 1-2: Event System
- [ ] Connect to NATS JetStream
- [ ] Implement event publishing for all graph operations
- [ ] Basic event replay functionality

### Day 3-4: Domain Modeling
- [ ] Define domain node types
- [ ] Implement domain validation rules
- [ ] Create domain-specific visual styles

### Day 5: Persistence
- [ ] Save graph state to NATS
- [ ] Load graph from saved state
- [ ] Auto-save functionality

## Critical Path Implementation Order

1. **Camera System First**
   - Get dual-mode viewing working before graph complexity
   - Ensures smooth user experience from the start

2. **Simple Graph Rendering**
   - Start with static test data
   - Focus on visual clarity in both modes

3. **Interaction Layer**
   - Mouse picking is foundational for all tools
   - Get selection working before manipulation

4. **Tools and UI**
   - Only after core interaction works
   - Keep tools simple initially

5. **Domain Integration**
   - Last step once foundation is solid
   - Can iterate on domain specifics

## Testing Strategy

### Unit Tests (Continuous)
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_camera_mode_transition() {
        // Test smooth interpolation
    }
    
    #[test]
    fn test_graph_event_generation() {
        // Verify events are created correctly
    }
}
```

### Integration Tests (Weekly)
- Full camera mode switching
- Graph creation workflow
- Event persistence and replay

### Performance Benchmarks
- Target: 60 FPS with 1000 nodes
- Measure: Frame time with increasing node counts
- Profile: Memory usage and allocations

## Common Pitfalls to Avoid

1. **Over-Engineering Early**
   - Start simple, iterate based on needs
   - Don't implement all domain features upfront

2. **Ignoring Performance**
   - Profile early and often
   - Use Bevy's built-in diagnostics

3. **Tight Coupling**
   - Keep graph logic separate from rendering
   - Use events for cross-system communication

4. **Complex State Management**
   - Let ECS handle state through components
   - Avoid global mutable state

## Success Metrics

### Week 1
- [ ] Can display and switch between 3D/2D views
- [ ] Basic graph structure in place

### Week 2
- [ ] Interactive camera controls working
- [ ] Can create and view simple graphs

### Week 3
- [ ] Full tool palette available
- [ ] Property editing functional

### Week 4
- [ ] Domain integration complete
- [ ] Persistence working

## Next Steps After MVP

1. **Performance Optimization**
   - Implement frustum culling
   - Add level-of-detail system
   - GPU instancing for large graphs

2. **Advanced Features**
   - Graph layout algorithms
   - Collaborative editing
   - AI-assisted design

3. **Domain Expansion**
   - Additional node/edge types
   - Complex validation rules
   - Domain-specific tools

Remember: Follow the CIM principles of composability - each component should be a self-contained "Lego block" that can be tested and used independently. 