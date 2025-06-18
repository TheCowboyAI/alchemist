# Dialog Domain Design

## Overview

The Dialog Domain is a standalone conversation management system that orchestrates multi-participant conversations through event-driven architecture. It manages conversation threads, maintains chain-of-thought context graphs, handles attachments, and enables fully interactive conversations between People and AI Agents.

## Core Concepts

### Dialog Aggregate
The Dialog is the aggregate root representing a complete conversation session with multiple participants, threads, and contexts.

```rust
pub struct Dialog {
    pub id: DialogId,
    pub title: String,
    pub description: Option<String>,
    pub status: DialogStatus,
    pub participants: HashMap<ParticipantId, Participant>,
    pub threads: Vec<ThreadId>,
    pub context_graph: ContextGraphId,
    pub policies: Vec<PolicyId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum DialogStatus {
    Active,
    Paused,
    Archived,
    Terminated,
}
```

### Participants
Participants can be either People (human users) or Agents (AI models), each with specific roles and capabilities.

```rust
pub enum Participant {
    Person {
        id: PersonId,
        name: String,
        role: ParticipantRole,
        permissions: HashSet<Permission>,
    },
    Agent {
        id: AgentId,
        name: String,
        model: ModelSelection,
        capabilities: HashSet<Capability>,
        system_prompt: Option<String>,
    },
}

pub struct ModelSelection {
    pub provider: String,        // e.g., "openai", "anthropic", "local"
    pub model_id: String,        // e.g., "gpt-4", "claude-3"
    pub parameters: ModelParams, // temperature, max_tokens, etc.
}

pub enum ParticipantRole {
    Moderator,
    Contributor,
    Observer,
    Expert(String), // Domain expert with specialty
}
```

### Conversation Threads
Threads organize messages into logical conversation flows with associated context.

```rust
pub struct ConversationThread {
    pub id: ThreadId,
    pub dialog_id: DialogId,
    pub title: Option<String>,
    pub messages: Vec<Message>,
    pub context_chain: Vec<ContextNode>,
    pub attachments: Vec<AttachmentId>,
    pub status: ThreadStatus,
}

pub struct Message {
    pub id: MessageId,
    pub thread_id: ThreadId,
    pub participant_id: ParticipantId,
    pub content: MessageContent,
    pub timestamp: DateTime<Utc>,
    pub in_reply_to: Option<MessageId>,
    pub reactions: Vec<Reaction>,
}

pub enum MessageContent {
    Text(String),
    Code { language: String, content: String },
    Query { query: String, response: Option<QueryResponse> },
    Attachment { attachment_id: AttachmentId, description: String },
    Mixed(Vec<MessageContent>),
}
```

### Context Graph
The Context Graph maintains chain-of-thought reasoning and provides relevant context from past conversations.

```rust
pub struct DialogContext {
    pub id: ContextId,
    pub dialog_id: DialogId,
    pub nodes: HashMap<NodeId, ContextNode>,
    pub edges: Vec<ContextEdge>,
    pub embeddings: HashMap<NodeId, Embedding>,
}

pub struct ContextNode {
    pub id: NodeId,
    pub content: ContextContent,
    pub source: ContextSource,
    pub relevance_score: f32,
    pub timestamp: DateTime<Utc>,
}

pub enum ContextContent {
    PastConversation { dialog_id: DialogId, summary: String },
    ConceptualKnowledge { concept_id: ConceptId, description: String },
    Document { document_id: DocumentId, excerpt: String },
    QueryResult { query: String, result: String },
}

pub enum ContextSource {
    Explicit,    // Manually attached by participant
    Inferred,    // AI-suggested based on conversation
    Retrieved,   // Retrieved from knowledge base
    Generated,   // Generated during conversation
}
```

### Dialog Policies
Policies govern conversation behavior, access control, and content management.

```rust
pub struct DialogPolicy {
    pub id: PolicyId,
    pub name: String,
    pub rules: Vec<PolicyRule>,
    pub priority: u32,
}

pub enum PolicyRule {
    AccessControl { participant: ParticipantMatcher, permissions: HashSet<Permission> },
    ContentFilter { filter_type: FilterType, action: FilterAction },
    RoutingRule { condition: RoutingCondition, target: ParticipantId },
    RetentionPolicy { duration: Duration, action: RetentionAction },
}
```

## Domain Events

### Core Dialog Events

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogEvent {
    // Dialog Lifecycle
    DialogCreated {
        dialog_id: DialogId,
        title: String,
        creator_id: ParticipantId,
        initial_context: Option<ContextId>,
    },
    DialogStatusChanged {
        dialog_id: DialogId,
        old_status: DialogStatus,
        new_status: DialogStatus,
        reason: String,
    },
    
    // Participant Management
    ParticipantJoined {
        dialog_id: DialogId,
        participant: Participant,
        invited_by: Option<ParticipantId>,
    },
    ParticipantLeft {
        dialog_id: DialogId,
        participant_id: ParticipantId,
        reason: LeaveReason,
    },
    ParticipantRoleChanged {
        dialog_id: DialogId,
        participant_id: ParticipantId,
        old_role: ParticipantRole,
        new_role: ParticipantRole,
    },
    
    // Messaging
    MessageSent {
        dialog_id: DialogId,
        thread_id: ThreadId,
        message: Message,
    },
    MessageEdited {
        dialog_id: DialogId,
        message_id: MessageId,
        old_content: MessageContent,
        new_content: MessageContent,
        edited_by: ParticipantId,
    },
    
    // Context Management
    ContextAttached {
        dialog_id: DialogId,
        context_node: ContextNode,
        attached_by: ParticipantId,
    },
    ContextGraphUpdated {
        dialog_id: DialogId,
        added_nodes: Vec<NodeId>,
        removed_nodes: Vec<NodeId>,
        updated_edges: Vec<ContextEdge>,
    },
    
    // Model Management
    ModelSwitched {
        dialog_id: DialogId,
        agent_id: AgentId,
        old_model: ModelSelection,
        new_model: ModelSelection,
        reason: String,
    },
    
    // Policy Application
    PolicyApplied {
        dialog_id: DialogId,
        policy_id: PolicyId,
        affected_participants: Vec<ParticipantId>,
    },
    PolicyViolation {
        dialog_id: DialogId,
        policy_id: PolicyId,
        violator_id: ParticipantId,
        action_taken: ViolationAction,
    },
}
```

## Commands

```rust
#[derive(Debug, Clone)]
pub enum DialogCommand {
    // Dialog Management
    CreateDialog {
        title: String,
        description: Option<String>,
        creator_id: ParticipantId,
        initial_participants: Vec<Participant>,
        policies: Vec<PolicyId>,
    },
    UpdateDialogStatus {
        dialog_id: DialogId,
        new_status: DialogStatus,
        reason: String,
    },
    
    // Participant Management
    InviteParticipant {
        dialog_id: DialogId,
        participant: Participant,
        invited_by: ParticipantId,
    },
    RemoveParticipant {
        dialog_id: DialogId,
        participant_id: ParticipantId,
        reason: String,
    },
    
    // Messaging
    SendMessage {
        dialog_id: DialogId,
        thread_id: ThreadId,
        sender_id: ParticipantId,
        content: MessageContent,
        attachments: Vec<AttachmentId>,
    },
    
    // Context Operations
    AttachContext {
        dialog_id: DialogId,
        context: ContextContent,
        source: ContextSource,
        attached_by: ParticipantId,
    },
    QueryContext {
        dialog_id: DialogId,
        query: String,
        requester_id: ParticipantId,
        scope: QueryScope,
    },
    
    // Model Operations
    SwitchModel {
        dialog_id: DialogId,
        agent_id: AgentId,
        new_model: ModelSelection,
        reason: String,
    },
    
    // Policy Operations
    ApplyPolicy {
        dialog_id: DialogId,
        policy_id: PolicyId,
        enforcer_id: ParticipantId,
    },
}
```

## Queries

```rust
#[derive(Debug, Clone)]
pub enum DialogQuery {
    // Dialog Queries
    GetDialog { dialog_id: DialogId },
    ListDialogs { participant_id: ParticipantId, status: Option<DialogStatus> },
    SearchDialogs { query: String, filters: SearchFilters },
    
    // Thread Queries
    GetThread { thread_id: ThreadId },
    ListThreads { dialog_id: DialogId },
    GetThreadMessages { thread_id: ThreadId, pagination: Pagination },
    
    // Context Queries
    GetContextGraph { dialog_id: DialogId },
    FindSimilarContext { content: String, threshold: f32 },
    GetContextHistory { dialog_id: DialogId, time_range: TimeRange },
    
    // Participant Queries
    GetParticipants { dialog_id: DialogId },
    GetParticipantHistory { participant_id: ParticipantId },
    
    // Analytics Queries
    GetDialogStats { dialog_id: DialogId },
    GetParticipationMetrics { participant_id: ParticipantId },
}
```

## Integration Points

### Cross-Domain Events

The Dialog Domain integrates with other CIM domains through events:

1. **Person Domain**: When a person joins a dialog
2. **Agent Domain**: When an AI agent participates
3. **Policy Domain**: When policies are applied to conversations
4. **Document Domain**: When documents are attached or referenced
5. **ConceptualSpaces Domain**: For semantic context and reasoning
6. **Workflow Domain**: For conversation workflows and automation

### NATS Subject Hierarchy

```
dialog.created
dialog.status.changed
dialog.participant.joined
dialog.participant.left
dialog.message.sent
dialog.context.attached
dialog.model.switched
dialog.policy.applied
```

## Value Objects

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DialogId(Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThreadId(Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageId(Uuid);

#[derive(Debug, Clone)]
pub struct Embedding(Vec<f32>);

#[derive(Debug, Clone)]
pub struct QueryResponse {
    pub query: String,
    pub results: Vec<QueryResult>,
    pub confidence: f32,
    pub sources: Vec<ContextSource>,
}

#[derive(Debug, Clone)]
pub struct Attachment {
    pub id: AttachmentId,
    pub filename: String,
    pub content_type: String,
    pub size: usize,
    pub cid: Cid,
}
```

## Projections

### ConversationView Projection
Optimized for displaying conversation threads with context.

```rust
pub struct ConversationView {
    pub dialog_id: DialogId,
    pub title: String,
    pub participants: Vec<ParticipantSummary>,
    pub recent_messages: Vec<MessageView>,
    pub active_context: Vec<ContextSummary>,
    pub unread_count: usize,
}
```

### ContextGraph Projection
Maintains the semantic relationships between conversation contexts.

```rust
pub struct ContextGraphProjection {
    pub dialog_id: DialogId,
    pub nodes: HashMap<NodeId, ContextNodeView>,
    pub edges: Vec<ContextEdgeView>,
    pub clusters: Vec<ContextCluster>,
}
```

### ParticipantActivity Projection
Tracks participant engagement and contributions.

```rust
pub struct ParticipantActivityProjection {
    pub participant_id: ParticipantId,
    pub dialogs: Vec<DialogSummary>,
    pub message_count: usize,
    pub context_contributions: usize,
    pub last_active: DateTime<Utc>,
}
```

## Implementation Considerations

### Performance
- Use embeddings for efficient context retrieval
- Cache frequently accessed context nodes
- Implement pagination for message history
- Use NATS JetStream for reliable message delivery

### Security
- Encrypt sensitive message content
- Implement fine-grained access control
- Audit all participant actions
- Validate content against policies

### Scalability
- Partition dialogs by ID for horizontal scaling
- Use read replicas for query projections
- Implement archival for old conversations
- Optimize context graph traversal

### User Experience
- Real-time message delivery via NATS
- Typing indicators and presence
- Rich message formatting
- Context suggestions based on conversation

## Testing Strategy

1. **Unit Tests**: Test aggregate logic, command validation
2. **Integration Tests**: Test cross-domain event handling
3. **Scenario Tests**: Test complete conversation flows
4. **Performance Tests**: Test with large context graphs
5. **Security Tests**: Test policy enforcement

## Future Enhancements

1. **Voice/Video Integration**: Support for multimedia conversations
2. **Translation Services**: Multi-language support
3. **Sentiment Analysis**: Track conversation tone
4. **Advanced Routing**: ML-based participant routing
5. **Conversation Templates**: Predefined dialog structures 