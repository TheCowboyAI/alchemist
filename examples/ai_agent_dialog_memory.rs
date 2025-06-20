//! AI Agent with Dialog Domain - Shows how agents use the Dialog domain for conversation tracking
//!
//! This example demonstrates:
//! - Creating dialogs for agent conversations
//! - Tracking turns and context
//! - Managing topics and context variables
//! - Using dialog events as episodic memory

use anyhow::Result;
use cim_domain_agent::{
    Agent, AgentType, Participant as AgentParticipant,
};
use cim_domain_dialog::{
    Dialog, DialogType, DialogStatus,
    StartDialog, AddTurn, SwitchContext, AddContextVariable,
    Participant, ParticipantType, ParticipantRole,
    Turn, TurnType, Message, MessageIntent,
    Topic, ContextVariable, ContextScope,
};
use cim_ipld::{ContentService, ContentType};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;
use tracing::{info, debug};

/// AI Agent with Dialog-based memory
struct DialogAgent {
    agent_id: Uuid,
    active_dialogs: HashMap<Uuid, Dialog>,
    content_service: ContentService,
}

impl DialogAgent {
    /// Create a new dialog-enabled agent
    async fn new(agent_id: Uuid, nats_client: async_nats::Client) -> Result<Self> {
        let content_service = ContentService::new(nats_client).await?;
        
        Ok(Self {
            agent_id,
            active_dialogs: HashMap::new(),
            content_service,
        })
    }
    
    /// Start a new conversation
    async fn start_conversation(&mut self, user_id: Uuid, initial_message: &str) -> Result<Uuid> {
        let dialog_id = Uuid::new_v4();
        
        // Create user participant
        let user = Participant {
            id: user_id,
            participant_type: ParticipantType::Human,
            role: ParticipantRole::Primary,
            name: "User".to_string(),
            metadata: HashMap::new(),
        };
        
        // Create dialog
        let dialog = Dialog::new(dialog_id, DialogType::Direct, user);
        
        // Add agent as participant
        let agent_participant = Participant {
            id: self.agent_id,
            participant_type: ParticipantType::AIAgent,
            role: ParticipantRole::Assistant,
            name: "AI Assistant".to_string(),
            metadata: HashMap::new(),
        };
        
        let mut dialog = dialog;
        dialog.add_participant(agent_participant)?;
        
        // Add initial user turn
        let user_turn = Turn::new(
            1,
            user_id,
            Message::text(initial_message).with_intent(MessageIntent::Question),
            TurnType::UserQuery,
        );
        
        dialog.add_turn(user_turn)?;
        
        // Detect initial topic
        let topic = self.detect_topic(initial_message).await?;
        dialog.switch_topic(topic)?;
        
        // Store dialog
        self.active_dialogs.insert(dialog_id, dialog);
        
        info!("Started new dialog: {}", dialog_id);
        Ok(dialog_id)
    }
    
    /// Process a user message in an existing dialog
    async fn process_message(
        &mut self,
        dialog_id: Uuid,
        user_id: Uuid,
        message: &str,
    ) -> Result<String> {
        let dialog = self.active_dialogs.get_mut(&dialog_id)
            .ok_or_else(|| anyhow::anyhow!("Dialog not found"))?;
        
        // Add user turn
        let turn_number = dialog.turns().len() as u32 + 1;
        let user_turn = Turn::new(
            turn_number,
            user_id,
            Message::text(message).with_intent(self.detect_intent(message)),
            TurnType::UserQuery,
        );
        
        dialog.add_turn(user_turn.clone())?;
        
        // Check if topic has changed
        if self.has_topic_changed(message, dialog.current_topic()).await? {
            let new_topic = self.detect_topic(message).await?;
            dialog.switch_topic(new_topic)?;
            info!("Switched to new topic: {:?}", dialog.current_topic());
        }
        
        // Extract and store context variables
        let variables = self.extract_context_variables(message).await?;
        for (name, value) in variables {
            let var = ContextVariable {
                name: name.clone(),
                value,
                scope: ContextScope::Dialog,
                set_at: Utc::now(),
                expires_at: None,
                source: user_id,
            };
            dialog.add_context_variable(var)?;
        }
        
        // Generate response based on dialog context
        let response = self.generate_contextual_response(dialog, message).await?;
        
        // Add agent turn
        let agent_turn = Turn::new(
            turn_number + 1,
            self.agent_id,
            Message::text(&response)
                .with_intent(MessageIntent::Answer)
                .with_embeddings(vec![0.1, 0.2, 0.3]), // Mock embeddings
            TurnType::AgentResponse,
        );
        
        dialog.add_turn(agent_turn)?;
        
        // Store conversation snippet in Object Store for future reference
        let snippet = self.create_conversation_snippet(dialog, 3).await?;
        let cid = self.content_service.store_content(
            snippet.as_bytes(),
            ContentType::Text,
            None,
        ).await?;
        debug!("Stored conversation snippet with CID: {}", cid);
        
        Ok(response)
    }
    
    /// Detect the intent of a message
    fn detect_intent(&self, message: &str) -> MessageIntent {
        // Simple intent detection based on keywords
        let lower = message.to_lowercase();
        
        if lower.contains("?") || lower.starts_with("what") || lower.starts_with("how") {
            MessageIntent::Question
        } else if lower.starts_with("please") || lower.contains("could you") {
            MessageIntent::Command
        } else if lower.contains("thanks") || lower.contains("thank you") {
            MessageIntent::Acknowledgment
        } else {
            MessageIntent::Statement
        }
    }
    
    /// Detect topic from message
    async fn detect_topic(&self, message: &str) -> Result<Topic> {
        // In a real implementation, this would use NLP/embeddings
        let keywords = message.split_whitespace()
            .filter(|w| w.len() > 4)
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>();
        
        let topic_name = keywords.first()
            .map(|k| k.to_string())
            .unwrap_or_else(|| "General".to_string());
        
        Ok(Topic::new(topic_name, keywords))
    }
    
    /// Check if topic has changed
    async fn has_topic_changed(&self, message: &str, current_topic: Option<&Topic>) -> Result<bool> {
        // Simple check - in reality would use semantic similarity
        if let Some(topic) = current_topic {
            let message_lower = message.to_lowercase();
            Ok(!topic.keywords.iter().any(|k| message_lower.contains(k)))
        } else {
            Ok(true)
        }
    }
    
    /// Extract context variables from message
    async fn extract_context_variables(&self, message: &str) -> Result<HashMap<String, serde_json::Value>> {
        let mut variables = HashMap::new();
        
        // Extract numbers
        if let Some(number) = message.split_whitespace()
            .find_map(|w| w.parse::<f64>().ok()) {
            variables.insert("last_number".to_string(), serde_json::json!(number));
        }
        
        // Extract time references
        if message.contains("tomorrow") {
            variables.insert("time_reference".to_string(), serde_json::json!("tomorrow"));
        }
        
        Ok(variables)
    }
    
    /// Generate response based on dialog context
    async fn generate_contextual_response(&self, dialog: &Dialog, message: &str) -> Result<String> {
        let context = dialog.context();
        let recent_turns = dialog.turns().iter().rev().take(5).collect::<Vec<_>>();
        
        // Build context summary
        let mut context_summary = String::new();
        if let Some(topic) = dialog.current_topic() {
            context_summary.push_str(&format!("Current topic: {}\n", topic.name));
        }
        
        context_summary.push_str(&format!("Turn count: {}\n", dialog.turns().len()));
        context_summary.push_str(&format!("Context variables: {:?}\n", context.variables.keys().collect::<Vec<_>>()));
        
        // In a real implementation, this would use an LLM with the full context
        Ok(format!(
            "Based on our conversation about '{}' (turn {}), here's my response to '{}'. \
             I'm aware of the following context: {}",
            dialog.current_topic().map(|t| &t.name).unwrap_or(&"general topics".to_string()),
            dialog.turns().len(),
            message,
            context_summary
        ))
    }
    
    /// Create a conversation snippet for storage
    async fn create_conversation_snippet(&self, dialog: &Dialog, last_n_turns: usize) -> Result<String> {
        let turns = dialog.turns();
        let start = turns.len().saturating_sub(last_n_turns);
        
        let mut snippet = String::new();
        snippet.push_str(&format!("Dialog {} - Snippet\n", dialog.id()));
        snippet.push_str(&format!("Topic: {:?}\n", dialog.current_topic().map(|t| &t.name)));
        snippet.push_str("---\n");
        
        for turn in &turns[start..] {
            let participant = dialog.participants()
                .get(&turn.participant_id)
                .map(|p| &p.name)
                .unwrap_or(&"Unknown".to_string());
            
            if let MessageContent::Text(text) = &turn.message.content {
                snippet.push_str(&format!("{}: {}\n", participant, text));
            }
        }
        
        Ok(snippet)
    }
    
    /// Get conversation summary
    async fn get_conversation_summary(&self, dialog_id: Uuid) -> Result<String> {
        let dialog = self.active_dialogs.get(&dialog_id)
            .ok_or_else(|| anyhow::anyhow!("Dialog not found"))?;
        
        let mut summary = String::new();
        summary.push_str(&format!("Dialog Summary (ID: {})\n", dialog_id));
        summary.push_str(&format!("Status: {:?}\n", dialog.status()));
        summary.push_str(&format!("Type: {:?}\n", dialog.dialog_type()));
        summary.push_str(&format!("Participants: {}\n", dialog.participants().len()));
        summary.push_str(&format!("Total turns: {}\n", dialog.turns().len()));
        
        if let Some(topic) = dialog.current_topic() {
            summary.push_str(&format!("Current topic: {} (relevance: {:.2})\n", 
                topic.name, 
                topic.current_relevance()
            ));
        }
        
        summary.push_str(&format!("Context variables: {:?}\n", 
            dialog.context().variables.keys().collect::<Vec<_>>()
        ));
        
        Ok(summary)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    // Connect to NATS
    let nats_client = async_nats::connect("nats://localhost:4222").await?;
    info!("Connected to NATS");
    
    // Create agent
    let agent_id = Uuid::new_v4();
    let mut agent = DialogAgent::new(agent_id, nats_client).await?;
    info!("Created dialog-enabled AI agent: {}", agent_id);
    
    // Simulate user conversation
    let user_id = Uuid::new_v4();
    
    // Start conversation
    let dialog_id = agent.start_conversation(
        user_id,
        "What is event sourcing and how does it work?"
    ).await?;
    
    println!("\n=== Conversation Started ===");
    println!("Dialog ID: {}", dialog_id);
    
    // Continue conversation
    let response1 = agent.process_message(
        dialog_id,
        user_id,
        "Can you explain CQRS as well?"
    ).await?;
    
    println!("\nUser: Can you explain CQRS as well?");
    println!("Agent: {}", response1);
    
    // Topic switch
    let response2 = agent.process_message(
        dialog_id,
        user_id,
        "Actually, let's talk about graph databases instead. What are they?"
    ).await?;
    
    println!("\nUser: Actually, let's talk about graph databases instead. What are they?");
    println!("Agent: {}", response2);
    
    // Reference context
    let response3 = agent.process_message(
        dialog_id,
        user_id,
        "How do they compare to what we discussed earlier?"
    ).await?;
    
    println!("\nUser: How do they compare to what we discussed earlier?");
    println!("Agent: {}", response3);
    
    // Get conversation summary
    let summary = agent.get_conversation_summary(dialog_id).await?;
    println!("\n=== Conversation Summary ===");
    println!("{}", summary);
    
    Ok(())
}

// Re-use MessageContent from dialog domain
use cim_domain_dialog::value_objects::MessageContent; 