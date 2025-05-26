# Panel System Consolidation Plan

## Current Issues

### 1. Duplicate Inspector Systems
- `graph_inspector_ui` in `graph_core/ui.rs` - Full graph inspector with left/right panels
- `inspector_panel_system` in `ui_panels/inspector_panel.rs` - Another inspector implementation
- Both show node lists, properties, and algorithm controls
- Conflicting UI layouts (multiple side panels)

### 2. Overlapping Functionality
- Control panel has graph stats that duplicate inspector
- Both systems handle node selection differently
- Algorithm controls appear in multiple places
- File operations scattered across panels

### 3. Context Separation Issues
- Graph-specific UI mixed with general UI
- Domain concepts (DDD, ECS) mixed with graph operations
- No clear separation between view modes and data operations

## Consolidation Strategy

### Phase 1: Remove Duplicate Inspector
1. **Keep**: `inspector_panel_system` as the main inspector
2. **Remove**: `graph_inspector_ui` from graph_core
3. **Move**: Algorithm controls to dedicated algorithm panel
4. **Merge**: Best features from both implementations

### Phase 2: Create Focused Panels

#### 1. Graph Inspector Panel (Right)
- Node/Edge properties only
- Selection state display
- Quick actions (delete, duplicate)
- Search/filter functionality

#### 2. Control Panel (Left)
- View mode controls
- Graph patterns
- Creation tools
- File operations

#### 3. Algorithm Panel (Bottom/Floating)
- Pathfinding controls
- Graph analysis results
- Layout algorithms
- Performance metrics

#### 4. Properties Panel (Separate from Inspector)
- Detailed property editing
- Batch operations
- Advanced node/edge configuration

### Phase 3: Context-Based UI

#### Graph Context
- Show only graph-related panels
- Hide domain-specific tools
- Focus on graph manipulation

#### DDD Context
- Show DDD-specific panels
- Graph becomes secondary
- Domain modeling tools prominent

#### ECS Context
- ECS-specific panels
- Component/System editors
- Performance monitoring

## Implementation Steps

### Step 1: Refactor Inspector
```rust
// Consolidate into single inspector
pub fn inspector_panel_system(
    // Show only current selection
    // Remove duplicate node lists
    // Focus on properties
)
```

### Step 2: Extract Algorithm Panel
```rust
// New dedicated algorithm panel
pub fn algorithm_panel_system(
    // All algorithm controls
    // Results visualization
    // Performance metrics
)
```

### Step 3: Improve Panel Manager
```rust
// Context-aware panel management
impl PanelManager {
    fn set_context(&mut self, context: AppContext) {
        // Show/hide panels based on context
    }
}
```

### Step 4: Remove Redundancy
- Remove `graph_inspector_ui` system
- Clean up duplicate state management
- Consolidate selection handling

## Benefits

1. **No More Conflicts**: Single source of truth for each UI element
2. **Better Performance**: Fewer overlapping systems
3. **Clearer UX**: Users know where to find features
4. **Maintainable**: Each panel has clear responsibility
5. **Context-Aware**: UI adapts to current task

## Migration Path

1. Create feature flag for new UI
2. Implement new panels alongside old
3. Migrate functionality incrementally
4. Remove old systems once stable
5. Update documentation

## Success Criteria

- [ ] No duplicate UI elements
- [ ] Clear panel responsibilities
- [ ] Context switching works smoothly
- [ ] Performance improved
- [ ] User feedback positive
