# Comprehensive Demo Plan for Information Alchemist

## Overview

Information Alchemist (IA) is a dogfooding application - it uses the exact same CIM patterns it enables users to create. Every feature must be demonstrated in a composable, single-responsibility manner that serves as both test and example.

## Demo Organization Principles

1. **Single Responsibility**: Each demo focuses on ONE feature
2. **Composable**: Demos can be combined to test complex scenarios
3. **Self-Documenting**: Each demo includes Mermaid diagrams
4. **Production-Ready**: Demo code is example implementation code
5. **Testable**: Each demo verifies its feature works correctly

## Phase 1: Infrastructure Foundation Demos

### Core NATS Integration
- [x] `demo_nats_connection` - Basic NATS connectivity
- [ ] `demo_nats_publish` - Publishing messages
- [ ] `demo_nats_subscribe` - Subscribing to subjects
- [ ] `demo_nats_request_reply` - Request/Reply pattern

### Event Store & Persistence
- [x] `demo_event_persistence` - Basic event storage
- [ ] `demo_cid_chain` - CID chain integrity
- [ ] `demo_event_replay` - Replaying event streams
- [ ] `demo_event_deduplication` - Duplicate prevention
- [ ] `demo_snapshot_restore` - Aggregate snapshots

### Object Store Integration
- [ ] `demo_object_store_put` - Storing objects
- [ ] `demo_object_store_get` - Retrieving objects
- [ ] `demo_object_store_cid` - CID-based storage

### Async/Sync Bridge
- [ ] `demo_async_sync_bridge` - Command bridging
- [ ] `demo_event_bridge` - Event bridging
- [ ] `demo_backpressure` - Flow control

### JetStream Features
- [ ] `demo_jetstream_consumer` - Pull consumer
- [ ] `demo_jetstream_ack` - Message acknowledgment
- [ ] `demo_jetstream_replay` - Stream replay

## Phase 2: Domain Model Demos

### Graph Aggregate
- [x] `demo_graph_create` - Basic graph creation
- [ ] `demo_graph_load` - Loading from events
- [ ] `demo_graph_snapshot` - Snapshotting

### Node Operations
- [ ] `demo_node_add` - Adding nodes
- [ ] `demo_node_remove` - Removing nodes
- [ ] `demo_node_update` - Updating node content
- [ ] `demo_node_move` - Moving nodes

### Edge Operations
- [ ] `demo_edge_connect` - Connecting nodes
- [ ] `demo_edge_disconnect` - Disconnecting nodes
- [ ] `demo_edge_relationship` - Edge relationships
- [ ] `demo_edge_validation` - Edge constraints

### Workflow Aggregate
- [ ] `demo_workflow_create` - Creating workflows
- [ ] `demo_workflow_step` - Adding steps
- [ ] `demo_workflow_transition` - Transitions
- [ ] `demo_workflow_execution` - Running workflows
- [ ] `demo_workflow_state` - State tracking

### Conceptual Space
- [x] `demo_conceptual_space_create` - Basic creation
- [ ] `demo_dimension_add` - Adding dimensions
- [ ] `demo_concept_embed` - Embedding concepts
- [ ] `demo_region_form` - Forming regions
- [ ] `demo_similarity_search` - Finding similar concepts

### ConceptGraph Integration
- [ ] `demo_concept_graph_create` - Creating concept graphs
- [ ] `demo_concept_mapping` - Mapping to visual space
- [ ] `demo_semantic_navigation` - Navigating by meaning

### Subgraph Operations
- [ ] `demo_subgraph_extract` - Extracting subgraphs
- [ ] `demo_subgraph_merge` - Merging graphs
- [ ] `demo_subgraph_split` - Splitting graphs
- [ ] `demo_subgraph_transform` - Transforming subgraphs

## Phase 3: Integration Pattern Demos

### Cross-Aggregate Operations
- [ ] `demo_graph_workflow_link` - Linking graphs to workflows
- [ ] `demo_workflow_concept_mapping` - Workflow semantic mapping
- [ ] `demo_multi_aggregate_transaction` - Coordinated updates

### Event-Driven Patterns
- [ ] `demo_event_choreography` - Event-driven workflow
- [ ] `demo_saga_pattern` - Distributed transactions
- [ ] `demo_event_sourced_projection` - Building projections

### Query Patterns
- [ ] `demo_graph_query` - Querying graphs
- [ ] `demo_workflow_query` - Querying workflows
- [ ] `demo_concept_query` - Semantic queries
- [ ] `demo_cross_aggregate_query` - Complex queries

## Phase 4: IA Dogfooding Demos

### Self-Visualization
- [ ] `demo_ia_self_graph` - IA visualizing its own structure
- [ ] `demo_ia_event_flow` - Visualizing event flows
- [ ] `demo_ia_concept_space` - IA's semantic space

### Workflow Building
- [ ] `demo_ia_workflow_designer` - Visual workflow design
- [ ] `demo_ia_workflow_test` - Testing workflows
- [ ] `demo_ia_workflow_deploy` - Deploying workflows

### Development Tools
- [ ] `demo_ia_event_inspector` - Event debugging
- [ ] `demo_ia_aggregate_explorer` - Aggregate state viewer
- [ ] `demo_ia_performance_monitor` - Performance tracking

### AI Integration
- [ ] `demo_ia_semantic_search` - Semantic code search
- [ ] `demo_ia_concept_suggestion` - Concept recommendations
- [ ] `demo_ia_workflow_optimization` - AI-driven optimization

## Phase 5: Advanced Composition Demos

### Complex Scenarios
- [ ] `demo_distributed_workflow` - Multi-context workflow
- [ ] `demo_event_sourced_cqrs` - Full CQRS implementation
- [ ] `demo_temporal_queries` - Time-travel queries
- [ ] `demo_concept_evolution` - Concept space learning

### Performance Patterns
- [ ] `demo_batch_operations` - Batch processing
- [ ] `demo_parallel_execution` - Parallel systems
- [ ] `demo_cache_optimization` - Caching strategies

### Error Handling
- [ ] `demo_error_recovery` - Error recovery patterns
- [ ] `demo_compensation` - Compensation logic
- [ ] `demo_circuit_breaker` - Circuit breaker pattern

## Implementation Strategy

### 1. Demo Structure
Each demo follows this structure:
```rust
// src/bin/demo_feature_name.rs
//! # Feature Name Demo
//!
//! Demonstrates: [specific feature]
//! Dependencies: [other demos it builds on]
//!
//! ## Mermaid Diagram
//! [diagram showing what happens]

async fn main() -> Result<()> {
    // 1. Setup
    // 2. Execute feature
    // 3. Verify results
    // 4. Display results
}
```

### 2. Composability Pattern
```rust
// Demos can be composed
async fn complex_scenario() {
    // Use individual demo functions
    let graph = demo_graph_create::create_graph().await?;
    let workflow = demo_workflow_create::create_workflow().await?;
    demo_graph_workflow_link::link(graph, workflow).await?;
}
```

### 3. Testing Pattern
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_demo_succeeds() {
        // Each demo must have tests
    }
}
```

## Success Criteria

1. **Coverage**: Every Phase 1 & 2 feature has a demo
2. **Composability**: Demos can be combined
3. **Documentation**: Each demo has clear docs & diagrams
4. **Testability**: All demos have tests
5. **Dogfooding**: IA uses these patterns internally

## Next Steps

1. Complete remaining Phase 1 infrastructure demos
2. Implement all Phase 2 domain model demos
3. Create integration pattern demos
4. Build IA dogfooding demos
5. Document composition patterns

## Progress Tracking

See `/doc/progress/demo-implementation-status.md` for current status of each demo.
