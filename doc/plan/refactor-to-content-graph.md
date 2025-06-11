# Refactor to ContentGraph

## Overview
Our current implementation uses a generic `Graph` aggregate that doesn't leverage CIM's CID/IPLD architecture. We need to refactor to use `ContentGraph` which properly integrates with our event sourcing and content-addressable storage.

## Current State Issues

1. **Generic Graph Aggregate** (`src/domain/aggregates/graph.rs`)
   - No CID chain integration
   - No IPLD storage
   - No semantic relationships
   - No content typing

2. **Mixed Terminology**
   - `Graph` vs `ContentGraph` vs `GraphContent`
   - Inconsistent usage across the codebase

3. **Missing CIM Features**
   - No automatic relationship discovery
   - No pattern detection
   - No business intelligence projections

## Target Architecture

### ContentGraph Aggregate
```rust
pub struct ContentGraph {
    // Identity
    pub id: GraphId,

    // Lazy CID evaluation for performance
    state_cid: LazyCid<GraphState>,
    pub previous_cid: Option<Cid>,  // CID chain

    // Content nodes with lazy CIDs
    pub nodes: HashMap<NodeId, ContentNode>,
    pub edges: HashMap<EdgeId, ContentEdge>,

    // Semantic relationships (memoized)
    semantic_clusters: VersionedCache<Vec<SemanticCluster>>,
    relationship_strengths: VersionedCache<HashMap<(NodeId, NodeId), f64>>,

    // Business intelligence (memoized)
    patterns: VersionedCache<Vec<DetectedPattern>>,
    metrics: VersionedCache<BusinessMetrics>,

    // Version for cache invalidation
    version: u64,
}

pub struct ContentNode {
    pub id: NodeId,
    // Lazy CID - only calculated when needed
    content: LazyCid<NodeContent>,
    pub content_type: ContentType,
    pub position: Position3D,
    pub conceptual_coordinates: Vec<f64>,
}

pub struct ContentEdge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub relationship: RelationshipPredicate,
    pub strength: f64,
    pub discovered_at: SystemTime,
}

impl ContentGraph {
    /// Get the current state CID (lazy evaluation)
    pub fn cid(&self) -> &Cid {
        self.state_cid.cid()
    }

    /// Get semantic clusters (memoized calculation)
    pub fn semantic_clusters(&self) -> Vec<SemanticCluster> {
        self.semantic_clusters.get_or_compute(|| {
            self.calculate_semantic_clusters()
        })
    }

    /// Invalidate all caches when graph changes
    fn increment_version(&mut self) {
        self.version += 1;
        self.semantic_clusters.invalidate();
        self.relationship_strengths.invalidate();
        self.patterns.invalidate();
        self.metrics.invalidate();
    }
}
```

## Refactoring Steps

### Phase 1: Create ContentGraph Aggregate
1. Create `src/domain/aggregates/content_graph.rs`
2. Implement CID chain support
3. Add IPLD storage integration
4. Implement semantic relationship tracking

### Phase 2: Update Commands
1. Rename commands to be content-focused:
   - `AddNode` → `AddContent`
   - `RemoveNode` → `RemoveContent`
   - `ConnectEdge` → `EstablishRelationship`

2. Add new commands:
   - `DiscoverRelationships`
   - `UpdateSemanticClusters`
   - `CalculateMetrics`

### Phase 3: Update Events
1. Content-focused events:
   - `ContentAdded { cid, type, position }`
   - `ContentRemoved { cid }`
   - `RelationshipDiscovered { source_cid, target_cid, predicate }`
   - `PatternDetected { pattern_type, confidence }`

### Phase 4: Integrate with IPLD
1. Store node content in IPLD object store
2. Use CIDs for content references
3. Implement content type detection
4. Add transformation tracking

### Phase 5: Add Business Intelligence
1. Implement pattern detectors
2. Add metric calculators
3. Create insight generators
4. Build dashboard projections

## Migration Strategy

### Step 1: Parallel Implementation
- Keep existing `Graph` aggregate
- Build `ContentGraph` alongside
- Test thoroughly

### Step 2: Adapter Layer
- Create adapter to convert between Graph and ContentGraph
- Allow gradual migration of features

### Step 3: Update Demos
- Create new demos showing ContentGraph features
- Show CID chain integrity
- Demonstrate pattern detection

### Step 4: Deprecate Graph
- Mark Graph as deprecated
- Update all references to use ContentGraph
- Remove Graph after full migration

## Benefits

1. **Content Integrity** - CID chains ensure tamper-proof history
2. **Automatic Intelligence** - Pattern detection and relationship discovery
3. **Business Value** - Built-in metrics and insights
4. **True CIM** - Aligns with our architecture principles
5. **Future-Proof** - Ready for AI integration and advanced analytics
6. **Performance** - Lazy CID evaluation and memoized calculations

## Performance Considerations

### Lazy CID Evaluation
- CIDs are only calculated when needed (e.g., for persistence or verification)
- Adding nodes is fast - no immediate CID calculation
- Batch operations don't trigger redundant calculations

### Memoized Calculations
- Semantic clusters calculated once per graph version
- Business metrics cached until graph changes
- Pattern detection results reused across queries

### Example Performance Impact
```rust
// Fast: No CID calculations
for i in 0..1000 {
    graph.add_content(content[i], position[i])?;
}

// CIDs calculated only when persisting
let event_store_cid = graph.cid(); // First calculation here

// Subsequent access is instant
let verification_cid = graph.cid(); // Returns cached value
```

## Demo Requirements

Create demos showing:
1. Content addition with automatic CID generation
2. Relationship discovery between content
3. Pattern detection in real-time
4. Business metric calculation
5. Dashboard projection generation

## Success Criteria

- [ ] All content stored with CIDs
- [ ] CID chains maintain integrity
- [ ] Relationships discovered automatically
- [ ] Patterns detected in real-time
- [ ] Business metrics calculated continuously
- [ ] Demos show clear value over generic Graph
