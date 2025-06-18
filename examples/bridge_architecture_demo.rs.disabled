//! Bridge Architecture Demonstration
//! 
//! This example demonstrates how CIM's bridge architecture enables
//! seamless integration with multiple AI providers through a generic
//! abstraction layer.

use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::future::Future;
use std::pin::Pin;

// Mock types for demonstration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequest {
    pub id: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub model: String,
    pub messages: Vec<Message>,
    pub parameters: ModelParameters,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelParameters {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub request_id: Uuid,
    pub correlation_id: Uuid,
    pub model: String,
    pub content: String,
    pub usage: Option<Usage>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug)]
pub struct MockError(String);

impl std::fmt::Display for MockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mock error: {}", self.0)
    }
}

impl std::error::Error for MockError {}

// Bridge types using enum instead of trait objects for simplicity
#[derive(Clone)]
pub enum Bridge {
    Ollama { host: String, port: u16 },
    OpenAI { api_key: String },
    Anthropic { api_key: String },
}

impl Bridge {
    pub fn name(&self) -> &str {
        match self {
            Bridge::Ollama { .. } => "Ollama",
            Bridge::OpenAI { .. } => "OpenAI",
            Bridge::Anthropic { .. } => "Anthropic",
        }
    }
    
    pub async fn query(&self, request: ModelRequest) -> Result<ModelResponse, MockError> {
        match self {
            Bridge::Ollama { host, port } => {
                println!("🦙 Ollama Bridge: Processing request to {}:{}", host, port);
                println!("   Model: {}", request.model);
                println!("   Correlation ID: {}", request.correlation_id);
                
                // Simulate API call
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                
                Ok(ModelResponse {
                    request_id: request.id,
                    correlation_id: request.correlation_id,
                    model: request.model,
                    content: format!("Mock Ollama response for: {}", 
                        request.messages.last().map(|m| &m.content).unwrap_or(&"".to_string())),
                    usage: Some(Usage {
                        prompt_tokens: 10,
                        completion_tokens: 20,
                        total_tokens: 30,
                    }),
                    timestamp: Utc::now(),
                })
            }
            Bridge::OpenAI { api_key } => {
                println!("🤖 OpenAI Bridge: Processing request");
                println!("   Model: {}", request.model);
                println!("   API Key: {}...", &api_key[..8]);
                
                // Simulate API call
                tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
                
                Ok(ModelResponse {
                    request_id: request.id,
                    correlation_id: request.correlation_id,
                    model: request.model,
                    content: format!("Mock OpenAI GPT response for: {}", 
                        request.messages.last().map(|m| &m.content).unwrap_or(&"".to_string())),
                    usage: Some(Usage {
                        prompt_tokens: 15,
                        completion_tokens: 25,
                        total_tokens: 40,
                    }),
                    timestamp: Utc::now(),
                })
            }
            Bridge::Anthropic { api_key } => {
                println!("🤖 Anthropic Bridge: Processing request");
                println!("   Model: {}", request.model);
                println!("   API Key: {}...", &api_key[..8]);
                
                // Simulate API call
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                
                Ok(ModelResponse {
                    request_id: request.id,
                    correlation_id: request.correlation_id,
                    model: request.model,
                    content: format!("Mock Anthropic Claude response for: {}", 
                        request.messages.last().map(|m| &m.content).unwrap_or(&"".to_string())),
                    usage: Some(Usage {
                        prompt_tokens: 20,
                        completion_tokens: 30,
                        total_tokens: 50,
                    }),
                    timestamp: Utc::now(),
                })
            }
        }
    }
}

// Bridge Manager for switching between providers
pub struct BridgeManager {
    bridges: HashMap<String, Bridge>,
    active_bridge: String,
}

impl BridgeManager {
    pub fn new() -> Self {
        Self {
            bridges: HashMap::new(),
            active_bridge: String::new(),
        }
    }
    
    pub fn register_bridge(&mut self, name: String, bridge: Bridge) {
        if self.active_bridge.is_empty() {
            self.active_bridge = name.clone();
        }
        self.bridges.insert(name, bridge);
    }
    
    pub fn switch_bridge(&mut self, name: &str) -> Result<(), String> {
        if self.bridges.contains_key(name) {
            self.active_bridge = name.to_string();
            println!("🔄 Switched to {} bridge", name);
            Ok(())
        } else {
            Err(format!("Bridge '{}' not found", name))
        }
    }
    
    pub async fn query(&self, mut request: ModelRequest) -> Result<ModelResponse, MockError> {
        let bridge = self.bridges.get(&self.active_bridge)
            .ok_or_else(|| MockError("No active bridge".to_string()))?;
        
        println!("\n📤 Sending request through {} bridge", self.active_bridge);
        
        // Add bridge info to metadata
        request.metadata.insert(
            "bridge".to_string(), 
            serde_json::Value::String(self.active_bridge.clone())
        );
        
        let response = bridge.query(request).await?;
        
        println!("📥 Received response from {}", self.active_bridge);
        println!("   Content: {}", response.content);
        if let Some(usage) = &response.usage {
            println!("   Tokens used: {}", usage.total_tokens);
        }
        
        Ok(response)
    }
}

// Simulate NATS event publishing
pub fn publish_event(event_type: &str, data: &serde_json::Value) {
    println!("\n📨 Publishing NATS event:");
    println!("   Subject: agent.{}", event_type);
    println!("   Data: {}", serde_json::to_string_pretty(data).unwrap());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CIM Bridge Architecture Demonstration\n");
    
    // Create bridge manager
    let mut manager = BridgeManager::new();
    
    // Register Ollama bridge
    manager.register_bridge(
        "ollama".to_string(), 
        Bridge::Ollama { 
            host: "localhost".to_string(), 
            port: 11434 
        }
    );
    
    // Register OpenAI bridge
    manager.register_bridge(
        "openai".to_string(), 
        Bridge::OpenAI { 
            api_key: "sk-mock-api-key-123456".to_string() 
        }
    );
    
    // Register Anthropic bridge
    manager.register_bridge(
        "anthropic".to_string(), 
        Bridge::Anthropic { 
            api_key: "ak-mock-api-key-789012".to_string() 
        }
    );
    
    // Create a sample request
    let correlation_id = Uuid::new_v4();
    let request = ModelRequest {
        id: Uuid::new_v4(),
        correlation_id,
        causation_id: None,
        model: "llama2".to_string(),
        messages: vec![
            Message {
                role: MessageRole::System,
                content: "You are a helpful assistant.".to_string(),
            },
            Message {
                role: MessageRole::User,
                content: "Explain event sourcing in simple terms.".to_string(),
            },
        ],
        parameters: ModelParameters {
            temperature: Some(0.7),
            max_tokens: Some(100),
        },
        metadata: HashMap::new(),
    };
    
    // Test with Ollama bridge
    println!("=== Testing Ollama Bridge ===");
    let response1 = manager.query(request.clone()).await?;
    
    // Publish completion event
    publish_event("query.completed", &serde_json::json!({
        "correlation_id": correlation_id,
        "model": response1.model,
        "bridge": "ollama",
        "tokens_used": response1.usage.as_ref().map(|u| u.total_tokens),
    }));
    
    // Switch to OpenAI bridge
    println!("\n=== Switching to OpenAI Bridge ===");
    manager.switch_bridge("openai")?;
    
    // Test with OpenAI bridge
    let mut request2 = request.clone();
    request2.model = "gpt-4".to_string();
    let response2 = manager.query(request2).await?;
    
    // Publish completion event
    publish_event("query.completed", &serde_json::json!({
        "correlation_id": correlation_id,
        "model": response2.model,
        "bridge": "openai",
        "tokens_used": response2.usage.as_ref().map(|u| u.total_tokens),
    }));
    
    // Switch to Anthropic bridge
    println!("\n=== Switching to Anthropic Bridge ===");
    manager.switch_bridge("anthropic")?;
    
    // Test with Anthropic bridge
    let mut request3 = request.clone();
    request3.model = "claude-3-opus".to_string();
    let response3 = manager.query(request3).await?;
    
    // Publish completion event
    publish_event("query.completed", &serde_json::json!({
        "correlation_id": correlation_id,
        "model": response3.model,
        "bridge": "anthropic",
        "tokens_used": response3.usage.as_ref().map(|u| u.total_tokens),
    }));
    
    // Demonstrate model switching event
    println!("\n=== Model Switching Event ===");
    publish_event("model.switched", &serde_json::json!({
        "agent_id": Uuid::new_v4(),
        "old_model": "llama2",
        "new_model": "claude-3-opus",
        "old_bridge": "ollama",
        "new_bridge": "anthropic",
        "reason": "User requested more capable model",
    }));
    
    // Show how Dialog domain would use this
    println!("\n=== Dialog Domain Integration ===");
    println!("The Dialog domain would:");
    println!("1. Send commands via NATS: agent.command.query");
    println!("2. Bridge receives command and routes to appropriate provider");
    println!("3. Response published to: agent.query.response");
    println!("4. Dialog domain updates conversation with response");
    println!("5. Context graph updated with new information");
    
    // Show composition example
    println!("\n=== Domain Composition with cim-compose ===");
    println!("Using cim-compose, we can create composed domains:");
    println!("- cim-domain-ollama = Agent Domain + Ollama Bridge");
    println!("- cim-domain-openai = Agent Domain + OpenAI Bridge");
    println!("- cim-domain-anthropic = Agent Domain + Anthropic Bridge");
    println!("\nThis enables:");
    println!("- Seamless provider switching");
    println!("- Unified event-driven interface");
    println!("- Consistent correlation tracking");
    println!("- Provider-agnostic dialog management");
    
    println!("\n✅ Bridge architecture demonstration complete!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_bridge_switching() {
        let mut manager = BridgeManager::new();
        
        let ollama = Box::new(MockOllamaBridge::new("localhost".to_string(), 11434));
        manager.register_bridge("ollama".to_string(), ollama as Box<dyn AIModelBridge<Config = (), Error = MockError>>);
        
        let openai = Box::new(MockOpenAIBridge::new("test-key".to_string()));
        manager.register_bridge("openai".to_string(), openai as Box<dyn AIModelBridge<Config = (), Error = MockError>>);
        
        assert!(manager.switch_bridge("ollama").is_ok());
        assert!(manager.switch_bridge("openai").is_ok());
        assert!(manager.switch_bridge("anthropic").is_err());
    }
} 