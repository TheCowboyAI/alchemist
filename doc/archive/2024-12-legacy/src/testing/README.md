# Testing Guide

This directory contains various test suites for the Alchemist project.

## Running Tests

### Run all tests (excluding ignored tests)
```bash
cargo test
```

### Run ignored tests only
```bash
cargo test -- --ignored
```

### Run all tests including ignored ones
```bash
cargo test -- --include-ignored
```

### Run specific test module
```bash
cargo test event_sourcing_tests
```

### Run with output
```bash
cargo test -- --nocapture
```

## Test Categories

### Feature Tests (`feature_tests.rs`)
These tests document missing features and are expected to fail with `#[should_panic]`. They serve as documentation for what features are claimed but not implemented.

### Integration Tests (`integration_tests.rs`)
End-to-end workflow tests that verify complete user scenarios. Many are marked as `#[ignore]` because they depend on unimplemented features.

### Performance Tests (`performance_tests.rs`)
Tests that verify performance claims. Most use `#[should_panic]` to document that performance optimizations are not implemented.

### Event Sourcing Tests (`event_sourcing_tests.rs`)
Tests for the event store and Merkle DAG functionality. Some are marked as `#[ignore]` due to integration issues.

### TDD Compliant ECS Tests (`tdd_compliant_ecs_tests.rs`)
Tests following strict TDD practices for Bevy ECS components. Some event handling tests are marked as `#[ignore]`.

### Headless Integration Test (`headless_integration_test.rs`)
Tests for UI interactions in headless mode. The UI interaction test is marked as `#[ignore]`.

## Ignored Tests

The following tests are currently marked as `#[ignore]` and need fixes:

1. **Selection Tests**
   - `test_select_all_event` - Event handling issues
   - `test_selection_with_animated_transforms` - Animated transforms integration

2. **Visualization Tests**
   - `test_node_visualization_with_different_render_modes` - Mesh component handling
   - `test_animation_components` - Time resource handling
   - `test_calculate_force_directed_layout_with_edge` - Edge handling in layout

3. **Event Sourcing Tests**
   - `test_event_capture_system` - Event adapter integration
   - `test_event_replay_system` - Replay system integration
   - `test_event_store_plugin_initialization` - Plugin initialization

4. **Other Tests**
   - `test_ui_interaction` - UI interaction in headless mode
   - `test_json_round_trip_preserves_all_data` - Import/export functionality
   - `test_graph_creation_event` - Event handling in TDD tests
   - `test_node_addition_event` - Event handling in TDD tests

## Running Tests in Nix

Since we're using NixOS with direnv:

```bash
# Ensure you're in the dev shell
direnv allow

# Run tests with nix
nix build .#checks.x86_64-linux.test

# Or use cargo in the dev shell
cargo test
```

## Test Coverage

To check test coverage (requires additional tooling):

```bash
cargo tarpaulin --out Html
```

## Debugging Failed Tests

1. Run the specific test with output:
   ```bash
   cargo test test_name -- --nocapture
   ```

2. Use `RUST_BACKTRACE=1` for stack traces:
   ```bash
   RUST_BACKTRACE=1 cargo test test_name
   ```

3. For Bevy-specific issues, ensure `BEVY_HEADLESS=1` is set:
   ```bash
   BEVY_HEADLESS=1 cargo test
   ```
