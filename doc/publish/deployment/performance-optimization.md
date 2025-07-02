# CIM Performance Optimization Guide

This guide provides comprehensive performance optimization strategies for CIM deployments.

## Performance Goals

- **Event Processing**: < 10ms p99 latency
- **Query Response**: < 100ms p99 for complex queries
- **Throughput**: 10,000+ events/second per node
- **Memory Efficiency**: < 4GB for typical workloads
- **Startup Time**: < 30 seconds to ready state

## Event Processing Optimization

### 1. Batch Processing

Configure event batching for optimal throughput:

```toml
# production.toml
[event_processing]
batch_size = 100
batch_timeout = "10ms"
max_concurrent_batches = 10

# Adaptive batching
enable_adaptive_batching = true
min_batch_size = 10
max_batch_size = 500
target_latency = "5ms"
```

### 2. Event Handler Optimization

```rust
// Use pre-allocated buffers
pub struct OptimizedEventHandler {
    event_buffer: Vec<DomainEvent>,
    command_buffer: Vec<Command>,
    reusable_allocations: AllocationPool,
}

impl OptimizedEventHandler {
    pub async fn handle_batch(&mut self, events: Vec<Event>) -> Result<()> {
        // Clear and reuse buffers
        self.event_buffer.clear();
        self.command_buffer.clear();

        // Process events in batch
        for event in events {
            // Avoid allocations in hot path
            match self.process_event_no_alloc(&event) {
                Ok(commands) => self.command_buffer.extend(commands),
                Err(e) => self.handle_error_no_alloc(e),
            }
        }

        // Batch publish results
        self.publish_commands_batch(&self.command_buffer).await?;
        Ok(())
    }
}
```

### 3. Memory Pool Usage

```rust
// Object pooling for frequent allocations
use crossbeam::queue::ArrayQueue;

pub struct EventPool {
    pool: ArrayQueue<Box<DomainEvent>>,
}

impl EventPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            pool: ArrayQueue::new(capacity),
        }
    }

    pub fn acquire(&self) -> Box<DomainEvent> {
        self.pool.pop().unwrap_or_else(|| Box::new(DomainEvent::default()))
    }

    pub fn release(&self, mut event: Box<DomainEvent>) {
        event.clear(); // Reset to default state
        let _ = self.pool.push(event); // Ignore if full
    }
}
```

## Query Optimization

### 1. Projection Design

Design projections for efficient queries:

```rust
// Denormalized projection for fast queries
pub struct GraphSummaryProjection {
    // Primary index
    graphs: HashMap<GraphId, GraphSummary>,
    
    // Secondary indices for common queries
    by_name: BTreeMap<String, GraphId>,
    by_creation_date: BTreeMap<DateTime<Utc>, Vec<GraphId>>,
    by_node_count: BTreeMap<usize, Vec<GraphId>>,
    
    // Pre-computed aggregates
    total_graphs: usize,
    total_nodes: usize,
    total_edges: usize,
}

impl GraphSummaryProjection {
    // O(1) lookup by ID
    pub fn get_by_id(&self, id: &GraphId) -> Option<&GraphSummary> {
        self.graphs.get(id)
    }
    
    // O(log n) range queries
    pub fn get_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&GraphSummary> {
        self.by_creation_date
            .range(start..=end)
            .flat_map(|(_, ids)| ids.iter())
            .filter_map(|id| self.graphs.get(id))
            .collect()
    }
}
```

### 2. Query Caching

Implement intelligent caching:

```toml
# production.toml
[query_cache]
enabled = true
max_entries = 10000
ttl_seconds = 300
cache_key_prefix = "cim:query:"

# Cache warming
warm_cache_on_startup = true
warm_cache_queries = [
    "get_all_graphs",
    "get_recent_nodes",
]
```

```rust
use moka::sync::Cache;

pub struct QueryCache {
    cache: Cache<String, Arc<QueryResult>>,
}

impl QueryCache {
    pub fn new(max_capacity: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(300))
            .build();
        
        Self { cache }
    }

    pub async fn get_or_compute<F, Fut>(&self, key: &str, compute: F) -> Arc<QueryResult>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = QueryResult>,
    {
        if let Some(cached) = self.cache.get(key) {
            return cached;
        }

        let result = Arc::new(compute().await);
        self.cache.insert(key.to_string(), result.clone());
        result
    }
}
```

### 3. Spatial Index Optimization

For spatial queries:

```rust
use rstar::{RTree, AABB};

pub struct SpatialIndex {
    rtree: RTree<SpatialNode>,
    grid_index: HashMap<GridCell, Vec<NodeId>>,
    lod_cache: HashMap<(BoundingBox, DetailLevel), Vec<NodeId>>,
}

impl SpatialIndex {
    pub fn find_in_radius(&self, center: Point3D, radius: f32) -> Vec<NodeId> {
        // Use grid for coarse filtering
        let grid_cells = self.get_affected_grid_cells(center, radius);
        let candidates: HashSet<NodeId> = grid_cells
            .iter()
            .flat_map(|cell| self.grid_index.get(cell))
            .flatten()
            .copied()
            .collect();

        // Fine filtering with R-tree
        let envelope = AABB::from_corners(
            [center.x - radius, center.y - radius, center.z - radius],
            [center.x + radius, center.y + radius, center.z + radius],
        );

        self.rtree
            .locate_in_envelope(&envelope)
            .filter(|node| node.distance_to(center) <= radius)
            .filter(|node| candidates.contains(&node.id))
            .map(|node| node.id)
            .collect()
    }
}
```

## NATS Optimization

### 1. Connection Pooling

```toml
# production.toml
[nats]
# Connection pool settings
min_connections = 5
max_connections = 20
connection_timeout = "5s"
idle_timeout = "300s"

# Performance tuning
write_buffer_size = "64KB"
read_buffer_size = "64KB"
pending_size = "8MB"
```

### 2. Subject Design

Optimize subject hierarchy:

```rust
// Good: Hierarchical subjects for efficient routing
const SUBJECTS: &[&str] = &[
    "event.graph.node.created",
    "event.graph.node.updated",
    "event.graph.edge.created",
    "cmd.graph.create_node",
    "query.graph.find_nodes",
];

// Bad: Flat subjects that don't leverage NATS routing
const BAD_SUBJECTS: &[&str] = &[
    "graph_node_created",
    "graph_node_updated",
    "create_node_command",
];
```

### 3. JetStream Configuration

```yaml
# Optimized JetStream configuration
jetstream:
  store_dir: "/fast-ssd/nats/jetstream"
  max_memory_store: 8GB
  max_file_store: 1TB
  
  # Performance settings
  sync_interval: "30s"  # Less frequent syncs
  max_outstanding_catchup: 128MB
  
  # Compression for storage efficiency
  compression: "s2"  # Fast compression
```

## Memory Optimization

### 1. Component Deduplication

```rust
pub struct ComponentStorage {
    // Intern common components
    interned_strings: StringInterner,
    component_pool: HashMap<ComponentHash, Arc<Component>>,
    reference_counts: HashMap<ComponentHash, usize>,
}

impl ComponentStorage {
    pub fn intern_component(&mut self, component: Component) -> ComponentRef {
        let hash = calculate_hash(&component);
        
        let arc = self.component_pool
            .entry(hash)
            .or_insert_with(|| Arc::new(component));
        
        *self.reference_counts.entry(hash).or_insert(0) += 1;
        
        ComponentRef {
            hash,
            data: arc.clone(),
        }
    }
}
```

### 2. Memory-Mapped Projections

For large read-only data:

```rust
use memmap2::MmapOptions;

pub struct MmapProjection {
    mmap: Mmap,
    index: HashMap<GraphId, (usize, usize)>, // offset, length
}

impl MmapProjection {
    pub fn load(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        
        // Load index from file header
        let index = Self::load_index(&mmap)?;
        
        Ok(Self { mmap, index })
    }
    
    pub fn get(&self, id: &GraphId) -> Option<GraphData> {
        let (offset, length) = self.index.get(id)?;
        let data = &self.mmap[*offset..*offset + *length];
        
        // Zero-copy deserialization
        bincode::deserialize(data).ok()
    }
}
```

### 3. Arena Allocation

For temporary allocations:

```rust
use bumpalo::Bump;

pub struct EventProcessingArena {
    arena: Bump,
}

impl EventProcessingArena {
    pub fn process_events<'a>(&'a self, events: &[Event]) -> Vec<&'a ProcessedEvent> {
        let mut results = Vec::new();
        
        for event in events {
            // Allocate in arena - no individual frees needed
            let processed = self.arena.alloc(ProcessedEvent {
                id: event.id,
                timestamp: event.timestamp,
                // ... process event
            });
            
            results.push(processed);
        }
        
        results
    }
    
    pub fn reset(&mut self) {
        self.arena.reset(); // Free all allocations at once
    }
}
```

## Concurrency Optimization

### 1. Lock-Free Data Structures

```rust
use crossbeam::queue::SegQueue;
use dashmap::DashMap;

pub struct LockFreeEventStore {
    // Lock-free queue for event ingestion
    event_queue: SegQueue<DomainEvent>,
    
    // Concurrent hashmap for projections
    projections: DashMap<ProjectionId, Projection>,
    
    // Atomic counters
    event_count: AtomicU64,
    error_count: AtomicU64,
}
```

### 2. Work Stealing

```rust
use rayon::prelude::*;

pub fn process_graph_batch(graphs: Vec<Graph>) -> Vec<ProcessedGraph> {
    graphs
        .into_par_iter()
        .map(|graph| {
            // CPU-intensive processing
            process_single_graph(graph)
        })
        .collect()
}

// Configure thread pool
rayon::ThreadPoolBuilder::new()
    .num_threads(num_cpus::get())
    .thread_name(|i| format!("cim-worker-{}", i))
    .build_global()
    .unwrap();
```

### 3. Async Runtime Tuning

```toml
# production.toml
[tokio_runtime]
worker_threads = 8
max_blocking_threads = 128
thread_stack_size = "2MB"
thread_keep_alive = "10s"
```

## Profiling and Benchmarking

### 1. Continuous Profiling

```rust
#[cfg(feature = "profiling")]
pub fn setup_continuous_profiling() {
    use pprof::ProfilerGuard;
    
    thread::spawn(|| {
        let mut guard: Option<ProfilerGuard> = None;
        
        loop {
            // Profile for 30 seconds every 5 minutes
            thread::sleep(Duration::from_secs(300));
            
            guard = Some(ProfilerGuard::new(100).unwrap());
            thread::sleep(Duration::from_secs(30));
            
            if let Some(g) = guard.take() {
                if let Ok(report) = g.report().build() {
                    let file = File::create(format!("profile-{}.pb", timestamp())).unwrap();
                    report.pprof().unwrap().write_to(&file).unwrap();
                }
            }
        }
    });
}
```

### 2. Micro-benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_event_processing(c: &mut Criterion) {
    let events = generate_test_events(1000);
    
    c.bench_function("process_events", |b| {
        b.iter(|| {
            process_event_batch(black_box(&events))
        })
    });
}

fn benchmark_spatial_query(c: &mut Criterion) {
    let index = create_test_spatial_index(10000);
    
    c.bench_function("spatial_radius_query", |b| {
        b.iter(|| {
            index.find_in_radius(
                black_box(Point3D::new(50.0, 50.0, 50.0)),
                black_box(10.0),
            )
        })
    });
}

criterion_group!(benches, benchmark_event_processing, benchmark_spatial_query);
criterion_main!(benches);
```

## Production Tuning Checklist

### System Level
- [ ] Disable CPU frequency scaling
- [ ] Set CPU governor to performance
- [ ] Configure huge pages for large allocations
- [ ] Tune network stack for low latency
- [ ] Disable swap or set swappiness to 0

### Application Level
- [ ] Enable release optimizations with LTO
- [ ] Profile-guided optimization build
- [ ] Strip debug symbols for production
- [ ] Enable jemalloc or mimalloc
- [ ] Configure appropriate log levels

### Monitoring
- [ ] Set up continuous profiling
- [ ] Monitor allocation rates
- [ ] Track query latencies
- [ ] Watch for lock contention
- [ ] Monitor GC pressure (if applicable)

## Performance Testing

### Load Testing Script

```bash
#!/bin/bash
# load-test.sh

# Configuration
DURATION="300s"
RATE="1000"
CONNECTIONS="100"

# Run load test
echo "Starting load test..."
vegeta attack \
  -duration=$DURATION \
  -rate=$RATE \
  -connections=$CONNECTIONS \
  -targets=targets.txt \
  -output=results.bin

# Generate report
vegeta report -type=text results.bin
vegeta plot results.bin > plot.html

# Check for performance regression
P99=$(vegeta report -type=json results.bin | jq .latencies.99th)
if (( $(echo "$P99 > 100000000" | bc -l) )); then
  echo "Performance regression detected! p99: ${P99}ns"
  exit 1
fi
```

---

*Last updated: January 2025*
*Version: 0.4.2* 