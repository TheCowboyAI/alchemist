# User Story Test Coverage Analysis

## Overview

This document analyzes the test coverage for each user story defined in `/doc/testing/user-stories.md` and identifies gaps that need to be addressed to meet our TDD requirements.

## Test Coverage by User Story

### ✅ Story 1: Create Event-Sourced Graph
**Status**: FULLY TESTED
- `test_graph_creation` - ✅ Implemented
- `test_event_sourcing_reconstruction` - ✅ Implemented
- Location: `tests/domain/aggregates/graph_tests.rs`

### ✅ Story 2: Maintain CID Chain Integrity
**Status**: FULLY TESTED
- `test_cid_chain_creation` - ✅ Implemented
- `test_chain_tampering_detection` - ✅ Implemented
- `test_cid_determinism` - ✅ Implemented
- Location: `cim-ipld/src/chain/mod.rs`, `tests/integration/cid_chain_tests.rs`

### ✅ Story 3: Handle Graph Commands
**Status**: FULLY TESTED
- 15+ command handler tests - ✅ Implemented
- Location: `src/domain/aggregates/graph.rs`

### ✅ Story 4: Add Nodes with Rich Metadata
**Status**: FULLY TESTED
- `test_handle_add_node_command` - ✅ Implemented
- `test_node_content_creation` - ✅ Implemented
- Location: `src/domain/aggregates/graph.rs`

### ✅ Story 5: Connect Nodes with Typed Edges
**Status**: FULLY TESTED
- `test_handle_connect_edge_command` - ✅ Implemented
- `test_handle_connect_edge_self_loop_error` - ✅ Implemented
- Location: `src/domain/aggregates/graph.rs`

### ✅ Story 6: Update Graph Elements (DDD Pattern)
**Status**: FULLY TESTED
- `test_handle_update_node_command` - ✅ Implemented
- `test_handle_move_node_command` - ✅ Implemented
- `test_value_object_patterns` - ✅ Implemented
- Location: `src/domain/aggregates/graph.rs`

### ✅ Story 7: Cascade Delete Dependencies
**Status**: FULLY TESTED
- `test_handle_remove_node_with_edges_cascade_delete` - ✅ Implemented
- Location: `src/domain/aggregates/graph.rs`

### ✅ Story 8: Persist Events to NATS JetStream
**Status**: FULLY TESTED (Added today)
- `test_distributed_event_store_*` - ✅ Implemented
- Location: `tests/integration/distributed_event_store_tests.rs`

### ✅ Story 9: Bridge Async NATS with Sync Bevy
**Status**: FULLY TESTED
- `test_event_bridge_bidirectional_flow` - ✅ Implemented
- Location: `tests/integration/nats_integration_tests.rs`

### ✅ Story 10: Store Large Content in Object Store
**Status**: FULLY TESTED
- `test_compression_threshold` - ✅ Implemented
- `test_content_storage_service_caching` - ✅ Implemented
- Location: `tests/integration/object_store_tests.rs`

### ✅ Story 11: Complete Command-to-Projection Flow
**Status**: FULLY TESTED
- `test_complete_command_to_projection_flow` - ✅ Implemented
- `test_multi_aggregate_event_flow` - ✅ Implemented
- Location: `tests/integration/projection_tests.rs`

### ⚠️ Story 12: Handle Concurrent Commands
**Status**: PARTIALLY TESTED
- `test_concurrent_command_processing` - ❌ Missing
- `test_concurrent_modifications` - ✅ Implemented
- **GAP**: Need concurrent command processing test

### ⚠️ Story 13: Recover from Failures
**Status**: PARTIALLY TESTED
- `test_event_store_recovery_after_crash` - ❌ Missing
- `test_event_deduplication` - ✅ Implemented
- **GAP**: Need crash recovery test

### ⚠️ Story 14: Visualize Graph in 3D
**Status**: PARTIALLY TESTED
- Visual verification - ❌ Manual only
- `test_render_modes` - ❌ Missing
- **GAP**: Need automated visual tests

### ⚠️ Story 15: Interact with Graph Elements
**Status**: PARTIALLY TESTED
- `test_closest_hit_selection` - ❌ Missing
- `test_camera_orbit_controls` - ❌ Missing
- **GAP**: Need interaction tests

### ✅ Story 16: Define Domain-Specific Content Types
**Status**: FULLY TESTED
- `test_*_content_creation` - ✅ Implemented
- `test_*_content_cid` - ✅ Implemented
- Location: `cim-ipld/tests/`

### ✅ Story 17: Chain Content for Integrity
**Status**: FULLY TESTED
- `test_content_chain_append` - ✅ Implemented
- `test_chain_validation` - ✅ Implemented
- Location: `cim-ipld/src/chain/mod.rs`

### ✅ Story 18: Comprehensive Test Coverage
**Status**: IN PROGRESS
- Current coverage: ~65%
- Target coverage: 95% (per TDD rules)
- **GAP**: Need to reach 95% coverage

### ✅ Story 19: Validate Business Invariants
**Status**: FULLY TESTED
- `test_graph_validation_*` - ✅ Implemented
- Domain invariant tests - ✅ Implemented
- Location: `src/domain/aggregates/graph.rs`

### ⚠️ Story 20: Handle Large Graphs Efficiently
**Status**: NOT TESTED
- Performance benchmarks - ❌ Missing
- Load tests - ❌ Missing
- **GAP**: Need performance test suite

## Missing User Stories

Based on TDD requirements and current implementation, we need additional user stories for:

### Story 21: Import Graph from External Formats
**As a** user
**I want** to import graphs from various formats (Mermaid, Cypher, etc.)
**So that** I can work with existing graph data

**Tests Needed:**
- `test_import_mermaid` (currently failing)
- `test_import_cypher`
- `test_import_arrows_app` (currently failing)
- `test_import_format_detection`

### Story 22: Handle All Commands and Events
**As a** developer
**I want** every command and event to have a handler
**So that** the system is complete per TDD rules

**Tests Needed:**
- Test for every command handler existence
- Test for every event handler existence
- Test for command rejection scenarios
- Test for event processing failures

### Story 23: Workflow State Machine
**As a** workflow designer
**I want** complete workflow state transitions
**So that** workflows execute reliably

**Tests Needed:**
- `test_workflow_state_transitions`
- `test_workflow_pause_resume`
- `test_workflow_failure_handling`
- `test_workflow_parallel_execution`

### Story 24: Query Handler Coverage
**As a** developer
**I want** query handlers for all projections
**So that** CQRS read side is complete

**Tests Needed:**
- `test_graph_summary_query`
- `test_node_search_query`
- `test_workflow_status_query`
- `test_event_history_query`

### Story 25: Animation and Physics Testing
**As a** developer
**I want** deterministic animation and physics
**So that** visual behavior is testable

**Tests Needed:**
- `test_animation_timing`
- `test_force_layout_convergence`
- `test_scheduled_command_execution`
- `test_animation_progress_tracking`

## Test Implementation Priority

### Priority 1: Fix Failing Tests (Immediate)
1. `test_graph_tag_operations` - Fix tag implementation
2. `test_import_arrows_app` - Implement import handler
3. `test_import_mermaid` - Implement import handler
4. `test_scheduled_command_timer` - Fix timing issues
5. `test_update_animation_progress` - Fix progress tracking
6. Force layout tests - Fix physics simulation

### Priority 2: Critical Missing Tests (This Week)
1. `test_concurrent_command_processing`
2. `test_event_store_recovery_after_crash`
3. Query handler tests
4. Import format tests
5. Workflow state machine tests

### Priority 3: Coverage Gaps (Next Week)
1. Performance benchmarks
2. Load tests for 10K nodes
3. Visual/interaction tests
4. Animation determinism tests

## Compliance with TDD Rules

### Current Violations:
1. **Test Coverage**: 65% vs 95% required
2. **Handler Coverage**: Not all commands/events have handlers with tests
3. **Failing Tests**: 13 tests failing (must be 0)

### Required Actions:
1. Fix all 13 failing tests immediately
2. Add missing command/event handler tests
3. Implement missing functionality with tests first
4. Increase coverage to 95%

## Test Pattern Requirements

Per TDD rules, all new tests must follow:

```rust
// Domain tests - NO Bevy/NATS dependencies
#[test]
fn test_domain_logic() {
    // Pure domain logic only
}

// ECS tests - Headless mode
#[test]
fn test_ecs_system() {
    let mut app = App::new();
    app.add_systems(Update, system_under_test);
    // BEVY_HEADLESS=1 required
}

// Integration tests - Full stack
#[tokio::test]
async fn test_integration() {
    // NATS + Bevy bridge tests
}
```

## Summary

- **Total Stories**: 20 existing + 5 new = 25
- **Fully Tested**: 12/25 (48%)
- **Partially Tested**: 6/25 (24%)
- **Not Tested**: 7/25 (28%)
- **Current Coverage**: ~65%
- **Required Coverage**: 95%
- **Failing Tests**: 13 (must be 0)

To achieve TDD compliance, we need to:
1. Fix 13 failing tests
2. Add ~30 missing tests
3. Implement missing handlers
4. Reach 95% coverage
