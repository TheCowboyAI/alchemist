# Test Inventory by Domain Categories

## Overview

This document provides a comprehensive inventory of all tests in the Information Alchemist project, organized by domain categories following our DDD structure.

**Total Test Count**: 114 tests (106 passing, 8 failing)

## Domain Categories

### 1. Graph Management Domain (Core)

#### Service Tests (`src/contexts/graph_management/tests.rs`)
- `test_create_graph_service` - Tests graph creation with metadata
- `test_add_node_to_graph` - Tests node addition to graphs
- `test_connect_nodes` - Tests edge creation between nodes
- `test_graph_validation_node_limit` - Tests node limit constraints
- `test_graph_validation_prevents_self_loops` - Tests self-loop prevention
- `test_graph_validation_prevents_duplicate_edges` - Tests duplicate edge prevention
- `test_establish_hierarchy_system` - Tests parent-child hierarchy

#### Repository Tests (`src/contexts/graph_management/tests.rs`)
- `test_graphs_repository` - Tests graph storage and retrieval
- `test_graph_events_repository` - Tests event storage and retrieval
- `test_nodes_repository` - Tests node repository operations
- `test_edges_repository` - Tests edge repository operations

#### Domain Model Tests (`src/contexts/graph_management/tests.rs`)
- `test_identity_creation` - Tests unique identity generation
- `test_spatial_position_creation` - Tests 3D position creation
- `test_graph_journey_defaults` - Tests journey initialization
- `test_metadata_structure` - Tests metadata validation
- `test_node_content_properties` - Tests node content structure
- `test_edge_relationship` - Tests edge relationship structure

#### Import/Export Tests (`src/contexts/graph_management/tests.rs`)
- `test_export_graph_to_json` - Tests JSON serialization
- `test_export_to_file` - Tests file writing
- `test_json_round_trip` - Tests data preservation
- `test_export_with_special_characters` - Tests special character handling

#### Storage Tests (`src/contexts/graph_management/storage.rs`)
- `test_create_graph` - Tests graph creation in storage
- `test_add_nodes` - Tests node addition to storage
- `test_connect_nodes` - Tests edge creation in storage
- `test_get_nodes` - Tests node retrieval
- `test_get_edges` - Tests edge retrieval
- `test_remove_graph` - Tests graph removal
- `test_error_handling` - Tests error conditions

### 2. Selection Domain (Supporting)

#### Selection State Tests (`src/contexts/selection/tests.rs`)
- `test_selection_state_initialization` - Tests initial state
- `test_node_selection` - Tests node selection/deselection
- `test_edge_selection` - Tests edge selection/deselection
- `test_clear_selection` - Tests clearing all selections
- `test_selection_mode_changes` - Tests mode transitions

#### Selection Event Tests (`src/contexts/selection/tests.rs`)
- `test_select_node_event_single_mode` - Tests single selection mode
- `test_select_node_event_multiple_mode` - Tests multiple selection mode
- `test_selection_cleared_event` - Tests clear event handling
- `test_deselect_node_event` - Tests deselection events
- `test_select_all_event` (ignored) - Tests select all functionality
- `test_selection_changed_event_fired` - Tests change notifications

#### Box Selection Tests (`src/contexts/selection/tests.rs`)
- `test_selection_box_creation` - Tests box creation
- `test_selection_box_bounds` - Tests boundary calculation
- `test_selection_box_contains` - Tests point containment
- `test_box_selection_with_scaled_nodes` (ignored) - Tests with transforms

#### Selection Visual Tests (`src/contexts/selection/tests.rs`)
- `test_selection_highlight_default` - Tests highlight defaults
- `test_selection_highlight_no_highlight_in_storage` (ignored) - Tests highlight storage

#### Ray Intersection Tests (`src/contexts/selection/tests.rs`)
- `test_ray_sphere_intersection_with_scale` - Tests scaled sphere intersection

### 3. Visualization Domain (Supporting)

#### Render Mode Tests (`src/contexts/visualization/tests.rs`)
- `test_render_mode_defaults` - Tests default render modes
- `test_render_modes` - Tests all render mode types
- `test_node_visualization_with_different_render_modes` (ignored) - Tests mode rendering

#### Visualization Component Tests (`src/contexts/visualization/tests.rs`)
- `test_visualization_capability_defaults` - Tests capability defaults
- `test_point_cloud_component_creation` - Tests point cloud creation
- `test_edge_point_cloud_component` - Tests edge point clouds
- `test_node_point_cloud_generation` - Tests node cloud generation
- `test_edge_point_cloud_generation` - Tests edge cloud generation
- `test_point_cloud_generation` - Tests general cloud generation

#### Edge Visualization Tests (`src/contexts/visualization/tests.rs`)
- `test_edge_types` - Tests edge type enumeration
- `test_edge_visual_defaults` - Tests edge visual defaults
- `test_edge_visual_bundle` - Tests edge component bundles
- `test_edge_animation_missing` - Tests animation components exist
- `test_edge_animation_components_dont_exist` - Verifies animation implementation
- `test_edge_animation_systems_exist` - Tests animation systems
- `test_edge_animations` - Tests edge animation behavior

#### Animation Tests (`src/contexts/visualization/tests.rs`)
- `test_graph_motion_defaults` - Tests motion defaults
- `test_animation_components` - Tests animation component structure

#### Input Handling Tests (`src/contexts/visualization/tests.rs`)
- `test_edge_type_keyboard_controls` - Tests edge type key bindings
- `test_render_mode_keyboard_controls` - Tests render mode key bindings
- `test_keyboard_controls_not_integrated` - Documents keyboard issues
- `test_visualization_state_update_systems` - Tests state updates

#### Camera Tests (`src/contexts/visualization/tests.rs`)
- `test_camera_setup` - Tests camera initialization
- `test_closest_hit_selection` - Tests hit detection

#### Visual Feedback Tests (`src/contexts/visualization/tests.rs`)
- `test_selection_visual_feedback` - Tests selection highlighting
- `test_raycasting_sphere_intersection` - Tests ray-sphere intersection

#### Layout Tests (`src/contexts/visualization/layout.rs`)
- `test_force_directed_config_defaults` - Tests layout configuration
- `test_layout_state_initialization` - Tests layout state
- `test_force_calculation` - Tests force physics
- `test_layout_convergence` - Tests layout stabilization

### 4. Event Store Domain (Infrastructure)

#### Event Store Tests (`src/contexts/event_store/store.rs`)
- `test_event_store_creation` - Tests store initialization
- `test_append_event` - Tests event appending
- `test_get_events_for_aggregate` - Tests event retrieval
- `test_event_sequence_numbers` - Tests sequence ordering
- `test_parent_cid_linking` - Tests Merkle DAG structure
- `test_traverse_from_cid` - Tests event traversal
- `test_cid_computation` - Tests content addressing

#### Event Sourcing Tests (`src/testing/event_sourcing_tests.rs`)
- `test_event_store_basic_operations` - Tests basic CRUD
- `test_event_capture_system` (ignored) - Tests event capture
- `test_merkle_dag_parent_linking` - Tests parent linking
- `test_event_replay_system` (ignored) - Tests replay functionality
- `test_event_traversal` - Tests event chain traversal
- `test_cid_determinism` - Tests CID consistency
- `test_event_store_plugin_initialization` (ignored) - Tests plugin setup
- `test_replay_missing_payload_error` - Tests error handling

#### Performance Tests (`src/testing/event_sourcing_tests.rs`)
- `test_event_store_scaling` - Tests performance with 1000 events

### 5. Core/Infrastructure Tests

#### Test Configuration (`src/test_config.rs`)
- `test_minimal_plugins_setup` - Tests minimal Bevy setup

#### Event Validation Helpers (`src/testing/event_validation_helpers.rs`)
- `test_event_builder_graph_created` - Tests graph event building
- `test_event_builder_node_added` - Tests node event building
- `test_event_builder_edge_connected` - Tests edge event building

#### Integration Tests (`src/testing/integration_tests.rs`)
- `test_graph_creation_integration` - Tests full graph creation flow
- `test_node_addition_integration` - Tests node addition flow
- `test_edge_connection_integration` - Tests edge connection flow

#### Headless Integration (`src/testing/headless_integration_test.rs`)
- `test_headless_graph_creation` - Tests headless mode operation

#### Repository Integration (`src/testing/repository_integration_tests.rs`)
- Various repository integration tests

#### Core Tests (`src/core/tests.rs`)
- `test_graph_identity_uniqueness` - Tests UUID generation
- `test_node_identity_creation` - Tests node ID creation
- `test_edge_identity_creation` - Tests edge ID creation

### 6. Testing Infrastructure

#### TDD Compliance Tests (`src/testing/tdd_compliant_ecs_tests.rs`)
- Various TDD-focused tests ensuring proper test structure

#### Domain Isolation Tests (`src/testing/domain_isolated_tests.rs`)
- Tests ensuring domain logic is isolated from Bevy dependencies

#### Feature Tests (`src/testing/feature_tests.rs`)
- End-to-end feature verification tests

#### Performance Tests (`src/testing/performance_tests.rs`)
- Performance benchmarks and scaling tests

## Test Status Summary

### By Domain
| Domain | Total | Passing | Failing | Ignored |
|--------|-------|---------|---------|---------|
| Graph Management | 28 | 28 | 0 | 0 |
| Selection | 23 | 20 | 0 | 3 |
| Visualization | 30 | 29 | 0 | 1 |
| Event Store | 16 | 11 | 0 | 5 |
| Core/Infrastructure | 17 | 17 | 0 | 0 |
| **Total** | **114** | **105** | **0** | **9** |

### Known Issues
1. **Event Store Integration**: 5 tests ignored due to event adapter integration issues
2. **Selection Tests**: 3 tests ignored due to animation/transform integration
3. **Visualization**: 1 test ignored due to mesh component handling

### Test Infrastructure Status
- ✅ Tests compile with Bevy main branch
- ✅ MinimalPlugins configuration working
- ✅ Headless mode enabled
- ⚠️ Some integration tests need event system fixes
- ⚠️ Animation-related tests need transform system updates

## Next Steps

1. **Fix Ignored Tests**: Focus on the 9 ignored tests
   - Event adapter integration (5 tests)
   - Animation/transform integration (3 tests)
   - Mesh component handling (1 test)

2. **Add Missing Coverage**:
   - UI interaction tests
   - File dialog integration tests
   - Multi-graph management tests
   - Performance optimization tests

3. **Improve Test Infrastructure**:
   - Better Bevy system mocking
   - Event simulation helpers
   - Transform animation test utilities

---
*Last Updated*: December 2024
*Test Runner*: Cargo with BEVY_HEADLESS=1
