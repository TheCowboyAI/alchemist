//! AI Agent with Memory - Demonstrates how agents use Event/Object stores as memory
//!
//! This example shows:
//! - Creating an AI agent with memory capabilities
//! - Storing experiences as events (episodic memory)
//! - Storing knowledge as objects (semantic memory)
//! - Building projections for reasoning (working memory)
//! - Querying memory to make decisions

use anyhow::Result;
use async_nats::jetstream;
use chrono::Utc;
use cim_domain_agent::{
    ActivateAgent, Agent, AgentCommandHandler, AgentMetadata, AgentType, CapabilitiesComponent,
    DeployAgent, EnableAgentTools, ToolDefinition,
};
use cim_ipld::{ContentService, ContentType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio;
use tracing::{debug, info};
use uuid::Uuid;

/// Events representing agent experiences (episodic memory)
#[derive(Debug, Clone, Serialize, Deserialize)]
enum AgentMemoryEvent {
    ConversationStarted {
        agent_id: Uuid,
        user_id: Uuid,
        topic: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    UserQueryReceived {
        agent_id: Uuid,
        query: String,
        context: HashMap<String, String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    KnowledgeAccessed {
        agent_id: Uuid,
        knowledge_cid: String,
        relevance_score: f32,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    ResponseGenerated {
        agent_id: Uuid,
        response: String,
        confidence: f32,
        sources: Vec<String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    FeedbackReceived {
        agent_id: Uuid,
        feedback_type: String,
        value: f32,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Agent's working memory projection
#[derive(Debug, Clone)]
struct AgentWorkingMemory {
    agent_id: Uuid,
    current_context: HashMap<String, String>,
    recent_queries: Vec<String>,
    knowledge_access_patterns: HashMap<String, f32>, // CID -> access frequency
    user_preferences: HashMap<String, String>,
    expertise_areas: HashSet<String>,
}

/// Memory-enabled AI Agent
struct MemoryAgent {
    agent: Agent,
    event_store: Arc<dyn EventStore>,
    object_store: Arc<ContentService>,
    working_memory: AgentWorkingMemory,
}

impl MemoryAgent {
    /// Create a new memory-enabled agent
    async fn new(agent_id: Uuid, owner_id: Uuid, nats_client: async_nats::Client) -> Result<Self> {
        // Initialize stores
        let jetstream = jetstream::new(nats_client.clone());
        let event_store = Arc::new(DistributedEventStore::new(jetstream).await?);
        let object_store = Arc::new(ContentService::new(nats_client).await?);

        // Create agent
        let agent = Agent::new(agent_id, AgentType::AI, owner_id);

        // Initialize working memory
        let working_memory = AgentWorkingMemory {
            agent_id,
            current_context: HashMap::new(),
            recent_queries: Vec::new(),
            knowledge_access_patterns: HashMap::new(),
            user_preferences: HashMap::new(),
            expertise_areas: HashSet::new(),
        };

        Ok(Self {
            agent,
            event_store,
            object_store,
            working_memory,
        })
    }

    /// Process a user query using memory
    async fn process_query(&mut self, query: &str, user_id: Uuid) -> Result<String> {
        info!("Processing query: {}", query);

        // 1. Store the query as an episodic memory event
        self.store_experience(AgentMemoryEvent::UserQueryReceived {
            agent_id: self.agent.id(),
            query: query.to_string(),
            context: self.working_memory.current_context.clone(),
            timestamp: Utc::now(),
        })
        .await?;

        // 2. Search semantic memory for relevant knowledge
        let relevant_knowledge = self.search_knowledge(query).await?;

        // 3. Access and track knowledge usage
        let mut sources = Vec::new();
        for (cid, relevance) in &relevant_knowledge {
            self.store_experience(AgentMemoryEvent::KnowledgeAccessed {
                agent_id: self.agent.id(),
                knowledge_cid: cid.clone(),
                relevance_score: *relevance,
                timestamp: Utc::now(),
            })
            .await?;
            sources.push(cid.clone());

            // Update access patterns in working memory
            *self
                .working_memory
                .knowledge_access_patterns
                .entry(cid.clone())
                .or_insert(0.0) += 1.0;
        }

        // 4. Generate response based on memory
        let response = self.generate_response(query, &relevant_knowledge).await?;

        // 5. Store the response as episodic memory
        self.store_experience(AgentMemoryEvent::ResponseGenerated {
            agent_id: self.agent.id(),
            response: response.clone(),
            confidence: 0.85, // Example confidence
            sources,
            timestamp: Utc::now(),
        })
        .await?;

        // 6. Update working memory
        self.working_memory.recent_queries.push(query.to_string());
        if self.working_memory.recent_queries.len() > 10 {
            self.working_memory.recent_queries.remove(0);
        }

        Ok(response)
    }

    /// Store an experience in episodic memory (Event Store)
    async fn store_experience(&self, event: AgentMemoryEvent) -> Result<()> {
        debug!("Storing experience: {:?}", event);

        // In a real implementation, this would serialize and store in Event Store
        // For now, we'll simulate it
        info!("Stored experience in Event Store");
        Ok(())
    }

    /// Search semantic memory (Object Store) for relevant knowledge
    async fn search_knowledge(&self, query: &str) -> Result<Vec<(String, f32)>> {
        debug!("Searching knowledge for: {}", query);

        // In a real implementation, this would:
        // 1. Use embeddings to find similar content
        // 2. Query the Object Store for relevant CIDs
        // 3. Return ranked results

        // Simulate finding relevant knowledge
        let mock_results = vec![
            (
                "bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku".to_string(),
                0.92,
            ),
            (
                "bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku".to_string(),
                0.87,
            ),
        ];

        Ok(mock_results)
    }

    /// Generate a response using retrieved knowledge
    async fn generate_response(&self, query: &str, knowledge: &[(String, f32)]) -> Result<String> {
        // In a real implementation, this would:
        // 1. Retrieve content from Object Store using CIDs
        // 2. Use LLM to generate response based on knowledge
        // 3. Include working memory context

        Ok(format!(
            "Based on {} relevant knowledge sources, here's my response to '{}': \
             This is a simulated response that would be generated by analyzing the retrieved content.",
            knowledge.len(),
            query
        ))
    }

    /// Learn from feedback to improve future responses
    async fn learn_from_feedback(&mut self, feedback_type: &str, value: f32) -> Result<()> {
        // Store feedback as episodic memory
        self.store_experience(AgentMemoryEvent::FeedbackReceived {
            agent_id: self.agent.id(),
            feedback_type: feedback_type.to_string(),
            value,
            timestamp: Utc::now(),
        })
        .await?;

        // Update working memory based on feedback
        if value > 0.8 {
            // Positive feedback - remember what worked
            if let Some(last_query) = self.working_memory.recent_queries.last() {
                // Extract topic and add to expertise
                let topic = extract_topic(last_query);
                self.working_memory.expertise_areas.insert(topic);
            }
        }

        Ok(())
    }

    /// Save important knowledge to semantic memory
    async fn store_knowledge(&self, content: &str, content_type: ContentType) -> Result<String> {
        info!("Storing knowledge in Object Store");

        // Store in Object Store and get CID
        let cid = self
            .object_store
            .store_content(
                content.as_bytes(),
                content_type,
                None, // Let it auto-detect domain
            )
            .await?;

        info!("Stored knowledge with CID: {}", cid);
        Ok(cid.to_string())
    }
}

/// Extract topic from query (simplified)
fn extract_topic(query: &str) -> String {
    // In a real implementation, this would use NLP
    query
        .split_whitespace()
        .next()
        .unwrap_or("general")
        .to_string()
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Connect to NATS
    let nats_client = async_nats::connect("nats://localhost:4222").await?;
    info!("Connected to NATS");

    // Create agent
    let agent_id = Uuid::new_v4();
    let owner_id = Uuid::new_v4();

    let mut agent = MemoryAgent::new(agent_id, owner_id, nats_client).await?;
    info!("Created memory-enabled AI agent: {}", agent_id);

    // Store some knowledge in semantic memory
    let doc_cid = agent
        .store_knowledge(
            "The Composable Information Machine (CIM) uses event sourcing and CQRS patterns.",
            ContentType::Markdown,
        )
        .await?;
    info!("Stored knowledge document: {}", doc_cid);

    // Simulate user interaction
    let user_id = Uuid::new_v4();

    // Start conversation
    agent
        .store_experience(AgentMemoryEvent::ConversationStarted {
            agent_id,
            user_id,
            topic: "CIM Architecture".to_string(),
            timestamp: Utc::now(),
        })
        .await?;

    // Process queries
    let response1 = agent.process_query("What is CIM?", user_id).await?;
    println!("\nQuery: What is CIM?");
    println!("Response: {}", response1);

    // Simulate positive feedback
    agent.learn_from_feedback("helpful", 0.9).await?;

    // Another query building on context
    let response2 = agent
        .process_query("How does it use event sourcing?", user_id)
        .await?;
    println!("\nQuery: How does it use event sourcing?");
    println!("Response: {}", response2);

    // Show working memory state
    println!("\n=== Agent Working Memory ===");
    println!("Recent queries: {:?}", agent.working_memory.recent_queries);
    println!(
        "Expertise areas: {:?}",
        agent.working_memory.expertise_areas
    );
    println!(
        "Knowledge access patterns: {:?}",
        agent.working_memory.knowledge_access_patterns
    );

    Ok(())
}

// Placeholder for EventStore trait - would come from cim-domain
trait EventStore: Send + Sync {
    // Event store methods
}

// Placeholder for DistributedEventStore - would come from infrastructure
struct DistributedEventStore;

impl DistributedEventStore {
    async fn new(_jetstream: jetstream::Context) -> Result<Self> {
        Ok(Self)
    }
}

impl EventStore for DistributedEventStore {}
