use anyhow::Result;
use async_nats::Client;
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222").await?;
    
    // Test 1: List available models using command
    println!("Testing: List available models");
    let command_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    let list_models_command = json!({
        "id": command_id,
        "command": {
            "ListModels": {
                "provider": null
            }
        },
        "correlation_id": correlation_id,
        "causation_id": null,
        "timestamp": Utc::now(),
        "metadata": {}
    });
    
    // Subscribe to events to get the response
    let mut event_sub = client.subscribe("bridge.event.models.listed").await?;
    
    // Send command
    client
        .publish("bridge.command.list_models", list_models_command.to_string().into())
        .await?;
    
    // Wait for response
    if let Some(msg) = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        event_sub.next()
    ).await? {
        let response_text = String::from_utf8(msg.payload.to_vec())?;
        println!("Models Response: {}\n", response_text);
    } else {
        println!("No response received for list models\n");
    }
    
    // Test 2: Chat with a model
    println!("Testing: Chat with model");
    let query_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    // Try with a smaller model that might be already downloaded
    let chat_command = json!({
        "id": query_id,
        "command": {
            "Query": {
                "model": "vicuna:latest", // Use vicuna which is available
                "messages": [
                    {
                        "role": "User",
                        "content": "What is the capital of France? Answer in one word.",
                        "name": null,
                        "metadata": null
                    }
                ],
                "parameters": {
                    "temperature": 0.7,
                    "max_tokens": 50
                }
            }
        },
        "correlation_id": correlation_id,
        "causation_id": null,
        "timestamp": Utc::now(),
        "metadata": {}
    });
    
    // Subscribe to completion events
    let mut completion_sub = client.subscribe("bridge.event.query.completed").await?;
    
    // Also subscribe to failure events to see if there's an error
    let mut failure_sub = client.subscribe("bridge.event.query.failed").await?;
    
    // Send command
    client
        .publish("bridge.command.query", chat_command.to_string().into())
        .await?;
    
    // Wait for response with longer timeout (60 seconds for model download)
    println!("Waiting for chat response (60s timeout)...");
    
    tokio::select! {
        Ok(Some(msg)) = tokio::time::timeout(
            std::time::Duration::from_secs(60),
            completion_sub.next()
        ) => {
            let response_text = String::from_utf8(msg.payload.to_vec())?;
            println!("Chat Response: {}\n", response_text);
        }
        Ok(Some(msg)) = tokio::time::timeout(
            std::time::Duration::from_secs(60),
            failure_sub.next()
        ) => {
            let response_text = String::from_utf8(msg.payload.to_vec())?;
            println!("Chat Failed: {}\n", response_text);
        }
        else => {
            println!("No response received for chat query (timeout)\n");
        }
    }
    
    // Test 3: Check active provider using query
    println!("Testing: Get active provider");
    let query_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    let provider_query = json!({
        "id": query_id,
        "query": "ActiveProvider",
        "correlation_id": correlation_id,
        "timestamp": Utc::now()
    });
    
    let response = client
        .request("bridge.query.active_provider", provider_query.to_string().into())
        .await?;
    
    let response_text = String::from_utf8(response.payload.to_vec())?;
    println!("Active Provider Response: {}\n", response_text);
    
    // Test 4: List all providers
    println!("Testing: List all providers");
    let command_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    
    let list_providers_command = json!({
        "id": command_id,
        "command": "ListProviders",
        "correlation_id": correlation_id,
        "causation_id": null,
        "timestamp": Utc::now(),
        "metadata": {}
    });
    
    // Subscribe to events
    let mut providers_sub = client.subscribe("bridge.event.providers.listed").await?;
    
    // Send command
    client
        .publish("bridge.command.list_providers", list_providers_command.to_string().into())
        .await?;
    
    // Wait for response
    if let Some(msg) = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        providers_sub.next()
    ).await? {
        let response_text = String::from_utf8(msg.payload.to_vec())?;
        println!("Providers Response: {}\n", response_text);
    } else {
        println!("No response received for list providers\n");
    }
    
    println!("All tests completed!");
    
    Ok(())
} 