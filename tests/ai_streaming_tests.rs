//! Tests for AI streaming responses

use alchemist::ai::{AiManager, StreamingResponse, StreamingResponseStream};
use alchemist::config::AlchemistConfig;
use anyhow::Result;
use mockito::Server;
use serde_json::json;
use std::collections::HashMap;
use futures::StreamExt;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_streaming_response() -> Result<()> {
    // Given: A mock server
    let mut server = Server::new_async().await;
    
    // And: A configured AI model
    let config = create_test_config(&server.url());
    let mut manager = AiManager::new(&config).await?;
    
    // And: A mock endpoint that returns a streaming response
    let _m = server.mock("POST", "/v1/messages")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(r#"data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}

data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}

data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":" world"}}

data: {"type":"content_block_stop","index":0}

data: {"type":"message_stop"}

"#)
        .create_async()
        .await;
    
    // When: Making a streaming request
    let mut stream = manager.stream_completion("claude-3", "Say hello").await?;
    
    // Then: Should receive chunks
    let mut chunks = Vec::new();
    while let Some(chunk) = stream.next().await {
        chunks.push(chunk?);
    }
    
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].content, "Hello");
    assert_eq!(chunks[1].content, " world");
    
    Ok(())
}

#[tokio::test]
async fn test_streaming_with_error() -> Result<()> {
    // Given: A mock server
    let mut server = Server::new_async().await;
    
    // And: A configured AI model
    let config = create_test_config(&server.url());
    let mut manager = AiManager::new(&config).await?;
    
    // And: A mock endpoint that returns an error mid-stream
    let _m = server.mock("POST", "/v1/messages")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(r#"data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}

data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}

data: {"type":"error","error":{"type":"stream_error","message":"Connection lost"}}

"#)
        .create_async()
        .await;
    
    // When: Making a streaming request
    let mut stream = manager.stream_completion("claude-3", "Say hello").await?;
    
    // Then: Should receive first chunk then error
    let chunk1 = stream.next().await.unwrap();
    assert!(chunk1.is_ok());
    assert_eq!(chunk1.unwrap().content, "Hello");
    
    let chunk2 = stream.next().await.unwrap();
    assert!(chunk2.is_err());
    assert!(chunk2.unwrap_err().to_string().contains("Connection lost"));
    
    Ok(())
}

#[tokio::test]
async fn test_streaming_timeout() -> Result<()> {
    // Given: A configured AI model with short timeout
    let mut config = AlchemistConfig::default();
    config.ai_models.insert("claude-3".to_string(), alchemist::config::AiModelConfig {
        provider: "anthropic".to_string(),
        endpoint: Some("http://localhost:19999/v1".to_string()), // Non-existent
        api_key_env: Some("TEST_API_KEY".to_string()),
        model_name: "claude-3-opus".to_string(),
        max_tokens: Some(4096),
        temperature: Some(0.7),
        timeout_seconds: Some(1),
        rate_limit: None,
        fallback_model: None,
        params: HashMap::new(),
    });
    std::env::set_var("TEST_API_KEY", "test-key-123");
    
    let mut manager = AiManager::new(&config).await?;
    
    // When: Making a streaming request (will timeout)
    let result = timeout(
        Duration::from_secs(2),
        manager.stream_completion("claude-3", "Say hello")
    ).await;
    
    // Then: Should timeout
    assert!(result.is_err() || result.unwrap().is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_streaming_with_system_prompt() -> Result<()> {
    // Given: A mock server
    let mut server = Server::new_async().await;
    
    // And: A configured AI model
    let config = create_test_config(&server.url());
    let mut manager = AiManager::new(&config).await?;
    
    // And: A mock that verifies system prompt
    let _m = server.mock("POST", "/v1/messages")
        .match_body(mockito::Matcher::Json(json!({
            "model": "claude-3-opus",
            "system": "You are a helpful assistant",
            "messages": [{"role": "user", "content": "Hello"}],
            "stream": true
        })))
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(r#"data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hi!"}}

data: {"type":"message_stop"}

"#)
        .create_async()
        .await;
    
    // When: Making a streaming request with system prompt
    let mut stream = manager.stream_completion_with_context(
        "claude-3",
        "Hello",
        Some("You are a helpful assistant")
    ).await?;
    
    // Then: Should receive response
    let chunk = stream.next().await.unwrap()?;
    assert_eq!(chunk.content, "Hi!");
    
    Ok(())
}

#[tokio::test]
async fn test_streaming_cancellation() -> Result<()> {
    // Given: A mock server
    let mut server = Server::new_async().await;
    
    // And: A configured AI model
    let config = create_test_config(&server.url());
    let mut manager = AiManager::new(&config).await?;
    
    // And: A mock endpoint that returns a long streaming response
    let _m = server.mock("POST", "/v1/messages")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(r#"data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"First"}}

data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":" Second"}}

data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":" Third"}}

data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":" Fourth"}}

data: {"type":"message_stop"}

"#)
        .create_async()
        .await;
    
    // When: Making a streaming request and cancelling early
    let mut stream = manager.stream_completion("claude-3", "Count to four").await?;
    
    // Get first chunk
    let chunk1 = stream.next().await.unwrap()?;
    assert_eq!(chunk1.content, "First");
    
    // Cancel the stream by dropping it
    drop(stream);
    
    // Then: Stream should be cancelled (no further processing)
    Ok(())
}

fn create_test_config(base_url: &str) -> AlchemistConfig {
    let mut config = AlchemistConfig::default();
    
    config.ai_models.insert("claude-3".to_string(), alchemist::config::AiModelConfig {
        provider: "anthropic".to_string(),
        endpoint: Some(format!("{}/v1", base_url)),
        api_key_env: Some("TEST_API_KEY".to_string()),
        model_name: "claude-3-opus".to_string(),
        max_tokens: Some(4096),
        temperature: Some(0.7),
        timeout_seconds: Some(30),
        rate_limit: None,
        fallback_model: None,
        params: HashMap::new(),
    });
    
    std::env::set_var("TEST_API_KEY", "test-key-123");
    config
}