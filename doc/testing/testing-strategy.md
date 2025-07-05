# CIM Testing Strategy

## Overview

The Composable Information Machine (CIM) employs a comprehensive testing strategy that ensures reliability, performance, and maintainability across all domains. This document outlines our testing approach, patterns, and best practices.

## Test Categories

### 1. Unit Tests (Domain Layer)
**Location**: `cim-domain-*/tests/`  
**Purpose**: Test individual components, aggregates, and value objects in isolation  
**Coverage Target**: 95%+

#### Key Principles:
- Test-first development (TDD)
- No external dependencies (no NATS, no Bevy)
- Pure domain logic testing
- Fast execution (<100ms per test)

#### Example Pattern:
```rust
#[test]
fn test_aggregate_command_handling() {
    let mut aggregate = GraphAggregate::new(GraphId::new());
    let command = AddNode { /* ... */ };
    
    let events = aggregate.handle_command(command).unwrap();
    
    assert_eq!(events.len(), 1);
    match &events[0] {
        DomainEvent::NodeAdded { .. } => (),
        _ => panic!("Wrong event type"),
    }
}
```

### 2. Integration Tests
**Location**: `tests/integration/`  
**Purpose**: Test cross-domain interactions and system integration  
**Coverage Target**: 80%+

#### Test Types:
- **Cross-Domain Flow**: Tests that span multiple domains
- **Event Bus Integration**: Verify event propagation
- **NATS Integration**: Test messaging infrastructure
- **Performance Benchmarks**: Ensure performance targets

#### Current Status:
- ✅ 14/17 integration tests passing
- ❌ 3 NATS tests need fixes (stream conflicts)
- ✅ Performance exceeds all targets

### 3. Component Tests (ECS Layer)
**Location**: `cim-domain-*/tests/systems/`  
**Purpose**: Test Bevy systems and component interactions  
**Environment**: `BEVY_HEADLESS=1`

#### Key Patterns:
```rust
fn test_ecs_system() {
    let mut app = App::new();
    app.add_systems(Update, system_under_test);
    
    // Setup test data
    app.world.spawn(TestEntity { /* ... */ });
    
    // Run system
    app.update();
    
    // Verify results
    let query = app.world.query::<&Component>();
    assert_eq!(query.iter(&app.world).count(), 1);
}
```

### 4. Error Handling Tests
**Location**: `tests/error_handling_test.rs`  
**Purpose**: Verify resilience and error recovery  

#### Patterns Tested:
- Retry with exponential backoff
- Circuit breaker implementation
- Timeout handling
- Graceful degradation
- Concurrent error handling
- Cascading failure prevention

## Testing Infrastructure

### Test Fixtures
**Location**: `tests/integration/fixtures.rs`

Provides common test infrastructure:
- `TestEventStore`: In-memory event storage
- `TestEventBus`: Simulated event propagation
- `TestNatsClient`: Mock NATS client
- Domain-specific test builders

### Test Helpers
**Location**: `tests/common/`

Shared utilities:
- ID generation helpers
- Event creation factories
- Assertion helpers
- Performance measurement tools

## Performance Testing

### Benchmarks
**Tool**: Criterion.rs  
**Location**: `benches/`

Current Performance Metrics:
- Event Creation: 762,710/sec (target: 100,000/sec) ✅
- Event Publishing: 882,103/sec (target: 10,000/sec) ✅
- Concurrent Operations: 1,978,904/sec ✅
- Event Filtering: 0.59ms (target: <10ms) ✅
- ID Generation: 2,962,139/sec (target: 1,000,000/sec) ✅

### Load Testing
**Tool**: Custom load generators  
**Scenarios**:
1. Sustained load (1000 events/sec for 1 hour)
2. Burst load (10,000 events/sec for 1 minute)
3. Mixed workload (reads + writes)
4. Failure injection (random service failures)

## Test Execution

### Local Development
```bash
# Run all unit tests
cargo test

# Run integration tests
cargo test --test '*' -- --ignored

# Run specific domain tests
cargo test -p cim-domain-graph

# Run with coverage
cargo tarpaulin --out Html

# Run benchmarks
cargo bench
```

### CI/CD Pipeline
```yaml
test:
  stage: test
  script:
    - cargo fmt -- --check
    - cargo clippy -- -D warnings
    - cargo test
    - cargo test -- --ignored
    - cargo bench --no-run
```

### NATS Testing
```bash
# Start NATS for testing
nats-server -js -sd /tmp/nats

# Run NATS integration tests
cargo test --test nats_integration_test -- --ignored

# Cleanup
nats-server --signal stop
```

## Test Patterns

### 1. Given-When-Then
```rust
#[test]
fn test_business_rule() {
    // Given
    let aggregate = setup_test_aggregate();
    
    // When
    let result = aggregate.process_command(command);
    
    // Then
    assert!(result.is_ok());
    assert_eq!(aggregate.state(), expected_state);
}
```

### 2. Property-Based Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_invariant(
        nodes in prop::collection::vec(node_strategy(), 0..100)
    ) {
        let graph = Graph::from_nodes(nodes);
        prop_assert!(graph.is_valid());
    }
}
```

### 3. Snapshot Testing
```rust
#[test]
fn test_projection_output() {
    let projection = build_test_projection();
    let output = projection.render();
    
    insta::assert_snapshot!(output);
}
```

## Test Data Management

### Fixtures
- Use builders for complex test data
- Keep test data minimal and focused
- Avoid sharing mutable test data

### Test Isolation
- Each test runs in isolation
- No shared state between tests
- Clean up external resources (NATS streams, files)

## Debugging Failed Tests

### 1. Verbose Output
```bash
RUST_LOG=debug cargo test failing_test -- --nocapture
```

### 2. Test Replay
```rust
// Save failing test inputs
#[test]
fn test_with_replay() {
    let input = generate_test_input();
    
    // Save for replay
    if let Err(e) = process(input.clone()) {
        std::fs::write("failing_input.json", 
            serde_json::to_string(&input).unwrap()
        ).unwrap();
        panic!("Test failed: {}", e);
    }
}
```

### 3. Time Travel Debugging
Use event sourcing to replay exact sequences:
```rust
let events = event_store.get_events_until(failure_time);
let state = replay_events(events);
// Inspect state at failure point
```

## Test Coverage

### Current Coverage by Domain:
- cim-domain-graph: 95%+ ✅
- cim-domain-identity: 92%+ ✅
- cim-domain-person: 90%+ ✅
- cim-domain-agent: 88%+ ✅
- cim-domain-git: 85%+ ✅
- Other domains: 80%+ average

### Coverage Goals:
- Unit tests: 95%+ coverage
- Integration tests: 80%+ coverage
- Critical paths: 100% coverage

## Best Practices

### Do's:
- ✅ Write tests first (TDD)
- ✅ Keep tests fast and focused
- ✅ Use descriptive test names
- ✅ Test edge cases and error paths
- ✅ Mock external dependencies
- ✅ Use property-based testing for invariants
- ✅ Document complex test scenarios

### Don'ts:
- ❌ Share state between tests
- ❌ Test implementation details
- ❌ Write brittle tests with hardcoded values
- ❌ Skip error case testing
- ❌ Ignore flaky tests
- ❌ Test multiple behaviors in one test

## Test Maintenance

### Regular Tasks:
1. **Weekly**: Review and fix flaky tests
2. **Monthly**: Update test data and fixtures
3. **Quarterly**: Performance regression testing
4. **Release**: Full integration test suite

### Test Refactoring:
- Extract common patterns to helpers
- Update tests when requirements change
- Remove obsolete tests
- Improve test performance

## Future Improvements

### Planned Enhancements:
1. **Mutation Testing**: Verify test quality
2. **Chaos Engineering**: Test system resilience
3. **Contract Testing**: For cross-domain interfaces
4. **Visual Regression**: For UI components
5. **Security Testing**: Penetration and fuzzing

### Tooling Upgrades:
- Integrate with observability platform
- Automated performance regression detection
- Test impact analysis
- Parallel test execution optimization

## Conclusion

Our comprehensive testing strategy ensures CIM maintains high quality and reliability. By following these patterns and practices, we can confidently evolve the system while preventing regressions and maintaining performance targets. 