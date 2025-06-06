# Value Object Immutability in DDD and Event Sourcing

## Overview

In Domain-Driven Design (DDD) with Event Sourcing, value objects are immutable by definition. This document explains how we handle value object changes in our architecture and why we never have "update" events for value objects.

## Core Principle

**Value Objects CANNOT be "updated" - they are replaced entirely.**

## Why This Matters

1. **Events are immutable facts** - They record what happened, not what changed
2. **Value Objects have no lifecycle** - They exist or don't exist, no in-between
3. **Clear event semantics** - Removal and addition are distinct business events
4. **Audit trail integrity** - Shows the complete replacement, not a partial mutation

## Implementation Pattern

### ❌ Wrong Approach - Update Events

```rust
// DON'T DO THIS
pub enum EdgeEvent {
    EdgeUpdated {
        edge_id: EdgeId,
        old_relationship: EdgeRelationship,
        new_relationship: EdgeRelationship,
    },
}

pub enum NodeEvent {
    NodeMoved {
        node_id: NodeId,
        old_position: Position3D,
        new_position: Position3D,
    },
}
```

### ✅ Correct Approach - Remove/Add Pattern

```rust
// DO THIS INSTEAD
pub enum EdgeEvent {
    EdgeRemoved { edge_id: EdgeId },
    EdgeAdded {
        edge_id: EdgeId,
        source: NodeId,
        target: NodeId,
        relationship: EdgeRelationship,
    },
}

pub enum NodeEvent {
    NodeRemoved { node_id: NodeId },
    NodeAdded {
        node_id: NodeId,
        position: Position3D,
        metadata: HashMap<String, Value>,
    },
}
```

## Examples

### Changing a Node's Position

```rust
// To move a node from one position to another:
let events = vec![
    DomainEvent::Node(NodeEvent::NodeRemoved { graph_id, node_id }),
    DomainEvent::Node(NodeEvent::NodeAdded {
        graph_id,
        node_id,
        position: new_position,
        metadata,
    }),
];
```

### Changing an Edge's Relationship

```rust
// To change an edge's relationship type:
let events = vec![
    DomainEvent::Edge(EdgeEvent::EdgeDisconnected {
        graph_id,
        edge_id: old_edge_id,
        source,
        target,
    }),
    DomainEvent::Edge(EdgeEvent::EdgeConnected {
        graph_id,
        edge_id: EdgeId::new(), // New ID for the new value object
        source,
        target,
        relationship: new_relationship,
    }),
];
```

## Helper Patterns

We provide helper functions in `domain::services::value_object_patterns` to make this pattern easier to implement:

```rust
use crate::domain::services::ValueObjectChangePatterns;

// Change node position
let events = ValueObjectChangePatterns::change_node_position(
    graph_id,
    node_id,
    old_position,
    new_position,
    metadata,
);

// Change edge relationship
let events = ValueObjectChangePatterns::change_edge_relationship(
    graph_id,
    old_edge_id,
    source,
    target,
    new_relationship,
);
```

## Entity vs Value Object Distinction

### Entities (Can be Updated)
- Have unique identity (ID)
- Have lifecycle (created, modified, deleted)
- Examples: Graph, Node (if it has NodeId), Edge (if it has EdgeId)

### Value Objects (Must be Replaced)
- No unique identity
- Compared by value
- Immutable after creation
- Examples: Position3D, EdgeRelationship, NodeContent, GraphMetadata

## Metadata Updates

Note that updating individual metadata keys is acceptable because we're not replacing the entire metadata collection:

```rust
// This is OK - updating individual keys in a collection
NodeMetadataUpdated {
    key: String,
    old_value: Option<Value>,
    new_value: Option<Value>,
}
```

## Benefits

1. **Clear Event Stream**: Each event represents a distinct business action
2. **Better Auditing**: Can see exactly when objects were removed and recreated
3. **Simpler Event Handlers**: No need to handle partial updates
4. **Consistency**: Aligns with DDD principles of immutable value objects

## Testing

Always test that value object changes generate the correct remove/add event pairs:

```rust
#[test]
fn test_position_change_generates_remove_add_events() {
    let events = change_node_position(...);

    assert_eq!(events.len(), 2);
    assert!(matches!(events[0], DomainEvent::Node(NodeEvent::NodeRemoved { .. })));
    assert!(matches!(events[1], DomainEvent::Node(NodeEvent::NodeAdded { .. })));
}
```

## Summary

When working with value objects in our event-sourced system:
- Never create "update" events for value objects
- Always use the remove/add pattern
- Generate new IDs when creating replacement value objects
- Use the helper functions in `value_object_patterns` module
- Test that changes generate the correct event pairs

This ensures our event stream accurately reflects the immutable nature of value objects and maintains the integrity of our Event Sourcing implementation.
