//! CIM Bridge CLI Client

use async_nats::Client;
use chrono::Utc;
use clap::{Parser, Subcommand};
use cim_bridge::*;
use futures::StreamExt;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "cim-bridge-cli")]
#[command(about = "CIM Bridge CLI - Interact with the bridge service")]
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
        #[arg(short, long, default_value = "llama3.2")]
        model: String,
        
        /// Temperature for generation
        #[arg(short, long)]
        temperature: Option<f32>,
    },
    
    /// Stream a response from the current provider
    Stream {
        /// The prompt to send
        prompt: String,
        
        /// Model to use (optional)
        #[arg(short, long, default_value = "llama3.2")]
        model: String,
    },
    
    /// List available providers
    ListProviders,
    
    /// List available models
    ListModels {
        /// Provider to list models for (optional)
        #[arg(short, long)]
        provider: Option<String>,
    },
    
    /// Check health of providers
    HealthCheck {
        /// Specific provider to check (optional)
        #[arg(short, long)]
        provider: Option<String>,
    },
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Connect to NATS
    println!("Connecting to NATS at {}...", cli.nats_url);
    let client = async_nats::connect(&cli.nats_url).await?;
    println!("Connected!");
    
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
        Commands::HealthCheck { provider } => {
            health_check(&client, provider).await?;
        }
    }
    
    Ok(())
}

async fn query_command(
    client: &Client,
    prompt: String,
    model: String,
    temperature: Option<f32>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let correlation_id = Uuid::new_v4();
    
    // Create command
    let mut parameters = ModelParameters::default();
    if let Some(temp) = temperature {
        parameters.temperature = Some(temp);
    }
    
    let command = CommandEnvelope {
        id: Uuid::new_v4(),
        command: BridgeCommand::Query {
            model: model.clone(),
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
    println!("Sending query to model {model}...");
    client
        .publish(
            "bridge.command.query",
            serde_json::to_vec(&command)?.into(),
        )
        .await?;
    
    // Wait for response with timeout
    let result = timeout(Duration::from_secs(30), async {
        while let Some(msg) = sub.next().await {
            match serde_json::from_slice::<EventEnvelope>(&msg.payload) {
                Ok(event) => {
                    if event.correlation_id != correlation_id {
                        continue;
                    }
                    
                    match event.event {
                        BridgeEvent::QueryStarted { provider, model, .. } => {
                            println!("✓ Query started with {provider} using model {model}");
                        }
                        BridgeEvent::QueryCompleted { response, .. } => {
                            println!("\n📝 Response:\n{}", response.content);
                            
                            if let Some(usage) = response.usage {
                                println!(
                                    "\n📊 Tokens: {} prompt + {} completion = {} total",
                                    usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                                );
                            }
                            
                            return Ok(());
                        }
                        BridgeEvent::QueryFailed { error, provider, .. } => {
                            eprintln!("❌ Query failed on {provider}: {error}");
                            return Err(error.into());
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse event: {e}");
                }
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
    model: String,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let correlation_id = Uuid::new_v4();
    
    // Create command
    let command = CommandEnvelope {
        id: Uuid::new_v4(),
        command: BridgeCommand::StreamQuery {
            model: model.clone(),
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
    println!("Streaming from model {model}...");
    client
        .publish(
            "bridge.command.stream_query",
            serde_json::to_vec(&command)?.into(),
        )
        .await?;
    
    // Process stream
    let mut total_tokens = 0u32;
    
    print!("\n📝 Response: ");
    use std::io::{self, Write};
    io::stdout().flush()?;
    
    while let Some(msg) = sub.next().await {
        match serde_json::from_slice::<EventEnvelope>(&msg.payload) {
            Ok(event) => {
                if event.correlation_id != correlation_id {
                    continue;
                }
                
                match event.event {
                    BridgeEvent::QueryStarted { provider, model, .. } => {
                        println!("\n✓ Streaming from {provider} using model {model}\n");
                    }
                    BridgeEvent::StreamChunk { chunk, .. } => {
                        print!("{}", chunk.content);
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
                        eprintln!("\n❌ Stream failed on {provider}: {error}");
                        return Err(error.into());
                    }
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("Failed to parse event: {e}");
            }
        }
    }
    
    if total_tokens > 0 {
        println!("📊 Total tokens used: {total_tokens}");
    }
    
    Ok(())
}

async fn list_providers(client: &Client) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let correlation_id = Uuid::new_v4();
    
    let command = CommandEnvelope {
        id: Uuid::new_v4(),
        command: BridgeCommand::ListProviders,
        correlation_id,
        causation_id: None,
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };
    
    // Subscribe to events
    let mut sub = client.subscribe("bridge.event.providers.listed").await?;
    
    // Send command
    println!("Fetching available providers...");
    client
        .publish(
            "bridge.command.list_providers",
            serde_json::to_vec(&command)?.into(),
        )
        .await?;
    
    // Wait for response
    let result = timeout(Duration::from_secs(5), async {
        while let Some(msg) = sub.next().await {
            match serde_json::from_slice::<EventEnvelope>(&msg.payload) {
                Ok(event) => {
                    if event.correlation_id != correlation_id {
                        continue;
                    }
                    
                    if let BridgeEvent::ProvidersListed { providers } = event.event {
                        println!("\n📋 Available providers:");
                        for provider in providers {
                            let status = if provider.is_active { "✓ ACTIVE" } else { "  " };
                            let configured = if provider.is_configured { "configured" } else { "not configured" };
                            println!(
                                "{} {} - {} ({})",
                                status, provider.name, provider.description, configured
                            );
                            println!("     Capabilities: {}", provider.capabilities.join(", "));
                        }
                        return Ok(());
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse event: {e}");
                }
            }
        }
        Err("No response received".into())
    })
    .await;
    
    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Request timed out".into()),
    }
}

async fn list_models(
    client: &Client,
    provider: Option<String>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let correlation_id = Uuid::new_v4();
    
    let command = CommandEnvelope {
        id: Uuid::new_v4(),
        command: BridgeCommand::ListModels { provider: provider.clone() },
        correlation_id,
        causation_id: None,
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };
    
    // Subscribe to events
    let mut sub = client.subscribe("bridge.event.models.listed").await?;
    
    // Send command
    println!("Fetching available models{}...", 
        provider.as_ref().map(|p| format!(" for {p}")).unwrap_or_default()
    );
    client
        .publish(
            "bridge.command.list_models",
            serde_json::to_vec(&command)?.into(),
        )
        .await?;
    
    // Wait for response
    let result = timeout(Duration::from_secs(10), async {
        while let Some(msg) = sub.next().await {
            match serde_json::from_slice::<EventEnvelope>(&msg.payload) {
                Ok(event) => {
                    if event.correlation_id != correlation_id {
                        continue;
                    }
                    
                    if let BridgeEvent::ModelsListed { provider, models } = event.event {
                        println!("\n📋 Available models for {provider}:");
                        for model in models {
                            println!("  • {}", model.name);
                            if let Some(desc) = model.description {
                                println!("    {desc}");
                            }
                            if !model.capabilities.is_empty() {
                                println!("    Capabilities: {}", model.capabilities.join(", "));
                            }
                        }
                        return Ok(());
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse event: {e}");
                }
            }
        }
        Err("No response received".into())
    })
    .await;
    
    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Request timed out".into()),
    }
}

async fn health_check(
    client: &Client,
    provider: Option<String>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let correlation_id = Uuid::new_v4();
    
    let command = CommandEnvelope {
        id: Uuid::new_v4(),
        command: BridgeCommand::HealthCheck { provider: provider.clone() },
        correlation_id,
        causation_id: None,
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };
    
    // Subscribe to events
    let mut sub = client.subscribe("bridge.event.provider.health").await?;
    
    // Send command
    println!("Checking health{}...", 
        provider.as_ref().map(|p| format!(" for {p}")).unwrap_or_default()
    );
    client
        .publish(
            "bridge.command.health_check",
            serde_json::to_vec(&command)?.into(),
        )
        .await?;
    
    // Wait for response
    let result = timeout(Duration::from_secs(5), async {
        while let Some(msg) = sub.next().await {
            match serde_json::from_slice::<EventEnvelope>(&msg.payload) {
                Ok(event) => {
                    if event.correlation_id != correlation_id {
                        continue;
                    }
                    
                    if let BridgeEvent::ProviderHealth { provider, status, details } = event.event {
                        print!("\n🏥 Health status for {provider}: ");
                        match status {
                            HealthStatus::Healthy => println!("✅ Healthy"),
                            HealthStatus::Degraded(msg) => println!("⚠️  Degraded - {msg}"),
                            HealthStatus::Unhealthy(msg) => println!("❌ Unhealthy - {msg}"),
                        }
                        
                        if let Some(details) = details {
                            println!("   Details: {details}");
                        }
                        
                        return Ok(());
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse event: {e}");
                }
            }
        }
        Err("No response received".into())
    })
    .await;
    
    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Request timed out".into()),
    }
} 