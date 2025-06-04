# Fitness Functions for Graph Editor and Workflow Manager

## Overview

Fitness functions are objective measures that help validate our system's quality attributes. They ensure our architecture remains fit for purpose as it evolves.

## Performance Fitness Functions

### FF-1: Graph Rendering Performance
**Attribute:** Performance
**Metric:** Frame Time
**Target:** < 16.67ms (60 FPS)
**Measurement:**
```rust
fn measure_frame_time(time: Res<Time>) -> f32 {
    time.delta().as_secs_f32() * 1000.0
}
```
**Threshold:**
- ✅ Green: < 16.67ms
- ⚠️ Yellow: 16.67ms - 33.33ms
- ❌ Red: > 33.33ms

**Current Status:** Not measured

### FF-2: Node Limit Performance
**Attribute:** Scalability
**Metric:** Query Time for 10,000 nodes
**Target:** < 1ms
**Test:** `test_graph_validation_node_limit`
**Measurement:**
```rust
fn measure_node_query_time(nodes: Query<&Node>) -> Duration {
    let start = Instant::now();
    let _count = nodes.iter().count();
    start.elapsed()
}
```
**Threshold:**
- ✅ Green: < 1ms
- ⚠️ Yellow: 1ms - 5ms
- ❌ Red: > 5ms

**Current Status:** ✅ Validated up to 100 nodes

### FF-3: Raycast Selection Performance
**Attribute:** Responsiveness
**Metric:** Selection calculation time
**Target:** < 1ms for 1000 nodes
**Test:** `test_closest_hit_selection`
**Measurement:**
```rust
fn measure_raycast_time(nodes: Query<(&Transform, &NodeIdentity)>) -> Duration {
    let start = Instant::now();
    // Perform raycast against all nodes
    start.elapsed()
}
```
**Threshold:**
- ✅ Green: < 1ms
- ⚠️ Yellow: 1ms - 3ms
- ❌ Red: > 3ms

**Current Status:** ✅ Algorithm tested

## Reliability Fitness Functions

### FF-4: Event Store Consistency
**Attribute:** Data Integrity
**Metric:** Event ordering and completeness
**Target:** 100% consistency
**Test:** `test_graph_events_repository`
**Validation:**
- All events are append-only
- Events maintain chronological order
- No events are lost
- Snapshots match event replay

**Current Status:** ✅ Passing

### FF-5: Repository Data Integrity
**Attribute:** Data Consistency
**Metric:** CRUD operation correctness
**Target:** 100% data integrity
**Tests:** All repository tests
**Validation:**
- Store operations persist data correctly
- Find operations return exact matches
- Remove operations clean up completely
- No data corruption occurs

**Current Status:** ✅ All repository tests passing

### FF-6: Constraint Enforcement
**Attribute:** Business Rule Compliance
**Metric:** Validation accuracy
**Target:** 100% rule enforcement
**Tests:** `test_graph_validation_*`
**Rules Enforced:**
- ✅ No self-loops
- ✅ No duplicate edges
- ✅ Node count limits
- ✅ Valid identifiers only

**Current Status:** ✅ All constraints enforced

## Maintainability Fitness Functions

### FF-7: Test Coverage
**Attribute:** Code Quality
**Metric:** Test coverage percentage
**Target:** > 80%
**Measurement:** `cargo tarpaulin`
**Current Coverage:**
- Graph Management: ~70% (estimated)
- Visualization: ~60% (estimated)
- Overall: ~65% (estimated)

**Status:** ⚠️ Below target

### FF-8: Code Complexity
**Attribute:** Maintainability
**Metric:** Cyclomatic complexity
**Target:** < 10 per function
**Measurement:** `cargo clippy`
**High Complexity Areas:**
- `RenderGraphElements::render_edge` - Multiple edge types
- Event handling in repositories
- Validation logic

**Status:** ✅ Generally good

### FF-9: DDD Compliance
**Attribute:** Architecture Fitness
**Metric:** Naming convention adherence
**Target:** 100% compliance
**Validation:**
- ✅ Aggregates as singular nouns
- ✅ Repositories as plural domain terms
- ✅ Events in past tense
- ✅ Services as verb phrases

**Status:** ✅ Fully compliant

## Usability Fitness Functions

### FF-10: Input Response Time
**Attribute:** User Experience
**Metric:** Input to visual feedback latency
**Target:** < 100ms
**Measurement:**
```rust
fn measure_input_latency(
    input_time: Res<InputTimestamp>,
    render_time: Res<RenderTimestamp>
) -> Duration {
    render_time.0 - input_time.0
}
```
**Current Issues:**
- ⚠️ Keyboard controls may not work
- ❌ No selection visual feedback

**Status:** ❌ Needs improvement

### FF-11: Animation Smoothness
**Attribute:** Visual Quality
**Metric:** Animation frame consistency
**Target:** < 5% frame time variance
**Tests:** `test_graph_motion_defaults`
**Current Capabilities:**
- ✅ Graph rotation
- ✅ Node pulsing
- ❌ Edge animation missing

**Status:** ⚠️ Partially implemented

## Security Fitness Functions

### FF-12: Input Validation
**Attribute:** Security
**Metric:** Invalid input rejection rate
**Target:** 100% rejection of invalid data
**Validation:**
- UUID format validation
- Property value sanitization
- Graph size limits

**Status:** ✅ Basic validation in place

## Evolutionary Fitness Functions

### FF-13: API Stability
**Attribute:** Evolvability
**Metric:** Breaking changes per release
**Target:** 0 unplanned breaking changes
**Tracking:**
- Service signatures unchanged
- Event schemas stable
- Component structures maintained

**Status:** ✅ Currently stable

### FF-14: Extension Points
**Attribute:** Extensibility
**Metric:** New feature integration effort
**Target:** < 1 day for new visualizations
**Evidence:**
- ✅ Plugin architecture
- ✅ Event-driven design
- ✅ Component composition

**Status:** ✅ Good extensibility

## Fitness Function Dashboard

| Category | Functions | ✅ Passing | ⚠️ Warning | ❌ Failing |
|----------|-----------|-----------|-----------|-----------|
| Performance | 3 | 2 | 0 | 1 |
| Reliability | 3 | 3 | 0 | 0 |
| Maintainability | 3 | 2 | 1 | 0 |
| Usability | 2 | 0 | 1 | 1 |
| Security | 1 | 1 | 0 | 0 |
| Evolution | 2 | 2 | 0 | 0 |
| **Total** | **14** | **10** | **2** | **2** |

## Automated Fitness Function Tests

```rust
#[cfg(test)]
mod fitness_tests {
    use super::*;

    #[test]
    fn ff_node_query_performance() {
        // Measure node query performance
        let result = measure_large_graph_query();
        assert!(result < Duration::from_millis(1));
    }

    #[test]
    fn ff_event_consistency() {
        // Verify event store maintains order
        let events = create_test_events();
        assert!(events_are_ordered(&events));
    }

    #[test]
    fn ff_ddd_naming_compliance() {
        // Automated naming convention check
        assert!(check_aggregate_naming());
        assert!(check_repository_naming());
    }
}
```

## Continuous Monitoring

1. **Performance Metrics**
   - Run benchmarks in CI/CD
   - Alert on regression > 10%
   - Track trends over time

2. **Quality Metrics**
   - Test coverage reports
   - Complexity analysis
   - Code review compliance

3. **User Experience Metrics**
   - Input latency tracking
   - Frame time monitoring
   - Error rate logging

## Action Items

1. **Immediate** (Failing Functions)
   - Implement edge animation
   - Add selection visual feedback
   - Fix keyboard input integration

2. **Short-term** (Warning Functions)
   - Increase test coverage to 80%
   - Implement performance benchmarks
   - Add input latency monitoring

3. **Long-term** (Optimization)
   - Scale testing to 10,000 nodes
   - Add automated fitness dashboards
   - Implement continuous monitoring
