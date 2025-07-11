# Renderer Integration Tests Summary

This document describes the comprehensive integration tests for the Alchemist renderer system located in `tests/renderer_integration_tests.rs`.

## Test Coverage

### 1. Renderer Manager Lifecycle (`test_renderer_manager_lifecycle`)
- Tests spawning new renderer windows (Bevy and Iced)
- Verifies renderer process management
- Tests listing active renderers
- Tests closing renderer windows
- Validates cleanup of renderer processes

### 2. Communication System (`test_renderer_communication`)
- Tests NATS-based communication between main process and renderers
- Verifies command sending to specific renderers
- Tests dialog-specific commands
- Tests broadcast commands to all renderers
- Tests renderer health checks (ping)

### 3. Renderer API (`test_renderer_api`)
- Tests renderer registration/unregistration
- Verifies command routing through the API
- Tests event handling from renderers
- Validates error handling for missing renderers

### 4. Renderer Type Selection (`test_different_renderer_types`)
- Tests automatic renderer type selection based on content
- Verifies Bevy is selected for 3D content (graphs, scenes)
- Verifies Iced is selected for 2D content (documents, dialogs, charts)

### 5. Data Serialization (`test_render_data_serialization`)
- Tests serialization/deserialization of all RenderData types:
  - Graph3D with nodes and edges
  - Documents with various formats
  - Dialogs with message history
  - Charts with data and options
  - Markdown content with themes
- Ensures data integrity through serialization round-trips

### 6. Concurrent Operations (`test_concurrent_renderers`)
- Tests spawning multiple renderers simultaneously
- Verifies manager handles concurrent operations correctly
- Tests mixed renderer types (Bevy and Iced) running together

### 7. Window State Management (`test_window_state_management`)
- Tests different window configurations:
  - Custom sizes and positions
  - Fullscreen mode
  - Resizable/non-resizable windows
  - Always-on-top windows
- Validates RenderConfig options are properly handled

### 8. Error Handling (`test_error_handling`)
- Tests closing non-existent renderers
- Tests updating data for missing renderers
- Tests handling of missing renderer binary
- Validates proper error messages and recovery

### 9. Dialog Event Flow (`test_dialog_event_flow`)
- Tests complete dialog interaction workflow:
  - System prompt updates
  - User message events
  - Loading state management
  - Token streaming for AI responses
  - Stream completion handling
- Simulates realistic dialog UI interactions

### 10. Process Cleanup (`test_cleanup_dead_processes`)
- Tests automatic cleanup of terminated renderer processes
- Validates process tracking and removal
- Ensures no zombie processes remain

### 11. Helper Functions (`test_renderer_helper_functions`)
- Tests convenience methods for spawning specific renderer types
- Validates helper functions for creating dialog messages
- Tests all specialized spawn methods (graph, document, markdown, chart)

### 12. NATS Client Integration (`test_renderer_client`)
- Tests renderer-side NATS client functionality
- Validates renderer registration with main process
- Tests event sending from renderer to main process
- Simulates renderer client lifecycle

## Mock Components

The tests include several mock components to simulate real renderer behavior:

### MockRenderer
- Simulates a renderer process without requiring the actual binary
- Allows testing of process management logic
- Can be extended to simulate various renderer behaviors

### Test Helpers
- `setup_test_nats()`: Creates a test NATS connection
- Assumes a local NATS server for integration testing
- Gracefully skips tests if NATS is unavailable

## Test Execution

### Prerequisites
- Local NATS server running on `nats://localhost:4222` (for communication tests)
- Tests gracefully handle missing dependencies

### Running the Tests
```bash
# Run all renderer integration tests
cargo test renderer_integration_tests

# Run specific test
cargo test test_renderer_manager_lifecycle

# Run with output
cargo test renderer_integration_tests -- --nocapture
```

## Key Features Tested

1. **Multi-Renderer Support**: Tests both Bevy (3D) and Iced (2D) renderers
2. **Async Communication**: Full async/await support with proper timeout handling
3. **Error Recovery**: Comprehensive error handling and graceful degradation
4. **Concurrent Operations**: Tests parallel renderer operations
5. **Event-Driven Architecture**: Tests event flow between processes
6. **Resource Cleanup**: Ensures proper cleanup of system resources

## Notes for CI/CD

- Tests handle missing renderer binaries gracefully
- NATS-dependent tests skip if server unavailable
- All tests use timeouts to prevent hanging
- Mock components allow testing without full system

## Future Enhancements

1. Add performance benchmarks for renderer spawning
2. Test resource usage with many concurrent renderers
3. Add stress tests for event streaming
4. Test renderer crash recovery
5. Add integration with actual renderer binaries in CI