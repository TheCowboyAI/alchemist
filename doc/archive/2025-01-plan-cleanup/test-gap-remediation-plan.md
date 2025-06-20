# Test Gap Remediation Plan

## Overview

This plan addresses the critical test gaps and missing functionality identified in the QA report. The plan follows TDD principles - we will write failing tests first, then implement the functionality to make them pass.

## Immediate Actions (Today)

### 1. Fix Currently Failing Tests

#### Import Command Handler Tests
**Files**: `/src/application/command_handlers/graph_import_handler.rs`
```rust
// Fix: Implement proper import command handling
pub fn handle_graph_command(cmd: &GraphCommand) -> Option<DomainEvent> {
    match cmd {
        GraphCommand::ImportGraph { graph_id, source, format, options } => {
            // Generate GraphImportRequested event instead of returning None
            Some(DomainEvent::Graph(GraphEvent::GraphImportRequested {
                graph_id: *graph_id,
                source: source.clone(),
                format: format.clone(),
                options: options.clone(),
                timestamp: SystemTime::now(),
            }))
        }
        // ... other cases
    }
}
```

#### Graph Aggregate Tests
**Files**: `/src/domain/aggregates/graph.rs`
- Fix metadata update to generate correct number of events
- Fix tag operations to handle events properly

#### Import Service Tests
**Files**: `/src/domain/services/graph_import.rs`
- Implement ArrowsApp format parser
- Implement Mermaid format parser

### 2. Create Missing Test Files

#### Integration Test Structure
```bash
tests/integration/
├── import_pipeline_tests.rs
├── query_handler_tests.rs
├── projection_sync_tests.rs
├── external_system_tests.rs
└── performance_tests.rs
```

## Week 1: Critical Business Logic Tests

### Day 1-2: Import Pipeline Tests

**File**: `/tests/integration/import_pipeline_tests.rs`
```rust
use ia::prelude::*;

#[test]
fn test_import_event_handler_processes_graph_import_requested() {
    // Given: A GraphImportRequested event
    let event = GraphEvent::GraphImportRequested {
        graph_id: GraphId::new(),
        source: ImportSource::File { path: "test.json".to_string() },
        format: "arrows_app".to_string(),
        options: ImportOptions::default(),
        timestamp: SystemTime::now(),
    };

    // When: The import handler processes it
    let result = process_import_event(event);

    // Then: It should generate NodeAdded and EdgeConnected events
    assert!(result.is_ok());
    let events = result.unwrap();
    assert!(events.iter().any(|e| matches!(e, DomainEvent::Graph(GraphEvent::NodeAdded { .. }))));
    assert!(events.iter().any(|e| matches!(e, DomainEvent::Graph(GraphEvent::EdgeConnected { .. }))));
}

#[test]
fn test_import_creates_entities_in_ecs() {
    // Test that import events create actual Bevy entities
    let mut app = create_test_app();

    // Send import command
    app.world.send_event(ImportGraphCommand { /* ... */ });

    // Process events
    app.update();

    // Verify entities were created
    let nodes = app.world.query::<&NodeEntity>().iter(&app.world).count();
    assert!(nodes > 0, "Import should create node entities");
}

#[test]
fn test_import_handles_invalid_data() {
    // Test graceful handling of malformed import data
    let invalid_json = r#"{ "invalid": "data" }"#;
    let result = import_from_json(invalid_json);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ImportError::InvalidFormat(_)));
}
```

### Day 3-4: Query Handler Tests

**File**: `/tests/integration/query_handler_tests.rs`
```rust
#[test]
fn test_find_nodes_by_type_query() {
    // Setup: Create graph with different node types
    let mut projection = NodeListProjection::new();
    projection.handle_event(NodeAdded {
        node_id: NodeId::new(),
        node_type: NodeType::Concept,
        // ...
    });

    // Execute query
    let query = FindNodesByType { node_type: NodeType::Concept };
    let result = handle_query(query, &projection);

    // Verify results
    assert_eq!(result.nodes.len(), 1);
    assert_eq!(result.nodes[0].node_type, NodeType::Concept);
}

#[test]
fn test_graph_traversal_queries() {
    // Test finding connected nodes
    let query = FindConnectedNodes {
        start_node: NodeId::new(),
        max_depth: 2,
    };
    let result = handle_query(query, &projection);
    assert!(result.is_ok());
}
```

### Day 5: Event Replay Tests

**File**: `/tests/integration/event_replay_tests.rs`
```rust
#[test]
fn test_event_replay_from_snapshot() {
    // Create snapshot at specific point
    let snapshot = create_snapshot_at_event(50);

    // Replay from snapshot
    let aggregate = replay_from_snapshot(snapshot, events[50..].to_vec());

    // Verify state matches full replay
    let full_replay = replay_all_events(events);
    assert_eq!(aggregate, full_replay);
}

#[test]
fn test_concurrent_replay_safety() {
    // Test that concurrent replays don't interfere
    let handles: Vec<_> = (0..10).map(|_| {
        thread::spawn(|| {
            replay_events(test_events())
        })
    }).collect();

    // All replays should succeed
    for handle in handles {
        assert!(handle.join().unwrap().is_ok());
    }
}
```

## Week 2: Integration and Performance Tests

### Day 1-2: End-to-End Tests

**File**: `/tests/integration/end_to_end_tests.rs`
```rust
#[test]
fn test_complete_import_to_query_flow() {
    // 1. Import a graph
    // 2. Process events through NATS
    // 3. Update projections
    // 4. Query the results
    // 5. Verify complete flow works
}

#[test]
fn test_concurrent_operations() {
    // Test multiple users modifying graph simultaneously
}
```

### Day 3-4: Performance Benchmarks

**File**: `/benches/graph_benchmarks.rs`
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_large_graph_operations(c: &mut Criterion) {
    c.bench_function("add_10k_nodes", |b| {
        b.iter(|| {
            let mut graph = Graph::new();
            for i in 0..10_000 {
                graph.add_node(black_box(create_test_node(i)));
            }
        });
    });
}

fn bench_event_processing_throughput(c: &mut Criterion) {
    c.bench_function("process_1k_events", |b| {
        let events = create_test_events(1000);
        b.iter(|| {
            process_events(black_box(&events))
        });
    });
}

criterion_group!(benches, bench_large_graph_operations, bench_event_processing_throughput);
criterion_main!(benches);
```

### Day 5: External System Tests

**File**: `/tests/integration/external_system_tests.rs`
```rust
#[test]
#[ignore] // Requires external services
fn test_neo4j_bidirectional_sync() {
    // Setup Neo4j test instance
    let neo4j = setup_test_neo4j();

    // Create graph in our system
    let graph = create_test_graph();

    // Sync to Neo4j
    let projection = Neo4jProjection::new(neo4j.url());
    projection.sync(graph).await.unwrap();

    // Modify in Neo4j
    neo4j.execute("MATCH (n) SET n.updated = true").await.unwrap();

    // Sync back
    let events = projection.ingest_changes().await.unwrap();
    assert!(!events.is_empty());
}
```

## Week 3: Comprehensive Test Suite

### Conceptual Space Tests
```rust
#[test]
fn test_conceptual_space_creation() {
    let space = ConceptualSpace::new(vec![
        QualityDimension::new("size", 0.0, 100.0),
        QualityDimension::new("complexity", 0.0, 10.0),
    ]);
    assert_eq!(space.dimensions().len(), 2);
}

#[test]
fn test_semantic_similarity_calculation() {
    let point1 = ConceptualPoint::new(vec![10.0, 5.0]);
    let point2 = ConceptualPoint::new(vec![15.0, 6.0]);
    let similarity = calculate_similarity(&point1, &point2);
    assert!(similarity > 0.8); // Should be similar
}
```

### Workflow State Machine Tests
```rust
#[test]
fn test_workflow_state_transitions() {
    let mut workflow = Workflow::new();
    workflow.start();
    assert_eq!(workflow.state(), WorkflowState::Running);

    workflow.complete_step("step1");
    assert_eq!(workflow.state(), WorkflowState::Running);

    workflow.complete_step("step2");
    assert_eq!(workflow.state(), WorkflowState::Completed);
}
```

## Implementation Priority

1. **Fix failing tests** (Day 1)
2. **Import pipeline tests** (Day 2-3)
3. **Query handler tests** (Day 4)
4. **Integration tests** (Week 2)
5. **Performance tests** (Week 2)
6. **External system tests** (Week 3)

## Success Metrics

- All 8 failing tests pass
- Test coverage increases to 80%+
- All critical paths have integration tests
- Performance benchmarks established
- No untested public APIs

## Test Infrastructure Setup

### 1. Add Test Dependencies
```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
proptest = "1.0"
test-case = "3.0"
serial_test = "2.0"
```

### 2. Configure Test Environment
```rust
// tests/common/mod.rs
pub fn setup_test_env() {
    std::env::set_var("BEVY_HEADLESS", "1");
    std::env::set_var("RUST_LOG", "debug");
}

pub fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(TestPlugin);
    app
}
```

### 3. Create Test Fixtures
```rust
// tests/fixtures/mod.rs
pub fn create_test_graph() -> Graph {
    // Standard test graph for consistency
}

pub fn create_test_events(count: usize) -> Vec<DomainEvent> {
    // Generate consistent test events
}
```

## Continuous Integration

### GitHub Actions Workflow
```yaml
name: Test Suite
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: cachix/install-nix-action@v22
      - run: nix develop --command cargo test --all
      - run: nix develop --command cargo bench --no-run

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: cachix/install-nix-action@v22
      - run: nix develop --command cargo tarpaulin --out Xml
      - uses: codecov/codecov-action@v3
```

## Monitoring Progress

Track progress in `/doc/progress/test-coverage-progress.json`:
```json
{
  "coverage": {
    "domain": 65,
    "application": 40,
    "infrastructure": 50,
    "presentation": 30,
    "overall": 46
  },
  "failing_tests": 8,
  "missing_features": [
    "import_pipeline",
    "query_handlers",
    "external_projections",
    "conceptual_spaces"
  ]
}
```

This plan provides a clear path to comprehensive test coverage while implementing missing functionality using TDD principles.
