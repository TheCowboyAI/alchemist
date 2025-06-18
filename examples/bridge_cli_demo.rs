//! CIM Bridge CLI Demo - Standalone Usage Example
//! 
//! This demonstrates using the CIM Bridge without any UI framework dependencies.
//! The bridge communicates purely through NATS subjects.

use async_nats::Client;
use chrono::Utc;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

/// Command envelope for NATS communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEnvelope {
    pub id: Uuid,
    pub command: BridgeCommand,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Bridge commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeCommand {
    Query {
        model: String,
        messages: Vec<Message>,
        parameters: ModelParameters,
    },
    StreamQuery {
        model: String,
        messages: Vec<Message>,
        parameters: ModelParameters,
    },
    SwitchProvider {
        provider: String,
        config: Option<serde_json::Value>,
    },
    ListProviders,
    ListModels {
        provider: Option<String>,
    },
    HealthCheck {
        provider: Option<String>,
    },
}

/// Event envelope for NATS communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub event: BridgeEvent,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Bridge events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeEvent {
    QueryStarted {
        query_id: Uuid,
        provider: String,
        model: String,
    },
    QueryCompleted {
        query_id: Uuid,
        response: ModelResponse,
    },
    QueryFailed {
        query_id: Uuid,
        error: String,
        provider: String,
    },
    StreamChunk {
        query_id: Uuid,
        chunk: StreamChunk,
        sequence: u32,
    },
    ProviderSwitched {
        old_provider: Option<String>,
        new_provider: String,
    },
    ProviderHealth {
        provider: String,
        status: HealthStatus,
        details: Option<String>,
    },
    MetricsUpdated {
        provider: String,
        metrics: ProviderMetrics,
    },
}

/// Message format for AI conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub name: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Function,
}

/// Model parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelParameters {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub seed: Option<u64>,
}

/// Model response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub query_id: Uuid,
    pub model: String,
    pub content: String,
    pub usage: Option<Usage>,
    pub finish_reason: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Stream chunk for streaming responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub content: String,
    pub is_final: bool,
    pub usage: Option<Usage>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

/// Provider metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_latency_ms: f64,
    pub tokens_processed: u64,
    pub last_request: Option<chrono::DateTime<chrono::Utc>>,
}

/// CLI Arguments
#[derive(Parser)]
#[command(name = "cim-bridge-cli")]
#[command(about = "CIM Bridge CLI - Interact with AI providers through NATS")]
struct Cli {
    /// NATS server URL
    #[arg(short, long, default_value = "nats://localhost:4222")]
    nats_url: String,

    /// Command to execute
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a query to the current provider
    Query {
        /// The prompt to send
        prompt: String,
        
        /// Model to use (optional)
        #[arg(short, long)]
        model: Option<String>,
        
        /// Temperature for generation
        #[arg(short, long)]
        temperature: Option<f32>,
    },
    
    /// Stream a response from the current provider
    Stream {
        /// The prompt to send
        prompt: String,
        
        /// Model to use (optional)
        #[arg(short, long)]
        model: Option<String>,
    },
    
    /// List available providers
    ListProviders,
    
    /// List available models
    ListModels {
        /// Provider to list models for (optional)
        #[arg(short, long)]
        provider: Option<String>,
    },
    
    /// Switch to a different provider
    SwitchProvider {
        /// Provider name
        provider: String,
    },
    
    /// Check health of providers
    HealthCheck {
        /// Specific provider to check (optional)
        #[arg(short, long)]
        provider: Option<String>,
    },
    
    /// Start an interactive chat session
    Chat {
        /// Model to use (optional)
        #[arg(short, long)]
        model: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Connect to NATS
    let client = async_nats::connect(&cli.nats_url).await?;
    println!("Connected to NATS at {}", cli.nats_url);
    
    // Execute command
    match cli.command {
        Commands::Query { prompt, model, temperature } => {
            query_command(&client, prompt, model, temperature).await?;
        }
        Commands::Stream { prompt, model } => {
            stream_command(&client, prompt, model).await?;
        }
        Commands::ListProviders => {
            list_providers(&client).await?;
        }
        Commands::ListModels { provider } => {
            list_models(&client, provider).await?;
        }
        Commands::SwitchProvider { provider } => {
            switch_provider(&client, provider).await?;
        }
        Commands::HealthCheck { provider } => {
            health_check(&client, provider).await?;
        }
        Commands::Chat { model } => {
            interactive_chat(&client, model).await?;
        }
    }
    
    Ok(())
}

async fn query_command(
    client: &Client,
    prompt: String,
    model: Option<String>,
    temperature: Option<f32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let correlation_id = Uuid::new_v4();
    
    // Create command
    let mut parameters = ModelParameters::default();
    if let Some(temp) = temperature {
        parameters.temperature = Some(temp);
    }
    
    let command = CommandEnvelope {
        id: Uuid::new_v4(),
        command: BridgeCommand::Query {
            model: model.unwrap_or_else(|| "default".to_string()),
            messages: vec![Message {
                role: MessageRole::User,
                content: prompt,
                name: None,
                metadata: None,
            }],
            parameters,
        },
        correlation_id,
        causation_id: None,
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };
    
    // Subscribe to response events
    let mut sub = client.subscribe("bridge.event.query.>").await?;
    
    // Send command
    client
        .publish(
            "bridge.command.query",
            serde_json::to_vec(&command)?.into(),
        )
        .await?;
    
    println!("Query sent, waiting for response...");
    
    // Wait for response with timeout
    let result = timeout(Duration::from_secs(30), async {
        while let Some(msg) = sub.next().await {
            let event: EventEnvelope = serde_json::from_slice(&msg.payload)?;
            
            if event.correlation_id != correlation_id {
                continue;
            }
            
            match event.event {
                BridgeEvent::QueryStarted { provider, model, .. } => {
                    println!("Query started with {} using model {}", provider, model);
                }
                BridgeEvent::QueryCompleted { response, .. } => {
                    println!("\nResponse:\n{}", response.content);
                    
                    if let Some(usage) = response.usage {
                        println!(
                            "\nTokens used: {} prompt + {} completion = {} total",
                            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                        );
                    }
                    
                    return Ok(());
                }
                BridgeEvent::QueryFailed { error, provider, .. } => {
                    eprintln!("Query failed on {}: {}", provider, error);
                    return Err(error.into());
                }
                _ => {}
            }
        }
        
        Err("No response received".into())
    })
    .await;
    
    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Query timed out after 30 seconds".into()),
    }
}

async fn stream_command(
    client: &Client,
    prompt: String,
    model: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let correlation_id = Uuid::new_v4();
    
    // Create command
    let command = CommandEnvelope {
        id: Uuid::new_v4(),
        command: BridgeCommand::StreamQuery {
            model: model.unwrap_or_else(|| "default".to_string()),
            messages: vec![Message {
                role: MessageRole::User,
                content: prompt,
                name: None,
                metadata: None,
            }],
            parameters: ModelParameters::default(),
        },
        correlation_id,
        causation_id: None,
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };
    
    // Subscribe to events
    let mut sub = client.subscribe("bridge.event.>").await?;
    
    // Send command
    client
        .publish(
            "bridge.command.stream_query",
            serde_json::to_vec(&command)?.into(),
        )
        .await?;
    
    println!("Streaming query sent...\n");
    
    // Process stream
    let mut total_tokens = 0;
    
    while let Some(msg) = sub.next().await {
        let event: EventEnvelope = serde_json::from_slice(&msg.payload)?;
        
        if event.correlation_id != correlation_id {
            continue;
        }
        
        match event.event {
            BridgeEvent::QueryStarted { provider, model, .. } => {
                println!("Streaming from {} using model {}\n", provider, model);
            }
            BridgeEvent::StreamChunk { chunk, .. } => {
                print!("{}", chunk.content);
                use std::io::{self, Write};
                io::stdout().flush()?;
                
                if chunk.is_final {
                    println!("\n");
                    if let Some(usage) = chunk.usage {
                        total_tokens = usage.total_tokens;
                    }
                    break;
                }
            }
            BridgeEvent::QueryFailed { error, provider, .. } => {
                eprintln!("\nStream failed on {}: {}", provider, error);
                return Err(error.into());
            }
            _ => {}
        }
    }
    
    if total_tokens > 0 {
        println!("Total tokens used: {}", total_tokens);
    }
    
    Ok(())
}

async fn list_providers(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    // For this demo, we'll just show the concept
    // In a real implementation, this would query the bridge
    println!("Available providers:");
    println!("  - ollama (local)");
    println!("  - openai (requires API key)");
    println!("  - anthropic (requires API key)");
    
    Ok(())
}

async fn list_models(
    client: &Client,
    provider: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let correlation_id = Uuid::new_v4();
    
    let command = CommandEnvelope {
        id: Uuid::new_v4(),
        command: BridgeCommand::ListModels { provider },
        correlation_id,
        causation_id: None,
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };
    
    // Send command
    client
        .publish(
            "bridge.command.list_models",
            serde_json::to_vec(&command)?.into(),
        )
        .await?;
    
    println!("Fetching available models...");
    
    // In a real implementation, we'd wait for the response
    // For now, show example output
    println!("\nAvailable models:");
    println!("  ollama:");
    println!("    - llama2");
    println!("    - codellama");
    println!("    - mistral");
    println!("  openai:");
    println!("    - gpt-4");
    println!("    - gpt-3.5-turbo");
    
    Ok(())
}

async fn switch_provider(
    client: &Client,
    provider: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let command = CommandEnvelope {
        id: Uuid::new_v4(),
        command: BridgeCommand::SwitchProvider {
            provider: provider.clone(),
            config: None,
        },
        correlation_id: Uuid::new_v4(),
        causation_id: None,
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };
    
    client
        .publish(
            "bridge.command.switch_provider",
            serde_json::to_vec(&command)?.into(),
        )
        .await?;
    
    println!("Switched to provider: {}", provider);
    
    Ok(())
}

async fn health_check(
    client: &Client,
    provider: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let correlation_id = Uuid::new_v4();
    
    let command = CommandEnvelope {
        id: Uuid::new_v4(),
        command: BridgeCommand::HealthCheck { provider },
        correlation_id,
        causation_id: None,
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };
    
    client
        .publish(
            "bridge.command.health_check",
            serde_json::to_vec(&command)?.into(),
        )
        .await?;
    
    println!("Health check requested...");
    
    // In a real implementation, we'd wait for health events
    // For now, show example output
    println!("\nProvider Health Status:");
    println!("  ollama: Healthy (localhost:11434)");
    println!("  openai: Healthy (api.openai.com)");
    println!("  anthropic: Not configured");
    
    Ok(())
}

async fn interactive_chat(
    client: &Client,
    model: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, Write};
    
    println!("Starting interactive chat session...");
    println!("Type 'exit' or 'quit' to end the session.\n");
    
    let mut messages = Vec::new();
    
    loop {
        // Get user input
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input == "exit" || input == "quit" {
            println!("Ending chat session.");
            break;
        }
        
        if input.is_empty() {
            continue;
        }
        
        // Add user message to history
        messages.push(Message {
            role: MessageRole::User,
            content: input.to_string(),
            name: None,
            metadata: None,
        });
        
        // Send query with full message history
        let correlation_id = Uuid::new_v4();
        
        let command = CommandEnvelope {
            id: Uuid::new_v4(),
            command: BridgeCommand::StreamQuery {
                model: model.clone().unwrap_or_else(|| "default".to_string()),
                messages: messages.clone(),
                parameters: ModelParameters::default(),
            },
            correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };
        
        // Subscribe to events
        let mut sub = client.subscribe("bridge.event.>").await?;
        
        // Send command
        client
            .publish(
                "bridge.command.stream_query",
                serde_json::to_vec(&command)?.into(),
            )
            .await?;
        
        print!("\nAssistant: ");
        io::stdout().flush()?;
        
        let mut response_content = String::new();
        
        // Process stream
        while let Some(msg) = sub.next().await {
            let event: EventEnvelope = serde_json::from_slice(&msg.payload)?;
            
            if event.correlation_id != correlation_id {
                continue;
            }
            
            match event.event {
                BridgeEvent::StreamChunk { chunk, .. } => {
                    print!("{}", chunk.content);
                    io::stdout().flush()?;
                    response_content.push_str(&chunk.content);
                    
                    if chunk.is_final {
                        println!("\n");
                        break;
                    }
                }
                BridgeEvent::QueryFailed { error, .. } => {
                    eprintln!("\nError: {}", error);
                    break;
                }
                _ => {}
            }
        }
        
        // Add assistant response to history
        if !response_content.is_empty() {
            messages.push(Message {
                role: MessageRole::Assistant,
                content: response_content,
                name: None,
                metadata: None,
            });
        }
    }
    
    Ok(())
} 