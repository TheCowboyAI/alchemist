# Bounded Context Refactoring Plan

## Overview

This plan outlines the systematic refactoring of the CIM codebase to achieve proper Domain-Driven Design (DDD) bounded context separation, following hexagonal architecture principles.

## Goals

1. **Eliminate circular dependencies** between modules
2. **Establish clear bounded contexts** aligned with business capabilities
3. **Implement hexagonal architecture** with ports and adapters
4. **Enable independent testing and deployment** of contexts
5. **Improve code maintainability** through clear separation of concerns

## New Module Structure

```
alchemist/
├── cim-core-domain/        # Shared domain primitives
├── cim-identity-context/   # Person, Organization management
├── cim-security-context/   # Agent, Policy management
├── cim-content-context/    # Document management
├── cim-workflow-context/   # Workflow and state machines
├── cim-knowledge-context/  # ConceptGraph and knowledge management
├── cim-infrastructure/     # Shared infrastructure (EventStore, NATS)
├── cim-component/          # Component trait and storage
├── cim-contextgraph/       # Graph abstractions (refactored)
├── cim-viz-bevy/          # Visualization layer (minimal changes)
├── cim-ipld/              # No changes needed
├── cim-subject/           # No changes needed
└── cim-compose/           # No changes needed
```

## Phase 1: Create Foundation Modules (Week 1)

### 1.1 Create `cim-component` Module

**Purpose**: Extract the Component trait to break circular dependencies

```toml
# cim-component/Cargo.toml
[package]
name = "cim-component"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.11", features = ["v4", "serde"] }
```

**Tasks**:
- [ ] Create new crate structure
- [ ] Move `Component` trait from `cim-domain`
- [ ] Move `ComponentStorage` implementation
- [ ] Add comprehensive tests
- [ ] Update all dependent crates

### 1.2 Create `cim-core-domain` Module

**Purpose**: Shared domain primitives without any infrastructure dependencies

```toml
# cim-core-domain/Cargo.toml
[package]
name = "cim-core-domain"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.11", features = ["v4", "serde"] }
thiserror = "2.0"
chrono = { version = "0.4", features = ["serde"] }
cim-component = { path = "../cim-component" }
```

**Structure**:
```
cim-core-domain/
├── src/
│   ├── lib.rs
│   ├── entity.rs           # Entity<T> trait and EntityId
│   ├── aggregate.rs        # AggregateRoot trait
│   ├── value_object.rs     # Value object patterns
│   ├── domain_event.rs     # Base DomainEvent trait
│   ├── command.rs          # Base Command trait
│   ├── query.rs            # Base Query trait
│   ├── errors.rs           # Core domain errors
│   └── identifiers.rs      # Shared ID types
```

**Tasks**:
- [ ] Extract base traits from `cim-domain`
- [ ] Define core domain interfaces
- [ ] Remove all infrastructure dependencies
- [ ] Add marker traits for phantom types

### 1.3 Create `cim-infrastructure` Module

**Purpose**: Consolidate all infrastructure concerns

```toml
# cim-infrastructure/Cargo.toml
[package]
name = "cim-infrastructure"
version = "0.1.0"
edition = "2021"

[dependencies]
async-nats = "0.35"
async-trait = "0.1"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
cid = { version = "0.11", features = ["serde"] }
lru = "0.12"

# Internal dependencies
cim-core-domain = { path = "../cim-core-domain" }
cim-ipld = { path = "../cim-ipld" }
cim-subject = { path = "../cim-subject" }
```

**Structure**:
```
cim-infrastructure/
├── src/
│   ├── lib.rs
│   ├── event_store/
│   │   ├── mod.rs
│   │   ├── trait.rs         # EventStore trait
│   │   ├── jetstream.rs     # JetStream implementation
│   │   └── memory.rs        # In-memory implementation
│   ├── messaging/
│   │   ├── mod.rs
│   │   ├── nats_client.rs
│   │   └── event_bus.rs
│   ├── persistence/
│   │   ├── mod.rs
│   │   ├── repository.rs    # Repository trait
│   │   └── snapshot.rs
│   └── cqrs/
│       ├── mod.rs
│       ├── command_bus.rs
│       └── query_bus.rs
```

**Tasks**:
- [ ] Move EventStore from `cim-domain`
- [ ] Move NATS client implementation
- [ ] Extract repository patterns
- [ ] Create port interfaces for domain use

## Phase 2: Create Bounded Contexts (Weeks 2-3)

### 2.1 Identity Context

```toml
# cim-identity-context/Cargo.toml
[package]
name = "cim-identity-context"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core dependencies
cim-core-domain = { path = "../cim-core-domain" }
cim-component = { path = "../cim-component" }

# Standard dependencies
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.11", features = ["v4", "serde"] }
thiserror = "2.0"
async-trait = "0.1"
```

**Structure**:
```
cim-identity-context/
├── src/
│   ├── lib.rs
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── person/
│   │   │   ├── mod.rs
│   │   │   ├── aggregate.rs
│   │   │   ├── commands.rs
│   │   │   ├── events.rs
│   │   │   └── value_objects.rs
│   │   └── organization/
│   │       ├── mod.rs
│   │       ├── aggregate.rs
│   │       ├── commands.rs
│   │       ├── events.rs
│   │       └── value_objects.rs
│   ├── application/
│   │   ├── mod.rs
│   │   ├── command_handlers.rs
│   │   ├── query_handlers.rs
│   │   └── services.rs
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   └── repositories.rs
│   └── ports/
│       ├── mod.rs
│       ├── inbound.rs      # Command/Query interfaces
│       └── outbound.rs     # Repository interfaces
```

**Migration Tasks**:
- [ ] Move Person aggregate and related types
- [ ] Move Organization aggregate and related types
- [ ] Extract identity-specific events
- [ ] Create command handlers with ports
- [ ] Define repository interfaces
- [ ] Add integration tests

### 2.2 Security Context

**Structure**: Similar to Identity Context

**Key Components**:
- Agent aggregate
- Policy aggregate
- Security-related events
- Permission management

**Migration Tasks**:
- [ ] Move Agent aggregate and components
- [ ] Move Policy aggregate and components
- [ ] Extract security-specific events
- [ ] Create security command handlers
- [ ] Define security repository interfaces

### 2.3 Content Context

**Key Components**:
- Document aggregate
- Content management events
- Access control
- Versioning

**Migration Tasks**:
- [ ] Move Document aggregate
- [ ] Extract content-specific events
- [ ] Create content command handlers
- [ ] Define content repository interfaces

### 2.4 Workflow Context

**Key Components**:
- Workflow aggregate
- State machine implementations
- Transition management

**Migration Tasks**:
- [ ] Move Workflow aggregate
- [ ] Move state machine types
- [ ] Extract workflow events
- [ ] Create workflow command handlers

### 2.5 Knowledge Context

**Key Components**:
- ConceptGraph aggregate
- Conceptual space management
- Knowledge relationships

**Migration Tasks**:
- [ ] Move ConceptGraph aggregate
- [ ] Extract knowledge events
- [ ] Create knowledge command handlers

## Phase 3: Refactor Existing Modules (Week 4)

### 3.1 Refactor `cim-contextgraph`

**Goals**:
- Remove dependency on `cim-domain`
- Use `cim-component` instead
- Focus on pure graph algorithms

**Tasks**:
- [ ] Update dependencies to use `cim-component`
- [ ] Remove workflow-specific logic (move to workflow context)
- [ ] Keep only generic graph algorithms
- [ ] Update tests

### 3.2 Update `cim-viz-bevy`

**Goals**:
- Update imports to use new contexts
- Implement proper adapters for each context

**Tasks**:
- [ ] Create adapters for each bounded context
- [ ] Update event routing
- [ ] Implement context-specific visualizations
- [ ] Update examples

## Phase 4: Integration and Anti-Corruption Layers (Week 5)

### 4.1 Create Integration Events

**Location**: Each context defines its own integration events

```rust
// cim-identity-context/src/integration/events.rs
pub enum IdentityIntegrationEvent {
    PersonCreated { person_id: PersonId, email: Email },
    OrganizationMemberAdded { org_id: OrgId, person_id: PersonId },
}
```

### 4.2 Implement Event Translators

```rust
// cim-security-context/src/integration/translators.rs
pub struct IdentityEventTranslator;

impl IdentityEventTranslator {
    pub fn translate(&self, event: IdentityIntegrationEvent) -> Option<SecurityCommand> {
        match event {
            IdentityIntegrationEvent::PersonCreated { person_id, .. } => {
                Some(SecurityCommand::CreateAgentForPerson { person_id })
            }
            _ => None,
        }
    }
}
```

### 4.3 Define Context Mappings

Create a context map showing relationships:
- Identity ← → Security (Partnership)
- Security → Content (Customer/Supplier)
- Workflow → All contexts (Open Host Service)

## Phase 5: Testing and Migration (Week 6)

### 5.1 Test Strategy

**Unit Tests**: Each context tested in isolation
```bash
cd cim-identity-context && cargo test
cd cim-security-context && cargo test
# etc...
```

**Integration Tests**: Test context interactions
```rust
// tests/integration/identity_security_integration.rs
#[tokio::test]
async fn test_person_creates_agent() {
    // Test that creating a person triggers agent creation
}
```

### 5.2 Migration Scripts

Create scripts to migrate existing data:
```rust
// scripts/migrate_domain_data.rs
async fn migrate_persons_to_identity_context() {
    // Load from old structure
    // Transform to new structure
    // Save in new context
}
```

## Phase 6: Documentation and Cleanup (Week 7)

### 6.1 Update Documentation

- [ ] Update README files for each context
- [ ] Create context-specific API documentation
- [ ] Document integration patterns
- [ ] Update architecture diagrams

### 6.2 Remove Old Code

- [ ] Delete old `cim-domain` module
- [ ] Remove deprecated imports
- [ ] Clean up unused dependencies

## Success Criteria

1. **No Circular Dependencies**
   ```bash
   cargo depgraph | grep -E "circular|cycle"  # Should return empty
   ```

2. **Independent Compilation**
   ```bash
   cd cim-identity-context && cargo build  # Should succeed without other contexts
   ```

3. **Isolated Tests**
   ```bash
   cd cim-security-context && cargo test  # Should pass without external dependencies
   ```

4. **Clear Context Boundaries**
   - Each context has its own aggregate roots
   - No shared domain models between contexts
   - Integration only through events

## Risk Mitigation

1. **Gradual Migration**: Keep old code working while building new
2. **Feature Flags**: Use flags to switch between old/new implementations
3. **Parallel Testing**: Run tests on both old and new code
4. **Rollback Plan**: Keep old module structure until fully migrated

## Timeline Summary

- **Week 1**: Foundation modules (component, core-domain, infrastructure)
- **Weeks 2-3**: Bounded contexts creation
- **Week 4**: Existing module refactoring
- **Week 5**: Integration and ACLs
- **Week 6**: Testing and migration
- **Week 7**: Documentation and cleanup

## Next Steps

1. Review and approve this plan
2. Create feature branch: `feature/bounded-context-refactoring`
3. Start with Phase 1: Foundation modules
4. Set up CI/CD for new module structure
5. Begin incremental migration

This refactoring will establish a solid foundation for long-term maintainability and enable true microservice architecture if needed in the future.
