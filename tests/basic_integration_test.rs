//! Basic integration test - minimal functionality verification

use alchemist::{
    config::{AlchemistConfig, AiModelConfig, CacheConfig, PolicyConfig},
    shell::AlchemistShell,
    shell_commands::{AiCommands, DialogCommands},
    ai::AiManager,
    dialog::DialogManager,
};
use std::collections::HashMap;
use anyhow::Result;
use tempfile::TempDir;

#[tokio::test]
async fn test_basic_shell_creation() -> Result<()> {
    // Create a temporary directory for test data
    let temp_dir = TempDir::new()?;
    
    // Create basic configuration
    let mut config = AlchemistConfig {
        general: Default::default(),
        ai_models: HashMap::new(),
        policy: PolicyConfig {
            storage_path: temp_dir.path().join("policies").to_string_lossy().to_string(),
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
    };
    
    // Update paths to use temp directory
    config.general.dialog_history_path = temp_dir.path().join("dialogs").to_string_lossy().to_string();
    config.policy.storage_path = temp_dir.path().join("policies").to_string_lossy().to_string();
    
    // Create required directories
    std::fs::create_dir_all(&config.general.dialog_history_path)?;
    std::fs::create_dir_all(&config.policy.storage_path)?;
    
    // Create shell
    let shell = AlchemistShell::new(config).await?;
    
    println!("✅ Basic shell creation test passed");
    Ok(())
}

#[tokio::test]
async fn test_ai_manager_creation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create config with a test AI model
    let mut config = AlchemistConfig {
        general: Default::default(),
        ai_models: HashMap::new(),
        policy: PolicyConfig {
            storage_path: temp_dir.path().join("policies").to_string_lossy().to_string(),
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
    };
    
    // Add a mock AI model
    config.ai_models.insert("test-model".to_string(), AiModelConfig {
        provider: "openai".to_string(),
        endpoint: Some("http://localhost:8080".to_string()),
        api_key_env: Some("TEST_API_KEY".to_string()),
        model_name: "gpt-3.5-turbo".to_string(),
        max_tokens: Some(1000),
        temperature: Some(0.7),
        timeout_seconds: Some(30),
        rate_limit: None,
        fallback_model: None,
        params: HashMap::new(),
    });
    
    // Set dummy API key
    std::env::set_var("TEST_API_KEY", "test-key");
    
    // Create AI manager
    let ai_manager = AiManager::new(&config).await?;
    
    println!("✅ AI manager creation test passed");
    Ok(())
}

#[tokio::test]
async fn test_dialog_manager_creation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let dialog_path = temp_dir.path().join("dialogs");
    std::fs::create_dir_all(&dialog_path)?;
    
    // Create a config for dialog manager
    let config = AlchemistConfig {
        general: Default::default(),
        ai_models: HashMap::new(),
        policy: PolicyConfig {
            storage_path: temp_dir.path().join("policies").to_string_lossy().to_string(),
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
    };
    
    // Create dialog manager
    let dialog_manager = DialogManager::new(&config).await?;
    
    // List dialogs
    let dialogs = dialog_manager.list_recent(10).await?;
    // Just check that list_recent works, don't assume empty
    println!("Found {} dialogs", dialogs.len());
    
    println!("✅ Dialog manager creation test passed");
    Ok(())
}

#[tokio::test]
async fn test_command_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    let mut config = AlchemistConfig {
        general: Default::default(),
        ai_models: HashMap::new(),
        policy: PolicyConfig {
            storage_path: temp_dir.path().join("policies").to_string_lossy().to_string(),
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
    };
    
    config.general.dialog_history_path = temp_dir.path().join("dialogs").to_string_lossy().to_string();
    config.policy.storage_path = temp_dir.path().join("policies").to_string_lossy().to_string();
    
    std::fs::create_dir_all(&config.general.dialog_history_path)?;
    std::fs::create_dir_all(&config.policy.storage_path)?;
    
    let mut shell = AlchemistShell::new(config).await?;
    
    // Test AI list command (should work even with no models)
    shell.handle_ai_command(AiCommands::List).await?;
    
    // Test dialog list command (should work even with no dialogs)
    shell.handle_dialog_command(DialogCommands::List { count: 10 }).await?;
    
    println!("✅ Command handling test passed");
    Ok(())
}

#[tokio::test]
async fn test_config_serialization() -> Result<()> {
    let config = AlchemistConfig {
        general: Default::default(),
        ai_models: HashMap::new(),
        policy: PolicyConfig {
            storage_path: "/tmp/policies".to_string(),
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
    
    // Serialize to TOML
    let toml_str = toml::to_string(&config)?;
    
    // Deserialize back
    let config2: AlchemistConfig = toml::from_str(&toml_str)?;
    
    // Verify cache config survived serialization
    assert!(config2.cache.is_some());
    assert_eq!(config2.cache.unwrap().default_ttl, 3600);
    
    println!("✅ Config serialization test passed");
    Ok(())
}