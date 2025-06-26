# Core Abstractions Implementation Plan

## Immediate Actions Required

### 1. Fix cim-infrastructure Dependencies

**File**: `cim-infrastructure/Cargo.toml`
```toml
[dependencies]
# Remove this line:
# cim-domain = { path = "../cim-domain" }

# Keep these:
async-nats = "0.38"
async-trait = "0.1"
futures = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.11", features = ["v4", "serde"] }
thiserror = "2.0"
tokio = { version = "1.42", features = ["full"] }
tracing = "0.1"
```

### 2. Create Core Traits Module

**New File**: `cim-core-traits/Cargo.toml`
```toml
[package]
name = "cim-core-traits"
version = "0.3.0"
edition = "2021"
description = "Core trait definitions for CIM modules"

[dependencies]
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
cid = "0.11"
thiserror = "2.0"
futures = "0.3"
```

**New File**: `cim-core-traits/src/lib.rs`
```rust
//! Core traits that define interfaces between CIM modules

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use cid::Cid;
use futures::Stream;
use std::error::Error;

/// Trait for content-addressed storage
#[async_trait]
pub trait ContentStore: Send + Sync {
    type Error: Error + Send + Sync + 'static;
    
    async fn put(&self, content: &[u8]) -> Result<Cid, Self::Error>;
    async fn get(&self, cid: &Cid) -> Result<Vec<u8>, Self::Error>;
    async fn has(&self, cid: &Cid) -> Result<bool, Self::Error>;
}

/// Trait for message bus operations
#[async_trait]
pub trait MessageBus: Send + Sync {
    type Error: Error + Send + Sync + 'static;
    type Message: Send + Sync;
    
    async fn publish<T>(&self, subject: &str, message: &T) -> Result<(), Self::Error>
    where
        T: Serialize + Send + Sync;
        
    async fn subscribe(&self, subject: &str) -> Result<Box<dyn Stream<Item = Self::Message> + Send + Unpin>, Self::Error>;
    
    async fn request<T, R>(&self, subject: &str, request: &T) -> Result<R, Self::Error>
    where
        T: Serialize + Send + Sync,
        R: for<'de> Deserialize<'de>;
}

/// Trait for event stores
#[async_trait]
pub trait EventStore: Send + Sync {
    type Error: Error + Send + Sync + 'static;
    type Event: Send + Sync;
    
    async fn append(&self, stream_id: &str, events: Vec<Self::Event>) -> Result<(), Self::Error>;
    async fn load(&self, stream_id: &str) -> Result<Vec<Self::Event>, Self::Error>;
    async fn load_from(&self, stream_id: &str, from_version: u64) -> Result<Vec<Self::Event>, Self::Error>;
}

/// Common error trait for CIM modules
pub trait CimError: Error + Send + Sync + 'static {
    /// Get a unique error code
    fn error_code(&self) -> &str;
    
    /// Check if the error is retryable
    fn is_retryable(&self) -> bool;
    
    /// Get the error category
    fn category(&self) -> ErrorCategory;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Infrastructure,
    Domain,
    Validation,
    NotFound,
    Conflict,
    Timeout,
}
```

### 3. Update cim-ipld to Remove NATS

**File**: `cim-ipld/Cargo.toml`
```toml
[dependencies]
# Core IPLD dependencies
cid = "0.11"
multihash = "0.19"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_cbor = "0.11"
blake3 = "1.5"
thiserror = "2.0"
bytes = "1.5"

# Remove NATS dependencies:
# async-nats = { version = "0.41", features = ["service"] }
# tokio = { version = "1.45", features = ["full"] }
# futures = "0.3"

# Keep content transformation dependencies
image = { version = "0.25", default-features = false, features = ["jpeg", "png", "webp"] }
pulldown-cmark = "0.13"
symphonia = { version = "0.5", features = ["mp3", "wav", "flac", "ogg"] }
regex = "1.11"

# Add core traits
cim-core-traits = { path = "../cim-core-traits" }
```

**New File**: `cim-ipld/src/store_trait.rs`
```rust
//! Storage trait for IPLD content

use async_trait::async_trait;
use cid::Cid;
use cim_core_traits::ContentStore;
use crate::error::Error;

/// Trait for IPLD object storage
#[async_trait]
pub trait IpldStore: ContentStore<Error = Error> {
    /// Store IPLD data with codec
    async fn put_ipld<T>(&self, value: &T, codec: u64) -> Result<Cid, Error>
    where
        T: serde::Serialize + Send + Sync;
        
    /// Get IPLD data
    async fn get_ipld<T>(&self, cid: &Cid) -> Result<T, Error>
    where
        T: for<'de> serde::Deserialize<'de>;
}
```

### 4. Keep cim-subject in cim-domain

**File**: `cim-domain/Cargo.toml`
```toml
[dependencies]
# Core dependencies
anyhow = "1.0"
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.11", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
async-trait = "0.1"
tokio = { version = "1.35", features = ["full"] }
futures = "0.3"

# Domain dependencies
cim-ipld = { path = "../cim-ipld" }
cim-component = { path = "../cim-component" }
cim-subject = { path = "../cim-subject" }  # Core abstraction
cim-core-traits = { path = "../cim-core-traits" }

# Content addressing
cid = { version = "0.11", features = ["serde"] }

# Other dependencies...
```

### 5. Create Infrastructure Implementations

**New File**: `cim-infrastructure/src/stores/ipld_store.rs`
```rust
//! IPLD store implementation using NATS object store

use async_trait::async_trait;
use cid::Cid;
use cim_core_traits::ContentStore;
use cim_ipld::IpldStore;
use crate::errors::{InfrastructureError, InfrastructureResult};

pub struct NatsIpldStore {
    // Implementation details
}

#[async_trait]
impl ContentStore for NatsIpldStore {
    type Error = InfrastructureError;
    
    async fn put(&self, content: &[u8]) -> Result<Cid, Self::Error> {
        // Implementation
    }
    
    async fn get(&self, cid: &Cid) -> Result<Vec<u8>, Self::Error> {
        // Implementation
    }
    
    async fn has(&self, cid: &Cid) -> Result<bool, Self::Error> {
        // Implementation
    }
}
```

### 6. Update cim-bridge to Use Infrastructure

**File**: `cim-bridge/Cargo.toml`
```toml
[dependencies]
# Add infrastructure dependency
cim-infrastructure = { path = "../cim-infrastructure" }
cim-core-traits = { path = "../cim-core-traits" }

# Existing dependencies...
```

## Module Interface Definitions

### cim-ipld Public API
```rust
// Pure content addressing, no infrastructure
pub use cid::Cid;
pub use chain::{ChainedContent, ContentChain};
pub use codec::{CimCodec, CodecRegistry};
pub use store_trait::IpldStore;
pub use error::{Error, Result};
```

### cim-infrastructure Public API
```rust
// All infrastructure implementations
pub use nats::{NatsClient, NatsConfig};
pub use stores::{NatsIpldStore, NatsEventStore};
pub use message_bus::NatsMessageBus;
pub use errors::{InfrastructureError, InfrastructureResult};
```

### cim-component Public API
```rust
// Component system unchanged
pub use component::{Component, ComponentStorage};
pub use error::{ComponentError, ComponentResult};
```

### cim-domain Public API
```rust
// Pure domain logic, no infrastructure
pub use entity::{Entity, EntityId, AggregateRoot};
pub use cqrs::{Command, Query, DomainEvent};
pub use handlers::{CommandHandler, QueryHandler};
pub use state_machine::{MooreMachine, MealyMachine};
```

### cim-bridge Public API
```rust
// AI integration using infrastructure
pub use providers::{OllamaProvider, OpenAIProvider};
pub use service::BridgeService;
pub use types::{CompletionRequest, CompletionResponse};
```

### cim-subject Public API
```rust
// Subject algebra and routing
pub use subject::{Subject, SubjectParts, SubjectBuilder};
pub use pattern::{Pattern, PatternMatcher};
pub use algebra::{SubjectAlgebra, AlgebraOperation};
pub use translator::{Translator, TranslationRule, MessageTranslator};
pub use correlation::{MessageIdentity, CorrelationId, CausationId};
```

## Testing Strategy

1. **Unit Tests**: Each module tested in isolation
2. **Integration Tests**: Test module interactions
3. **Contract Tests**: Verify trait implementations
4. **End-to-End Tests**: Full stack validation

## Migration Timeline

1. **Week 1**: Create cim-core-traits and update dependencies
2. **Week 2**: Refactor cim-ipld and cim-infrastructure
3. **Week 3**: Update cim-domain and cim-bridge
4. **Week 4**: Integration testing and documentation

This implementation plan ensures clean separation of concerns while maintaining backward compatibility where possible. 