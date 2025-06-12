# Phase 1: Foundation Modules - Detailed Tasks

## Overview
Phase 1 establishes the foundation modules that will enable proper bounded context separation. This phase is critical as it breaks circular dependencies and creates the base infrastructure.

## Task 1.1: Create `cim-component` Module

### Setup Tasks
- [ ] Create new directory `cim-component/`
- [ ] Initialize with `cargo init --lib`
- [ ] Add to workspace in root `Cargo.toml`
- [ ] Create initial module structure

### Implementation Tasks

#### 1.1.1 Extract Component Trait
```rust
// cim-component/src/lib.rs
use std::any::{Any, TypeId};
use std::fmt::Debug;

/// Core trait for attachable components
pub trait Component: Any + Send + Sync + Debug {
    /// Get the component as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Get mutable reference as Any
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Get the TypeId of this component
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    /// Clone the component into a box
    fn clone_box(&self) -> Box<dyn Component>;
}

// Blanket implementation
impl<T> Component for T
where
    T: Any + Send + Sync + Debug + Clone + 'static
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
}
```

#### 1.1.2 Move ComponentStorage
```rust
// cim-component/src/storage.rs
use crate::Component;
use std::any::TypeId;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ComponentStorage {
    components: HashMap<TypeId, Box<dyn Component>>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn add_component<T: Component>(&mut self, component: T) {
        self.components.insert(component.type_id(), Box::new(component));
    }

    pub fn get_component<T: Component>(&self) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|c| c.as_any().downcast_ref::<T>())
    }

    pub fn has_component<T: Component>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<T>())
    }

    pub fn remove_component<T: Component>(&mut self) -> Option<Box<T>> {
        self.components
            .remove(&TypeId::of::<T>())
            .and_then(|c| c.as_any().downcast_ref::<T>().map(|_| {
                // This is safe because we just verified the type
                unsafe { Box::from_raw(Box::into_raw(c) as *mut T) }
            }))
    }
}
```

#### 1.1.3 Add Tests
```rust
// cim-component/src/tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestComponent {
        value: String,
    }

    #[test]
    fn test_component_storage() {
        let mut storage = ComponentStorage::new();

        storage.add_component(TestComponent {
            value: "test".to_string(),
        });

        assert!(storage.has_component::<TestComponent>());

        let component = storage.get_component::<TestComponent>().unwrap();
        assert_eq!(component.value, "test");
    }
}
```

### Update Dependencies
- [ ] Update `cim-domain/Cargo.toml` to remove Component code
- [ ] Update `cim-contextgraph/Cargo.toml` to use `cim-component`
- [ ] Fix all import statements

## Task 1.2: Create `cim-core-domain` Module

### Setup Tasks
- [ ] Create new directory `cim-core-domain/`
- [ ] Initialize with `cargo init --lib`
- [ ] Add to workspace
- [ ] Create module structure

### Implementation Tasks

#### 1.2.1 Entity Trait
```rust
// cim-core-domain/src/entity.rs
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

/// Unique identifier for entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Core trait for entities with identity
pub trait Entity: Debug + Send + Sync {
    type Id: Debug + Clone + PartialEq + Eq + Hash + Send + Sync;

    /// Get the entity's unique identifier
    fn id(&self) -> &Self::Id;

    /// Check if this entity equals another by ID
    fn same_identity_as(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
```

#### 1.2.2 Aggregate Root Trait
```rust
// cim-core-domain/src/aggregate.rs
use crate::entity::Entity;
use crate::domain_event::DomainEvent;

/// Marker trait for aggregate roots
pub trait AggregateRoot: Entity {
    type Event: DomainEvent;

    /// Get uncommitted events
    fn uncommitted_events(&self) -> &[Self::Event];

    /// Mark events as committed
    fn mark_events_committed(&mut self);

    /// Apply an event to update state
    fn apply_event(&mut self, event: &Self::Event);
}
```

#### 1.2.3 Domain Event Trait
```rust
// cim-core-domain/src/domain_event.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Base trait for domain events
pub trait DomainEvent: Debug + Clone + Send + Sync {
    /// Get the aggregate ID this event belongs to
    fn aggregate_id(&self) -> String;

    /// Get the event timestamp
    fn occurred_at(&self) -> DateTime<Utc>;

    /// Get the event type name
    fn event_type(&self) -> &'static str;
}
```

#### 1.2.4 Command and Query Traits
```rust
// cim-core-domain/src/command.rs
pub trait Command: Debug + Send + Sync {
    type Result;

    /// Get the aggregate ID this command targets
    fn aggregate_id(&self) -> String;
}

// cim-core-domain/src/query.rs
pub trait Query: Debug + Send + Sync {
    type Result;
}
```

#### 1.2.5 Core Errors
```rust
// cim-core-domain/src/errors.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),

    #[error("Concurrency conflict: expected version {expected}, got {actual}")]
    ConcurrencyConflict { expected: u64, actual: u64 },
}

pub type DomainResult<T> = Result<T, DomainError>;
```

## Task 1.3: Create `cim-infrastructure` Module

### Setup Tasks
- [ ] Create new directory `cim-infrastructure/`
- [ ] Initialize with `cargo init --lib`
- [ ] Add to workspace
- [ ] Create module structure

### Implementation Tasks

#### 1.3.1 EventStore Trait
```rust
// cim-infrastructure/src/event_store/trait.rs
use async_trait::async_trait;
use cim_core_domain::DomainEvent;

#[async_trait]
pub trait EventStore: Send + Sync {
    type Event: DomainEvent;
    type Error;

    /// Append events to the store
    async fn append_events(
        &self,
        aggregate_id: &str,
        events: Vec<Self::Event>,
        expected_version: Option<u64>,
    ) -> Result<(), Self::Error>;

    /// Get events for an aggregate
    async fn get_events(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<Self::Event>, Self::Error>;

    /// Get events from a specific version
    async fn get_events_from_version(
        &self,
        aggregate_id: &str,
        from_version: u64,
    ) -> Result<Vec<Self::Event>, Self::Error>;
}
```

#### 1.3.2 Repository Trait
```rust
// cim-infrastructure/src/persistence/repository.rs
use async_trait::async_trait;
use cim_core_domain::{AggregateRoot, DomainResult};

#[async_trait]
pub trait Repository<A: AggregateRoot>: Send + Sync {
    /// Load an aggregate by ID
    async fn get(&self, id: &A::Id) -> DomainResult<A>;

    /// Save an aggregate
    async fn save(&self, aggregate: &A) -> DomainResult<()>;

    /// Check if aggregate exists
    async fn exists(&self, id: &A::Id) -> DomainResult<bool>;
}
```

#### 1.3.3 Move JetStream EventStore
- [ ] Copy `jetstream_event_store.rs` from `cim-domain`
- [ ] Update imports to use new traits
- [ ] Remove domain-specific logic
- [ ] Make it generic over event types

#### 1.3.4 Move NATS Client
- [ ] Copy `nats_client.rs` from `cim-domain`
- [ ] Remove domain-specific code
- [ ] Create generic messaging interfaces

## Testing Strategy

### Unit Tests
- [ ] Test Component trait implementations
- [ ] Test ComponentStorage operations
- [ ] Test Entity and AggregateRoot traits
- [ ] Test EventStore implementations

### Integration Tests
```rust
// tests/integration/component_integration.rs
#[test]
fn test_component_across_modules() {
    // Test that components work across module boundaries
}
```

## Migration Checklist

### Before Starting
- [ ] Create feature branch: `feature/phase-1-foundation`
- [ ] Backup current state
- [ ] Notify team of refactoring start

### During Implementation
- [ ] Keep existing code working
- [ ] Run tests after each major change
- [ ] Document any API changes
- [ ] Update examples as needed

### After Completion
- [ ] All tests passing
- [ ] No circular dependencies
- [ ] Documentation updated
- [ ] PR created and reviewed

## Success Metrics

1. **Compilation**: All modules compile independently
2. **Tests**: 100% of existing tests still pass
3. **Dependencies**: No circular dependencies detected
4. **Performance**: No performance regression

## Rollback Plan

If issues arise:
1. Keep old modules intact during development
2. Use feature flags to switch between implementations
3. Have clear git tags for rollback points
4. Document any breaking changes

This detailed task breakdown ensures a systematic approach to Phase 1, minimizing risk while establishing the foundation for proper bounded context separation.
