# Alchemist Performance Optimization Guide

## Overview

This guide covers performance optimization techniques, benchmarking, and best practices for the Alchemist system.

## Performance Benchmarks

### Running Benchmarks

```bash
# Run all benchmarks
./scripts/run_benchmarks.sh

# Run benchmarks with integration tests
./scripts/run_benchmarks.sh --with-tests

# Run specific benchmark
cargo bench --bench alchemist_benchmarks -- event_creation

# Generate HTML reports
cargo bench
# Open target/criterion/report/index.html
```

### Benchmark Categories

1. **Event Creation & Serialization**
   - Dashboard update events
   - Dialog events
   - JSON serialization

2. **Policy Engine**
   - Single policy evaluation
   - Multiple policy evaluation
   - Cache hit performance

3. **Dashboard Operations**
   - Data creation
   - Metric updates
   - Event filtering

4. **Event Monitoring**
   - Event creation
   - Filtering large datasets
   - Tag-based queries

5. **String Operations**
   - Command parsing
   - Shell word splitting

6. **JSON Operations**
   - Small/medium/large payloads
   - Serialization/deserialization

## Performance Targets

Based on progress.json metrics:

| Operation | Target | Current | Status |
|-----------|--------|---------|--------|
| Event Creation | 100k/sec | 779k/sec | ✅ Exceeded |
| Event Publishing | 10k/sec | 1M/sec | ✅ Exceeded |
| Concurrent Ops | 100k/sec | 2.3M/sec | ✅ Exceeded |
| Query Response | <150ms | <10ms | ✅ Exceeded |
| Memory/Event | <10KB | 1.3KB | ✅ Exceeded |

## Optimization Techniques

### 1. Event System Optimization

#### Batch Processing
```rust
// Instead of sending individual events
for event in events {
    sender.send(event).await?;
}

// Use batch sending
sender.send_batch(events).await?;
```

#### Event Pooling
```rust
// Reuse event allocations
let mut event_pool = Vec::with_capacity(1000);
event_pool.push(EventBuilder::dashboard_update(data));
```

### 2. Policy Engine Optimization

#### Caching Strategy
- Policy evaluation results cached for 5 minutes
- Cache key includes claims + context hash
- LRU eviction for memory efficiency

```rust
// Configure cache size
let engine = PolicyEngine::new(policies, Duration::from_secs(300))
    .with_cache_size(10000);
```

#### Rule Ordering
- Place most restrictive conditions first
- Order policies by priority
- Short-circuit evaluation on first match

### 3. Dashboard Performance

#### Differential Updates
```rust
// Only send changed fields
let update = DashboardUpdate {
    total_events: Some(new_count),
    ..Default::default()
};
```

#### Event Windowing
```rust
// Limit recent events
const MAX_RECENT_EVENTS: usize = 100;
if dashboard.recent_events.len() > MAX_RECENT_EVENTS {
    dashboard.recent_events.drain(0..50); // Remove oldest half
}
```

### 4. Renderer Communication

#### Channel Sizing
```rust
// Size channels appropriately
let (tx, rx) = mpsc::channel(1000); // Buffer for burst traffic
```

#### Async Processing
```rust
// Non-blocking event handling
tokio::spawn(async move {
    while let Some(event) = rx.recv().await {
        process_event(event);
    }
});
```

### 5. Memory Optimization

#### String Interning
```rust
// Reuse common strings
lazy_static! {
    static ref DOMAIN_GRAPH: String = "graph".to_string();
}
```

#### Arena Allocation
```rust
// For temporary allocations
let arena = Arena::new();
let node = arena.alloc(GraphNode { ... });
```

## Profiling Tools

### CPU Profiling
```bash
# Using perf
cargo build --release
perf record -g target/release/alchemist
perf report

# Using flamegraph
cargo install flamegraph
cargo flamegraph --bench alchemist_benchmarks
```

### Memory Profiling
```bash
# Using heaptrack
heaptrack target/release/alchemist
heaptrack --analyze heaptrack.alchemist.12345.gz

# Using valgrind
valgrind --tool=massif target/release/alchemist
ms_print massif.out.12345
```

### Async Profiling
```rust
// Using tokio-console
#[tokio::main]
async fn main() {
    console_subscriber::init();
    // ... rest of app
}
```

## Best Practices

### 1. Avoid Blocking Operations
```rust
// Bad
let data = std::fs::read_to_string(path)?;

// Good
let data = tokio::fs::read_to_string(path).await?;
```

### 2. Minimize Allocations
```rust
// Bad
let items: Vec<_> = data.iter().map(|x| x.clone()).collect();

// Good
let items: Vec<_> = data.iter().cloned().collect();
```

### 3. Use Appropriate Data Structures
- `HashMap` for lookups
- `BTreeMap` for ordered iteration
- `Vec` for sequential access
- `VecDeque` for queues

### 4. Leverage Concurrency
```rust
// Process in parallel
let handles: Vec<_> = items
    .into_iter()
    .map(|item| tokio::spawn(process_item(item)))
    .collect();

let results = futures::future::join_all(handles).await;
```

## Monitoring Performance

### Metrics Collection
```rust
// Track operation timing
let start = Instant::now();
let result = expensive_operation().await?;
let duration = start.elapsed();
metrics.record_duration("operation_name", duration);
```

### Performance Dashboards
- Use Alchemist's built-in performance monitor
- Export metrics to Prometheus/Grafana
- Set up alerting for degradation

## Common Bottlenecks

### 1. Event Queue Overflow
**Symptom**: Dropped events, high memory usage
**Solution**: Increase channel capacity, add backpressure

### 2. Policy Evaluation Slowdown
**Symptom**: Increased latency over time
**Solution**: Clear policy cache, optimize rules

### 3. Renderer Lag
**Symptom**: UI unresponsive, delayed updates
**Solution**: Batch updates, reduce render frequency

### 4. Memory Leaks
**Symptom**: Growing memory usage
**Solution**: Check for circular references, unbounded collections

## Performance Testing

### Load Testing
```bash
# Generate load
cargo run --example performance_demo -- --events 10000 --duration 60
```

### Stress Testing
```bash
# Run stress tests
cargo test stress_test -- --ignored --nocapture
```

### Regression Testing
```bash
# Compare with baseline
cargo bench --bench alchemist_benchmarks -- --baseline master
```

## Optimization Checklist

- [ ] Enable release mode builds
- [ ] Configure appropriate channel sizes
- [ ] Implement event batching
- [ ] Enable policy caching
- [ ] Limit collection sizes
- [ ] Use async operations
- [ ] Profile before optimizing
- [ ] Measure after changes
- [ ] Document performance characteristics
- [ ] Set up continuous benchmarking

## Troubleshooting Performance Issues

### High CPU Usage
1. Check for busy loops
2. Profile with flamegraph
3. Look for inefficient algorithms
4. Verify async task scheduling

### High Memory Usage
1. Check for unbounded growth
2. Look for memory leaks
3. Verify cache sizes
4. Monitor allocations

### Network Latency
1. Check NATS configuration
2. Verify message sizes
3. Monitor network throughput
4. Consider compression

## Future Optimizations

1. **SIMD Operations**: For batch processing
2. **Zero-Copy Serialization**: Using rkyv or bincode
3. **GPU Acceleration**: For graph algorithms
4. **Distributed Processing**: Horizontal scaling
5. **Persistent Caching**: Redis/RocksDB integration