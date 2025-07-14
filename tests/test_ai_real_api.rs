//! Integration tests for AI models with real API keys

#[cfg(test)]
mod ai_integration_tests {
    use alchemist::{
        ai::{AiManager, ModelStatus},
        config::{AlchemistConfig, AiModelConfig},
    };
    use std::env;
    
    /// Helper to create test config with real API keys
    fn create_test_config() -> AlchemistConfig {
        // Load API keys from environment or .env file
        dotenv::dotenv().ok();
        
        let mut config = AlchemistConfig::default();
        
        // Add Anthropic models
        if env::var("ANTHROPIC_API_KEY").is_ok() {
            config.ai_models.insert("claude-3-sonnet".to_string(), AiModelConfig {
                provider: "anthropic".to_string(),
                endpoint: None,
                api_key_env: Some("ANTHROPIC_API_KEY".to_string()),
                model_name: "claude-3-sonnet-20240229".to_string(),
                max_tokens: Some(4096),
                temperature: Some(0.7),
                timeout_seconds: Some(30),
                rate_limit: None,
                fallback_model: None,
                params: std::collections::HashMap::new(),
            });
            
            config.ai_models.insert("claude-3-opus".to_string(), AiModelConfig {
                provider: "anthropic".to_string(),
                endpoint: None,
                api_key_env: Some("ANTHROPIC_API_KEY".to_string()),
                model_name: "claude-3-opus-20240229".to_string(),
                max_tokens: Some(4096),
                temperature: Some(0.7),
                timeout_seconds: Some(30),
                rate_limit: None,
                fallback_model: None,
                params: std::collections::HashMap::new(),
            });
        }
        
        // Add OpenAI models
        if env::var("OPENAI_API_KEY").is_ok() {
            config.ai_models.insert("gpt-4".to_string(), AiModelConfig {
                provider: "openai".to_string(),
                endpoint: None,
                api_key_env: Some("OPENAI_API_KEY".to_string()),
                model_name: "gpt-4-turbo-preview".to_string(),
                max_tokens: Some(4096),
                temperature: Some(0.7),
                timeout_seconds: Some(30),
                rate_limit: None,
                fallback_model: None,
                params: std::collections::HashMap::new(),
            });
            
            config.ai_models.insert("gpt-3.5-turbo".to_string(), AiModelConfig {
                provider: "openai".to_string(),
                endpoint: None,
                api_key_env: Some("OPENAI_API_KEY".to_string()),
                model_name: "gpt-3.5-turbo".to_string(),
                max_tokens: Some(4096),
                temperature: Some(0.7),
                timeout_seconds: Some(30),
                rate_limit: None,
                fallback_model: None,
            });
        }
        
        config
    }
    
    #[tokio::test]
    #[ignore] // Run with: cargo test --ignored test_anthropic_connection
    async fn test_anthropic_connection() {
        let config = create_test_config();
        let ai_manager = AiManager::new(&config).await.unwrap();
        
        // Test Claude 3 Sonnet
        let result = ai_manager.test_model("claude-3-sonnet").await.unwrap();
        assert!(result.success, "Claude 3 Sonnet test failed: {:?}", result.error);
        assert_eq!(result.status, ModelStatus::Available);
        println!("Claude 3 Sonnet - Latency: {}ms", result.latency_ms);
        
        // Test Claude 3 Opus
        let result = ai_manager.test_model("claude-3-opus").await.unwrap();
        assert!(result.success, "Claude 3 Opus test failed: {:?}", result.error);
        assert_eq!(result.status, ModelStatus::Available);
        println!("Claude 3 Opus - Latency: {}ms", result.latency_ms);
    }
    
    #[tokio::test]
    #[ignore] // Run with: cargo test --ignored test_openai_connection
    async fn test_openai_connection() {
        let config = create_test_config();
        let ai_manager = AiManager::new(&config).await.unwrap();
        
        // Test GPT-4
        let result = ai_manager.test_model("gpt-4").await.unwrap();
        assert!(result.success, "GPT-4 test failed: {:?}", result.error);
        assert_eq!(result.status, ModelStatus::Available);
        println!("GPT-4 - Latency: {}ms", result.latency_ms);
        
        // Test GPT-3.5 Turbo
        let result = ai_manager.test_model("gpt-3.5-turbo").await.unwrap();
        assert!(result.success, "GPT-3.5 Turbo test failed: {:?}", result.error);
        assert_eq!(result.status, ModelStatus::Available);
        println!("GPT-3.5 Turbo - Latency: {}ms", result.latency_ms);
    }
    
    #[tokio::test]
    #[ignore] // Run with: cargo test --ignored test_streaming_response
    async fn test_streaming_response() {
        use futures::StreamExt;
        
        let config = create_test_config();
        let ai_manager = AiManager::new(&config).await.unwrap();
        
        // Test streaming with Claude
        let mut stream = ai_manager.stream_completion(
            "claude-3-sonnet", 
            "Write a haiku about Rust programming"
        ).await.unwrap();
        
        let mut response = String::new();
        let mut token_count = 0;
        
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(token) => {
                    response.push_str(&token);
                    token_count += 1;
                    print!("{}", token); // Show streaming in action
                }
                Err(e) => panic!("Streaming error: {}", e),
            }
        }
        
        println!("\n\nComplete response ({} tokens):\n{}", token_count, response);
        assert!(!response.is_empty(), "No response received");
        assert!(token_count > 0, "No tokens received");
    }
    
    #[tokio::test]
    #[ignore] // Run with: cargo test --ignored test_dialog_conversation
    async fn test_dialog_conversation() {
        use alchemist::dialog::{DialogManager, MessageRole};
        
        let config = create_test_config();
        let mut dialog_manager = DialogManager::new(&config).await.unwrap();
        let ai_manager = AiManager::new(&config).await.unwrap();
        
        // Create a new dialog
        let dialog_id = dialog_manager.new_dialog_cli(
            Some("Test Conversation".to_string()), 
            Some("claude-3-sonnet".to_string())
        ).await.unwrap();
        
        // Add a user message
        dialog_manager.add_message(
            &dialog_id,
            MessageRole::User,
            "What is Rust programming language best known for?".to_string(),
            None
        ).await.unwrap();
        
        // Get AI response
        let prompt = "User: What is Rust programming language best known for?\n\nAssistant:";
        let response = ai_manager.get_completion("claude-3-sonnet", prompt).await.unwrap();
        
        // Add AI response to dialog
        dialog_manager.add_message(
            &dialog_id,
            MessageRole::Assistant,
            response.clone(),
            Some(serde_json::json!({
                "model": "claude-3-sonnet",
                "tokens": response.split_whitespace().count()
            }))
        ).await.unwrap();
        
        // Verify dialog has messages
        let messages = dialog_manager.get_all_messages(&dialog_id).await.unwrap();
        assert_eq!(messages.len(), 2);
        assert!(!response.is_empty(), "AI response is empty");
        
        println!("Dialog test successful!");
        println!("User: What is Rust programming language best known for?");
        println!("AI: {}", response);
    }
}