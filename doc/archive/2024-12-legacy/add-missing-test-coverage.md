# Plan: Add Missing Test Coverage

## Overview

This plan addresses the critical gap between claimed functionality and actual implementation by adding comprehensive test coverage that will expose missing features.

## Objectives

1. Write failing tests for all claimed features
2. Establish proper integration test infrastructure
3. Add performance benchmarks
4. Create end-to-end workflow tests
5. Implement proper TDD practices going forward

## Phase 1: Feature Existence Tests (Week 1)

### 1.1 Create Feature Test Module
```rust
// src/testing/feature_tests.rs
mod feature_tests {
    mod visualization {
        #[test]
        fn test_2d_mode_exists() { /* WILL FAIL */ }

        #[test]
        fn test_3d_to_2d_switching() { /* WILL FAIL */ }

        #[test]
        fn test_2d_overview_rendering() { /* WILL FAIL */ }
    }

    mod subgraph_composition {
        #[test]
        fn test_load_multiple_graphs() { /* WILL FAIL */ }

        #[test]
        fn test_maintain_subgraph_structure() { /* WILL FAIL */ }

        #[test]
        fn test_compose_graphs() { /* WILL FAIL */ }
    }

    mod collaboration {
        #[test]
        fn test_multi_user_connection() { /* WILL FAIL */ }

        #[test]
        fn test_real_time_sync() { /* WILL FAIL */ }

        #[test]
        fn test_conflict_resolution() { /* WILL FAIL */ }
    }

    mod ai_integration {
        #[test]
        fn test_ai_agent_exists() { /* WILL FAIL */ }

        #[test]
        fn test_pattern_recognition() { /* WILL FAIL */ }

        #[test]
        fn test_optimization_suggestions() { /* WILL FAIL */ }
    }

    mod plugin_system {
        #[test]
        fn test_wasm_plugin_loading() { /* WILL FAIL */ }

        #[test]
        fn test_custom_algorithm_plugin() { /* WILL FAIL */ }

        #[test]
        fn test_visualization_plugin() { /* WILL FAIL */ }
    }
}
```

### 1.2 Expected Results
- All tests will FAIL
- This documents the gap between claims and reality
- Provides clear targets for implementation

## Phase 2: Integration Tests (Week 1-2)

### 2.1 End-to-End Workflow Tests
```rust
// src/testing/integration_tests.rs
#[test]
fn test_complete_graph_editing_workflow() {
    // 1. Start application
    // 2. Create new graph
    // 3. Add nodes interactively
    // 4. Add edges interactively
    // 5. Apply layout
    // 6. Save graph
    // 7. Close and restart
    // 8. Load saved graph
    // 9. Verify identical state
}

#[test]
fn test_import_edit_export_cycle() {
    // 1. Import graph from file
    // 2. Modify graph (add/remove nodes)
    // 3. Export to new file
    // 4. Import the exported file
    // 5. Verify modifications preserved
}
```

### 2.2 File I/O Round-Trip Tests
```rust
#[test]
fn test_json_round_trip_preserves_all_data() {
    // Create complex graph with all features
    // Export to JSON
    // Import from JSON
    // Deep equality check
}

#[test]
fn test_import_from_user_selected_file() {
    // Currently hardcoded to CIM.json
    // Should test file dialog integration
}
```

## Phase 3: Performance Tests (Week 2)

### 3.1 Benchmark Infrastructure
```rust
// benches/performance.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_large_graph_rendering(c: &mut Criterion) {
    c.bench_function("render_10k_nodes", |b| {
        b.iter(|| {
            // Test with 10,000 nodes
        });
    });

    c.bench_function("render_100k_nodes", |b| {
        b.iter(|| {
            // Test with 100,000 nodes
        });
    });

    c.bench_function("render_250k_nodes_60fps", |b| {
        b.iter(|| {
            // Test claimed 250k+ at 60 FPS
        });
    });
}
```

### 3.2 Memory Usage Tests
```rust
#[test]
fn test_memory_usage_large_graphs() {
    // Monitor memory with increasing graph sizes
    // Verify no memory leaks
    // Check performance degradation curve
}
```

## Phase 4: User Interaction Tests (Week 2-3)

### 4.1 Graph Editing Tests
```rust
#[test]
fn test_add_node_interactively() {
    // Right-click to add node (if implemented)
    // Verify node appears at cursor position
}

#[test]
fn test_delete_selected_nodes() {
    // Select nodes
    // Press delete
    // Verify removal
}

#[test]
fn test_edge_creation_by_dragging() {
    // Drag from one node to another
    // Verify edge created
}
```

### 4.2 UI Responsiveness Tests
```rust
#[test]
fn test_ui_remains_responsive_during_layout() {
    // Apply force-directed layout
    // Verify UI doesn't freeze
    // Check frame rate maintained
}
```

## Phase 5: Event System Tests (Week 3)

### 5.1 Event Audit Trail
```rust
#[test]
fn test_all_changes_create_events() {
    // Perform various operations
    // Verify event generated for each
    // Check event contains sufficient data for replay
}

#[test]
fn test_event_replay_recreates_state() {
    // Capture event stream
    // Reset to initial state
    // Replay events
    // Verify identical final state
}
```

## Implementation Strategy

### Week 1: Setup and Feature Tests
1. Create test infrastructure
2. Write all failing feature tests
3. Document current vs expected behavior
4. Update CI to run new tests (expecting failures)

### Week 2: Integration and Performance
1. Add integration test framework
2. Create performance benchmarks
3. Write file I/O tests
4. Add memory profiling

### Week 3: Interaction and Events
1. Add UI interaction tests
2. Create event system tests
3. Write audit trail tests
4. Complete test documentation

## Success Criteria

1. **Transparency**: All missing features have failing tests
2. **Coverage**: Every claimed feature has at least one test
3. **Performance**: Benchmarks establish baseline and targets
4. **Integration**: End-to-end workflows are tested
5. **TDD Ready**: Infrastructure supports test-first development

## Expected Outcomes

- 50+ new failing tests documenting missing features
- Clear roadmap of what needs implementation
- Performance baselines established
- Integration test framework ready
- Honest picture of project status

## Next Steps After This Plan

1. Update README to reflect actual functionality
2. Create implementation plan for missing features
3. Prioritize features based on user needs
4. Begin TDD implementation of highest priority features
