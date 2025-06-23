# Event Correlation and Causation Algebra

## Overview

This document defines the formal algebra for event correlation and causation in CIM's event-driven architecture. This algebra provides a flexible grouping mechanism for event routing, transaction tracking, and distributed tracing.

## Core Concepts

### Message Types
All messages in the system are one of:
- **Command**: An intent to change state
- **Query**: A request for information  
- **Event**: A fact that something has happened

### Identity Fields

#### MessageId
- Every message has a unique identifier
- Generated at message creation time
- Immutable throughout the message lifecycle

#### CorrelationId
- Groups related messages within a transaction or workflow
- For the first message in a transaction: `CorrelationId = MessageId` (self-correlation)
- All subsequent messages in the transaction inherit the same CorrelationId
- Non-transactional messages: `CorrelationId = MessageId`

#### CausationId
- Identifies the message that directly caused this message to be created
- For the first message in a transaction: `CausationId = MessageId = CorrelationId`
- For subsequent messages: `CausationId = MessageId of the causing message`
- Forms a causation chain/tree within a correlation group

**Note**: Timestamps are metadata OUTSIDE the correlation algebra. The algebra deals only with identity relationships, not temporal ordering.

## The Algebra

### Basic Formula
```
EventRoutingKey = Subject + CorrelationId + CausationId
```

### Properties

#### 1. Self-Correlation Property
For any initial message `m₀`:
```
m₀.CorrelationId = m₀.MessageId
m₀.CausationId = m₀.MessageId
```

#### 2. Transaction Propagation Property
For any message `mᵢ` caused by message `mⱼ` within transaction `T`:
```
mᵢ.CorrelationId = T.CorrelationId = m₀.MessageId
mᵢ.CausationId = mⱼ.MessageId
```

#### 3. Singleton Property
For any non-transactional message `m`:
```
m.CorrelationId = m.CausationId = m.MessageId
```

#### 4. Causation Chain Property
Within a correlation group, messages form a directed acyclic graph (DAG) where:
- Nodes are messages
- Edges represent causation relationships
- The root node has `CausationId = CorrelationId`

### Operations

#### 1. Group By Correlation
```rust
fn group_by_correlation(messages: Vec<Message>) -> HashMap<CorrelationId, Vec<Message>> {
    messages.into_iter()
        .fold(HashMap::new(), |mut acc, msg| {
            acc.entry(msg.correlation_id).or_default().push(msg);
            acc
        })
}
```

#### 2. Build Causation Tree
```rust
fn build_causation_tree(messages: Vec<Message>) -> CausationTree {
    let mut tree = CausationTree::new();
    
    // Find root (where causation_id == correlation_id)
    let root = messages.iter()
        .find(|m| m.causation_id == m.correlation_id)
        .expect("Every correlation group must have a root");
    
    // Build tree structure
    tree.add_root(root);
    for msg in messages {
        if msg.message_id != root.message_id {
            tree.add_child(msg.causation_id, msg);
        }
    }
    
    tree
}
```

#### 3. Transaction Boundary Detection
```rust
fn is_transaction_boundary(msg: &Message) -> bool {
    msg.correlation_id == msg.message_id
}
```

## Implementation Guidelines

### 1. Message Creation

```rust
impl Message {
    /// Create a new root message (starts a transaction)
    pub fn new_root(payload: impl Into<MessagePayload>) -> Self {
        let id = MessageId::new();
        Self {
            message_id: id.clone(),
            correlation_id: id.clone(),
            causation_id: id,
            payload: payload.into(),
        }
    }
    
    /// Create a message caused by another message
    pub fn new_caused_by(
        payload: impl Into<MessagePayload>,
        causing_message: &Message,
    ) -> Self {
        Self {
            message_id: MessageId::new(),
            correlation_id: causing_message.correlation_id.clone(),
            causation_id: causing_message.message_id.clone(),
            payload: payload.into(),
        }
    }
}
```

### 2. Event Routing

```rust
pub struct EventRouter {
    routes: HashMap<RouteKey, Vec<Handler>>,
}

#[derive(Hash, Eq, PartialEq)]
pub struct RouteKey {
    subject: Subject,
    correlation_pattern: Option<CorrelationPattern>,
}

pub enum CorrelationPattern {
    /// Route all messages with specific correlation
    Exact(CorrelationId),
    /// Route only transaction roots
    RootOnly,
    /// Route only caused messages (not roots)
    CausedOnly,
    /// Route by causation depth
    DepthRange(Range<usize>),
}
```

### 3. NATS Integration

```rust
impl NatsEventBridge {
    pub async fn publish_with_correlation(
        &self,
        subject: &str,
        message: Message,
    ) -> Result<()> {
        let mut headers = HeaderMap::new();
        headers.insert("X-Message-ID", message.message_id.to_string());
        headers.insert("X-Correlation-ID", message.correlation_id.to_string());
        headers.insert("X-Causation-ID", message.causation_id.to_string());
        // Timestamp is added separately as metadata, not part of correlation
        
        self.client
            .publish_with_headers(subject, headers, message.encode()?)
            .await?;
            
        Ok(())
    }
}
```

## Use Cases

### 1. Distributed Transaction Tracing
```rust
// Start a new order transaction
let create_order_cmd = Command::new_root(CreateOrder { ... });

// All subsequent messages share correlation
let validate_payment_cmd = Command::new_caused_by(
    ValidatePayment { ... },
    &create_order_cmd
);

let reserve_inventory_cmd = Command::new_caused_by(
    ReserveInventory { ... },
    &create_order_cmd
);

// Events maintain the chain
let order_created_event = Event::new_caused_by(
    OrderCreated { ... },
    &create_order_cmd
);
```

### 2. Saga Orchestration
```rust
pub struct SagaOrchestrator {
    correlation_id: CorrelationId,
    steps: Vec<SagaStep>,
}

impl SagaOrchestrator {
    pub fn execute_step(&self, step: &SagaStep, causing_event: &Event) -> Command {
        Command::new_caused_by(step.command.clone(), causing_event)
    }
}
```

### 3. Event Sourcing with Causation
```rust
pub struct EventStore {
    pub fn get_transaction_events(&self, correlation_id: &CorrelationId) -> Vec<Event> {
        self.events
            .iter()
            .filter(|e| e.correlation_id == *correlation_id)
            .cloned()
            .collect()
    }
    
    pub fn get_causation_chain(&self, message_id: &MessageId) -> Vec<Event> {
        let mut chain = vec![];
        let mut current_id = Some(message_id.clone());
        
        while let Some(id) = current_id {
            if let Some(event) = self.find_by_message_id(&id) {
                chain.push(event.clone());
                current_id = if event.causation_id != event.message_id {
                    Some(event.causation_id.clone())
                } else {
                    None // Reached root
                };
            } else {
                break;
            }
        }
        
        chain.reverse(); // Root first
        chain
    }
}
```

## Validation Rules

1. **Correlation Consistency**: All messages with the same CorrelationId must form a connected graph through causation relationships

2. **Root Uniqueness**: Each correlation group must have exactly one root message where `MessageId = CorrelationId = CausationId`

3. **Causation Validity**: A message's CausationId must reference a MessageId that exists within the same correlation group

4. **No Cycles**: The causation graph must be acyclic (no message can cause itself, directly or indirectly)

5. **Identity Immutability**: MessageId, CorrelationId, and CausationId are immutable once assigned

## Benefits

1. **Transaction Tracking**: Easy identification of all messages within a business transaction
2. **Distributed Tracing**: Natural support for distributed system observability
3. **Event Replay**: Ability to replay transactions in correct causal order
4. **Debugging**: Clear visualization of cause-and-effect relationships
5. **Audit Trail**: Complete lineage of every system action
6. **Flexible Routing**: Route messages based on correlation patterns, not just subjects

## Migration Strategy

For existing systems without correlation/causation:

1. **Phase 1**: Add fields as optional, default to self-correlation
2. **Phase 2**: Update message creation to properly set causation
3. **Phase 3**: Implement routing based on correlation patterns
4. **Phase 4**: Add validation and monitoring
5. **Phase 5**: Make fields required

## Monitoring and Observability

### Metrics
- Transaction completion rate by correlation
- Average causation chain depth
- Orphaned messages (invalid causation references)
- Transaction duration (first to last message in correlation)

### Alerts
- Correlation groups exceeding size threshold
- Causation chains exceeding depth threshold
- Messages with future-dated causation
- Circular causation detected

## Conclusion

This algebra provides a robust foundation for event correlation and causation tracking in distributed systems. By enforcing these patterns consistently, we gain powerful capabilities for transaction tracking, debugging, and system observability while maintaining the flexibility of event-driven architectures. 