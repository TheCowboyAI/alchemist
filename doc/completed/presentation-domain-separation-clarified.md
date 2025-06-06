# Presentation vs Domain Event Separation - Clarified

**Date**: January 2, 2025
**Status**: Documentation Complete, Implementation Planned

## What We Clarified

### Core Principle: NOT EVERY EVENT IS A DOMAIN EVENT

We established a critical architectural distinction:

1. **Presentation Events** - Stay entirely within Bevy:
   - Animation frames (60fps updates)
   - Mouse movements and drag operations
   - Force-directed layout iterations
   - UI state changes (menus, selections)
   - Visual-only transformations

2. **Domain Events** - Persisted to NATS/Event Store:
   - Graph structure changes (GraphCreated, NodeAdded, EdgeConnected)
   - Business state transitions
   - Model recognition (K7 identified, morphism applied)
   - User-confirmed saves

### Key Insight: Aggregation Pattern

Instead of sending every UI operation to the domain, we aggregate changes:

```rust
// Presentation: Many operations
- User drags node (100s of position updates)
- Force-directed layout runs (1000s of iterations)
- Animations play (60fps)

// Domain: Single meaningful event
- NodesPositioned { positions: final_positions }
```

## Graph Models and Structure Preservation

### Recognized Models
We defined a taxonomy of graph models our system understands:

1. **Mathematical Graphs**
   - Complete graphs (Kn) - K3, K4, K5, K7
   - Cycle graphs (Cn) - C3, C4, C5

2. **State Machines**
   - Mealy machines (output depends on state + input)
   - Moore machines (output depends only on state)

3. **Domain-Specific Models**
   - Address graphs (represent address value objects)
   - Workflow graphs (business processes)
   - Concept graphs (knowledge representation)

### Structure-Preserving Morphisms
We can transform between compatible models while preserving their essential properties:
- ToComplete - Add missing edges
- ToCycle - Form cycle structure
- SubdivideEdges - Replace edges with paths
- MealyToMoore - Convert between state machine types

## Benefits Achieved

1. **Performance**: Animations run at 60fps without flooding event store
2. **Clarity**: Domain events represent only business-meaningful changes
3. **Flexibility**: UI can experiment freely before committing
4. **Recognition**: System understands and can work with known structures
5. **Transformation**: Powerful graph operations via morphisms

## What's Next

We created a comprehensive refactoring plan (`/doc/plan/refactor-to-presentation-domain-separation.md`) with 6 phases:

1. Event Classification - Separate presentation from domain events
2. Graph Model Recognition - Implement pattern matching
3. Structure-Preserving Morphisms - Enable transformations
4. Presentation Layer Refactoring - Add aggregation patterns
5. Testing Updates - Comprehensive test coverage
6. Documentation Updates - User guides and examples

## Key Deliverables

1. **Design Documents**:
   - `/doc/design/presentation-vs-domain-events.md` - Core separation principles
   - `/doc/design/graph-models-and-morphisms.md` - Mathematical foundation

2. **Implementation Plan**:
   - `/doc/plan/refactor-to-presentation-domain-separation.md` - Detailed roadmap

3. **Progress Tracking**:
   - Updated `/doc/progress/progress.json` with new milestone

## Impact on Development

This clarification provides the foundation for:
- Clean domain model focused on business logic
- High-performance UI with rich interactions
- Model-based graph operations
- Clear boundaries between layers
- Preparation for AI agent integration

The architecture now properly separates concerns while maintaining the power and flexibility needed for our graph manipulation and workflow design goals.
