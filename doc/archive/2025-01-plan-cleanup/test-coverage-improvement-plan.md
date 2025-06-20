# Test Coverage Improvement Plan

## Overview

This plan addresses the critical test coverage gaps identified in our user story analysis. We must follow strict TDD principles: write failing tests first, then implement functionality.

## Current State
- **Total Tests**: 90 (77 passing, 13 failing)
- **Coverage**: ~65% (target: 95%)
- **User Stories**: 25 total (12 fully tested, 8 partial, 5 untested)

## Phase 1: Fix Failing Tests (Immediate - Day 1)

### 1.1 Domain Tests (3 failing)
```rust
// test_graph_tag_operations
// Issue: Tag operations not implemented
// Fix: Implement tag commands in GraphAggregate

// test_import_arrows_app
// Issue: Import handler missing
// Fix: Implement ArrowsAppImporter

// test_import_mermaid
// Issue: Import handler missing
// Fix: Implement MermaidImporter
```

### 1.2 Presentation Tests (7 failing)
```rust
// Force layout tests (3)
// Issue: Physics calculations incorrect
// Fix: Correct force calculations and damping

// Animation tests (2)
// Issue: Timer and progress tracking
// Fix: Proper time handling in tests

// Import processor test (1)
// Issue: Event forwarding logic
// Fix: Implement proper event routing
```

### 1.3 Integration Test (1 failing)
```rust
// test_no_conflict_with_proper_event_forwarding
// Issue: Event forwarding not implemented
// Fix: Complete event forwarding in processor
```

## Phase 2: Add Missing Handler Tests (Day 2-3)

### 2.1 Command Handler Coverage
```rust
// src/domain/commands/mod.rs - Add tests for ALL commands
#[cfg(test)]
mod handler_existence_tests {
    #[test]
    fn test_all_commands_have_handlers() {
        // Verify every Command variant has a handler
    }

    #[test]
    fn test_unknown_command_rejection() {
        // Test proper error for unknown commands
    }
}
```

### 2.2 Event Handler Coverage
```rust
// src/domain/events/mod.rs - Add tests for ALL events
#[cfg(test)]
mod event_handler_tests {
    #[test]
    fn test_all_events_have_handlers() {
        // Verify every Event variant can be handled
    }

    #[test]
    fn test_event_processing_failures() {
        // Test error handling in event processors
    }
}
```

## Phase 3: Implement Missing Features with TDD (Day 4-7)

### 3.1 Import Functionality
```rust
// Step 1: Write failing tests
#[test]
fn test_import_format_detection() {
    let mermaid = "graph TD\n  A --> B";
    assert_eq!(detect_format(mermaid), ImportFormat::Mermaid);
}

// Step 2: Implement minimal code to pass
pub fn detect_format(content: &str) -> ImportFormat {
    if content.starts_with("graph") {
        ImportFormat::Mermaid
    } else {
        ImportFormat::Unknown
    }
}
```

### 3.2 Query Handlers
```rust
// Write query handler tests first
#[test]
fn test_graph_summary_query() {
    let query = GraphSummaryQuery { graph_id };
    let result = handle_query(query).await?;
    assert_eq!(result.node_count, 5);
    assert_eq!(result.edge_count, 4);
}

// Then implement handlers
pub async fn handle_graph_summary_query(
    query: GraphSummaryQuery,
    projection: &GraphProjection,
) -> Result<GraphSummary> {
    // Implementation
}
```

### 3.3 Concurrent Command Processing
```rust
#[tokio::test]
async fn test_concurrent_command_processing() {
    let store = create_test_store().await;

    // Send 10 commands concurrently
    let handles: Vec<_> = (0..10).map(|i| {
        let store = store.clone();
        tokio::spawn(async move {
            store.process_command(create_node_command(i)).await
        })
    }).collect();

    // All should succeed
    for handle in handles {
        assert!(handle.await?.is_ok());
    }
}
```

## Phase 4: Performance and Load Tests (Day 8-10)

### 4.1 Benchmark Tests
```rust
// benches/graph_performance.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_large_graph(c: &mut Criterion) {
    c.bench_function("create_10k_nodes", |b| {
        b.iter(|| create_nodes(10_000))
    });
}
```

### 4.2 Load Tests
```rust
#[test]
fn test_handle_10k_nodes_100k_edges() {
    let mut graph = GraphAggregate::new(GraphId::new());

    // Add 10k nodes
    for i in 0..10_000 {
        let cmd = AddNodeCommand { /* ... */ };
        graph.handle_command(cmd).unwrap();
    }

    // Verify performance metrics
    assert!(graph.nodes.len() == 10_000);
}
```

## Phase 5: Visual and Interaction Tests (Day 11-12)

### 5.1 Deterministic Animation Tests
```rust
#[test]
fn test_animation_determinism() {
    let mut app = create_test_app();

    // Run animation for exact time
    app.world.resource_mut::<Time>()
        .advance_by(Duration::from_secs(1));

    // Check exact position
    let transform = app.world.query::<&Transform>()
        .single(&app.world);
    assert_eq!(transform.translation.x, 5.0); // Exact value
}
```

### 5.2 Interaction Tests
```rust
#[test]
fn test_mouse_picking() {
    let mut app = create_test_app();

    // Simulate mouse click
    app.world.send_event(MouseClick {
        position: Vec2::new(100.0, 100.0),
    });

    app.update();

    // Verify selection
    let selected = app.world.query::<&Selected>().iter(&app.world).count();
    assert_eq!(selected, 1);
}
```

## Implementation Guidelines

### TDD Workflow
1. **Red**: Write failing test first
2. **Green**: Write minimal code to pass
3. **Refactor**: Clean up while keeping tests green

### Test Organization
```
tests/
├── unit/           # Pure domain logic
├── integration/    # Cross-boundary tests
├── performance/    # Benchmarks and load tests
└── e2e/           # Full system tests
```

### Coverage Tracking
```bash
# Run with coverage
cargo tarpaulin --out Html --output-dir coverage

# Check coverage meets 95%
cargo tarpaulin --print-summary --fail-under 95
```

## Success Criteria

1. **All tests passing** (0 failures)
2. **95% test coverage** achieved
3. **All user stories tested** (25/25)
4. **All commands/events have handlers**
5. **Performance benchmarks established**

## Timeline

- **Day 1**: Fix all 13 failing tests
- **Day 2-3**: Add handler existence tests
- **Day 4-7**: Implement missing features (TDD)
- **Day 8-10**: Add performance tests
- **Day 11-12**: Add visual/interaction tests
- **Day 13**: Final coverage verification

## Risks and Mitigations

1. **Risk**: Fixing tests reveals deeper issues
   - **Mitigation**: Time-box fixes, document blockers

2. **Risk**: Performance tests fail on CI
   - **Mitigation**: Set environment-specific thresholds

3. **Risk**: Visual tests are flaky
   - **Mitigation**: Use deterministic time, fixed seeds

## Next Steps

1. Start with Phase 1 immediately
2. Run `cargo test` after each fix
3. Update progress.json after each phase
4. Document any architectural changes needed
