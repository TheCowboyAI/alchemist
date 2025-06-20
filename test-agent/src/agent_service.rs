//! CIM Alchemist Agent Service - Continuous listener

use async_nats;
use futures::StreamExt;
use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio;
use tracing::{error, info, warn};
use tracing_subscriber;

#[derive(Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
    #[allow(dead_code)]
    done: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct AgentMessage {
    id: String,
    content: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

const CIM_CONTEXT: &str = r#"You are the Alchemist, an AI assistant specialized in the Composable Information Machine (CIM) architecture. 

CIM is a revolutionary distributed system architecture that transforms how we build, visualize, and reason about information systems. It combines:

- Event-Driven Architecture: All state changes flow through immutable events (ZERO CRUD violations)
- Graph-Based Workflows: Visual representation of business processes and knowledge
- Conceptual Spaces: Geometric representation of semantic relationships
- AI-Native Design: Built for seamless integration with intelligent agents
- Self-Referential Capability: Systems that can visualize and reason about themselves

Key principles:
- Events provide the foundation of truth
- Graphs make workflows visual and composable
- Conceptual Spaces add semantic understanding
- ECS (Entity Component System) enables real-time interaction
- AI enhances with intelligence

Please provide helpful, accurate answers about CIM architecture, its domains (Graph, Identity, Person, Agent, Git, Location, ConceptualSpaces, Workflow), and implementation patterns."#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("CIM Alchemist Agent Service Starting...");

    // Connect to Ollama
    let client = reqwest::Client::new();

    // Test Ollama connection
    match client.get("http://localhost:11434/api/tags").send().await {
        Ok(response) => {
            if response.status().is_success() {
                info!("✓ Connected to Ollama");
            } else {
                error!("Ollama returned error: {}", response.status());
                return Err("Failed to connect to Ollama".into());
            }
        }
        Err(e) => {
            error!("Failed to connect to Ollama: {}", e);
            return Err("Ollama not available".into());
        }
    }

    // Connect to NATS
    let nats_client = match async_nats::connect("nats://localhost:4222").await {
        Ok(client) => {
            info!("✓ Connected to NATS");
            client
        }
        Err(e) => {
            error!("Failed to connect to NATS: {}", e);
            return Err("NATS not available".into());
        }
    };

    // Subscribe to multiple subjects
    let subjects = vec![
        "cim.dialog.alchemist.request",
        "cim.agent.alchemist.commands",
        "cim.agent.alchemist.queries",
        "cim.agent.alchemist.health",
    ];

    for subject in &subjects {
        info!("Subscribing to: {}", subject);
    }

    // Health check responder
    let health_client = nats_client.clone();
    tokio::spawn(async move {
        let mut health_sub = health_client
            .subscribe("cim.agent.alchemist.health")
            .await
            .unwrap();

        while let Some(msg) = health_sub.next().await {
            let health_response = serde_json::json!({
                "status": "healthy",
                "agent": "alchemist",
                "version": "0.1.0",
                "timestamp": chrono::Utc::now(),
            });

            if let Some(reply) = msg.reply {
                let _ = health_client
                    .publish(reply, serde_json::to_vec(&health_response).unwrap().into())
                    .await;
                info!("Responded to health check");
            }
        }
    });

    // Main dialog processor
    let mut dialog_sub = nats_client
        .subscribe("cim.dialog.alchemist.request")
        .await?;
    info!("Agent ready and listening for requests...");

    while let Some(msg) = dialog_sub.next().await {
        let nats_clone = nats_client.clone();
        let client_clone = client.clone();

        // Process each message in a separate task
        tokio::spawn(async move {
            match serde_json::from_slice::<AgentMessage>(&msg.payload) {
                Ok(request) => {
                    info!("Received question: {}", request.content);

                    // Build prompt with context
                    let prompt = format!(
                        "{}\n\nUser Question: {}\n\nPlease provide a helpful and concise answer:",
                        CIM_CONTEXT, request.content
                    );

                    // Generate response
                    let ollama_request = OllamaGenerateRequest {
                        model: "vicuna:latest".to_string(),
                        prompt,
                        stream: false,
                    };

                    match client_clone
                        .post("http://localhost:11434/api/generate")
                        .json(&ollama_request)
                        .timeout(std::time::Duration::from_secs(60))
                        .send()
                        .await
                    {
                        Ok(response) => {
                            match response.json::<OllamaGenerateResponse>().await {
                                Ok(ollama_response) => {
                                    let reply = AgentMessage {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        content: ollama_response.response.trim().to_string(),
                                        timestamp: chrono::Utc::now(),
                                    };

                                    // Send to response channel if specified
                                    if let Some(reply_to) = msg.reply {
                                        let payload = serde_json::to_vec(&reply).unwrap();
                                        if let Err(e) =
                                            nats_clone.publish(reply_to, payload.into()).await
                                        {
                                            error!("Failed to send reply: {}", e);
                                        } else {
                                            info!("Sent response");
                                        }
                                    } else {
                                        // Publish to general response channel
                                        let payload = serde_json::to_vec(&reply).unwrap();
                                        if let Err(e) = nats_clone
                                            .publish(
                                                "cim.dialog.alchemist.response",
                                                payload.into(),
                                            )
                                            .await
                                        {
                                            error!("Failed to publish response: {}", e);
                                        } else {
                                            info!("Published response");
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to parse Ollama response: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to generate response: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to parse message: {}", e);
                }
            }
        });
    }

    Ok(())
}
