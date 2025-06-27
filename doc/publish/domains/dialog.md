# Dialog Domain

## Overview

The Dialog Domain manages conversational interfaces and natural language interactions within CIM. It provides structured dialog management, intent recognition, context tracking, and multi-turn conversation capabilities for both human-to-system and human-to-human communications.

## Key Concepts

### Conversation
- **Definition**: A sequence of related messages between participants
- **Properties**: ID, participants, context, state, history
- **Types**: Chat, voice, structured dialog, free-form
- **Lifecycle**: Initiated → Active → Paused → Completed

### Message
- **Definition**: A single unit of communication in a conversation
- **Components**: Content, sender, timestamp, intent, entities
- **Types**: Text, voice, rich media, system message
- **Processing**: NLU analysis, intent extraction, response generation

### Dialog Flow
- **Definition**: Structured conversation path with states and transitions
- **Components**: States, intents, actions, conditions
- **Design**: Visual flow editor, conditional branching
- **Execution**: State machine, context-aware transitions

### Conversation Context
- **Definition**: Accumulated state and understanding during dialog
- **Contents**: Variables, entities, history, user preferences
- **Persistence**: Session-based or long-term memory
- **Usage**: Personalization, continuity, disambiguation

## Domain Events

### Commands
- `cmd.dialog.start_conversation` - Begin new dialog
- `cmd.dialog.send_message` - Add message to conversation
- `cmd.dialog.process_intent` - Handle recognized intent
- `cmd.dialog.update_context` - Modify conversation state
- `cmd.dialog.end_conversation` - Complete dialog

### Events
- `event.dialog.conversation_started` - Dialog initiated
- `event.dialog.message_received` - New message added
- `event.dialog.intent_recognized` - Intent identified
- `event.dialog.response_generated` - Reply created
- `event.dialog.conversation_ended` - Dialog completed

### Queries
- `query.dialog.get_conversation` - Retrieve dialog history
- `query.dialog.find_by_intent` - Search by intent type
- `query.dialog.get_context` - Current conversation state
- `query.dialog.analyze_sentiment` - Emotional analysis

## API Reference

### DialogAggregate
```rust
pub struct DialogAggregate {
    pub id: ConversationId,
    pub participants: Vec<ParticipantId>,
    pub messages: Vec<Message>,
    pub context: ConversationContext,
    pub flow_state: Option<FlowState>,
    pub metadata: DialogMetadata,
}
```

### Key Methods
- `start_conversation()` - Initialize dialog
- `add_message()` - Process new message
- `recognize_intent()` - Analyze message intent
- `generate_response()` - Create reply
- `update_context()` - Modify state

## Conversation Management

### Starting Conversations
```rust
// Initialize chat conversation
let conversation = StartConversation {
    participants: vec![user_id, assistant_id],
    conversation_type: ConversationType::Chat,
    initial_context: ConversationContext {
        language: "en".to_string(),
        timezone: "America/New_York".to_string(),
        preferences: user_preferences,
        variables: HashMap::new(),
    },
    flow_id: Some(customer_support_flow),
};

// Voice conversation
let voice_dialog = StartVoiceDialog {
    caller_id: phone_number,
    ivr_flow: ivr_flow_id,
    speech_config: SpeechConfig {
        language: "en-US".to_string(),
        voice: "neural-voice-1".to_string(),
        speed: 1.0,
        pitch: 0.0,
    },
};
```

### Message Processing
```rust
// Process incoming message
let message = ProcessMessage {
    conversation_id,
    message: Message {
        sender_id,
        content: MessageContent::Text(
            "I need help with my order #12345".to_string()
        ),
        timestamp: SystemTime::now(),
        channel: Channel::Web,
    },
};

// NLU analysis result
let analysis = MessageAnalysis {
    intent: Intent {
        name: "order_inquiry".to_string(),
        confidence: 0.92,
    },
    entities: vec![
        Entity {
            entity_type: "order_number".to_string(),
            value: "12345".to_string(),
            position: (28, 33),
        },
    ],
    sentiment: Sentiment::Neutral,
    language: "en".to_string(),
};

// Generate response
let response = GenerateResponse {
    conversation_id,
    analysis,
    response_type: ResponseType::Natural,
    include_suggestions: true,
};
```

## Dialog Flow Design

### Flow Definition
```rust
// Customer support flow
let support_flow = DialogFlow {
    id: FlowId::new(),
    name: "Customer Support".to_string(),
    states: vec![
        FlowState {
            id: "greeting".to_string(),
            message: "Hello! How can I help you today?".to_string(),
            transitions: vec![
                Transition {
                    intent: "order_inquiry".to_string(),
                    next_state: "order_help".to_string(),
                },
                Transition {
                    intent: "technical_support".to_string(),
                    next_state: "tech_help".to_string(),
                },
            ],
        },
        FlowState {
            id: "order_help".to_string(),
            message: "I'll help you with your order. What's your order number?".to_string(),
            entity_prompt: Some("order_number".to_string()),
            action: Some(Action::LookupOrder),
        },
    ],
    fallback_state: "clarification".to_string(),
};

// Conditional branching
let conditional_flow = FlowState {
    id: "check_status".to_string(),
    conditions: vec![
        Condition {
            expression: "order.status == 'shipped'".to_string(),
            next_state: "tracking_info".to_string(),
        },
        Condition {
            expression: "order.status == 'processing'".to_string(),
            next_state: "processing_info".to_string(),
        },
    ],
    default_next: "status_unknown".to_string(),
};
```

### Multi-turn Dialogs
```rust
// Form filling flow
let form_flow = FormDialog {
    id: "reservation_form".to_string(),
    fields: vec![
        FormField {
            name: "date".to_string(),
            prompt: "What date would you like to reserve?".to_string(),
            entity_type: "date".to_string(),
            validation: Some(Validation::FutureDate),
            required: true,
        },
        FormField {
            name: "party_size".to_string(),
            prompt: "How many people?".to_string(),
            entity_type: "number".to_string(),
            validation: Some(Validation::Range(1, 20)),
            required: true,
        },
        FormField {
            name: "special_requests".to_string(),
            prompt: "Any special requests?".to_string(),
            entity_type: "text".to_string(),
            required: false,
        },
    ],
    confirmation: "Great! I have a reservation for {party_size} on {date}. {special_requests}".to_string(),
};

// Slot filling
let slot_filling = SlotFillingStrategy {
    missing_slot_prompts: HashMap::from([
        ("date", "What date works for you?"),
        ("time", "What time would you prefer?"),
    ]),
    confirmation_required: true,
    allow_corrections: true,
};
```

## Natural Language Understanding

### Intent Recognition
```rust
// Intent classifier configuration
let intent_classifier = IntentClassifier {
    model: "intent-classifier-v2".to_string(),
    confidence_threshold: 0.7,
    fallback_intent: "unclear".to_string(),
    training_data: TrainingData {
        intents: vec![
            IntentExamples {
                intent: "greeting".to_string(),
                examples: vec![
                    "hello", "hi", "hey", "good morning",
                    "how are you", "what's up",
                ],
            },
            IntentExamples {
                intent: "order_inquiry".to_string(),
                examples: vec![
                    "where is my order",
                    "track my package",
                    "order status",
                    "when will it arrive",
                ],
            },
        ],
    },
};

// Entity extraction
let entity_extractor = EntityExtractor {
    patterns: vec![
        EntityPattern {
            entity_type: "order_number".to_string(),
            pattern: Regex::new(r"\b\d{5}\b").unwrap(),
        },
        EntityPattern {
            entity_type: "email".to_string(),
            pattern: Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap(),
        },
    ],
    ml_models: vec!["ner-model-v3"],
};
```

### Context Management
```rust
// Conversation context
let context = ConversationContext {
    // User information
    user_profile: UserProfile {
        name: "John Doe".to_string(),
        preferences: HashMap::from([
            ("communication_style", "formal"),
            ("language", "en"),
        ]),
        history_summary: "Regular customer, 5 previous orders".to_string(),
    },
    
    // Current conversation state
    current_topic: "order_inquiry".to_string(),
    mentioned_entities: HashMap::from([
        ("order_number", "12345"),
        ("product", "laptop"),
    ]),
    
    // Dialog variables
    variables: HashMap::from([
        ("order_status", Value::String("shipped")),
        ("tracking_number", Value::String("1Z999AA1")),
    ]),
    
    // Conversation history summary
    history_summary: vec![
        "User asked about order 12345",
        "System provided shipping status",
    ],
};

// Context updates
let update = UpdateContext {
    conversation_id,
    updates: vec![
        ContextUpdate::SetVariable("resolved", Value::Bool(true)),
        ContextUpdate::AddEntity("case_number", "CS-789"),
        ContextUpdate::SetTopic("feedback"),
    ],
};
```

## Response Generation

### Natural Language Generation
```rust
// Response templates
let response_template = ResponseTemplate {
    intent: "order_shipped".to_string(),
    templates: vec![
        "Your order {order_id} has been shipped and will arrive by {delivery_date}.",
        "Great news! Order {order_id} is on its way. Expected delivery: {delivery_date}.",
        "I've checked and your order {order_id} shipped today. You should receive it by {delivery_date}.",
    ],
    variables: vec!["order_id", "delivery_date"],
};

// Dynamic response generation
let response_generator = ResponseGenerator {
    personality: Personality {
        tone: Tone::Friendly,
        formality: Formality::Casual,
        verbosity: Verbosity::Concise,
    },
    capabilities: vec![
        Capability::Empathy,
        Capability::Clarification,
        Capability::Suggestions,
    ],
};

// Rich responses
let rich_response = RichResponse {
    text: "Here's your order status:".to_string(),
    attachments: vec![
        Attachment::Card(Card {
            title: "Order #12345".to_string(),
            subtitle: "Shipped via FedEx".to_string(),
            image_url: Some("https://example.com/package.png".to_string()),
            buttons: vec![
                Button {
                    text: "Track Package".to_string(),
                    action: Action::Url("https://track.fedex.com/...".to_string()),
                },
            ],
        }),
    ],
    suggestions: vec![
        "Check delivery instructions",
        "Contact support",
        "View order details",
    ],
};
```

## Integration Features

### Channel Integration
```rust
// Multi-channel support
pub enum Channel {
    Web,
    Mobile,
    Voice,
    SMS,
    Email,
    Slack,
    Teams,
}

// Channel-specific formatting
impl MessageFormatter for Channel {
    fn format(&self, response: &Response) -> String {
        match self {
            Channel::SMS => truncate_for_sms(response),
            Channel::Voice => convert_to_ssml(response),
            Channel::Slack => format_slack_blocks(response),
            _ => response.text.clone(),
        }
    }
}
```

### Agent Handoff
```rust
// Transfer to human agent
let handoff = HandoffToAgent {
    conversation_id,
    reason: HandoffReason::CustomerRequest,
    priority: Priority::High,
    context_summary: "Customer upset about delayed order".to_string(),
    suggested_agent_skills: vec!["escalation", "refunds"],
};

// Agent availability check
let available_agents = FindAvailableAgents {
    skills: vec!["customer_service", "order_support"],
    language: "en".to_string(),
    max_wait_time: Duration::minutes(5),
};
```

## Analytics and Insights

### Conversation Analytics
```rust
// Analyze conversations
let analytics = AnalyzeConversations {
    time_range: TimeRange::LastDays(30),
    metrics: vec![
        Metric::AverageLength,
        Metric::ResolutionRate,
        Metric::SentimentTrend,
        Metric::IntentDistribution,
    ],
};

// Analytics results
let results = ConversationAnalytics {
    total_conversations: 5234,
    average_messages: 8.3,
    resolution_rate: 0.78,
    average_duration: Duration::minutes(12),
    top_intents: vec![
        ("order_inquiry", 34.5),
        ("technical_support", 28.2),
        ("billing", 18.7),
    ],
    sentiment_breakdown: SentimentBreakdown {
        positive: 0.42,
        neutral: 0.45,
        negative: 0.13,
    },
};
```

## Use Cases

### Customer Service
- Automated support chat
- FAQ handling
- Ticket creation
- Escalation management

### Virtual Assistants
- Task automation
- Information retrieval
- Scheduling
- Reminders

### Voice Interfaces
- IVR systems
- Voice commands
- Call routing
- Transcription

### Internal Communications
- Team chat
- Announcement distribution
- Meeting scheduling
- Knowledge sharing

## Performance Characteristics

- **Message Processing**: <100ms latency
- **Intent Recognition**: <50ms
- **Context Retrieval**: <10ms
- **Concurrent Conversations**: 10,000+

## Best Practices

1. **Context Preservation**: Maintain conversation continuity
2. **Fallback Handling**: Graceful unclear intent handling
3. **Response Variety**: Avoid repetitive responses
4. **Privacy**: Secure handling of conversation data
5. **Testing**: Comprehensive dialog flow testing

## Related Domains

- **Agent Domain**: AI-powered responses
- **Person Domain**: User preferences
- **Workflow Domain**: Task automation
- **Policy Domain**: Conversation policies
