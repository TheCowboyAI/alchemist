//! Simple agent implementation for CIM questions

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Event sent when user asks a question
#[derive(Event, Clone, Debug)]
pub struct AgentQuestionEvent {
    pub question: String,
}

/// Event sent when agent responds
#[derive(Event, Clone, Debug)]
pub struct AgentResponseEvent {
    pub response: String,
}

/// Event sent when agent encounters an error
#[derive(Event, Clone, Debug)]
pub struct AgentErrorEvent {
    pub error: String,
}

/// Simple Ollama client
#[derive(Clone)]
pub struct OllamaClient {
    base_url: String,
    model: String,
    runtime: Arc<Runtime>,
}

impl OllamaClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            base_url,
            model,
            runtime: Arc::new(
                Runtime::new().expect("Failed to create Tokio runtime")
            ),
        }
    }

    pub fn ask(&self, question: &str) -> Result<String, String> {
        let url = format!("{}/api/generate", self.base_url);
        
        // Add CIM context to the question
        let prompt = format!(
            "You are an AI assistant helping with CIM (Composable Information Machine). \
             CIM is a revolutionary distributed system architecture that transforms how we build, \
             visualize, and reason about information systems. It combines Event-Driven Architecture, \
             Graph-Based Workflows, Conceptual Spaces, and AI-Native Design. \
             CIM has 8 production-ready domains: Graph, Identity, Person, Agent, Git, Location, \
             ConceptualSpaces, and Workflow. All domains use event sourcing with NATS JetStream. \
             \n\nUser question: {}",
            question
        );

        let request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
            options: OllamaOptions {
                temperature: 0.7,
                num_predict: 500,
            },
        };

        // Use blocking in runtime to avoid nested runtime issues
        self.runtime.block_on(async {
            let client = reqwest::Client::new();
            let response = client
                .post(&url)
                .json(&request)
                .send()
                .await
                .map_err(|e| format!("Failed to send request: {}", e))?;

            if !response.status().is_success() {
                return Err(format!("API error: {}", response.status()));
            }

            let result: OllamaResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            Ok(result.response)
        })
    }
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: i32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

/// Resource for the Ollama client
#[derive(Resource)]
pub struct AgentResource {
    client: OllamaClient,
}

impl Default for AgentResource {
    fn default() -> Self {
        Self {
            client: OllamaClient::new(
                "http://localhost:11434".to_string(),
                "vicuna:latest".to_string(),
            ),
        }
    }
}

/// Plugin for the simple agent
pub struct SimpleAgentPlugin;

impl Plugin for SimpleAgentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AgentResource>()
            .add_event::<AgentQuestionEvent>()
            .add_event::<AgentResponseEvent>()
            .add_event::<AgentErrorEvent>()
            .add_systems(Update, process_questions);
    }
}

/// System that processes questions
fn process_questions(
    mut question_events: EventReader<AgentQuestionEvent>,
    mut response_events: EventWriter<AgentResponseEvent>,
    mut error_events: EventWriter<AgentErrorEvent>,
    agent: Res<AgentResource>,
) {
    for event in question_events.read() {
        info!("Processing question: {}", event.question);
        
        match agent.client.ask(&event.question) {
            Ok(response) => {
                info!("Agent response: {}", response);
                response_events.write(AgentResponseEvent { response });
            }
            Err(error) => {
                error!("Agent error: {}", error);
                error_events.write(AgentErrorEvent { error });
            }
        }
    }
} 