# Fix Domain Model for CIM Compliance

## Overview
The `demo_cim_rules_violations` has revealed critical violations of CIM principles in our current domain model. This plan outlines the necessary refactoring to bring our model into compliance with Event Sourcing and DDD best practices.

## Violations Found

### 1. Update Commands (CRITICAL)
- ❌ `GraphCommand::UpdateNode` - Allows partial updates with optional fields
- ❌ `NodeCommand::MoveNode` - Misleading name suggesting mutation
- ❌ `EdgeCommand::UpdateEdge` - Same partial update problem

### 2. Mutation Events (CRITICAL)
- ❌ `NodeEvent::NodeMoved` - Implies position was "updated"
- ❌ `NodeEvent::NodeContentChanged` - Suggests value object mutation
- ❌ `EdgeEvent::EdgeUpdated` - Violates immutability

### 3. Missing Validation (HIGH)
- ❌ Position3D accepts NaN, Infinity values
- ❌ No validation on node types
- ❌ No validation on edge relationships

### 4. CRUD-style Commands (MEDIUM)
- ❌ Commands named after database operations
- ❌ No business intent in command names

## Refactoring Plan

### Phase 1: Remove Update Commands and Events

#### 1.1 GraphCommand Refactoring
```rust
// REMOVE these commands:
- UpdateNode { node_id, new_position, new_content }

// No replacement needed - use Remove + Add pattern
```

#### 1.2 NodeCommand Refactoring
```rust
// REMOVE:
- MoveNode { node_id, position }

// REPLACE WITH business-intent commands:
+ RepositionNodeInWorkflow { node_id, position, reason }
+ PlaceNodeInLayout { node_id, position, layout_strategy }
```

#### 1.3 Event Refactoring
```rust
// REMOVE these events:
- NodeMoved { old_position, new_position }
- NodeContentChanged { old_content, new_content }
- EdgeUpdated { old_relationship, new_relationship }

// Events should only be:
- NodeRemoved { node_id, position, content }
- NodeAdded { node_id, position, content }
- EdgeRemoved { edge_id, source, target, relationship }
- EdgeAdded { edge_id, source, target, relationship }
```

### Phase 2: Add Proper Validation

#### 2.1 Position3D Validation
```rust
impl Position3D {
    pub fn new(x: f32, y: f32, z: f32) -> Result<Self, DomainError> {
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return Err(DomainError::InvalidPosition(
                "Position coordinates must be finite values".to_string()
            ));
        }
        Ok(Self { x, y, z })
    }
}
```

#### 2.2 NodeType Validation
```rust
impl NodeType {
    pub fn new(value: String) -> Result<Self, DomainError> {
        if value.is_empty() {
            return Err(DomainError::InvalidNodeType("Node type cannot be empty"));
        }
        if value.len() > 100 {
            return Err(DomainError::InvalidNodeType("Node type too long"));
        }
        Ok(Self(value))
    }
}
```

### Phase 3: Implement CIM-Compliant Patterns

#### 3.1 Position Change Pattern
```rust
// To change a node's position:
pub fn reposition_node(&mut self, node_id: NodeId, new_position: Position3D) -> Result<Vec<DomainEvent>> {
    // 1. Validate node exists
    let node = self.nodes.get(&node_id)
        .ok_or(DomainError::NodeNotFound)?;

    // 2. Store complete node state
    let old_position = node.position.clone();
    let content = node.content.clone();
    let node_type = node.node_type.clone();

    // 3. Remove the node
    self.nodes.remove(&node_id);

    // 4. Add node at new position
    self.nodes.insert(node_id, Node {
        position: new_position.clone(),
        content: content.clone(),
        node_type: node_type.clone(),
    });

    // 5. Generate proper events
    Ok(vec![
        DomainEvent::Node(NodeEvent::NodeRemoved {
            graph_id: self.id,
            node_id,
            position: old_position,
            content: content.clone(),
            node_type: node_type.clone(),
        }),
        DomainEvent::Node(NodeEvent::NodeAdded {
            graph_id: self.id,
            node_id,
            position: new_position,
            content,
            node_type,
        }),
    ])
}
```

#### 3.2 Business-Intent Commands
```rust
pub enum WorkflowCommand {
    PlaceStepInWorkflow {
        workflow_id: WorkflowId,
        step_id: NodeId,
        position: Position3D,
        step_type: WorkflowStepType,
    },
    ConnectWorkflowSteps {
        workflow_id: WorkflowId,
        from_step: NodeId,
        to_step: NodeId,
        condition: Option<TransitionCondition>,
    },
    MarkStepCompleted {
        workflow_id: WorkflowId,
        step_id: NodeId,
        completion_time: SystemTime,
        output: serde_json::Value,
    },
}
```

### Phase 4: Update Tests

#### 4.1 Create Failing Tests First
- Test that NaN positions are rejected
- Test that empty node types are rejected
- Test that update commands don't exist
- Test that events are properly paired (Remove + Add)

#### 4.2 Update Existing Tests
- Remove tests for update commands
- Update tests to use Remove/Add pattern
- Add validation tests

### Phase 5: Update Documentation

#### 5.1 Update Domain Model Documentation
- Document why we don't have update commands
- Explain the Remove/Add pattern
- Show examples of business-intent commands

#### 5.2 Update API Documentation
- Remove references to update operations
- Add migration guide for existing code

## Implementation Order

1. **Add validation to value objects** (Position3D, NodeType, etc.)
2. **Create new business-intent commands**
3. **Remove update commands from enums**
4. **Remove mutation events from enums**
5. **Update aggregate methods to use Remove/Add pattern**
6. **Update all tests**
7. **Update documentation**
8. **Run all demos to verify compliance**

## Success Criteria

- [ ] `demo_cim_rules_violations` shows all violations as fixed
- [ ] No "Update" or "Change" commands in the codebase
- [ ] No "Moved" or "Changed" events in the codebase
- [ ] All value objects have proper validation
- [ ] All commands express business intent
- [ ] All tests pass with new patterns
- [ ] Documentation reflects CIM principles

## Timeline

- Phase 1-2: 2 hours (Critical fixes)
- Phase 3: 3 hours (Pattern implementation)
- Phase 4: 2 hours (Test updates)
- Phase 5: 1 hour (Documentation)

Total: ~8 hours of focused work

## Notes

This is a breaking change that will require updates to any code using the domain model. However, it's essential for maintaining the integrity of our Event Sourcing architecture and ensuring that our system can evolve correctly over time.
