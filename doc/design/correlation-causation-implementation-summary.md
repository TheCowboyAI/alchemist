# Correlation/Causation Implementation Summary

## Overview

This document summarizes the implementation of correlation and causation tracking across the CIM system, completed on January 4, 2025.

## Key Changes

### 1. Created `cim-subject` Module

The correlation/causation functionality was moved to a dedicated module for better separation of concerns:

```rust
// cim-subject/src/correlation.rs
pub struct CorrelationId(pub IdType);
pub struct CausationId(pub IdType);

pub struct MessageIdentity {
    pub message_id: IdType,
    pub correlation_id: CorrelationId,
    pub causation_id: CausationId,
}
```

### 2. SerializableCid Wrapper

Created a wrapper to handle Cid serialization since the `cid` crate doesn't implement serde traits:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SerializableCid(pub Cid);

impl Serialize for SerializableCid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.to_string().serialize(serializer)
    }
}
```

### 3. MessageFactory Pattern

Implemented a factory to ensure proper correlation/causation chain creation:

```rust
pub struct MessageFactory;

impl MessageFactory {
    /// Create a root message (self-correlated)
    pub fn create_root<T>(payload: T) -> Message<T> {
        let id = IdType::Uuid(Uuid::new_v4());
        Message {
            identity: MessageIdentity {
                message_id: id.clone(),
                correlation_id: CorrelationId(id.clone()),
                causation_id: CausationId(id),
            },
            payload,
        }
    }

    /// Create a message caused by another
    pub fn create_caused_by<T, P>(payload: T, parent: &Message<P>) -> Message<T> {
        Message {
            identity: MessageIdentity {
                message_id: IdType::Uuid(Uuid::new_v4()),
                correlation_id: parent.identity.correlation_id.clone(),
                causation_id: CausationId(parent.identity.message_id.clone()),
            },
            payload,
        }
    }
}
```

### 4. Message Algebra

Created structures for managing and validating correlation chains:

```rust
pub struct CorrelationChain {
    pub correlation_id: CorrelationId,
    pub messages: Vec<MessageNode>,
}

pub struct MessageAlgebra {
    chains: HashMap<CorrelationId, CorrelationChain>,
}
```

### 5. NATS Integration

Updated the NATS translator to include correlation headers:

```rust
pub struct NatsMessage {
    pub subject: String,
    pub payload: Vec<u8>,
    pub headers: HashMap<String, String>,
}

impl SubjectTranslator {
    pub fn translate_with_correlation<T: Serialize>(
        &self,
        message: &Message<T>,
        subject: &Subject,
    ) -> Result<NatsMessage, TranslatorError> {
        let mut headers = HashMap::new();
        headers.insert("X-Message-ID".to_string(), message.identity.message_id.to_string());
        headers.insert("X-Correlation-ID".to_string(), message.identity.correlation_id.0.to_string());
        headers.insert("X-Causation-ID".to_string(), message.identity.causation_id.0.to_string());
        
        // ... rest of translation
    }
}
```

## Updated Domains

### Phase 2: Core Infrastructure
- ✅ Updated `cim-domain` to use types from `cim-subject`
- ✅ Made correlation/causation fields required (not optional)
- ✅ Updated CommandEnvelope and QueryEnvelope to use MessageIdentity

### Phase 3: Domain Updates

Successfully updated 8 domains with the new correlation/causation API:

1. **Graph Domain** (41/41 tests passing)
   - Updated command/query handlers to use envelope.identity.correlation_id
   - Maintained backward compatibility

2. **Identity Domain** (27/27 tests passing)
   - Created CQRS adapters for existing handlers
   - Added correlation tracking support

3. **Person Domain** (22/22 tests passing)
   - Created command/query wrappers
   - Fixed API compatibility issues

4. **Agent Domain** (5/5 tests passing)
   - Already had CQRS support
   - Updated correlation_id access pattern

5. **Git Domain** (10/10 tests passing)
   - Created CQRS adapters for all commands
   - Added Command trait implementations

6. **ConceptualSpaces Domain** (0 tests, compiles)
   - Fixed correlation_id references
   - Updated to use envelope.identity pattern

7. **Location Domain** (5/5 tests passing)
   - Fixed command handler correlation references
   - Updated event publisher calls

8. **Policy Domain** (22/22 tests passing)
   - Updated command handler correlation references
   - Fixed all envelope accesses

## Key Implementation Rules

### 1. Correlation/Causation Rules
- **Root messages**: MessageId = CorrelationId = CausationId (self-correlation)
- **Caused messages**: Inherit CorrelationId from parent, CausationId = parent.MessageId
- **All messages** MUST have correlation and causation IDs (not optional)

### 2. API Pattern
```rust
// OLD (incorrect)
envelope.correlation_id

// NEW (correct)
envelope.identity.correlation_id
```

### 3. Factory Usage
```rust
// ALWAYS use MessageFactory
let root_cmd = MessageFactory::create_root(CreateOrder { ... });
let caused_event = MessageFactory::create_caused_by(OrderCreated { ... }, &root_cmd);

// NEVER create messages directly
let bad = Event { correlation_id: Uuid::new_v4(), ... }; // ❌ WRONG
```

## Benefits

1. **Centralized Management**: All correlation logic in one module
2. **Type Safety**: Strong typing prevents correlation mistakes
3. **Flexibility**: Supports both UUID and CID-based identifiers
4. **Traceability**: Complete message lineage tracking
5. **NATS Integration**: Headers automatically included in all messages

## Remaining Work

1. Update remaining domains (Dialog, Document, Nix, Organization, Workflow)
2. Create comprehensive correlation chain tests
3. Implement correlation tracking dashboard in Bevy
4. Add correlation validation to event store

## References

- `/doc/design/event-correlation-causation-algebra.md` - Design specification
- `cim-subject/src/correlation.rs` - Core implementation
- `cim-subject/src/message_algebra.rs` - Chain management
- `cim-domain/src/cqrs.rs` - CQRS integration 