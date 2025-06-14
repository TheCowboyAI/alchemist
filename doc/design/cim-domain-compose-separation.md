# CIM Domain vs CIM Compose - Separation of Responsibilities

## Overview

This document clarifies the distinct responsibilities of `cim-domain` and `cim-compose` to eliminate overlap and confusion.

## Core Principle

- **`cim-domain`**: Provides the fundamental DDD building blocks (Entity, Aggregate, ValueObject, etc.)
- **`cim-compose`**: Provides graph-based composition of those building blocks

## Dependency Flow

The correct dependency flow is:

```
┌─────────────────┐
│   cim-domain    │  (Core DDD types)
└────────┬────────┘
         │
         ├──────────────┬──────────────┬──────────────┐
         ▼              ▼              ▼              ▼
┌─────────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│ cim-domain-     │ │ cim-domain- │ │ cim-domain- │ │ cim-domain- │
│ document        │ │ graph       │ │ person      │ │ workflow    │
└─────────────────┘ └─────────────┘ └─────────────┘ └─────────────┘
         ▲              ▲              ▲              ▲
         └──────────────┴──────────────┴──────────────┘
                                │
                        ┌───────▼────────┐
                        │  cim-compose   │  (Composition layer)
                        └────────────────┘
```

**Key Points:**
- Domain modules (document, graph, person, etc.) depend ONLY on `cim-domain`
- Domain modules do NOT depend on `cim-compose`
- `cim-compose` depends on domain modules to provide composition capabilities
- This prevents circular dependencies and maintains clean architecture

## cim-domain: Core DDD Building Blocks

### Purpose
Defines the fundamental Domain-Driven Design patterns and types that all domains use.

### Responsibilities
1. **Core Types**
   - `Entity<T>` - Types with identity and lifecycle
   - `AggregateRoot<T>` - Consistency boundaries
   - `EntityId<T>` - Type-safe identifiers with phantom types
   - Value Objects - Immutable types without identity

2. **CQRS Infrastructure**
   - `Command` trait - Intent to change state
   - `Query` trait - Request for data
   - `DomainEvent` trait - Things that happened
   - `CommandHandler` trait - Process commands
   - `QueryHandler` trait - Process queries
   - `EventHandler` trait - React to events

3. **State Machines**
   - Moore and Mealy machine implementations
   - State transitions and guards
   - Event outputs

4. **Infrastructure Abstractions**
   - `EventStore` trait
   - `Repository` trait
   - `EventPublisher` trait
   - `ReadModel` trait

### What It Does NOT Do
- Does NOT implement specific domain logic
- Does NOT know about graphs, workflows, or agents
- Does NOT compose elements together
- Does NOT provide concrete implementations (except test doubles)

## cim-compose: Graph-Based Composition

### Purpose
Provides the ability to compose domain elements into graph structures using category theory principles.

### Responsibilities
1. **Graph Structures**
   - `GraphComposition<N, R>` - The main composition type
   - `CompositionNode<N>` - Nodes in the graph
   - `CompositionEdge<R>` - Edges/relationships
   - `NodeId` and `EdgeId` - Local identifiers within graphs

2. **Composition Operations**
   - Sequential composition (`then`)
   - Parallel composition (`parallel`)
   - Choice composition (`choice`)
   - Custom composition patterns

3. **Category Theory Operations**
   - `GraphMorphism` - Transform one graph to another
   - `GraphFunctor` - Map functions over graphs
   - `GraphMonad` - Wrap graphs in contexts
   - `Composable` trait - Define composition rules

4. **Composition Types**
   - `Atomic` - Single node graphs
   - `Composite` - Multi-node structures
   - `Functor` - Transformation graphs
   - `Monad` - Contextual graphs
   - `Domain` - DDD-specific compositions

5. **Domain Compositions** (Feature-gated)
   - Implements `Composable` trait for domain aggregates
   - Provides domain-specific composition utilities
   - Enables knowledge graph construction

### What It Does NOT Do
- Does NOT define what an Entity or Aggregate is
- Does NOT handle commands, events, or queries
- Does NOT implement domain logic
- Does NOT know about specific domains (without features enabled)

## Domain Modules: Concrete Implementations

### Purpose
Implement specific domain logic using both `cim-domain` building blocks and `cim-compose` patterns.

### Examples

#### cim-domain-graph
```rust
use cim_domain::{AggregateRoot, Entity, Command, DomainEvent};

// Uses Entity from cim-domain
pub struct GraphAggregate {
    entity: Entity<GraphMarker>,
    // ...
}

// Pure domain logic - no composition dependencies
impl GraphAggregate {
    pub fn add_node(&mut self, node: Node) -> Result<Vec<DomainEvent>> {
        // Business logic here
    }
}
```

#### cim-compose (with graph feature)
```rust
// cim-compose depends on domain modules
use cim_domain_graph::GraphAggregate;
use cim_compose::{GraphComposition, Composable};

// Composition logic lives in cim-compose
impl Composable for GraphAggregate {
    fn to_graph(&self) -> GraphComposition {
        // Convert domain aggregate to graph representation
    }
}
```

## Migration Plan

### Phase 1: Clean up cim-compose ✅
1. Remove duplicate `Entity` and `EntityId` definitions
2. Import these from `cim-domain` instead
3. Keep only graph-specific types (NodeId, EdgeId, etc.)

### Phase 2: Update domain modules ✅
1. Ensure all domain modules import core types from `cim-domain`
2. Remove any dependencies on `cim-compose` from domain modules
3. Add domain modules as dependencies to `cim-compose` (with features)

### Phase 3: Documentation ✅
1. Update all module documentation
2. Create examples showing proper usage
3. Add compile-fail tests for improper usage

## Usage Examples

### Correct Usage
```rust
// In a domain module (e.g., cim-domain-document)
use cim_domain::{Entity, AggregateRoot, Command, DomainEvent};

pub struct DocumentAggregate {
    // Uses Entity from cim-domain
    entity: Entity<DocumentMarker>,
    components: ComponentStorage,
}

// Pure domain logic
impl DocumentAggregate {
    pub fn ingest_content(&mut self, content: Vec<u8>) -> Result<Vec<DomainEvent>> {
        // Business logic only
    }
}
```

```rust
// In cim-compose (with document feature enabled)
use cim_domain_document::DocumentAggregate;
use cim_compose::{GraphComposition, Composable};

impl Composable for DocumentAggregate {
    fn to_graph(&self) -> GraphComposition {
        let mut graph = GraphComposition::aggregate("Document", self.id().to_string());

        // Add nodes for document components
        if let Some(info) = self.get_component::<DocumentInfo>() {
            graph = graph.add_node(/* ... */);
        }

        graph
    }
}
```

### Incorrect Usage (Anti-patterns)
```rust
// ❌ DON'T: Domain module depending on cim-compose
// In cim-domain-document/Cargo.toml
[dependencies]
cim-compose = { path = "../cim-compose" } // WRONG!

// ❌ DON'T: Composition logic in domain module
impl DocumentAggregate {
    fn to_graph(&self) -> GraphComposition { // WRONG - this belongs in cim-compose
        // ...
    }
}

// ❌ DON'T: Make cim-domain depend on cim-compose
// Core types should not know about graph composition
```

## Benefits of This Separation

1. **Clear Responsibilities**: Each crate has a single, well-defined purpose
2. **No Circular Dependencies**: Clean dependency flow from core → domains → composition
3. **Reusability**: Can use domain modules without graph composition
4. **Flexibility**: Can compose any domain type into graphs
5. **Type Safety**: Phantom types and traits ensure correct usage
6. **Testability**: Each layer can be tested independently
7. **Feature Flags**: Selective compilation of domain compositions

## Summary

- **cim-domain**: "What things ARE" (Entity, Aggregate, Event, etc.)
- **Domain modules**: "What things DO" (Business logic, specific implementations)
- **cim-compose**: "How things COMBINE" (Graphs, Morphisms, Composition)

This separation follows the Single Responsibility Principle and enables clean, maintainable architecture.
