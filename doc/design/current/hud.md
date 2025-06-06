# Heads Up Display (HUD) - Power Tool for Graph Understanding

## Overview

The HUD is a critical presentation layer component that provides real-time, contextual information about graphs without cluttering the main visualization. It serves as a power tool for understanding graph structure, recognizing patterns, and monitoring transformations.

## Core Principles

1. **Non-Intrusive**: Information appears when needed, fades when not
2. **Context-Aware**: Shows relevant data based on current focus and activity
3. **Performance-First**: Updates via presentation events only (no domain events)
4. **Configurable**: Users can customize what they see and when

## HUD Information Layers

### 1. Graph Model Recognition Layer

**Purpose**: Instantly identify the type of graph you're working with

```rust
#[derive(Component)]
pub struct ModelRecognitionHUD {
    pub detected_model: Option<GraphModel>,
    pub confidence: f32,
    pub structural_properties: StructuralProperties,
}

pub struct StructuralProperties {
    pub is_complete: bool,
    pub is_cycle: bool,
    pub is_bipartite: bool,
    pub is_planar: bool,
    pub chromatic_number: Option<usize>,
}
```

**Display Elements**:
- Model badge (e.g., "K7 - Complete Graph")
- Confidence meter for partial matches
- Available morphisms for current model
- Structural property indicators

### 2. Real-Time Statistics Layer

**Purpose**: Live metrics that update as you interact

```rust
#[derive(Component)]
pub struct GraphStatisticsHUD {
    pub node_count: usize,
    pub edge_count: usize,
    pub connected_components: usize,
    pub average_degree: f32,
    pub density: f32,
    pub diameter: Option<usize>,
}
```

**Display Elements**:
- Node/Edge counters with delta indicators (+3 nodes)
- Graph density visualization (sparse ← → dense)
- Component count with visual separation
- Degree distribution mini-chart

### 3. Selection Context Layer

**Purpose**: Deep information about selected elements

```rust
#[derive(Component)]
pub struct SelectionHUD {
    pub selected_nodes: Vec<NodeId>,
    pub selected_edges: Vec<EdgeId>,
    pub subgraph_properties: Option<SubgraphAnalysis>,
    pub suggested_operations: Vec<SuggestedOperation>,
}

pub struct SubgraphAnalysis {
    pub forms_clique: bool,
    pub is_independent_set: bool,
    pub cut_size: usize,
    pub modularity_score: f32,
}
```

**Display Elements**:
- Selection count and type
- Subgraph pattern detection
- Available operations (extract, contract, etc.)
- Relationship analysis between selected nodes

### 4. Transformation Preview Layer

**Purpose**: Preview morphisms before applying them

```rust
#[derive(Component)]
pub struct TransformationHUD {
    pub available_morphisms: Vec<GraphMorphism>,
    pub preview_active: bool,
    pub impact_analysis: TransformationImpact,
}

pub struct TransformationImpact {
    pub nodes_affected: usize,
    pub edges_affected: usize,
    pub reversible: bool,
    pub complexity_change: ComplexityDelta,
}
```

**Display Elements**:
- Morphism selector with descriptions
- Before/after preview toggle
- Impact metrics (nodes/edges added/removed)
- Undo/redo stack visualization

### 5. Performance Monitor Layer

**Purpose**: Track system performance and event flow

```rust
#[derive(Component)]
pub struct PerformanceHUD {
    pub fps: f32,
    pub presentation_events_per_second: usize,
    pub pending_domain_commands: usize,
    pub animation_queue_depth: usize,
    pub force_layout_iterations: usize,
}
```

**Display Elements**:
- FPS counter with graph
- Event flow visualization (presentation → aggregation → domain)
- Animation queue status
- Force-directed layout convergence meter

### 6. Conceptual Space Layer

**Purpose**: Show semantic relationships and clustering

```rust
#[derive(Component)]
pub struct ConceptualHUD {
    pub active_dimensions: Vec<String>,
    pub cluster_count: usize,
    pub semantic_coherence: f32,
    pub suggested_layouts: Vec<LayoutType>,
}
```

**Display Elements**:
- Active conceptual dimensions
- Cluster membership indicators
- Semantic similarity heat map
- Layout suggestion based on semantics

## HUD Layout System

### Adaptive Positioning

```rust
pub enum HUDPosition {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    Floating { x: f32, y: f32 },
    FollowSelection { offset: Vec2 },
}

#[derive(Component)]
pub struct HUDLayout {
    pub position: HUDPosition,
    pub opacity: f32,
    pub scale: f32,
    pub auto_hide: bool,
    pub fade_distance: f32,
}
```

### Smart Visibility

The HUD uses intelligent visibility rules:

1. **Proximity Fade**: Elements fade as camera moves away
2. **Activity Fade**: Information appears during relevant activities
3. **Focus Fade**: Non-relevant information dims when focusing
4. **Performance Fade**: Reduces detail when FPS drops

## Implementation Architecture

### ECS Components

```rust
// HUD Bundle for each information layer
#[derive(Bundle)]
pub struct HUDBundle {
    pub hud_type: HUDType,
    pub visibility: Visibility,
    pub layout: HUDLayout,
    pub style: HUDStyle,
    pub update_frequency: UpdateFrequency,
}

// Configurable update rates
pub enum UpdateFrequency {
    Realtime,           // Every frame
    Throttled(f32),     // Max updates per second
    OnChange,           // Only when data changes
    OnDemand,           // User-triggered
}
```

### Update Systems

```rust
// Presentation-only system (no domain events)
fn update_statistics_hud(
    graph_query: Query<(&GraphNode, &GraphEdge)>,
    mut hud_query: Query<&mut GraphStatisticsHUD>,
    time: Res<Time>,
) {
    // Calculate statistics from current ECS state
    // Update HUD components
    // No domain events generated
}

// Aggregated updates for expensive calculations
fn update_model_recognition_hud(
    graph_state: Res<GraphState>,
    mut hud_query: Query<&mut ModelRecognitionHUD>,
    mut last_update: Local<f32>,
    time: Res<Time>,
) {
    // Throttle expensive recognition
    if time.elapsed_seconds() - *last_update < 0.5 {
        return;
    }

    // Perform recognition
    // Update HUD
    *last_update = time.elapsed_seconds();
}
```

## User Interaction

### HUD Controls

```rust
pub struct HUDControls {
    pub toggle_key: KeyCode,           // Default: Tab
    pub cycle_detail_key: KeyCode,     // Default: D
    pub pin_hud_key: KeyCode,         // Default: P
    pub reset_layout_key: KeyCode,    // Default: R
}
```

### Customization Options

Users can:
1. **Toggle Layers**: Show/hide specific information layers
2. **Adjust Detail**: Simple → Detailed → Expert modes
3. **Pin Information**: Keep specific data visible
4. **Save Layouts**: Store preferred HUD configurations

## Visual Design

### Styling System

```rust
#[derive(Component)]
pub struct HUDStyle {
    pub background_color: Color,
    pub text_color: Color,
    pub accent_color: Color,
    pub font_size: f32,
    pub padding: f32,
    pub corner_radius: f32,
    pub blur_background: bool,
}

// Predefined themes
pub enum HUDTheme {
    Minimal,      // Clean, low contrast
    Technical,    // Detailed, high contrast
    Accessible,   // High visibility, large text
    Custom(HUDStyle),
}
```

### Visual Hierarchy

1. **Primary Info**: Large, high contrast (model type, selection)
2. **Secondary Info**: Medium, medium contrast (statistics)
3. **Tertiary Info**: Small, low contrast (performance)
4. **Alerts**: Bright, animated (warnings, suggestions)

## Integration with Graph Operations

### Event Flow Visualization

The HUD can show the event flow in real-time:

```
Presentation Events          Domain Boundary           Domain Events
─────────────────           ───────────────           ─────────────
Mouse Drag (100/s)    →     Aggregation      →       NodesMoved (1)
Force Layout (60/s)   →     Batch & Filter   →       LayoutApplied (1)
Selection (instant)   →     Immediate        →       NodesSelected (1)
```

### Model-Aware Suggestions

Based on recognized models, suggest operations:
- **K7 Detected**: "Convert to bipartite K(3,4)"
- **Cycle Detected**: "Add chord to create wheel graph"
- **Tree Detected**: "Balance tree structure"

## Performance Considerations

1. **Lazy Calculation**: Only compute visible HUD elements
2. **Caching**: Cache expensive calculations (diameter, chromatic number)
3. **LOD System**: Reduce detail at distance or low FPS
4. **Async Updates**: Non-critical calculations in background

## Example Use Cases

### 1. Graph Exploration
- See model type instantly
- Understand structure at a glance
- Identify patterns and anomalies

### 2. Interactive Editing
- Preview operations before applying
- See impact of changes in real-time
- Maintain context during complex edits

### 3. Performance Tuning
- Monitor event flow rates
- Identify performance bottlenecks
- Optimize based on metrics

### 4. Learning Tool
- Understand graph properties
- See relationships between concepts
- Learn through visual feedback

## Future Enhancements

1. **AI Integration**: HUD shows AI agent suggestions
2. **Collaboration**: See other users' focus areas
3. **History**: Timeline of graph evolution
4. **Automation**: Trigger actions from HUD
5. **Export**: Generate reports from HUD data

The HUD transforms from a simple information display into a comprehensive power tool for graph understanding, making complex graph operations intuitive and accessible while maintaining our architectural principles of clean separation between presentation and domain concerns.


