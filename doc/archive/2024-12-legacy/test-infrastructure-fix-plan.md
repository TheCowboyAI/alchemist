# Test Infrastructure Fix Plan

## Problem Statement

The Information Alchemist project is feature complete and functional, but the test suite fails to compile due to Bevy render pipeline linker errors. This prevents automated testing and quality assurance verification.

## Root Cause Analysis

### Primary Issue: Bevy Render Pipeline Dependencies
```
error: undefined symbol: _$LT$bevy_render..view..ViewDepthTexture$u20$as$u20$bevy_ecs..component..Component$GT$::register_required_components
error: undefined symbol: _$LT$bevy_render..experimental..occlusion_culling..OcclusionCullingSubview$u20$as$u20$bevy_ecs..component..Component$GT$::register_required_components
```

**Cause**: Tests create minimal Bevy `App` instances without full plugin initialization, but the codebase references components that require the complete render pipeline.

### Secondary Issues
1. **Incomplete Test Setup**: Tests only add events, missing core Bevy plugins
2. **Feature Flag Mismatch**: Test compilation includes render features not initialized in test environment
3. **Dependency Conflicts**: Experimental Bevy features referenced but not properly configured

## Solution Strategy

### Phase 1: Immediate Fix (Critical Priority)

#### 1.1 Create Test-Specific Bevy Configuration
**File**: `src/testing/test_app_builder.rs` (new)

```rust
use bevy::prelude::*;

/// Builder for test applications with proper Bevy setup
pub struct TestAppBuilder {
    headless: bool,
    minimal: bool,
}

impl TestAppBuilder {
    pub fn new() -> Self {
        Self {
            headless: true,
            minimal: false,
        }
    }

    pub fn headless(mut self, headless: bool) -> Self {
        self.headless = headless;
        self
    }

    pub fn minimal(mut self, minimal: bool) -> Self {
        self.minimal = minimal;
        self
    }

    pub fn build(self) -> App {
        let mut app = App::new();

        if self.minimal {
            // Minimal setup for unit tests
            app.add_plugins(MinimalPlugins);
        } else {
            // Full setup for integration tests
            if self.headless {
                app.add_plugins(DefaultPlugins.set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    close_when_requested: false,
                }));
            } else {
                app.add_plugins(DefaultPlugins);
            }
        }

        // Add our domain events
        app.add_event::<crate::contexts::graph_management::events::GraphCreated>()
           .add_event::<crate::contexts::graph_management::events::NodeAdded>()
           .add_event::<crate::contexts::graph_management::events::EdgeConnected>();

        app
    }
}

/// Helper function for domain-only tests
pub fn create_minimal_test_app() -> App {
    TestAppBuilder::new().minimal(true).build()
}

/// Helper function for integration tests
pub fn create_headless_test_app() -> App {
    TestAppBuilder::new().headless(true).build()
}
```

#### 1.2 Update Test Setup Functions
**File**: `src/contexts/graph_management/tests.rs`

```rust
// Replace existing setup_test_app() function
fn setup_test_app() -> App {
    crate::testing::test_app_builder::create_minimal_test_app()
}

// For tests that need full ECS
fn setup_integration_test_app() -> App {
    crate::testing::test_app_builder::create_headless_test_app()
}
```

#### 1.3 Add Cargo.toml Test Features
**File**: `Cargo.toml`

```toml
[features]
default = []
dev = [
  "bevy/asset_processor",
  "bevy/file_watcher"
]
# Add test-specific features
test-minimal = []
test-headless = [
  "bevy/bevy_render",
  "bevy/bevy_core_pipeline"
]
```

### Phase 2: Test Coverage Implementation (High Priority)

#### 2.1 Domain-Isolated Unit Tests
**Approach**: Tests that don't require Bevy ECS at all

**File**: `src/contexts/graph_management/domain_tests.rs` (new)

```rust
#[cfg(test)]
mod domain_tests {
    use super::*;

    #[test]
    fn test_graph_identity_creation() {
        let id1 = GraphIdentity::new();
        let id2 = GraphIdentity::new();
        assert_ne!(id1, id2);
        assert_ne!(id1.0, uuid::Uuid::nil());
    }

    #[test]
    fn test_spatial_position_operations() {
        let pos = SpatialPosition::at_3d(1.0, 2.0, 3.0);
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
        assert_eq!(pos.z, 3.0);
    }

    #[test]
    fn test_node_content_validation() {
        let content = NodeContent {
            label: "Test Node".to_string(),
            category: "test".to_string(),
            properties: Default::default(),
        };
        assert!(!content.label.is_empty());
        assert!(!content.category.is_empty());
    }
}
```

#### 2.2 Service Logic Tests
**Approach**: Test service logic without ECS dependencies

**File**: `src/contexts/graph_management/service_tests.rs` (new)

```rust
#[cfg(test)]
mod service_tests {
    use super::*;

    #[test]
    fn test_validate_graph_constraints() {
        let validator = ValidateGraph;

        // Test node limit validation logic
        let result = validator.validate_node_count(50, 1000);
        assert!(result.is_ok());

        let result = validator.validate_node_count(1001, 1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_edge_validation_logic() {
        let validator = ValidateGraph;
        let source = NodeIdentity::new();
        let target = NodeIdentity::new();

        // Test self-loop detection
        let result = validator.validate_edge_connection(source, source);
        assert!(result.is_err());

        // Test valid connection
        let result = validator.validate_edge_connection(source, target);
        assert!(result.is_ok());
    }
}
```

#### 2.3 Integration Tests with Proper Setup
**File**: `src/testing/integration_tests.rs` (new)

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::testing::test_app_builder::create_headless_test_app;

    #[test]
    fn test_full_graph_workflow() {
        let mut app = create_headless_test_app();

        // Add our plugins
        app.add_plugins(crate::contexts::graph_management::plugin::GraphManagementPlugin);

        // Test complete workflow
        // ... implementation
    }
}
```

### Phase 3: Test Infrastructure Improvements (Medium Priority)

#### 3.1 Mock Framework for External Dependencies
**File**: `src/testing/mocks.rs` (new)

```rust
/// Mock NATS client for testing
pub struct MockNatsClient {
    pub sent_messages: Vec<String>,
}

impl MockNatsClient {
    pub fn new() -> Self {
        Self {
            sent_messages: Vec::new(),
        }
    }

    pub fn send_message(&mut self, message: String) {
        self.sent_messages.push(message);
    }
}

/// Mock file system for import/export tests
pub struct MockFileSystem {
    pub files: std::collections::HashMap<String, String>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            files: std::collections::HashMap::new(),
        }
    }

    pub fn add_file(&mut self, path: String, content: String) {
        self.files.insert(path, content);
    }

    pub fn read_file(&self, path: &str) -> Option<&String> {
        self.files.get(path)
    }
}
```

#### 3.2 Test Utilities and Helpers
**File**: `src/testing/test_utilities.rs` (new)

```rust
/// Utilities for creating test data
pub struct TestDataBuilder;

impl TestDataBuilder {
    pub fn sample_graph_metadata() -> GraphMetadata {
        GraphMetadata {
            name: "Test Graph".to_string(),
            description: "Test Description".to_string(),
            domain: "test".to_string(),
            created: std::time::SystemTime::now(),
            modified: std::time::SystemTime::now(),
            tags: vec!["test".to_string()],
        }
    }

    pub fn sample_node_content() -> NodeContent {
        NodeContent {
            label: "Test Node".to_string(),
            category: "test".to_string(),
            properties: Default::default(),
        }
    }

    pub fn sample_edge_relationship(source: NodeIdentity, target: NodeIdentity) -> EdgeRelationship {
        EdgeRelationship {
            source,
            target,
            category: "test".to_string(),
            strength: 1.0,
            properties: Default::default(),
        }
    }
}
```

### Phase 4: Continuous Integration Setup (Medium Priority)

#### 4.1 NixOS Test Configuration
**File**: `nix/test-runner.nix` (new)

```nix
{ pkgs, rust-toolchain, nonRustDeps }:

pkgs.writeShellScriptBin "run-tests" ''
  export BEVY_HEADLESS=1
  export RUST_BACKTRACE=1

  echo "Running domain tests..."
  ${rust-toolchain}/bin/cargo test --lib --features test-minimal -- domain_tests

  echo "Running service tests..."
  ${rust-toolchain}/bin/cargo test --lib --features test-minimal -- service_tests

  echo "Running integration tests..."
  ${rust-toolchain}/bin/cargo test --lib --features test-headless -- integration_tests

  echo "All tests completed!"
''
```

#### 4.2 Test Coverage Reporting
**File**: `nix/coverage-report.nix` (new)

```nix
{ pkgs, rust-toolchain }:

pkgs.writeShellScriptBin "coverage-report" ''
  export BEVY_HEADLESS=1
  export CARGO_INCREMENTAL=0
  export RUSTFLAGS="-Cinstrument-coverage"
  export LLVM_PROFILE_FILE="coverage-%p-%m.profraw"

  ${rust-toolchain}/bin/cargo test --features test-minimal,test-headless

  ${pkgs.llvm}/bin/llvm-profdata merge -sparse coverage-*.profraw -o coverage.profdata
  ${pkgs.llvm}/bin/llvm-cov show target/debug/deps/ia-* -instr-profile=coverage.profdata --format=html --output-dir=coverage-report

  echo "Coverage report generated in coverage-report/"
''
```

## Implementation Timeline

### Week 1: Critical Fixes
- [ ] Day 1-2: Implement `TestAppBuilder` and update test setup
- [ ] Day 3-4: Fix compilation errors in existing tests
- [ ] Day 5: Verify all tests compile and basic functionality works

### Week 2: Test Coverage
- [ ] Day 1-2: Implement domain-isolated unit tests
- [ ] Day 3-4: Create service logic tests
- [ ] Day 5: Add integration tests with proper Bevy setup

### Week 3: Infrastructure
- [ ] Day 1-2: Implement mock framework
- [ ] Day 3-4: Add test utilities and helpers
- [ ] Day 5: Set up NixOS test runner and coverage reporting

## Success Criteria

### Immediate (Week 1)
- [ ] All tests compile without errors
- [ ] Basic test suite runs successfully
- [ ] `cargo test` completes without linker errors

### Short-term (Week 2)
- [ ] 80%+ test coverage for domain logic
- [ ] All service functions have unit tests
- [ ] Integration tests verify end-to-end workflows

### Long-term (Week 3)
- [ ] Automated test runner in NixOS
- [ ] Coverage reporting integrated
- [ ] CI/CD pipeline ready for future development

## Risk Mitigation

### Risk: Bevy Version Compatibility
**Mitigation**: Pin Bevy version and test with specific feature combinations

### Risk: Test Performance
**Mitigation**: Use minimal setups for unit tests, full setup only for integration tests

### Risk: Flaky Tests
**Mitigation**: Implement proper test isolation and deterministic test data

## Monitoring and Validation

### Daily Checks
- [ ] All tests compile
- [ ] Test execution time < 30 seconds
- [ ] No test failures

### Weekly Reviews
- [ ] Test coverage metrics
- [ ] Performance benchmarks
- [ ] Code quality metrics

---

**Priority**: 🔥 CRITICAL
**Estimated Effort**: 3 weeks
**Dependencies**: None
**Blockers**: None

*This plan addresses the critical test infrastructure issues identified in the QA report and establishes a foundation for ongoing quality assurance.*
