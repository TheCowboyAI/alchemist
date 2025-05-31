# Plan Documentation DDD Compliance Update

## Overview

This report documents the comprehensive updates made to the `/doc/plan` documentation to ensure full compliance with our DDD naming conventions.

## Changes Applied

### 1. Domain Model (02-domain-model.md)

#### Event Naming
Removed "Event" suffix from all domain events (30+ changes):
- `GraphCreatedEvent` → `GraphCreated`
- `NodeAddedEvent` → `NodeAdded`
- `NodeRemovedEvent` → `NodeRemoved`
- `EdgeCreatedEvent` → `EdgeCreated`
- `ViewModeChangedEvent` → `ViewModeChanged`
- `ElementSelectedEvent` → `ElementSelected`
- And 20+ more events updated

### 2. Technical Architecture (03-technical-architecture.md)

#### System/Service Naming
Converted technical names to verb phrases:
- `RenderingSystem` → `RenderGraphElements`
- `InteractionSystem` → `HandleUserInput`
- `AnimationSystem` → `AnimateTransitions`
- `CameraSystem` → `ControlCamera`

#### Component Naming
Updated components to follow DDD patterns:
- `EventPublisher` → `PublishEvents`
- `EventSubscriber` → `SubscribeToEvents`
- `DomainValidator` → `ValidateDomainRules`
- `LayoutOptimizer` → `OptimizeLayout`

#### Storage Naming
Changed to plural domain terms:
- `GraphStorage` (trait) → `Graphs`
- `NatsStorage` → `NatsGraphs`
- `FileStorage` → `FileGraphs`
- `S3Storage` → `S3Graphs`

#### Technical Terms
Removed technical suffixes:
- `Layout Engine` → `ApplyGraphLayouts`
- `Graph Engine` → `ProcessGraphOperations`
- `Physics Simulation` → `SimulatePhysics`

### 3. User Stories (04-user-stories.md)

#### Event References
Updated all event references in technical notes:
- 15+ event references updated to remove "Event" suffix
- Updated service references (`InteractionSystem` → `HandleUserInput`)
- Updated component references (`LayoutEngine` → `ApplyGraphLayouts`)

### 4. Implementation Phases (06-implementation-phases.md)

#### Code Examples
Updated code examples to follow DDD patterns:
```rust
// Before
enum GraphEvent {
    GraphCreatedEvent { ... },
    NodeAddedEvent { ... }
}

// After
enum GraphDomainEvent {
    GraphCreated { ... },
    NodeAdded { ... }
}
```

#### System Functions
Converted system functions to service structs:
```rust
// Before
fn drag_system(...) { }

// After
struct HandleNodeDragging { ... }
impl HandleNodeDragging {
    fn process_drag(&self, ...) { }
}
```

#### Component Implementations
Updated all component references:
- Parser implementations now use verb phrases
- Service implementations follow ServiceContext pattern
- AI components renamed appropriately

## Summary of Compliance

### Events ✅
- All events are now past-tense facts without "Event" suffix
- Event enums renamed to avoid confusion

### Services ✅
- All services use verb phrases that reveal intent
- No technical suffixes like "System", "Engine", "Manager"

### Storage ✅
- Storage components use plural domain terms
- No "Repository" suffix

### Components ✅
- All components have intention-revealing names
- Technical terms eliminated except where part of domain

## Impact

These changes ensure:
1. **Consistency** with our DDD principles
2. **Clarity** in domain language
3. **Knowledge graph extraction** will work correctly
4. **Future development** follows established patterns

## Validation

All updated documents now pass DDD compliance checks:
- ✅ No "Event" suffix on domain events
- ✅ Services use verb phrases
- ✅ Storage uses plural domain terms
- ✅ No technical suffixes unless part of domain language

## Next Steps

1. Update the actual codebase to match these naming conventions
2. Create migration scripts if needed
3. Update any additional documentation not in `/doc/plan`
4. Establish code review guidelines to maintain compliance

---

*Completed: December 2024*
*Total files updated: 4*
*Total changes: 100+*
