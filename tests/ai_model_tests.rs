//! Tests for AI model connection verification and management

use alchemist::{
    ai::{AiManager, ModelStatus, TestResult},
    config::{AlchemistConfig, AiModelConfig, CacheConfig, PolicyConfig},
};
use anyhow::Result;
use std::collections::HashMap;

#[tokio::test]
async fn test_ai_manager_creation() -> Result<()> {
    // Test that we can create an AI manager with a config
    let config = create_test_config();
    let _manager = AiManager::new(&config).await?;
    
    println!("✅ AI manager creation test passed");
    Ok(())
}

#[tokio::test]
async fn test_ai_model_configuration() -> Result<()> {
    // Test that we can configure AI models
    let mut config = create_test_config();
    
    // Add multiple models
    config.ai_models.insert("gpt-4".to_string(), AiModelConfig {
        provider: "openai".to_string(),
        endpoint: Some("https://api.openai.com/v1".to_string()),
        api_key_env: Some("OPENAI_API_KEY".to_string()),
        model_name: "gpt-4".to_string(),
        max_tokens: Some(4096),
        temperature: Some(0.7),
        timeout_seconds: Some(30),
        rate_limit: None,
        fallback_model: None,
        params: HashMap::new(),
    });
    
    config.ai_models.insert("claude-3".to_string(), AiModelConfig {
        provider: "anthropic".to_string(),
        endpoint: Some("https://api.anthropic.com/v1".to_string()),
        api_key_env: Some("ANTHROPIC_API_KEY".to_string()),
        model_name: "claude-3-opus".to_string(),
        max_tokens: Some(4096),
        temperature: Some(0.7),
        timeout_seconds: Some(30),
        rate_limit: None,
        fallback_model: Some("gpt-4".to_string()),
        params: HashMap::new(),
    });
    
    assert_eq!(config.ai_models.len(), 2);
    assert!(config.ai_models.contains_key("gpt-4"));
    assert!(config.ai_models.contains_key("claude-3"));
    
    // Check fallback configuration
    assert_eq!(
        config.ai_models.get("claude-3").unwrap().fallback_model,
        Some("gpt-4".to_string())
    );
    
    println!("✅ AI model configuration test passed");
    Ok(())
}

#[tokio::test]
async fn test_test_result_structure() {
    // Test that TestResult has the expected fields
    let result = TestResult {
        status: ModelStatus::Available,
        success: true,
        latency_ms: 100,
        error: None,
        model_used: "test-model".to_string(),
        used_fallback: false,
    };
    
    assert!(result.success);
    assert_eq!(result.model_used, "test-model");
    assert!(matches!(result.status, ModelStatus::Available));
    assert!(!result.used_fallback);
    
    println!("✅ TestResult structure test passed");
}

#[tokio::test]
async fn test_model_status_enum() {
    // Test ModelStatus enum variants
    let statuses = vec![
        ModelStatus::Unknown,
        ModelStatus::Available,
        ModelStatus::Unavailable,
        ModelStatus::RateLimited,
    ];
    
    for status in statuses {
        match status {
            ModelStatus::Unknown => println!("Status: Unknown"),
            ModelStatus::Available => println!("Status: Available"),
            ModelStatus::Unavailable => println!("Status: Unavailable"),
            ModelStatus::RateLimited => println!("Status: RateLimited"),
            _ => println!("Status: Other"),
        }
    }
    
    println!("✅ ModelStatus enum test passed");
}

#[tokio::test]
async fn test_config_with_cache() -> Result<()> {
    // Test configuration with cache settings
    let config = AlchemistConfig {
        general: Default::default(),
        ai_models: HashMap::new(),
        policy: PolicyConfig {
            storage_path: "/tmp/test_policies".to_string(),
            validation_enabled: true,
            evaluation_timeout: 5000,
            cache_ttl: Some(300),
        },
        deployments: HashMap::new(),
        domains: Default::default(),
        cache: Some(CacheConfig {
            redis_url: Some("redis://localhost:6379".to_string()),
            default_ttl: 3600,
            max_memory_items: 1000,
        }),
    };
    
    // Verify cache configuration
    assert!(config.cache.is_some());
    let cache = config.cache.unwrap();
    assert_eq!(cache.redis_url, Some("redis://localhost:6379".to_string()));
    assert_eq!(cache.default_ttl, 3600);
    assert_eq!(cache.max_memory_items, 1000);
    
    println!("✅ Config with cache test passed");
    Ok(())
}

fn create_test_config() -> AlchemistConfig {
    AlchemistConfig {
        general: Default::default(),
        ai_models: HashMap::new(),
        policy: PolicyConfig {
            storage_path: "/tmp/test_policies".to_string(),
            validation_enabled: true,
            evaluation_timeout: 5000,
            cache_ttl: Some(300),
        },
        deployments: HashMap::new(),
        domains: Default::default(),
        cache: Some(CacheConfig {
            redis_url: None,
            default_ttl: 3600,
            max_memory_items: 1000,
        }),
    }
}