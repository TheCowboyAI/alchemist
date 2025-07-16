# CLAUDE.md for cim-domain

This file provides guidance to Claude Code (claude.ai/code) when working with the cim-domain repository - the core Domain Model for the Composable Information Machine (CIM).

## Critical Context

**cim-domain is the HEART of CIM** - it defines the foundational domain-driven architecture that all other domains and components depend on. This is our core module that MUST maintain 100% test coverage and production readiness.

## Project Overview

**cim-domain** is the core domain modeling framework for the Composable Information Machine. It provides:
- Event-driven architecture foundation with CQRS implementation
- Domain-Driven Design (DDD) primitives (Aggregates, Commands, Events, Queries)
- Category Theory-based inter-domain communication
- Event sourcing with NATS JetStream integration
- Cross-domain integration patterns
- Production-ready infrastructure for building domains

## Architecture Principles

### Core Design Patterns
1. **Event-First Design** - Everything is an event, no CRUD operations
2. **CQRS Pattern** - Complete separation of commands and queries
3. **Domain Isolation** - No shared state between domains
4. **Category Theory** - Functors, morphisms, and natural transformations for domain communication
5. **Event Sourcing** - Complete audit trail with time-travel debugging

### Key Components

#### 1. Entity System (`src/entity.rs`)
- `Entity<T>` trait for domain entities
- `EntityId<T>` for type-safe identifiers
- `AggregateRoot` trait for aggregate consistency
- Snapshot support for performance optimization

#### 2. Command/Event/Query (`src/commands.rs`, `src/events.rs`, `src/queries.rs`)
- `DomainCommand` trait with command handling
- `DomainEvent` trait with CID-based integrity
- `DomainQuery` trait for read models
- Handler traits for processing

#### 3. CQRS Implementation (`src/cqrs/`)
- Command bus with async handling
- Event store with replay capability
- Query processors with caching
- Projection support for read models

#### 4. Category Theory (`src/category/`)
- Domain categories as objects
- Morphisms for transformations
- Functors for domain mapping
- Natural transformations for complex mappings
- Limits/colimits for composition

#### 5. Cross-Domain Integration (`src/integration/`)
- Service registry with dependency injection
- Domain bridges for event routing
- Cross-domain rules engine
- Semantic analyzer for concept alignment

#### 6. State Machines (`src/state_machine.rs`)
- Moore and Mealy machine implementations
- State transition validation
- Event-driven state changes
- Aggregate behavior modeling

## Development Rules

### MANDATORY Test-Driven Development
1. **Write failing tests first** - No code without tests
2. **100% test coverage required** - Every public API must be tested
3. **Integration tests for cross-domain** - Test domain interactions
4. **Doc tests for examples** - Every major component needs usage examples

### Code Organization
```
cim-domain/
├── src/
│   ├── lib.rs              # Public API exports
│   ├── entity.rs           # Core entity traits
│   ├── commands.rs         # Command definitions
│   ├── events.rs           # Event definitions
│   ├── queries.rs          # Query definitions
│   ├── error.rs            # Error types
│   ├── cqrs/              # CQRS implementation
│   ├── category/          # Category theory
│   ├── domain/            # Domain utilities
│   ├── integration/       # Cross-domain
│   └── state_machine.rs   # State machines
├── crates/
│   ├── cim-component/     # ECS component system
│   ├── cim-ipld/          # Content addressing
│   └── cim-subject/       # NATS subject algebra
└── tests/                 # Integration tests
```

### API Design Guidelines
1. **Type Safety First** - Use phantom types for domain separation
2. **Zero-Cost Abstractions** - Performance is critical
3. **Explicit Over Implicit** - Clear intent in APIs
4. **Builder Patterns** - For complex object construction
5. **Result Types** - All operations return Result<T, DomainError>

## Usage Patterns

### Creating a New Domain
```rust
// 1. Define your aggregate
#[derive(Debug, Clone)]
pub struct MyAggregate {
    id: EntityId<Self>,
    // ... fields
}

impl Entity<Self> for MyAggregate {
    fn id(&self) -> EntityId<Self> { self.id }
}

impl AggregateRoot for MyAggregate {
    type Command = MyCommand;
    type Event = MyEvent;
    type Error = MyError;
    
    fn handle_command(&mut self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        // Command handling logic
    }
    
    fn apply_event(&mut self, event: &Self::Event) {
        // State mutation from events
    }
}

// 2. Define commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MyCommand {
    Create { name: String },
    Update { field: String },
}

impl DomainCommand for MyCommand {
    type Aggregate = MyAggregate;
    // ... implementation
}

// 3. Define events  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MyEvent {
    Created { id: EntityId<MyAggregate>, name: String },
    Updated { id: EntityId<MyAggregate>, field: String },
}

impl DomainEvent for MyEvent {
    // ... implementation
}
```

### Cross-Domain Communication
```rust
// Use functors for domain mapping
let functor = IdentityFunctor::<SourceDomain, TargetDomain>::new();
let transformed = functor.map(source_object)?;

// Use domain bridges for event routing
let bridge = DomainBridge::new("source.domain", "target.domain");
bridge.route_event(event).await?;
```

## Testing Requirements

### Unit Tests
- Test each aggregate's command handling
- Test event application and state changes
- Test invariant enforcement
- Test error conditions

### Integration Tests
- Test cross-domain event flow
- Test saga orchestration
- Test projection updates
- Test query execution

### Performance Tests
- Event creation/publishing benchmarks
- Command processing throughput
- Query response times
- Memory usage per aggregate

## Current Status

- **196 tests passing** (100% pass rate)
- **9 doc tests** demonstrating usage
- **Zero compilation warnings** in domain logic
- **Production ready** - Used by 14+ domains

## Dependencies

### Internal (within cim-domain workspace)
- `cim-component` - ECS component system
- `cim-ipld` - Content addressing with CIDs
- `cim-subject` - NATS subject algebra

### External
- `tokio` - Async runtime
- `serde` - Serialization
- `async-trait` - Async traits
- `thiserror` - Error handling
- `uuid` - Entity IDs
- `chrono` - Timestamps

## Important Conventions

1. **All operations are async** - Use `async/await` throughout
2. **All errors use `DomainError`** - Consistent error handling
3. **All events have `occurred_at`** - Temporal ordering
4. **All commands have `metadata`** - Tracing and correlation
5. **All queries return `Vec<T>`** - Even single results

## Performance Targets

- Event creation: >500,000/sec
- Event publishing: >100,000/sec  
- Command processing: <10ms p99
- Query execution: <5ms p99
- Memory per aggregate: <10KB

## Future Enhancements

1. **Snapshot Store** - Implement aggregate snapshots
2. **Event Replay** - Time-travel debugging
3. **CID Chains** - Cryptographic event integrity
4. **Distributed Sagas** - Multi-domain transactions
5. **GraphQL Integration** - Query language support

## Working with this Module

When making changes:
1. **Always run tests first** - `cargo test`
2. **Check benchmarks** - `cargo bench`
3. **Update docs** - `cargo doc --open`
4. **Run examples** - `cargo run --example <name>`
5. **Verify downstream** - Test dependent domains

This module is the foundation of CIM - treat it with appropriate care and rigor.