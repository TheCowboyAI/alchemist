# Graph Domain Event-Driven Architecture Fix

## Critical Issue Addressed

**User Feedback**: "we don't do 'CRUD' We do Events, Events don't do 'updates' they throw events with a difference of what we add and remove from the component or entity."

## Problem Identified

The initial Graph domain implementation incorrectly included CRUD-style "update" operations that violated the event-sourcing architecture principles:

❌ **Anti-Patterns Removed:**
- `UpdateNode` commands for changing metadata
- `NodeUpdated` events 
- Direct mutation of value objects
- Single events for state changes

## Solution Implemented

### 1. Command Changes
**Before:**
```rust
UpdateNode {
    node_id: NodeId,
    metadata: HashMap<String, Value>,
}
```

**After:**
```rust
ChangeNodeMetadata {
    node_id: NodeId,
    new_metadata: HashMap<String, Value>,  // Replaces ALL metadata
}
```

### 2. Event Changes
**Before:**
```rust
NodeUpdated {
    node_id: NodeId,
    metadata: HashMap<String, Value>,
}
```

**After:**
```rust
// Sequence of two events:
NodeRemoved { node_id }
NodeAdded { node_id, node_type, new_metadata }
```

### 3. Aggregate Changes
**Before:**
```rust
pub fn update_node(&mut self, node_id: NodeId, metadata: HashMap<String, Value>) {
    let node = self.nodes.get_mut(&node_id).unwrap();
    node.metadata = metadata;  // Direct mutation!
}
```

**After:**
```rust
pub fn change_node_metadata(&mut self, node_id: NodeId, new_metadata: HashMap<String, Value>) {
    // Remove old node entirely
    let old_node = self.nodes.remove(&node_id).unwrap();
    
    // Create completely new node with new metadata
    let new_node = GraphNode::new(node_id, old_node.node_type, new_metadata);
    self.nodes.insert(node_id, new_node);
}
```

## Event Sourcing Principles Enforced

### 1. Value Object Immutability
✅ **Never mutate** - Always replace entirely
✅ **Complete replacement** - New objects, not partial updates
✅ **Event sequence** - Remove old, add new

### 2. Audit Trail Integrity
✅ **Two events** show exactly what changed:
   - `NodeRemoved` - What was taken away
   - `NodeAdded` - What was added back
✅ **Complete history** - No hidden state changes
✅ **Replay capability** - Events can reconstruct exact state

### 3. Domain Event Semantics
✅ **Events are facts** - "This happened" not "This changed"
✅ **Past tense** - "NodeAdded" not "AddNode"
✅ **Immutable records** - Events never change after creation

## Architecture Benefits

### 1. True Event Sourcing
- Perfect audit trail of all changes
- Ability to replay events to any point in time
- No hidden state mutations

### 2. Value Object Consistency
- Components/entities are never partially updated
- All changes are atomic and complete
- No inconsistent intermediate states

### 3. Domain Clarity
- Events represent business facts, not technical operations
- Clear separation between commands (intent) and events (facts)
- Business logic in aggregates, not in event handlers

## Test Coverage

All **37 tests passing** with the new event-driven implementation:
- Command handling tests verify proper event generation
- Aggregate tests confirm value object immutability
- Projection tests handle remove/add event sequences
- Integration tests verify end-to-end consistency

## Implementation Pattern

This pattern should be applied to **all domains** in CIM:

```rust
// ❌ NEVER DO THIS
fn update_entity(&mut self, changes: Changes) {
    self.field = changes.field;  // Mutation!
}

// ✅ ALWAYS DO THIS
fn change_entity(&mut self, new_data: EntityData) -> Vec<DomainEvent> {
    // 1. Remove old entity
    let old = self.entities.remove(&id);
    
    // 2. Create new entity
    let new = Entity::new(id, new_data);
    self.entities.insert(id, new);
    
    // 3. Return remove + add events
    vec![
        DomainEvent::EntityRemoved { id },
        DomainEvent::EntityAdded { id, data: new_data }
    ]
}
```

## Verification

The fix ensures CIM strictly follows event-sourcing principles:
- ✅ No CRUD operations
- ✅ No partial updates
- ✅ No value object mutations
- ✅ Complete audit trail
- ✅ Proper event semantics

This architectural correction is fundamental to CIM's event-driven foundation and ensures consistency across all domains. 