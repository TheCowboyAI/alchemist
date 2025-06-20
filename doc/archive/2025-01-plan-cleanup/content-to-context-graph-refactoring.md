# ContentGraph to ContextGraph Refactoring Plan

## Overview

This refactoring renames `ContentGraph` to `ContextGraph` to better reflect its true purpose: representing bounded contexts with a designated context root entity that serves as the semantic anchor.

## Refactoring Steps

### Phase 1: Core Domain Model Updates

1. **Rename the aggregate**
   - `src/domain/aggregates/content_graph.rs` → `context_graph.rs`
   - `ContentGraph` struct → `ContextGraph`
   - Add `context_root: NodeId` field
   - Add `context_type: ContextType` field

2. **Update value objects**
   - `ContentNode` → `ContextNode`
   - `ContentEdge` → `ContextEdge`
   - `NodeContent` → `NodeContext` (represents context within a node)

3. **Update events**
   - `src/domain/events/content_graph.rs` → `context_graph.rs`
   - `ContentGraphCreated` → `ContextGraphCreated`
   - `ContentAdded` → `ContextAdded`
   - Add `context_root` field to creation event

4. **Update commands**
   - `src/domain/commands/content_graph_commands.rs` → `context_graph_commands.rs`
   - `ContentGraphCommand` → `ContextGraphCommand`
   - `CreateContentGraph` → `CreateContextGraph`

### Phase 2: Infrastructure Updates

1. **Event mappings**
   - Update subject router mappings
   - Update event store handlers
   - Update NATS subject patterns

2. **Persistence layer**
   - Update repository interfaces
   - Update projection names

### Phase 3: Application Layer Updates

1. **Command handlers**
   - Rename command handler functions
   - Update command processing logic

2. **Query handlers**
   - Update query handler names
   - Update projection queries

### Phase 4: Presentation Layer Updates

1. **Bevy components**
   - Update component names
   - Update system names

2. **Events**
   - Update presentation event names

### Phase 5: Test Updates

1. **Unit tests**
   - Update test names
   - Update test data builders

2. **Integration tests**
   - Update integration test scenarios

## File Mapping

| Old File | New File |
|----------|----------|
| `content_graph.rs` | `context_graph.rs` |
| `content_graph_events.rs` | `context_graph_events.rs` |
| `content_graph_commands.rs` | `context_graph_commands.rs` |

## Type Mapping

| Old Type | New Type |
|----------|----------|
| `ContentGraph` | `ContextGraph` |
| `ContentNode` | `ContextNode` |
| `ContentEdge` | `ContextEdge` |
| `NodeContent` | `NodeContext` |
| `ContentGraphError` | `ContextGraphError` |

## Event Mapping

| Old Event | New Event |
|-----------|-----------|
| `ContentGraphCreated` | `ContextGraphCreated` |
| `ContentAdded` | `ContextAdded` |
| `ContentRemoved` | `ContextRemoved` |

## Command Mapping

| Old Command | New Command |
|-------------|-------------|
| `CreateContentGraph` | `CreateContextGraph` |
| `AddContent` | `AddContext` |
| `RemoveContent` | `RemoveContext` |

## Implementation Order

1. Create new files with Context naming
2. Copy and update code from Content files
3. Update all imports
4. Update all references
5. Remove old Content files
6. Run tests and fix any issues

## Validation Checklist

- [ ] All Content* types renamed to Context*
- [ ] context_root field added to ContextGraph
- [ ] All events updated
- [ ] All commands updated
- [ ] All tests passing
- [ ] Documentation updated
- [ ] Examples updated

## Benefits After Refactoring

1. **Clearer semantics** - ContextGraph better represents bounded contexts
2. **Root-centric design** - Every context has a clear entry point
3. **Better DDD alignment** - Matches aggregate root pattern
4. **Improved navigation** - Start from root to explore context
5. **Recursive clarity** - Nested contexts with their own roots
