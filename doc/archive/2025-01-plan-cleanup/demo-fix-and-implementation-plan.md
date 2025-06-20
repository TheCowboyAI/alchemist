# Demo Fix and Implementation Plan

## Current Status
- Total Demos Planned: 75
- Completed (but broken): 4 (5.3%)
- Need to implement: 71 (94.7%)

## Phase 0: Fix Existing Demos (Priority 1)

All 4 existing demos have compilation errors due to API changes:

### 1. demo_nats_connection
**Issues:**
- NatsConfig structure changed (auth/tls moved to SecurityConfig)
- JetStreamConfig import path changed
- jetstream() method now returns Result instead of Option

**Fix:**
- Update config structure to use SecurityConfig
- Fix import paths
- Handle Result type properly

### 2. demo_event_persistence
**Issues:**
- Same NatsConfig issues
- DistributedEventStore::new() now requires 2 arguments
- append_events() method removed
- GraphCreated is not a direct DomainEvent variant
- get_events() and get_latest_cid() methods removed

**Fix:**
- Update config structure
- Add DistributedEventStoreConfig parameter
- Use proper event store methods
- Use GraphEvent::GraphCreated variant

### 3. demo_graph_create
**Issues:**
- Same NatsConfig and DistributedEventStore issues
- Graph::new() now requires 3 arguments (name, description)
- metadata field expects HashMap not Option
- apply_event() returns () not Result
- append_events() method removed
- Graph structure changed (no description/tags fields)

**Fix:**
- Update all constructor calls
- Fix metadata initialization
- Remove error handling for apply_event
- Use proper event store methods

### 4. demo_conceptual_space_create
**Issues:**
- Same NatsConfig and DistributedEventStore issues
- ConceptualSpace::new() now requires 4 arguments
- append_events() method removed
- concept_mappings field doesn't exist

**Fix:**
- Update constructor with all required arguments
- Use proper event store methods
- Remove references to non-existent fields

## Phase 1: Infrastructure Demos (14 remaining)

### Priority Order:
1. **demo_async_sync_bridge** - Critical for Bevy integration
2. **demo_event_bridge** - Critical for event flow
3. **demo_cid_chain** - Event integrity
4. **demo_event_replay** - Event sourcing foundation
5. **demo_nats_publish** - Basic messaging
6. **demo_nats_subscribe** - Basic messaging
7. **demo_nats_request_reply** - Sync communication
8. **demo_event_deduplication** - Event processing
9. **demo_snapshot_restore** - State recovery
10. **demo_object_store_put** - Large object storage
11. **demo_object_store_get** - Large object retrieval
12. **demo_object_store_cid** - Content addressing
13. **demo_backpressure** - Flow control
14. **demo_jetstream_consumer** - Production messaging
15. **demo_jetstream_ack** - Message acknowledgment
16. **demo_jetstream_replay** - Historical replay

## Phase 2: Domain Model Demos (25 remaining)

### Graph Operations (7 demos):
1. **demo_node_add** - Basic graph operation
2. **demo_node_remove** - Node deletion
3. **demo_node_update** - Node modification
4. **demo_node_move** - Position changes
5. **demo_edge_connect** - Edge creation
6. **demo_edge_disconnect** - Edge removal
7. **demo_edge_relationship** - Relationship types
8. **demo_edge_validation** - Business rules
9. **demo_graph_load** - Loading from events
10. **demo_graph_snapshot** - State persistence

### Workflow Operations (5 demos):
11. **demo_workflow_create** - Workflow initialization
12. **demo_workflow_step** - Step management
13. **demo_workflow_transition** - State transitions
14. **demo_workflow_execution** - Running workflows
15. **demo_workflow_state** - State tracking

### Conceptual Space Operations (5 demos):
16. **demo_dimension_add** - Quality dimensions
17. **demo_concept_embed** - Concept positioning
18. **demo_region_form** - Category creation
19. **demo_similarity_search** - Semantic search

### ConceptGraph Operations (4 demos):
20. **demo_concept_graph_create** - Graph initialization
21. **demo_concept_mapping** - Concept relationships
22. **demo_semantic_navigation** - Path finding

### Subgraph Operations (4 demos):
23. **demo_subgraph_extract** - Subgraph extraction
24. **demo_subgraph_merge** - Combining subgraphs
25. **demo_subgraph_split** - Dividing subgraphs
26. **demo_subgraph_transform** - Transformations

## Phase 3: Integration Patterns (10 demos)

1. **demo_event_choreography** - Multi-context flows
2. **demo_event_orchestration** - Centralized coordination
3. **demo_saga_pattern** - Distributed transactions
4. **demo_cqrs_projection** - Read model updates
5. **demo_event_replay_projection** - Rebuilding state
6. **demo_context_bridge** - Cross-context communication
7. **demo_anti_corruption_layer** - External integration
8. **demo_event_transformation** - Message translation
9. **demo_aggregate_coordination** - Multi-aggregate operations
10. **demo_eventual_consistency** - Consistency patterns

## Phase 4: IA Dogfooding (12 demos)

1. **demo_ia_self_graph** - Visualize IA's own structure
2. **demo_ia_event_stream** - Show IA's event flow
3. **demo_ia_development_workflow** - Development process
4. **demo_ia_concept_space** - IA's conceptual model
5. **demo_ia_context_map** - Bounded contexts
6. **demo_ia_aggregate_view** - Domain aggregates
7. **demo_ia_projection_status** - Read model health
8. **demo_ia_performance_metrics** - System metrics
9. **demo_ia_error_analysis** - Error patterns
10. **demo_ia_evolution_timeline** - Development history
11. **demo_ia_dependency_graph** - Module dependencies
12. **demo_ia_test_coverage** - Testing visualization

## Phase 5: Advanced Composition (11 demos)

1. **demo_multi_graph_composition** - Combining graphs
2. **demo_workflow_composition** - Workflow chaining
3. **demo_concept_space_merge** - Space combination
4. **demo_event_stream_merge** - Stream composition
5. **demo_projection_composition** - Composite views
6. **demo_context_composition** - Context integration
7. **demo_rule_composition** - Business rule chains
8. **demo_visualization_composition** - UI composition
9. **demo_performance_composition** - Optimization patterns
10. **demo_error_composition** - Error handling chains
11. **demo_test_composition** - Test scenario building

## Implementation Strategy

### For Each Demo:
1. Create single-responsibility binary in src/bin/
2. Include comprehensive documentation
3. Show both success and failure cases
4. Make it runnable in headless mode
5. Include performance metrics
6. Add to integration test suite

### Common Patterns:
- Use builder pattern for complex setups
- Include timing and performance output
- Show event flow visually when possible
- Demonstrate error recovery
- Include cleanup/teardown

### Testing Requirements:
- Each demo must pass in CI/CD
- Must work with BEVY_HEADLESS=1
- Should complete in < 30 seconds
- Must handle missing NATS gracefully

## Success Metrics
- All 75 demos compile and run
- Each demo clearly demonstrates its feature
- Demos can be composed together
- IA uses these patterns internally
- Documentation auto-generated from demos

## Timeline
- Week 1: Fix existing 4 demos + implement 14 infrastructure demos
- Week 2: Implement 25 domain model demos
- Week 3: Implement 10 integration pattern demos
- Week 4: Implement 12 dogfooding demos
- Week 5: Implement 11 advanced composition demos + final testing
