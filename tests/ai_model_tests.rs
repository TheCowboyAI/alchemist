//! Tests for AI model connection verification and management

use alchemist::ai::{AiManager, ModelStatus, TestResult};
use alchemist::config::{AlchemistConfig, AiModelConfig};
use anyhow::Result;
use mockito::Server;
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_ai_model_connection_success() -> Result<()> {
    // Given: A mock server
    let mut server = Server::new_async().await;
    
    // And: A configured AI model
    let config = create_test_config(&server.url());
    let mut manager = AiManager::new(&config).await?;
    
    // And: A mock endpoint that responds successfully
    let _m = server.mock("POST", "/v1/messages")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "id": "msg_123",
            "content": [{
                "type": "text",
                "text": "Test response"
            }],
            "model": "claude-3-opus",
            "usage": {
                "input_tokens": 10,
                "output_tokens": 5
            }
        }).to_string())
        .create_async()
        .await;
    
    // When: Testing the connection
    let result = manager.test_connection("claude-3").await?;
    
    // Then: The test should succeed
    assert_eq!(result.status, ModelStatus::Available);
    assert!(result.success);
    assert!(result.latency_ms >= 0); // Can be 0 for very fast mocked responses
    assert_eq!(result.error, None);
    
    Ok(())
}

#[tokio::test]
async fn test_ai_model_connection_failure() -> Result<()> {
    // Given: A mock server
    let mut server = Server::new_async().await;
    
    // And: A configured AI model
    let config = create_test_config(&server.url());
    let mut manager = AiManager::new(&config).await?;
    
    // And: A mock endpoint that returns an error
    let _m = server.mock("POST", "/v1/messages")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "error": {
                "type": "authentication_error",
                "message": "Invalid API key"
            }
        }).to_string())
        .create_async()
        .await;
    
    // When: Testing the connection
    let result = manager.test_connection("claude-3").await?;
    
    // Then: The test should fail with details
    assert_eq!(result.status, ModelStatus::Error);
    assert!(!result.success);
    assert!(result.error.is_some());
    assert!(result.error.unwrap().contains("Invalid API key"));
    
    Ok(())
}

#[tokio::test]
async fn test_ai_model_connection_timeout() -> Result<()> {
    // Given: A configured AI model with short timeout pointing to non-existent server
    let mut config = AlchemistConfig::default();
    config.ai_models.insert("claude-3".to_string(), AiModelConfig {
        provider: "anthropic".to_string(),
        endpoint: Some("http://localhost:19999/v1".to_string()), // Non-existent port
        api_key_env: Some("TEST_API_KEY".to_string()),
        model_name: "claude-3-opus".to_string(),
        max_tokens: Some(4096),
        temperature: Some(0.7),
        timeout_seconds: Some(1), // Short timeout
        rate_limit: None,
        fallback_model: None,
        params: HashMap::new(),
    });
    std::env::set_var("TEST_API_KEY", "test-key-123");
    
    let mut manager = AiManager::new(&config).await?;
    
    // When: Testing the connection (will timeout trying to connect)
    let result = manager.test_connection("claude-3").await?;
    
    // Then: The test should timeout or error
    assert!(!result.success);
    assert!(matches!(result.status, ModelStatus::Timeout | ModelStatus::Error));
    
    Ok(())
}

#[tokio::test]
async fn test_batch_model_testing() -> Result<()> {
    // Given: Mock servers
    let mut server = Server::new_async().await;
    
    // And: Multiple configured AI models
    let config = create_test_config_multiple(&server.url());
    let mut manager = AiManager::new(&config).await?;
    
    // And: Mock endpoints for each model
    let _m1 = server.mock("POST", "/v1/messages")
        .match_header("x-api-key", "test-key-123")
        .with_status(200)
        .with_body(json!({"id": "msg_1"}).to_string())
        .create_async()
        .await;
        
    let _m2 = server.mock("POST", "/v1/chat/completions")
        .match_header("authorization", "Bearer test-key-456")
        .with_status(200)
        .with_body(json!({"id": "chat_1"}).to_string())
        .create_async()
        .await;
    
    // When: Testing all connections
    let results = manager.test_all_connections().await?;
    
    // Then: Should have results for all models
    assert_eq!(results.len(), 2);
    assert!(results.get("claude-3").unwrap().success);
    assert!(results.get("gpt-4").unwrap().success);
    
    Ok(())
}

#[tokio::test]
async fn test_model_rate_limiting() -> Result<()> {
    // Given: A mock server
    let mut server = Server::new_async().await;
    
    // And: A model with rate limits
    let mut config = create_test_config(&server.url());
    config.ai_models.get_mut("claude-3").unwrap().rate_limit = Some(2); // 2 per minute
    let mut manager = AiManager::new(&config).await?;
    
    // And: A mock that responds to all requests
    let _m = server.mock("POST", "/v1/messages")
        .with_status(200)
        .with_body(json!({"id": "msg_1"}).to_string())
        .expect_at_least(2)
        .create_async()
        .await;
    
    // When: Making multiple requests
    let mut results = vec![];
    for _ in 0..3 {
        results.push(manager.test_connection("claude-3").await?);
    }
    
    // Then: Third request should be rate limited
    assert!(results[0].success);
    assert!(results[1].success);
    assert!(!results[2].success);
    assert_eq!(results[2].status, ModelStatus::RateLimited);
    
    Ok(())
}

#[tokio::test]
async fn test_model_fallback() -> Result<()> {
    // Given: Mock servers
    let mut server = Server::new_async().await;
    
    // And: Models with fallback configuration
    let mut config = create_test_config_multiple(&server.url());
    config.ai_models.get_mut("claude-3").unwrap().fallback_model = Some("gpt-4".to_string());
    let mut manager = AiManager::new(&config).await?;
    
    // And: Primary model fails, fallback succeeds
    let _m1 = server.mock("POST", "/v1/messages")
        .with_status(500)
        .create_async()
        .await;
        
    let _m2 = server.mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_body(json!({"id": "chat_1"}).to_string())
        .create_async()
        .await;
    
    // When: Testing with fallback enabled
    let result = manager.test_connection_with_fallback("claude-3").await?;
    
    // Then: Should succeed with fallback
    assert!(result.success);
    assert_eq!(result.model_used, "gpt-4");
    assert!(result.used_fallback);
    
    Ok(())
}

fn create_test_config(base_url: &str) -> AlchemistConfig {
    let mut config = AlchemistConfig::default();
    
    config.ai_models.insert("claude-3".to_string(), AiModelConfig {
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

fn create_test_config_multiple(base_url: &str) -> AlchemistConfig {
    let mut config = create_test_config(base_url);
    
    config.ai_models.insert("gpt-4".to_string(), AiModelConfig {
        provider: "openai".to_string(),
        endpoint: Some(format!("{}/v1", base_url)),
        api_key_env: Some("TEST_API_KEY_2".to_string()),
        model_name: "gpt-4-turbo".to_string(),
        max_tokens: Some(4096),
        temperature: Some(0.7),
        timeout_seconds: Some(30),
        rate_limit: None,
        fallback_model: None,
        params: HashMap::new(),
    });
    
    std::env::set_var("TEST_API_KEY", "test-key-123");
    std::env::set_var("TEST_API_KEY_2", "test-key-456");
    config
}