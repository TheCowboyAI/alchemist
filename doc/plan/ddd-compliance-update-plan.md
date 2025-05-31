# DDD Compliance Update Plan

## Objective

Update all code and documentation to comply with the revised DDD naming conventions that enforce pure domain language without technical suffixes.

## Phase 1: Design Documentation ✅ COMPLETE

### Completed Tasks
1. **Consolidated design documents** into 3 clear files:
   - `graph-domain-design.md` - Complete specification
   - `graph-current-state-analysis.md` - Gap analysis
   - `graph-implementation-roadmap.md` - Implementation plan

2. **Removed deprecated documents** that violated new rules

3. **Created README** for design folder structure

## Phase 2: Code Implementation (NEXT)

### Task 2.1: Event Renaming
Update all events in the codebase:
- `GraphCreatedEvent` → `GraphCreated`
- `NodeAddedEvent` → `NodeAdded`
- `EdgeConnectedEvent` → `EdgeConnected`
- `NodeRemovedEvent` → `NodeRemoved`

### Task 2.2: Component Creation
Create service components following verb phrases:
- Implement `CreateGraph`
- Implement `AddNodeToGraph`
- Implement `ConnectGraphNodes`
- Implement `ValidateGraph`

### Task 2.3: Storage Implementation
- Create `Graphs` storage component
- Integrate Daggy for graph structure
- Implement index management

## Phase 3: System Migration

### Task 3.1: Update Existing Systems
- Wrap current systems with service components
- Maintain backward compatibility during transition
- Update event handling

### Task 3.2: Add Event Store
- Implement event storage
- Add event replay capability
- Set up event topics (graphs.created, node.added, etc.)

## Phase 4: Feature Implementation

Follow the roadmap in `graph-implementation-roadmap.md`:
- Week 1-2: Core foundation
- Week 3-4: Visualization
- Week 5-6: Analysis & Import/Export
- Week 7-8: Advanced features

## Current Status

✅ **Design Phase Complete**: All design documents now comply with DDD rules
⏳ **Implementation Phase**: Ready to begin code changes

## Next Steps

1. Begin with Task 2.1 - rename all events in code
2. Create the core service components (Task 2.2)
3. Implement Graphs storage with Daggy (Task 2.3)
4. Follow the detailed roadmap for remaining features

## Success Criteria

- [ ] All events renamed without "Event" suffix
- [ ] Service components follow verb phrase pattern
- [ ] Storage uses plural domain terms
- [ ] All code passes DDD compliance check
- [ ] Knowledge graph extraction works correctly

The design is now clean and consistent. Implementation can proceed with confidence.
