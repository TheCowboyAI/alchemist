# Phase 1 & 2 Comprehensive Testing Plan

## Overview

This plan ensures all Phase 1 and Phase 2 features are fully tested and demonstrated in the IA application, not just as isolated demos.

## Phase 1: Distributed Event Infrastructure

### 1.1 NATS Integration Foundation
**Status**: Completed
**Tests Required**:
- [x] NATS client connection and health checks
- [x] JWT authentication
- [x] TLS configuration
- [x] Event bridge bidirectional flow
- [ ] **Demo**: Show NATS connection status in UI

### 1.2 JetStream Event Store
**Status**: Completed
**Tests Required**:
- [x] Event persistence to JetStream
- [x] Event retrieval by aggregate ID
- [x] Stream configuration
- [ ] **Demo**: Store and retrieve graph events through UI

### 1.3 CID Chain Implementation
**Status**: Completed
**Tests Required**:
- [x] CID generation for events
- [x] Chain validation
- [x] Tampering detection
- [ ] **Demo**: Show CID chain visualization in UI

### 1.4 Event Replay Mechanism
**Status**: Completed
**Tests Required**:
- [x] Replay from specific CID
- [x] Replay from timestamp
- [ ] **Demo**: UI controls for event replay

## Phase 2: Graph Domain Model

### 2.1 Graph Aggregate
**Status**: Completed
**Tests Required**:
- [x] Graph creation, update, deletion
- [x] Business rule validation
- [x] Max capacity limits
- [ ] **Demo**: Create and manipulate graphs in UI

### 2.2 Node/Edge Entities
**Status**: Completed
**Tests Required**:
- [x] Node CRUD operations
- [x] Edge connections
- [x] Cascade delete
- [ ] **Demo**: Interactive node/edge manipulation

### 2.3 Workflow Aggregate
**Status**: Completed
**Tests Required**:
- [x] Workflow state machine
- [x] Step types and transitions
- [ ] Workflow execution
- [ ] **Demo**: Visual workflow designer

### 2.4 Conceptual Space Aggregate
**Status**: Completed
**Tests Required**:
- [ ] Quality dimension management
- [ ] Concept mapping
- [ ] Region definition
- [ ] Similarity calculations
- [ ] **Demo**: Conceptual space visualization

### 2.5 ConceptGraph Implementation
**Status**: Completed
**Tests Required**:
- [x] Graph composition operations
- [x] Category theory morphisms
- [ ] **Demo**: Graph composition UI

### 2.6 Subgraph Operations
**Status**: Completed
**Tests Required**:
- [x] Subgraph creation and management
- [x] Spatial mapping
- [ ] Merge/split operations
- [ ] Collapse/expand
- [ ] **Demo**: Interactive subgraph operations

## Required Demos for Main Application

### 1. Graph Building Demo
- Create a new graph
- Add nodes with different types
- Connect nodes with edges
- Apply force-directed layout
- Save to NATS event store
- Show CID chain

### 2. ConceptGraph Demo
- Create a ConceptGraph with quality dimensions
- Apply category theory operations
- Compose multiple graphs
- Show morphism transformations

### 3. ConceptualSpace Demo
- Create a conceptual space
- Add quality dimensions
- Map concepts to positions
- Calculate similarities
- Define regions

### 4. Workflow Designer Demo
- Design a workflow visually
- Add different step types
- Connect steps
- Validate workflow
- Execute workflow
- Show state transitions

### 5. Subgraph Operations Demo
- Create subgraphs
- Merge multiple subgraphs
- Split subgraph with gesture
- Collapse/expand subgraphs
- Drag and drop operations

### 6. Event Sourcing Demo
- Show event log
- Replay from point in time
- Show CID chain integrity
- Demonstrate event-driven updates

### 7. Import/Export Demo
- Import from markdown/Mermaid
- Import from JSON
- Export graph to various formats
- Show imported graph visualization

## Implementation Priority

1. **Critical** (Must have for Phase completion):
   - [ ] Fix all compilation errors in tests
   - [ ] Ensure all aggregates have working command handlers
   - [ ] Verify event persistence and replay
   - [ ] Create UI controls for basic operations

2. **Important** (Core functionality):
   - [ ] Implement missing ConceptualSpace operations
   - [ ] Complete workflow execution
   - [ ] Add subgraph merge/split UI
   - [ ] Create demo mode with presets

3. **Nice to have** (Polish):
   - [ ] Performance optimizations
   - [ ] Advanced visualizations
   - [ ] Export capabilities
   - [ ] Undo/redo functionality

## Test Execution Plan

### Week 1: Fix Compilation and Basic Tests
- Fix all compilation errors
- Ensure all unit tests pass
- Create missing unit tests for untested features

### Week 2: Integration Tests
- Test full command → event → projection flow
- Test NATS integration end-to-end
- Test UI → domain → infrastructure flow

### Week 3: Feature Demos
- Implement UI controls for all features
- Create demo scenarios
- Document demo workflows

### Week 4: Polish and Documentation
- Performance testing
- User documentation
- Video demonstrations
- Final QA pass

## Success Criteria

- [ ] All tests compile and pass (100%)
- [ ] Test coverage > 80%
- [ ] All Phase 1 & 2 features accessible from UI
- [ ] Demo mode shows all capabilities
- [ ] Documentation complete
- [ ] No critical bugs
- [ ] Performance acceptable (< 100ms response time)

## Current Blockers

1. Compilation errors in test files
2. Missing UI controls for many features
3. ConceptualSpace operations not fully implemented
4. Workflow execution incomplete
5. No demo mode or presets
