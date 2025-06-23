# Event Correlation Implementation Guide

## Overview

This guide provides concrete implementation patterns for enforcing the Event Correlation and Causation algebra across all CIM modules. All event streams MUST implement these patterns.

## Required Implementation

### 1. Base Message Types

```rust
use uuid::Uuid;

/// All messages must implement this trait
pub trait Message: Send + Sync {
    fn message_id(&self) -> &MessageId;
    fn correlation_id(&self) -> &CorrelationId;
    fn causation_id(&self) -> &CausationId;
    fn message_type(&self) -> MessageType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageId(Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CorrelationId(Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CausationId(Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Command,
    Query,
    Event,
}

impl MessageId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<MessageId> for CorrelationId {
    fn from(id: MessageId) -> Self {
        CorrelationId(id.0)
    }
}

impl From<MessageId> for CausationId {
    fn from(id: MessageId) -> Self {
        CausationId(id.0)
    }
}
```

### 2. Message Factory Pattern

Every module MUST use this factory pattern for message creation:

```rust
pub struct MessageFactory;

impl MessageFactory {
    /// Create a root message (starts a new transaction)
    pub fn create_root<T: Into<MessagePayload>>(payload: T) -> impl Message {
        let message_id = MessageId::new();
        BaseMessage {
            message_id,
            correlation_id: message_id.into(),
            causation_id: message_id.into(),
            payload: payload.into(),
        }
    }
    
    /// Create a message caused by another message
    pub fn create_caused_by<T: Into<MessagePayload>>(
        payload: T,
        causing_message: &impl Message,
    ) -> impl Message {
        BaseMessage {
            message_id: MessageId::new(),
            correlation_id: causing_message.correlation_id().clone(),
            causation_id: causing_message.message_id().clone().into(),
            payload: payload.into(),
        }
    }
    
    /// Create a message with explicit correlation (for external integration)
    pub fn create_with_correlation<T: Into<MessagePayload>>(
        payload: T,
        correlation_id: CorrelationId,
        causation_id: Option<CausationId>,
    ) -> impl Message {
        let message_id = MessageId::new();
        BaseMessage {
            message_id,
            correlation_id,
            causation_id: causation_id.unwrap_or_else(|| message_id.into()),
            payload: payload.into(),
        }
    }
}
```

### 3. Event Stream Implementation

All event streams MUST implement correlation tracking:

```rust
use async_nats::HeaderMap;

pub struct CorrelatedEventStream {
    client: async_nats::Client,
    subject_prefix: String,
}

impl CorrelatedEventStream {
    pub async fn publish(&self, message: impl Message) -> Result<()> {
        let subject = format!(
            "{}.{}",
            self.subject_prefix,
            message.message_type().as_str()
        );
        
        let mut headers = HeaderMap::new();
        headers.insert("X-Message-ID", message.message_id().to_string());
        headers.insert("X-Correlation-ID", message.correlation_id().to_string());
        headers.insert("X-Causation-ID", message.causation_id().to_string());
        headers.insert("X-Message-Type", message.message_type().as_str());
        
        let payload = serde_json::to_vec(&message)?;
        
        self.client
            .publish_with_headers(subject, headers, payload.into())
            .await?;
            
        Ok(())
    }
    
    pub async fn subscribe_correlated(
        &self,
        correlation_id: CorrelationId,
    ) -> Result<CorrelationSubscription> {
        // Subscribe to all messages with this correlation ID
        let subject = format!("{}.*", self.subject_prefix);
        let sub = self.client.subscribe(subject).await?;
        
        Ok(CorrelationSubscription {
            subscription: sub,
            correlation_id,
        })
    }
}

pub struct CorrelationSubscription {
    subscription: async_nats::Subscription,
    correlation_id: CorrelationId,
}

impl CorrelationSubscription {
    pub async fn next_in_correlation(&mut self) -> Option<impl Message> {
        while let Some(msg) = self.subscription.next().await {
            if let Some(corr_id) = msg.headers
                .as_ref()
                .and_then(|h| h.get("X-Correlation-ID"))
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
            {
                if corr_id == self.correlation_id.0 {
                    // Parse and return the message
                    return Some(parse_message(msg));
                }
            }
        }
        None
    }
}
```

### 4. Command Handler Pattern

All command handlers MUST propagate correlation:

```rust
#[async_trait]
pub trait CommandHandler<C: Command> {
    async fn handle(&self, command: C) -> Result<Vec<Event>>;
}

pub struct CorrelatedCommandHandler<H> {
    inner: H,
}

#[async_trait]
impl<C: Command, H: CommandHandler<C>> CommandHandler<C> for CorrelatedCommandHandler<H> {
    async fn handle(&self, command: C) -> Result<Vec<Event>> {
        let events = self.inner.handle(command).await?;
        
        // Ensure all events are properly correlated
        Ok(events.into_iter()
            .map(|event| {
                MessageFactory::create_caused_by(event, &command)
            })
            .collect())
    }
}
```

### 5. Validation Middleware

Implement validation to ensure correlation integrity:

```rust
pub struct CorrelationValidator;

impl CorrelationValidator {
    pub fn validate_message(&self, msg: &impl Message) -> Result<()> {
        // Rule 1: Message ID must be unique
        if msg.message_id() == &MessageId(Uuid::nil()) {
            return Err(Error::InvalidMessageId);
        }
        
        // Rule 2: Root messages must have self-correlation
        if msg.correlation_id() == msg.causation_id().into() 
           && msg.message_id() != msg.correlation_id().into() {
            return Err(Error::InvalidRootMessage);
        }
        
        Ok(())
    }
    
    pub fn validate_causation_chain(&self, messages: &[impl Message]) -> Result<()> {
        // Build causation graph
        let mut graph = HashMap::new();
        for msg in messages {
            graph.insert(msg.message_id(), msg.causation_id());
        }
        
        // Check for cycles
        for msg in messages {
            if self.has_cycle(msg.message_id(), &graph) {
                return Err(Error::CausationCycle);
            }
        }
        
        // Verify single root per correlation
        let mut roots_by_correlation = HashMap::new();
        for msg in messages {
            if msg.message_id() == msg.causation_id().into() {
                roots_by_correlation
                    .entry(msg.correlation_id())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
        }
        
        for (corr_id, count) in roots_by_correlation {
            if count != 1 {
                return Err(Error::MultipleRoots(corr_id));
            }
        }
        
        Ok(())
    }
    
    fn has_cycle(&self, start: &MessageId, graph: &HashMap<&MessageId, &CausationId>) -> bool {
        let mut visited = HashSet::new();
        let mut current = start;
        
        while visited.insert(current) {
            if let Some(next) = graph.get(current) {
                current = &MessageId(next.0);
                if current == start {
                    return true;
                }
            } else {
                break;
            }
        }
        
        false
    }
}
```

### 6. Testing Patterns

All modules MUST include correlation tests:

```rust
#[cfg(test)]
mod correlation_tests {
    use super::*;
    
    #[test]
    fn test_root_message_correlation() {
        let msg = MessageFactory::create_root(TestPayload::default());
        
        assert_eq!(msg.correlation_id(), msg.message_id().into());
        assert_eq!(msg.causation_id(), msg.message_id().into());
    }
    
    #[test]
    fn test_caused_message_correlation() {
        let root = MessageFactory::create_root(TestPayload::default());
        let caused = MessageFactory::create_caused_by(TestPayload::default(), &root);
        
        assert_eq!(caused.correlation_id(), root.correlation_id());
        assert_eq!(caused.causation_id(), root.message_id().into());
        assert_ne!(caused.message_id(), root.message_id());
    }
    
    #[tokio::test]
    async fn test_correlation_propagation_through_handler() {
        let handler = CorrelatedCommandHandler::new(TestHandler);
        let command = MessageFactory::create_root(TestCommand::default());
        
        let events = handler.handle(command).await.unwrap();
        
        for event in events {
            assert_eq!(event.correlation_id(), command.correlation_id());
            assert_eq!(event.causation_id(), command.message_id().into());
        }
    }
    
    #[test]
    fn test_causation_chain_validation() {
        let validator = CorrelationValidator;
        
        let root = MessageFactory::create_root(TestPayload::default());
        let child1 = MessageFactory::create_caused_by(TestPayload::default(), &root);
        let child2 = MessageFactory::create_caused_by(TestPayload::default(), &child1);
        
        let messages = vec![root, child1, child2];
        assert!(validator.validate_causation_chain(&messages).is_ok());
    }
}
```

## Migration Checklist

For each module:

- [ ] Update message structs to include correlation/causation fields
- [ ] Replace direct message creation with MessageFactory
- [ ] Update event publishers to include correlation headers
- [ ] Add correlation validation to command handlers
- [ ] Update tests to verify correlation propagation
- [ ] Add correlation-based queries to projections
- [ ] Update documentation with correlation examples

## Monitoring Implementation

```rust
pub struct CorrelationMetrics {
    transaction_starts: Counter,
    messages_per_correlation: Histogram,
    causation_chain_depth: Histogram,
    orphaned_messages: Counter,
}

impl CorrelationMetrics {
    pub fn record_message(&self, msg: &impl Message) {
        if msg.message_id() == msg.correlation_id().into() {
            self.transaction_starts.increment();
        }
    }
    
    pub fn record_correlation_group(&self, messages: &[impl Message]) {
        self.messages_per_correlation.record(messages.len() as f64);
        
        // Calculate max causation depth
        let depth = self.calculate_max_depth(messages);
        self.causation_chain_depth.record(depth as f64);
    }
}
```

## Common Pitfalls to Avoid

1. **Creating orphaned messages** - Always use MessageFactory
2. **Breaking correlation chains** - Pass causing message to all handlers
3. **Ignoring external correlations** - Honor correlation IDs from external systems
4. **Missing validation** - Validate all incoming messages
5. **Forgetting headers** - Always include correlation headers in NATS messages

## Enforcement

Add this to your CI pipeline:

```bash
# Check for direct message construction (should use factory)
! grep -r "MessageId::new()" --include="*.rs" | grep -v "MessageFactory"

# Ensure all event publishers include headers
! grep -r "publish(" --include="*.rs" | grep -v "publish_with_headers"

# Verify correlation tests exist
find . -name "*test*.rs" -exec grep -l "test_.*correlation" {} \; | wc -l
``` 