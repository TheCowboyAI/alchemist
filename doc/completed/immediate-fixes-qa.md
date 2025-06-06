# Immediate Fixes Plan - QA Issues

## Overview

This plan addresses the critical issues identified in the Comprehensive QA Report, focusing on getting the project back to a compilable and testable state.

## Priority 1: Fix Compilation Error (URGENT)

### Issue
The integration test `tests/nats_object_store_integration.rs` is failing because it's using an outdated API for `NatsObjectStore::new()`.

### Fix
Update line 31 in the test file:
```rust
// FROM:
let object_store = Arc::new(NatsObjectStore::new(jetstream).await?);

// TO:
let object_store = Arc::new(NatsObjectStore::new(jetstream, 1024).await?);
```

### Verification
```bash
cargo test --test nats_object_store_integration -- --nocapture
```

## Priority 2: Fix Linting Warnings

### cim-ipld Warnings (7)
1. Variables in format strings
2. Unnecessary type casting
3. Clone on Copy types
4. Length comparison to zero

### Fix Commands
```bash
cd cim-ipld
cargo clippy --fix --lib -p cim-ipld
```

### bevy_render Warnings (2)
1. Unused imports in patched files
2. Unnecessary qualification

### Manual Fixes Required
- Remove unused imports from Component trait implementations
- Remove unnecessary module qualifications

## Priority 3: Create Domain Tests

### Test Structure
```
tests/
├── domain/
│   ├── aggregates/
│   │   ├── graph_aggregate_test.rs
│   │   └── mod.rs
│   ├── commands/
│   │   ├── graph_commands_test.rs
│   │   └── mod.rs
│   └── events/
│       ├── graph_events_test.rs
│       └── mod.rs
└── mod.rs
```

### Example Domain Test
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::aggregates::GraphAggregate;
    use crate::domain::commands::CreateGraph;
    use crate::domain::events::GraphCreated;

    #[test]
    fn test_create_graph_command() {
        // Given
        let mut aggregate = GraphAggregate::new();
        let command = CreateGraph {
            id: GraphId::new(),
            name: "Test Graph".to_string(),
        };

        // When
        let events = aggregate.handle(command).unwrap();

        // Then
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::GraphCreated(e) => {
                assert_eq!(e.name, "Test Graph");
            }
            _ => panic!("Wrong event type"),
        }
    }
}
```

## Priority 4: Configure Test Coverage

### Add to Cargo.toml
```toml
[dev-dependencies]
cargo-tarpaulin = "0.27"
```

### Create .tarpaulin.toml
```toml
[default]
exclude-files = ["*/tests/*", "*/benches/*", "*/examples/*"]
ignore-panics = true
ignore-tests = false
timeout = "300s"
```

### Run Coverage
```bash
cargo tarpaulin --out Html --output-dir target/coverage
```

## Implementation Timeline

### Day 1 (Immediate)
- [ ] Fix compilation error in nats_object_store_integration.rs
- [ ] Run cargo clippy --fix on cim-ipld
- [ ] Fix bevy_render warnings manually
- [ ] Verify all code compiles

### Day 2 (Domain Tests)
- [ ] Create test directory structure
- [ ] Write GraphAggregate tests
- [ ] Write Command validation tests
- [ ] Write Event application tests

### Day 3 (Coverage & Documentation)
- [ ] Configure cargo-tarpaulin
- [ ] Run coverage report
- [ ] Document test patterns
- [ ] Update progress.json

## Success Criteria

1. **Compilation**: `cargo build` passes without errors
2. **Linting**: `cargo clippy` shows 0 warnings
3. **Tests**: At least 10 domain tests passing
4. **Coverage**: Measurable coverage (target 80% for domain)

## Verification Commands

```bash
# Full verification suite
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo tarpaulin --print-summary
nix build
```

## Notes

- Keep tests focused on domain logic without infrastructure dependencies
- Use test doubles for external dependencies
- Follow TDD going forward - write tests first
- Update progress.json after each milestone
