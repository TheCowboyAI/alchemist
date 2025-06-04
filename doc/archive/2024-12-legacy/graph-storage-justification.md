# Graph Storage Design Justification

## Why Not Use Petgraph?

### The Core Principle
**Bevy ECS IS a graph database** when you use Entity references. Adding petgraph would create another graph model to synchronize, which is exactly the problem we're solving.

## Current Design Analysis

### What the HashSets Actually Do
```rust
#[derive(Component)]
pub struct GraphIndex {
    pub nodes: HashSet<Entity>,  // Just for O(1) membership checks
    pub edges: HashSet<Entity>,  // Just for O(1) membership checks
}

#[derive(Component)]
pub struct NodeIndex {
    pub outgoing: Vec<Entity>,  // Edge entities for traversal
    pub incoming: Vec<Entity>,  // Edge entities for traversal
}
```

These aren't the graph structure - they're just indices for fast queries. The actual graph is:
- **Nodes**: Entities with Node components
- **Edges**: Entities with Edge components containing source/target Entity refs
- **Relationships**: Direct Entity references (pointers)

### ECS as a Graph Database

```rust
// This IS a graph edge in ECS
#[derive(Component)]
pub struct Edge {
    pub source: Entity,  // Direct pointer to source node
    pub target: Entity,  // Direct pointer to target node
    pub weight: f32,
}

// Graph traversal is just ECS queries
fn traverse_from(start: Entity, edges: Query<&Edge>) -> Vec<Entity> {
    edges.iter()
        .filter(|e| e.source == start)
        .map(|e| e.target)
        .collect()
}
```

## Comparison: ECS vs Petgraph

### Using Petgraph (Don't Do This)
```rust
// BAD: Creates another model to sync
pub struct GraphStorage {
    petgraph: Graph<NodeData, EdgeData>,
    entity_to_node: HashMap<Entity, NodeIndex>,
    node_to_entity: HashMap<NodeIndex, Entity>,
}

// Now you have to sync ECS ↔ Petgraph
fn sync_to_petgraph(world: &World, storage: &mut GraphStorage) {
    // Synchronization overhead!
}
```

### Using Pure ECS (Current Design)
```rust
// GOOD: Single source of truth
fn dijkstra_in_ecs(
    start: Entity,
    end: Entity,
    edges: Query<&Edge>,
    nodes: Query<&Node>,
) -> Option<Vec<Entity>> {
    // Implement directly on ECS queries
    // No synchronization needed!
}
```

## When Petgraph Makes Sense: As a View

### Option 1: On-Demand Computation View
```rust
/// Build petgraph temporarily for complex algorithms
pub fn compute_centrality(
    graph_entity: Entity,
    world: &World,
) -> HashMap<Entity, f32> {
    // Build petgraph view
    let pg = build_petgraph_view(graph_entity, world);

    // Run algorithm
    let centrality = petgraph::algo::betweenness_centrality(&pg);

    // Map results back to entities
    map_results_to_entities(centrality)

    // Petgraph is discarded - no persistent state
}
```

### Option 2: Cached Algorithm Results
```rust
#[derive(Component)]
pub struct GraphAlgorithmCache {
    pub shortest_paths: Option<HashMap<(Entity, Entity), Vec<Entity>>>,
    pub centrality: Option<HashMap<Entity, f32>>,
    pub communities: Option<Vec<HashSet<Entity>>>,
}

// Compute on demand, cache results
fn get_shortest_path(
    cache: &mut GraphAlgorithmCache,
    start: Entity,
    end: Entity,
    world: &World,
) -> Option<Vec<Entity>> {
    if cache.shortest_paths.is_none() {
        compute_all_shortest_paths(world, cache);
    }
    cache.shortest_paths.as_ref()?.get(&(start, end)).cloned()
}
```

## Performance Comparison

### Memory Usage
- **Petgraph + ECS**: 2x memory (duplicate structure)
- **Pure ECS**: 1x memory (single structure)
- **ECS + View**: 1x memory + temporary spike during computation

### Traversal Performance
```rust
// ECS traversal (with NodeIndex)
// O(1) to find outgoing edges, then O(k) to iterate
fn ecs_neighbors(node: Entity, index: &NodeIndex) -> &[Entity] {
    &index.outgoing
}

// Petgraph traversal
// Similar O(1) + O(k), but requires indirection through NodeIndex
fn petgraph_neighbors(node: NodeIndex, graph: &Graph) -> Neighbors {
    graph.neighbors(node)
}
```

### Cache Efficiency
- **ECS**: Entities are allocated in archetypes, good cache locality
- **Petgraph**: Separate allocation, potential cache misses
- **Winner**: ECS when traversing with other components

## Recommendation

### For Simple Operations (90% of cases)
Use pure ECS with index components:
- Graph membership checks
- Basic traversal
- Local neighborhood queries
- Simple path finding

### For Complex Algorithms (10% of cases)
Build temporary petgraph views:
- All-pairs shortest path
- Complex centrality measures
- Community detection
- Maximum flow

### Implementation Example
```rust
pub trait GraphAlgorithms {
    /// Simple traversal - pure ECS
    fn neighbors(&self, node: Entity) -> Vec<Entity>;

    /// Complex algorithm - temporary petgraph
    fn betweenness_centrality(&self) -> HashMap<Entity, f32> {
        let pg = self.as_petgraph();
        let result = petgraph::algo::betweenness_centrality(&pg);
        self.map_from_petgraph(result)
    }
}
```

## Conclusion

The HashSets in the design are not the graph model - they're just indices for O(1) lookups. The actual graph is the ECS entity relationships. Using petgraph as primary storage would recreate the synchronization problem we're solving.

However, using petgraph as a temporary computational view for complex algorithms is a valid pattern that maintains our single source of truth principle.
