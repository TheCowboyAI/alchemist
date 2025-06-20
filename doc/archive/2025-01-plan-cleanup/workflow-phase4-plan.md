# Workflow Phase 4 Implementation Plan

## Overview

Phase 4 integrates the workflow system with Bevy presentation layer and NATS messaging, enabling real-time visualization and distributed execution of workflows.

## Current State

### Completed Components:
1. **cim-domain/workflow**: Core workflow traits, state machines, aggregates
2. **cim-contextgraph/workflow_graph**: Graph-based workflow representation
3. **presentation/components/workflow_visualization**: Bevy components for workflow visualization
4. **presentation/systems/workflow_visualization**: Bevy systems for rendering workflows

### Missing Integration:
1. NATS subject mapping for workflow events
2. Workflow execution service connecting domain to NATS
3. Bridge between workflow domain events and Bevy visualization
4. Real-time workflow state synchronization

## Implementation Tasks

### 1. NATS Subject Mapping for Workflows

Create workflow-specific subject patterns:

```rust
// src/infrastructure/nats/workflow_subjects.rs
pub struct WorkflowSubjectMapper {
    // Maps workflow events to NATS subjects
    // Pattern: workflow.{workflow_id}.{event_type}
}
```

### 2. Workflow Execution Service

Create service to handle workflow execution via NATS:

```rust
// src/application/services/workflow_execution.rs
pub struct WorkflowExecutionService {
    // Manages workflow instances
    // Processes transitions via NATS messages
    // Publishes workflow events
}
```

### 3. Workflow Event Bridge

Connect workflow domain events to Bevy:

```rust
// src/infrastructure/event_bridge/workflow_bridge.rs
pub struct WorkflowEventBridge {
    // Translates workflow domain events to presentation events
    // Updates Bevy components based on workflow state changes
}
```

### 4. Workflow Command Handler

Process workflow commands from UI:

```rust
// src/application/command_handlers/workflow_command_handler.rs
pub struct WorkflowCommandHandler {
    // Handles StartWorkflow, ExecuteTransition, etc.
    // Integrates with event store
}
```

### 5. Example Workflow Implementation

Create a concrete workflow example:

```rust
// examples/approval_workflow.rs
// Document approval workflow with:
// - States: Draft, Review, Approved, Rejected
// - Transitions with guards
// - NATS integration
// - Bevy visualization
```

## Success Criteria

1. ✅ Workflows can be started and executed via NATS messages
2. ✅ Workflow state changes are visualized in real-time in Bevy
3. ✅ Transitions respect guards and produce appropriate events
4. ✅ Multiple workflow instances can run concurrently
5. ✅ Example demonstrates end-to-end workflow execution

## Timeline

- Task 1-2: NATS integration (2 hours)
- Task 3-4: Event bridge and command handling (2 hours)
- Task 5: Example implementation and testing (1 hour)
