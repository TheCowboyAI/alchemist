# Test Implementation Plan

## Overview

This plan outlines the strategy for implementing tests based on the 227 user stories (207 domain + 20 cross-domain) to achieve 80%+ test coverage across all CIM modules.

## Current State Analysis

### Test Coverage by Domain
- **Graph Domain**: ~50% (best coverage)
- **Agent Domain**: ~20%
- **Identity Domain**: ~30%
- **Document Domain**: ~50%
- **Location Domain**: ~40%
- **Organization Domain**: ~30%
- **ConceptualSpaces Domain**: ~25%
- **Workflow Domain**: ~40%
- **Person Domain**: ~30%
- **Policy Domain**: ~35%
- **Overall Average**: ~35%

### Target State
- **Minimum Coverage**: 80% per domain
- **Integration Tests**: 90% of cross-domain flows
- **E2E Tests**: All critical user journeys

## Implementation Phases

### Phase 1: Core Domain Tests (Days 7-8)
Focus on domains with lowest coverage and highest criticality.

#### Priority 1: Identity Domain (30% → 80%)
```rust
// Test Structure
mod identity_tests {
    mod aggregate_tests;      // 15 tests
    mod command_tests;        // 20 tests
    mod event_tests;          // 15 tests
    mod projection_tests;     // 10 tests
    mod integration_tests;    // 10 tests
}
```

**Key Tests to Implement:**
- `test_person_identity_creation` (Story I1)
- `test_authentication_flow` (Story I2)
- `test_mfa_enforcement` (Story I4)
- `test_session_management` (Story I5)
- `test_privacy_controls` (Story I11)

#### Priority 2: Agent Domain (20% → 80%)
```rust
mod agent_tests {
    mod lifecycle_tests;      // 20 tests
    mod capability_tests;     // 15 tests
    mod permission_tests;     // 15 tests
    mod tool_tests;          // 10 tests
    mod query_tests;         // 10 tests
}
```

**Key Tests to Implement:**
- `test_agent_deployment` (Story A1)
- `test_capability_management` (Story A5)
- `test_permission_enforcement` (Story A9)
- `test_tool_integration` (Story A13)
- `test_agent_queries` (Story A17)

#### Priority 3: Policy Domain (35% → 80%)
```rust
mod policy_tests {
    mod definition_tests;     // 15 tests
    mod enforcement_tests;    // 20 tests
    mod permission_tests;     // 15 tests
    mod compliance_tests;     // 15 tests
    mod security_tests;       // 15 tests
}
```

**Key Tests to Implement:**
- `test_access_policy_definition` (Story PO1)
- `test_policy_enforcement` (Story PO4)
- `test_rate_limiting` (Story PO5)
- `test_compliance_monitoring` (Story PO10)
- `test_mfa_policies` (Story PO15)

### Phase 2: Supporting Domain Tests (Days 8-9)

#### Priority 4: Person Domain (30% → 80%)
```rust
mod person_tests {
    mod profile_tests;        // 15 tests
    mod skill_tests;         // 10 tests
    mod relationship_tests;   // 15 tests
    mod preference_tests;     // 10 tests
    mod activity_tests;       // 10 tests
}
```

#### Priority 5: Organization Domain (30% → 80%)
```rust
mod organization_tests {
    mod creation_tests;       // 10 tests
    mod member_tests;        // 15 tests
    mod team_tests;          // 15 tests
    mod resource_tests;      // 10 tests
    mod compliance_tests;    // 10 tests
}
```

#### Priority 6: ConceptualSpaces Domain (25% → 80%)
```rust
mod conceptual_tests {
    mod space_tests;         // 15 tests
    mod concept_tests;       // 15 tests
    mod region_tests;        // 15 tests
    mod similarity_tests;    // 15 tests
    mod learning_tests;      // 10 tests
}
```

### Phase 3: Advanced Domain Tests (Days 9-10)

#### Priority 7: Workflow Domain (40% → 80%)
```rust
mod workflow_tests {
    mod design_tests;        // 15 tests
    mod execution_tests;     // 20 tests
    mod task_tests;         // 15 tests
    mod pattern_tests;      // 10 tests
    mod monitoring_tests;   // 10 tests
}
```

#### Priority 8: Document Domain (50% → 80%)
```rust
mod document_tests {
    mod creation_tests;      // 10 tests
    mod version_tests;      // 15 tests
    mod collaboration_tests; // 15 tests
    mod intelligence_tests; // 10 tests
    mod search_tests;       // 10 tests
}
```

#### Priority 9: Location Domain (40% → 80%)
```rust
mod location_tests {
    mod management_tests;    // 10 tests
    mod spatial_tests;      // 15 tests
    mod geofence_tests;     // 15 tests
    mod service_tests;      // 10 tests
    mod privacy_tests;      // 10 tests
}
```

### Phase 4: Integration Tests (Days 11-12)

#### Cross-Domain Integration Tests
```rust
mod integration_tests {
    mod identity_agent_tests;           // X1, X2
    mod document_workflow_tests;        // X3, X4
    mod organization_policy_tests;      // X5, X6
    mod location_agent_tests;          // X7, X8
    mod conceptual_document_tests;     // X9, X10
    mod multi_domain_tests;            // X11-X15
    mod testing_pattern_tests;         // X16-X18
    mod data_consistency_tests;        // X19-X20
}
```

**Key Integration Tests:**
- `test_agent_authentication_flow` (Story X1)
- `test_document_approval_workflow` (Story X3)
- `test_org_policy_enforcement` (Story X5)
- `test_location_agent_trigger` (Story X7)
- `test_semantic_document_search` (Story X9)
- `test_project_collaboration` (Story X11)

### Phase 5: E2E Tests (Days 13-14)

#### End-to-End Scenarios
```rust
mod e2e_tests {
    mod user_journey_tests {
        // Complete user registration through first action
        async fn test_new_user_journey();
        
        // Document creation through approval
        async fn test_document_lifecycle();
        
        // Project setup through completion
        async fn test_project_management();
        
        // Agent deployment through task execution
        async fn test_agent_automation();
    }
    
    mod system_scenarios {
        // Multi-domain transaction flows
        async fn test_complex_transactions();
        
        // Failure and recovery scenarios
        async fn test_system_resilience();
        
        // Performance under load
        async fn test_system_performance();
    }
}
```

## Test Implementation Guidelines

### 1. Test Structure Template
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    /// Test for User Story XX: [Story Title]
    /// 
    /// # Mermaid Diagram
    /// ```mermaid
    /// graph TD
    ///     A[Setup] --> B[Action]
    ///     B --> C[Verify]
    /// ```
    #[tokio::test]
    async fn test_story_implementation() {
        // Given: Setup test context
        let mut test_context = TestContext::new();
        
        // When: Execute the action
        let result = test_context.execute_action().await;
        
        // Then: Verify outcomes
        assert!(result.is_ok());
        assert_events_generated(&test_context, vec![
            "ExpectedEvent1",
            "ExpectedEvent2"
        ]);
    }
}
```

### 2. Test Data Builders
```rust
// Builder pattern for test data
pub struct TestPersonBuilder {
    name: String,
    email: String,
    // ... other fields
}

impl TestPersonBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
    
    pub fn build(self) -> Person {
        Person::new(self.name, self.email)
    }
}
```

### 3. Event Verification Helpers
```rust
pub fn assert_events_generated(
    context: &TestContext,
    expected_events: Vec<&str>
) {
    let events = context.get_events();
    assert_eq!(events.len(), expected_events.len());
    
    for (event, expected) in events.iter().zip(expected_events.iter()) {
        assert_eq!(event.event_type(), *expected);
    }
}
```

### 4. Integration Test Helpers
```rust
pub struct IntegrationTestContext {
    nats_client: NatsClient,
    event_store: EventStore,
    domains: HashMap<String, DomainContext>,
}

impl IntegrationTestContext {
    pub async fn setup_multi_domain() -> Self {
        // Setup NATS
        let nats = setup_test_nats().await;
        
        // Initialize domains
        let mut domains = HashMap::new();
        domains.insert("identity", setup_identity_domain().await);
        domains.insert("agent", setup_agent_domain().await);
        // ... other domains
        
        Self { nats_client: nats, domains }
    }
}
```

## Test Metrics and Monitoring

### Coverage Tracking
```toml
# tarpaulin.toml
[default]
workspace = true
all-features = true
engine = "llvm"
exclude-files = ["*/tests/*", "*/examples/*"]
```

### Daily Progress Tracking
| Day | Domains                 | Tests Written | Coverage Increase | Target |
| --- | ----------------------- | ------------- | ----------------- | ------ |
| 7   | Identity, Agent, Policy | 150           | 35% → 50%         | 50%    |
| 8   | Person, Org, Conceptual | 140           | 50% → 65%         | 65%    |
| 9   | Workflow, Document      | 120           | 65% → 75%         | 75%    |
| 10  | Location, Graph         | 90            | 75% → 80%         | 80%    |
| 11  | Integration             | 80            | N/A               | 90%    |
| 12  | Integration             | 60            | N/A               | 90%    |
| 13  | E2E                     | 40            | N/A               | 100%   |
| 14  | E2E                     | 30            | N/A               | 100%   |

### Success Criteria
- [ ] All domains at 80%+ unit test coverage
- [ ] All cross-domain stories have integration tests
- [ ] All critical paths have E2E tests
- [ ] Performance benchmarks established
- [ ] Failure scenarios tested
- [ ] Documentation updated with test examples

## Test Execution Strategy

### Continuous Integration
```yaml
# .github/workflows/test.yml
test:
  strategy:
    matrix:
      test-suite:
        - unit
        - integration
        - e2e
  steps:
    - run: cargo test --workspace --test-suite ${{ matrix.test-suite }}
    - run: cargo tarpaulin --out Xml
    - uses: codecov/codecov-action@v3
```

### Local Development
```bash
# Run all tests
cargo test --workspace

# Run specific domain tests
cargo test -p cim-domain-identity

# Run with coverage
cargo tarpaulin --workspace --out Html

# Run integration tests
cargo test --test integration_tests

# Run E2E tests
cargo test --test e2e_tests -- --test-threads=1
```

## Risk Mitigation

### Identified Risks
1. **Complex Event Flows**: Mitigate with comprehensive event helpers
2. **Async Testing**: Use tokio::test and proper async patterns
3. **Test Data Management**: Implement builders and fixtures
4. **Performance Impact**: Parallelize where possible
5. **Flaky Tests**: Implement retry logic for network operations

### Contingency Plans
- If coverage targets slip, focus on critical paths first
- If integration tests are complex, create simplified scenarios
- If E2E tests are slow, implement smoke test suite
- If specific domains are challenging, pair program

This plan provides a structured approach to achieving 80%+ test coverage while ensuring quality and maintainability of the test suite. 