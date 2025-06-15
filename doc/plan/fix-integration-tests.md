# Fix Integration Tests Plan

## Date: 2025-01-12

## Overview
The integration tests have many compilation errors that need to be fixed systematically. This plan outlines the approach to fix all integration test issues.

## Main Issues Identified

### 1. Import Path Issues
- `Position3D` is in `cim_domain`, not `cim_domain_graph`
- `NodeType` is in `cim_domain`, not `cim_domain_graph`
- `StepType` is in `cim_domain_workflow`, not `cim_domain_graph`
- Many other types need correct import paths

### 2. DomainEvent Trait Usage
- `DomainEvent` is a trait, not a type
- Need to use `Box<dyn DomainEvent>` for collections
- Function parameters need generic constraints or trait objects

### 3. Missing Dependencies
- `axum` - needed for webhook tests
- `reqwest` - needed for HTTP client tests
- `wiremock` - needed for mock server tests

### 4. API Changes
- `NodeUpdated` is now a tuple variant, not a struct variant
- Some fields have been moved or renamed

## Fix Strategy

### Phase 1: Fix Import Paths
1. Update all imports to use correct modules
2. Add missing imports for assertion functions
3. Fix module visibility issues

### Phase 2: Fix DomainEvent Usage
1. Change `Vec<DomainEvent>` to `Vec<Box<dyn DomainEvent>>`
2. Update function signatures to use trait objects or generics
3. Fix event construction to match actual API

### Phase 3: Handle External Dependencies
1. Comment out tests that require external dependencies (axum, reqwest)
2. Mark them with `#[ignore]` and add TODO comments
3. Focus on tests that can run without external services

### Phase 4: Fix API Usage
1. Update event construction to match actual event structures
2. Fix projection API usage
3. Update query result handling

## Implementation Order

1. **fixtures.rs** - Fix the test infrastructure first
2. **simple_test.rs** - Ensure basic test works
3. **event_flow_tests.rs** - Fix event handling tests
4. **domain_integration_tests.rs** - Fix domain integration
5. **query_handler_tests.rs** - Fix query tests
6. **projection_sync_tests.rs** - Fix projection tests
7. **import_pipeline_tests.rs** - Fix import tests
8. **performance_benchmarks.rs** - Fix performance tests
9. **external_system_tests.rs** - Comment out external dependency tests
10. **end_to_end_workflow_tests.rs** - Fix workflow tests

## Expected Outcome

After implementing these fixes:
- All integration tests should compile
- Tests that don't require external services should pass
- Tests requiring external services should be properly marked with `#[ignore]`
- The test infrastructure should be robust and maintainable

## Next Steps

1. Start with fixtures.rs to fix the test infrastructure
2. Work through each test file systematically
3. Run tests after each fix to verify progress
4. Document any additional issues discovered during fixing 