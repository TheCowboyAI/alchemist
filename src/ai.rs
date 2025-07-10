//! AI model management for Alchemist

use anyhow::{Result, Context};
use std::collections::HashMap;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};
use reqwest::Client;
use serde_json::json;

use crate::{
    config::{AlchemistConfig, AiModelConfig},
    shell_commands::AiCommands,
};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use futures::{Stream, StreamExt, stream};
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};
use bytes::Bytes;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModel {
    pub name: String,
    pub config: AiModelConfig,
    pub status: ModelStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelStatus {
    Available,
    Unavailable,
    Unknown,
    Testing,
    Error,
    Timeout,
    RateLimited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub status: ModelStatus,
    pub success: bool,
    pub latency_ms: u64,
    pub error: Option<String>,
    pub model_used: String,
    pub used_fallback: bool,
}

pub struct AiManager {
    models: DashMap<String, AiModel>,
    client: Client,
    default_model: Option<String>,
    rate_limits: DashMap<String, RateLimiter>,
}

#[derive(Debug)]
struct RateLimiter {
    max_requests: u32,
    window_start: Instant,
    request_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingResponse {
    pub content: String,
    pub is_final: bool,
    pub token_count: Option<u32>,
}

pub struct StreamingResponseStream {
    inner: Pin<Box<dyn Stream<Item = Result<StreamingResponse>> + Send>>,
}

impl Stream for StreamingResponseStream {
    type Item = Result<StreamingResponse>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

impl AiManager {
    pub async fn new(config: &AlchemistConfig) -> Result<Self> {
        let models = DashMap::new();
        let rate_limits = DashMap::new();
        
        // Load configured models
        for (name, model_config) in &config.ai_models {
            models.insert(name.clone(), AiModel {
                name: name.clone(),
                config: model_config.clone(),
                status: ModelStatus::Unknown,
            });
            
            // Initialize rate limiter if configured
            if let Some(rate_limit) = model_config.rate_limit {
                rate_limits.insert(name.clone(), RateLimiter {
                    max_requests: rate_limit,
                    window_start: Instant::now(),
                    request_count: 0,
                });
            }
        }
        
        Ok(Self {
            models,
            client: Client::new(),
            default_model: config.general.default_ai_model.clone(),
            rate_limits,
        })
    }
    
    pub async fn handle_command(&mut self, command: AiCommands) -> Result<()> {
        match command {
            AiCommands::List => {
                self.list_models_cli().await?;
            }
            AiCommands::Add { name, provider, endpoint } => {
                self.add_model(name, provider, endpoint).await?;
            }
            AiCommands::Remove { name } => {
                self.remove_model(&name).await?;
            }
            AiCommands::Test { name } => {
                self.test_model(&name).await?;
            }
        }
        Ok(())
    }
    
    pub async fn list_models(&self) -> Result<Vec<(String, AiModelConfig)>> {
        let mut models = Vec::new();
        for entry in self.models.iter() {
            models.push((entry.key().clone(), entry.value().config.clone()));
        }
        models.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(models)
    }
    
    async fn list_models_cli(&self) -> Result<()> {
        println!("🤖 Configured AI Models:");
        println!("─────────────────────────────");
        
        if self.models.is_empty() {
            println!("No models configured. Use 'alchemist ai add' to add a model.");
            return Ok(());
        }
        
        for entry in self.models.iter() {
            let model = entry.value();
            let default_marker = if Some(&model.name) == self.default_model.as_ref() {
                " (default)"
            } else {
                ""
            };
            
            let status_icon = match model.status {
                ModelStatus::Available => "✅",
                ModelStatus::Unavailable => "❌",
                ModelStatus::Testing => "🔄",
                ModelStatus::Unknown => "❓",
                ModelStatus::Error => "⚠️",
                ModelStatus::Timeout => "⏱️",
                ModelStatus::RateLimited => "🚫",
            };
            
            println!(
                "{} {} - {} ({}){}",
                status_icon,
                model.name,
                model.config.model_name,
                model.config.provider,
                default_marker
            );
            
            if let Some(endpoint) = &model.config.endpoint {
                println!("     Endpoint: {}", endpoint);
            }
            
            if let Some(api_key_env) = &model.config.api_key_env {
                let has_key = std::env::var(api_key_env).is_ok();
                println!("     API Key: {} ({})", api_key_env, if has_key { "set" } else { "not set" });
            }
        }
        
        Ok(())
    }
    
    async fn add_model(&mut self, name: String, provider: String, endpoint: Option<String>) -> Result<()> {
        info!("Adding AI model: {} ({})", name, provider);
        
        // Create model config based on provider
        let config = match provider.as_str() {
            "openai" => AiModelConfig {
                provider: provider.clone(),
                endpoint: endpoint.or_else(|| Some("https://api.openai.com/v1".to_string())),
                api_key_env: Some("OPENAI_API_KEY".to_string()),
                model_name: "gpt-4-turbo-preview".to_string(),
                max_tokens: Some(4096),
                temperature: Some(0.7),
                timeout_seconds: Some(30),
                rate_limit: None,
                fallback_model: None,
                params: HashMap::new(),
            },
            "anthropic" => AiModelConfig {
                provider: provider.clone(),
                endpoint: endpoint.or_else(|| Some("https://api.anthropic.com/v1".to_string())),
                api_key_env: Some("ANTHROPIC_API_KEY".to_string()),
                model_name: "claude-3-opus-20240229".to_string(),
                max_tokens: Some(4096),
                temperature: Some(0.7),
                timeout_seconds: Some(30),
                rate_limit: None,
                fallback_model: None,
                params: HashMap::new(),
            },
            "ollama" => AiModelConfig {
                provider: provider.clone(),
                endpoint: endpoint.or_else(|| Some("http://localhost:11434".to_string())),
                api_key_env: None,
                model_name: "llama2".to_string(),
                max_tokens: Some(2048),
                temperature: Some(0.8),
                timeout_seconds: Some(60),
                rate_limit: None,
                fallback_model: None,
                params: HashMap::new(),
            },
            _ => {
                error!("Unknown provider: {}", provider);
                return Err(anyhow::anyhow!("Unknown provider: {}. Supported: openai, anthropic, ollama", provider));
            }
        };
        
        let model = AiModel {
            name: name.clone(),
            config,
            status: ModelStatus::Unknown,
        };
        
        self.models.insert(name.clone(), model);
        
        println!("✅ Added model: {}", name);
        println!("   Run 'alchemist ai test {}' to verify connection", name);
        
        Ok(())
    }
    
    async fn remove_model(&mut self, name: &str) -> Result<()> {
        if self.models.remove(name).is_some() {
            println!("✅ Removed model: {}", name);
            
            if Some(name) == self.default_model.as_deref() {
                self.default_model = None;
                warn!("Removed default model. Please set a new default.");
            }
        } else {
            println!("❌ Model not found: {}", name);
        }
        
        Ok(())
    }
    
    async fn test_model(&mut self, name: &str) -> Result<()> {
        let model = self.models.get(name)
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", name))?
            .clone();
        
        println!("🔄 Testing model: {} ...", name);
        
        // Update status to testing
        if let Some(mut entry) = self.models.get_mut(name) {
            entry.status = ModelStatus::Testing;
        }
        
        let result = match model.config.provider.as_str() {
            "openai" => self.test_openai(&model).await,
            "anthropic" => self.test_anthropic(&model).await,
            "ollama" => self.test_ollama(&model).await,
            _ => Err(anyhow::anyhow!("Unknown provider")),
        };
        
        // Update status based on result
        let status = match &result {
            Ok(_) => {
                println!("✅ Model {} is available and responding", name);
                ModelStatus::Available
            }
            Err(e) => {
                println!("❌ Model {} test failed: {}", name, e);
                ModelStatus::Unavailable
            }
        };
        
        if let Some(mut entry) = self.models.get_mut(name) {
            entry.status = status;
        }
        
        result.map(|_| ())
    }
    
    async fn test_openai(&self, model: &AiModel) -> Result<()> {
        let api_key = if let Some(env_var) = &model.config.api_key_env {
            std::env::var(env_var)
                .context(format!("API key not found in environment variable: {}", env_var))?
        } else {
            return Err(anyhow::anyhow!("No API key configured"));
        };
        
        let endpoint = model.config.endpoint.as_deref().unwrap_or("https://api.openai.com/v1");
        let url = format!("{}/chat/completions", endpoint);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&json!({
                "model": model.config.model_name,
                "messages": [{"role": "user", "content": "Say 'test successful' and nothing else"}],
                "max_tokens": 10,
                "temperature": 0
            }))
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }
        
        Ok(())
    }
    
    async fn test_anthropic(&self, model: &AiModel) -> Result<()> {
        let api_key = if let Some(env_var) = &model.config.api_key_env {
            std::env::var(env_var)
                .context(format!("API key not found in environment variable: {}", env_var))?
        } else {
            return Err(anyhow::anyhow!("No API key configured"));
        };
        
        let endpoint = model.config.endpoint.as_deref().unwrap_or("https://api.anthropic.com/v1");
        let url = format!("{}/messages", endpoint);
        
        let response = self.client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&json!({
                "model": model.config.model_name,
                "messages": [{"role": "user", "content": "Say 'test successful' and nothing else"}],
                "max_tokens": 10
            }))
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Anthropic API error: {}", error_text));
        }
        
        Ok(())
    }
    
    async fn test_ollama(&self, model: &AiModel) -> Result<()> {
        let endpoint = model.config.endpoint.as_deref().unwrap_or("http://localhost:11434");
        let url = format!("{}/api/generate", endpoint);
        
        let response = self.client
            .post(&url)
            .json(&json!({
                "model": model.config.model_name,
                "prompt": "Say 'test successful' and nothing else",
                "stream": false
            }))
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Ollama API error: {}", error_text));
        }
        
        Ok(())
    }
    
    pub async fn get_model(&self, name: &str) -> Result<AiModel> {
        self.models
            .get(name)
            .map(|entry| entry.clone())
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", name))
    }
    
    pub fn get_default_model(&self) -> Option<String> {
        self.default_model.clone()
    }
    
    /// Test connection to a specific model
    pub async fn test_connection(&mut self, name: &str) -> Result<TestResult> {
        let start = Instant::now();
        
        // Check rate limit
        if let Some(mut limiter) = self.rate_limits.get_mut(name) {
            let now = Instant::now();
            let window_duration = Duration::from_secs(60); // 1 minute window
            
            if now.duration_since(limiter.window_start) > window_duration {
                // Reset window
                limiter.window_start = now;
                limiter.request_count = 0;
            }
            
            if limiter.request_count >= limiter.max_requests {
                return Ok(TestResult {
                    status: ModelStatus::RateLimited,
                    success: false,
                    latency_ms: 0,
                    error: Some("Rate limit exceeded".to_string()),
                    model_used: name.to_string(),
                    used_fallback: false,
                });
            }
            
            limiter.request_count += 1;
        }
        
        let model = self.models.get(name)
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", name))?
            .clone();
        
        // Get timeout duration
        let timeout_duration = Duration::from_secs(
            model.config.timeout_seconds.unwrap_or(30) as u64
        );
        
        // Test with timeout
        let result = timeout(timeout_duration, self.test_model_internal(&model)).await;
        
        let latency_ms = start.elapsed().as_millis() as u64;
        
        match result {
            Ok(Ok(_)) => Ok(TestResult {
                status: ModelStatus::Available,
                success: true,
                latency_ms,
                error: None,
                model_used: name.to_string(),
                used_fallback: false,
            }),
            Ok(Err(e)) => Ok(TestResult {
                status: ModelStatus::Error,
                success: false,
                latency_ms,
                error: Some(e.to_string()),
                model_used: name.to_string(),
                used_fallback: false,
            }),
            Err(_) => Ok(TestResult {
                status: ModelStatus::Timeout,
                success: false,
                latency_ms,
                error: Some("Connection timeout".to_string()),
                model_used: name.to_string(),
                used_fallback: false,
            }),
        }
    }
    
    /// Test connection with fallback support
    pub async fn test_connection_with_fallback(&mut self, name: &str) -> Result<TestResult> {
        let primary_result = self.test_connection(name).await?;
        
        if primary_result.success {
            return Ok(primary_result);
        }
        
        // Try fallback if configured
        let fallback_model = self.models.get(name)
            .and_then(|m| m.config.fallback_model.clone());
        
        if let Some(fallback) = fallback_model {
            info!("Primary model {} failed, trying fallback {}", name, fallback);
            let mut fallback_result = self.test_connection(&fallback).await?;
            fallback_result.used_fallback = true;
            return Ok(fallback_result);
        }
        
        Ok(primary_result)
    }
    
    /// Test all configured models
    pub async fn test_all_connections(&mut self) -> Result<HashMap<String, TestResult>> {
        let mut results = HashMap::new();
        
        let model_names: Vec<String> = self.models.iter()
            .map(|entry| entry.key().clone())
            .collect();
        
        for name in model_names {
            let result = self.test_connection(&name).await?;
            results.insert(name, result);
        }
        
        Ok(results)
    }
    
    /// Internal method to test a model
    async fn test_model_internal(&self, model: &AiModel) -> Result<()> {
        match model.config.provider.as_str() {
            "openai" => self.test_openai(model).await,
            "anthropic" => self.test_anthropic(model).await,
            "ollama" => self.test_ollama(model).await,
            _ => Err(anyhow::anyhow!("Unknown provider: {}", model.config.provider)),
        }
    }
    
    /// Stream completion from an AI model
    pub async fn stream_completion(&self, model_name: &str, prompt: &str) -> Result<StreamingResponseStream> {
        self.stream_completion_with_context(model_name, prompt, None).await
    }
    
    /// Stream completion with system context
    pub async fn stream_completion_with_context(
        &self,
        model_name: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> Result<StreamingResponseStream> {
        let model = self.models.get(model_name)
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_name))?
            .clone();
            
        match model.config.provider.as_str() {
            "anthropic" => self.stream_anthropic(&model, prompt, system_prompt).await,
            "openai" => self.stream_openai(&model, prompt, system_prompt).await,
            _ => Err(anyhow::anyhow!("Streaming not supported for provider: {}", model.config.provider)),
        }
    }
    
    async fn stream_anthropic(
        &self,
        model: &AiModel,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> Result<StreamingResponseStream> {
        let api_key = if let Some(env_var) = &model.config.api_key_env {
            std::env::var(env_var)
                .context(format!("API key not found in environment variable: {}", env_var))?
        } else {
            return Err(anyhow::anyhow!("No API key configured"));
        };
        
        let endpoint = model.config.endpoint.as_deref().unwrap_or("https://api.anthropic.com/v1");
        let url = format!("{}/messages", endpoint);
        
        let mut body = json!({
            "model": model.config.model_name,
            "messages": [{"role": "user", "content": prompt}],
            "stream": true
        });
        
        if let Some(system) = system_prompt {
            body["system"] = json!(system);
        }
        
        if let Some(max_tokens) = model.config.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }
        
        let response = self.client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Anthropic API error: {}", error_text));
        }
        
        // For now, collect the entire response and parse it
        // In a real implementation, we'd properly parse SSE chunks
        let body = response.text().await?;
        let mut responses = Vec::new();
        
        for line in body.lines() {
            if let Some(resp) = Self::parse_anthropic_line(line) {
                if let Ok(r) = resp {
                    responses.push(r);
                }
            }
        }
        
        let stream = stream::iter(responses.into_iter().map(Ok));
        
        Ok(StreamingResponseStream {
            inner: Box::pin(stream),
        })
    }
    
    async fn stream_openai(
        &self,
        model: &AiModel,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> Result<StreamingResponseStream> {
        let api_key = if let Some(env_var) = &model.config.api_key_env {
            std::env::var(env_var)
                .context(format!("API key not found in environment variable: {}", env_var))?
        } else {
            return Err(anyhow::anyhow!("No API key configured"));
        };
        
        let endpoint = model.config.endpoint.as_deref().unwrap_or("https://api.openai.com/v1");
        let url = format!("{}/chat/completions", endpoint);
        
        let mut messages = vec![];
        if let Some(system) = system_prompt {
            messages.push(json!({"role": "system", "content": system}));
        }
        messages.push(json!({"role": "user", "content": prompt}));
        
        let mut body = json!({
            "model": model.config.model_name,
            "messages": messages,
            "stream": true
        });
        
        if let Some(max_tokens) = model.config.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }
        
        if let Some(temperature) = model.config.temperature {
            body["temperature"] = json!(temperature);
        }
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }
        
        // For now, collect the entire response and parse it
        // In a real implementation, we'd properly parse SSE chunks
        let body = response.text().await?;
        let mut responses = Vec::new();
        
        for line in body.lines() {
            if let Some(resp) = Self::parse_openai_line(line) {
                if let Ok(r) = resp {
                    responses.push(r);
                }
            }
        }
        
        let stream = stream::iter(responses.into_iter().map(Ok));
        
        Ok(StreamingResponseStream {
            inner: Box::pin(stream),
        })
    }
    
    fn parse_anthropic_line(line: &str) -> Option<Result<StreamingResponse>> {
        if line.starts_with("data: ") {
            let json_str = &line[6..];
            if json_str.trim().is_empty() {
                return None;
            }
            
            match serde_json::from_str::<serde_json::Value>(json_str) {
                Ok(data) => {
                    if let Some(event_type) = data.get("type").and_then(|t| t.as_str()) {
                        match event_type {
                            "content_block_delta" => {
                                if let Some(text) = data.get("delta")
                                    .and_then(|d| d.get("text"))
                                    .and_then(|t| t.as_str()) {
                                    return Some(Ok(StreamingResponse {
                                        content: text.to_string(),
                                        is_final: false,
                                        token_count: None,
                                    }));
                                }
                            }
                            "message_stop" => {
                                return Some(Ok(StreamingResponse {
                                    content: String::new(),
                                    is_final: true,
                                    token_count: None,
                                }));
                            }
                            "error" => {
                                if let Some(error) = data.get("error") {
                                    let msg = error.get("message")
                                        .and_then(|m| m.as_str())
                                        .unwrap_or("Unknown error");
                                    return Some(Err(anyhow::anyhow!("Stream error: {}", msg)));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => return Some(Err(anyhow::anyhow!("JSON parse error: {}", e))),
            }
        }
        
        None
    }
    
    fn parse_openai_line(line: &str) -> Option<Result<StreamingResponse>> {
        if line.starts_with("data: ") {
            let json_str = &line[6..];
            if json_str.trim() == "[DONE]" {
                return Some(Ok(StreamingResponse {
                    content: String::new(),
                    is_final: true,
                    token_count: None,
                }));
            }
            
            if json_str.trim().is_empty() {
                return None;
            }
            
            match serde_json::from_str::<serde_json::Value>(json_str) {
                Ok(data) => {
                    if let Some(choices) = data.get("choices").and_then(|c| c.as_array()) {
                        if let Some(choice) = choices.first() {
                            if let Some(delta) = choice.get("delta") {
                                if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                    return Some(Ok(StreamingResponse {
                                        content: content.to_string(),
                                        is_final: false,
                                        token_count: None,
                                    }));
                                }
                            }
                        }
                    }
                }
                Err(e) => return Some(Err(anyhow::anyhow!("JSON parse error: {}", e))),
            }
        }
        
        None
    }
}