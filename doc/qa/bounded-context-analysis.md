# Bounded Context and Domain Separation Analysis

## Current Module Structure

### 1. Core Modules and Their Responsibilities

#### cim-ipld (Infrastructure Layer)
- **Purpose**: Content-addressed storage and CID chain management
- **Dependencies**: None (leaf module)
- **Responsibilities**:
  - IPLD implementation
  - CID generation and verification
  - Content chain management
  - NATS object store integration
- **Assessment**: ✅ Well-bounded, focused on infrastructure concerns

#### cim-subject (Infrastructure Layer)
- **Purpose**: NATS subject algebra and message routing
- **Dependencies**: None (leaf module)
- **Responsibilities**:
  - Subject pattern matching
  - Message translation
  - Routing algebra
  - Permission management
- **Assessment**: ✅ Well-bounded, focused on messaging infrastructure

#### cim-compose (Pure Domain Layer)
- **Purpose**: Graph composition based on category theory
- **Dependencies**: None (leaf module)
- **Responsibilities**:
  - Category theory abstractions
  - Graph composition patterns
  - Functor implementations
- **Assessment**: ✅ Excellent separation, pure domain logic

#### cim-domain (Domain Layer)
- **Purpose**: Core DDD building blocks and business logic
- **Dependencies**: cim-ipld, cim-subject
- **Responsibilities**:
  - Entity/Aggregate definitions
  - Domain events and commands
  - CQRS implementation
  - Business domain models (Person, Organization, etc.)
  - Infrastructure abstractions (EventStore, etc.)
- **Assessment**: ⚠️ Mixed concerns - contains both domain and infrastructure

#### cim-contextgraph (Domain/Application Layer)
- **Purpose**: Graph abstractions and algorithms
- **Dependencies**: cim-ipld, cim-domain
- **Responsibilities**:
  - ContextGraph implementation
  - CID DAG for event sourcing
  - Workflow graph modeling
  - Graph algorithms
- **Assessment**: ⚠️ Circular dependency with cim-domain

#### cim-viz-bevy (Presentation Layer)
- **Purpose**: Bevy ECS visualization
- **Dependencies**: cim-contextgraph, cim-domain
- **Responsibilities**:
  - Visual components
  - Bevy systems
  - Async/sync bridge
  - Rendering logic
- **Assessment**: ✅ Clear presentation layer separation

## Identified Issues

### 1. Circular Dependencies
```
cim-contextgraph → cim-domain (uses Component, workflow types)
cim-domain → cim-contextgraph (would need graph types, but avoided)
```

### 2. Mixed Responsibilities in cim-domain
The `cim-domain` module contains:
- Pure domain logic (entities, aggregates, value objects)
- Infrastructure concerns (event store, NATS client)
- Application services (command handlers, query handlers)
- Bevy bridge code (presentation concern)

### 3. Unclear Aggregate Boundaries
Multiple domain concepts are mixed in cim-domain:
- Person/Organization (HR context?)
- Agent/Policy (Security context?)
- Document (Content management context?)
- Workflow (Process context?)
- ConceptGraph (Knowledge context?)

### 4. Infrastructure Leakage
- Domain events know about NATS subjects
- Command handlers directly use event store
- No clear ports/adapters pattern

## Recommended Improvements

### 1. Split cim-domain into Bounded Contexts

#### cim-core-domain (Pure Domain Layer)
```
cim-core-domain/
├── src/
│   ├── lib.rs
│   ├── entity.rs          # Base Entity trait
│   ├── aggregate.rs       # AggregateRoot trait
│   ├── value_object.rs    # Value object patterns
│   ├── domain_event.rs    # Base event types
│   ├── command.rs         # Base command types
│   └── errors.rs          # Domain errors
```

#### cim-identity-context (Bounded Context)
```
cim-identity-context/
├── src/
│   ├── lib.rs
│   ├── domain/
│   │   ├── person.rs
│   │   ├── organization.rs
│   │   └── events.rs
│   ├── application/
│   │   ├── commands.rs
│   │   └── queries.rs
│   └── infrastructure/
│       └── repositories.rs
```

#### cim-security-context (Bounded Context)
```
cim-security-context/
├── src/
│   ├── lib.rs
│   ├── domain/
│   │   ├── agent.rs
│   │   ├── policy.rs
│   │   └── events.rs
│   ├── application/
│   │   ├── commands.rs
│   │   └── queries.rs
│   └── infrastructure/
│       └── repositories.rs
```

#### cim-content-context (Bounded Context)
```
cim-content-context/
├── src/
│   ├── lib.rs
│   ├── domain/
│   │   ├── document.rs
│   │   └── events.rs
│   ├── application/
│   │   ├── commands.rs
│   │   └── queries.rs
│   └── infrastructure/
│       └── repositories.rs
```

#### cim-workflow-context (Bounded Context)
```
cim-workflow-context/
├── src/
│   ├── lib.rs
│   ├── domain/
│   │   ├── workflow.rs
│   │   ├── state_machine.rs
│   │   └── events.rs
│   ├── application/
│   │   ├── commands.rs
│   │   └── queries.rs
│   └── infrastructure/
│       └── repositories.rs
```

#### cim-infrastructure (Infrastructure Layer)
```
cim-infrastructure/
├── src/
│   ├── lib.rs
│   ├── event_store/
│   │   ├── mod.rs
│   │   ├── jetstream.rs
│   │   └── memory.rs
│   ├── messaging/
│   │   ├── mod.rs
│   │   └── nats_client.rs
│   └── persistence/
│       ├── mod.rs
│       └── repositories.rs
```

### 2. Resolve Circular Dependencies

#### Option A: Move Component trait to cim-core-domain
```rust
// cim-core-domain/src/component.rs
pub trait Component: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}
```

#### Option B: Create cim-component as a separate crate
```
cim-component/
├── src/
│   ├── lib.rs
│   └── storage.rs
```

### 3. Implement Hexagonal Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Presentation Layer                    │
│                    (cim-viz-bevy)                       │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                   Application Layer                      │
│         (Command/Query Handlers per Context)            │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                     Domain Layer                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │Identity  │ │Security  │ │Content   │ │Workflow  │  │
│  │Context   │ │Context   │ │Context   │ │Context   │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                  Infrastructure Layer                    │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │cim-ipld  │ │cim-      │ │cim-infra │ │cim-      │  │
│  │          │ │subject   │ │structure │ │compose   │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
└─────────────────────────────────────────────────────────┘
```

### 4. Define Clear Context Boundaries

#### Context Mapping
```
Identity Context ←→ Security Context
    - Person can be an Agent
    - Organization owns Policies

Security Context ←→ Content Context
    - Policies govern Documents
    - Documents have access control

Workflow Context ←→ All Contexts
    - Workflows orchestrate cross-context processes
    - Each context can have internal workflows
```

### 5. Anti-Corruption Layers

```rust
// Example: Identity context consuming Security events
pub struct SecurityEventTranslator {
    pub fn translate_agent_event(&self, event: AgentEvent) -> Option<PersonEvent> {
        match event {
            AgentEvent::AgentDeactivated { agent_id } => {
                // Map to identity context concept
                Some(PersonEvent::AccessRevoked { person_id: self.map_agent_to_person(agent_id)? })
            }
            _ => None, // Ignore events not relevant to identity context
        }
    }
}
```

## Migration Strategy

### Phase 1: Extract Infrastructure (2 weeks)
1. Create `cim-infrastructure` crate
2. Move event store, NATS client, repositories
3. Update dependencies

### Phase 2: Split Domain Contexts (4 weeks)
1. Create context-specific crates
2. Move domain models to appropriate contexts
3. Implement context boundaries

### Phase 3: Implement ACLs (2 weeks)
1. Add translation layers between contexts
2. Define integration events vs internal events
3. Test context isolation

### Phase 4: Refactor Application Layer (2 weeks)
1. Move command/query handlers to contexts
2. Implement proper ports/adapters
3. Update Bevy bridge

## Success Metrics

1. **No circular dependencies** between modules
2. **Clear ownership** of domain concepts
3. **Isolated test suites** per context
4. **Independent deployment** capability
5. **Reduced coupling** between contexts

## Conclusion

The current module structure shows good initial separation but suffers from:
- Mixed responsibilities in cim-domain
- Unclear bounded context boundaries
- Some circular dependencies
- Infrastructure concerns mixed with domain logic

The proposed restructuring would create:
- Clear bounded contexts aligned with business capabilities
- Proper hexagonal architecture
- Better testability and maintainability
- True module independence

This refactoring would be a significant effort but would greatly improve the long-term maintainability and scalability of the codebase.
