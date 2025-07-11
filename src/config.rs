//! Configuration management for Alchemist

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::collections::HashMap;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlchemistConfig {
    /// General settings
    pub general: GeneralConfig,
    
    /// AI model configurations
    pub ai_models: HashMap<String, AiModelConfig>,
    
    /// Policy settings
    pub policy: PolicyConfig,
    
    /// Deployment targets
    pub deployments: HashMap<String, DeploymentConfig>,
    
    /// Domain registry
    pub domains: DomainRegistryConfig,
    
    /// Cache configuration
    pub cache: Option<CacheConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Redis URL for caching
    pub redis_url: Option<String>,
    
    /// Default TTL in seconds
    pub default_ttl: u64,
    
    /// Maximum memory cache size
    pub max_memory_items: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Default AI model
    pub default_ai_model: Option<String>,
    
    /// Dialog history location
    pub dialog_history_path: String,
    
    /// Progress file location
    pub progress_file_path: String,
    
    /// NATS connection URL
    pub nats_url: Option<String>,
    
    /// Log level
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModelConfig {
    /// Provider type (openai, anthropic, ollama)
    pub provider: String,
    
    /// API endpoint
    pub endpoint: Option<String>,
    
    /// API key (will be loaded from env var)
    pub api_key_env: Option<String>,
    
    /// Model name (e.g., gpt-4, claude-3)
    pub model_name: String,
    
    /// Max tokens
    pub max_tokens: Option<u32>,
    
    /// Temperature
    pub temperature: Option<f32>,
    
    /// Timeout in seconds
    pub timeout_seconds: Option<u32>,
    
    /// Rate limit (requests per minute)
    pub rate_limit: Option<u32>,
    
    /// Fallback model name
    pub fallback_model: Option<String>,
    
    /// Custom parameters
    pub params: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Policy storage path
    pub storage_path: String,
    
    /// Enable policy validation
    pub validation_enabled: bool,
    
    /// Policy evaluation timeout (ms)
    pub evaluation_timeout: u64,
    
    /// Cache TTL in seconds
    pub cache_ttl: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    /// CIM instance name
    pub name: String,
    
    /// NATS URL for this CIM
    pub nats_url: String,
    
    /// JetStream configuration
    pub jetstream: JetStreamConfig,
    
    /// Deployed domains
    pub domains: Vec<String>,
    
    /// Environment (dev, staging, prod)
    pub environment: String,
    
    /// Custom service configurations
    pub services: Option<HashMap<String, String>>,
    
    /// Custom agent configurations
    pub agents: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JetStreamConfig {
    /// Stream name prefix
    pub stream_prefix: String,
    
    /// Retention policy
    pub retention_days: u32,
    
    /// Max message size
    pub max_msg_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRegistryConfig {
    /// Available domains
    pub available: Vec<DomainConfig>,
    
    /// Domain relationships
    pub relationships: Vec<DomainRelationship>,
}

impl Default for DomainRegistryConfig {
    fn default() -> Self {
        Self {
            available: vec![],
            relationships: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    /// Domain name
    pub name: String,
    
    /// Domain description
    pub description: String,
    
    /// Module path
    pub module_path: String,
    
    /// Enabled by default
    pub enabled: bool,
    
    /// Dependencies
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRelationship {
    /// Source domain
    pub source: String,
    
    /// Target domain
    pub target: String,
    
    /// Relationship type
    pub relationship_type: String,
    
    /// Bidirectional
    pub bidirectional: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_ai_model: None,
            dialog_history_path: "~/.alchemist/dialogs".to_string(),
            progress_file_path: "doc/progress/progress.json".to_string(),
            nats_url: Some("nats://localhost:4222".to_string()),
            log_level: "info".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service name
    pub name: String,
    
    /// Service executable
    pub executable: String,
    
    /// Service arguments
    pub args: Vec<String>,
    
    /// Service port
    pub port: Option<u16>,
    
    /// Environment variables
    pub environment: HashMap<String, String>,
    
    /// Resource limits
    pub resources: ResourceLimits,
    
    /// Health check configuration
    pub health_check: Option<HealthCheckConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent name
    pub name: String,
    
    /// Agent type
    pub agent_type: String,
    
    /// Capabilities
    pub capabilities: Vec<String>,
    
    /// Configuration
    pub config: HashMap<String, serde_json::Value>,
    
    /// Environment variables
    pub environment: HashMap<String, String>,
    
    /// Resource limits
    pub resources: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// CPU limit (cores)
    pub cpu: Option<f32>,
    
    /// Memory limit (MB)
    pub memory: Option<u32>,
    
    /// Disk limit (MB)
    pub disk: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check endpoint
    pub endpoint: String,
    
    /// Check interval (seconds)
    pub interval: u32,
    
    /// Timeout (seconds)
    pub timeout: u32,
    
    /// Failure threshold
    pub failure_threshold: u32,
}

impl Default for AlchemistConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            ai_models: HashMap::new(),
            policy: PolicyConfig {
                storage_path: "~/.alchemist/policies".to_string(),
                validation_enabled: true,
                evaluation_timeout: 5000,
                cache_ttl: Some(300),
            },
            deployments: HashMap::new(),
            domains: DomainRegistryConfig {
                available: vec![
                    DomainConfig {
                        name: "graph".to_string(),
                        description: "Core graph operations and spatial positioning".to_string(),
                        module_path: "cim-domain-graph".to_string(),
                        enabled: true,
                        dependencies: vec![],
                    },
                    DomainConfig {
                        name: "workflow".to_string(),
                        description: "Business process execution and state machines".to_string(),
                        module_path: "cim-domain-workflow".to_string(),
                        enabled: true,
                        dependencies: vec!["graph".to_string()],
                    },
                    DomainConfig {
                        name: "agent".to_string(),
                        description: "AI provider integration and semantic search".to_string(),
                        module_path: "cim-domain-agent".to_string(),
                        enabled: true,
                        dependencies: vec!["graph".to_string()],
                    },
                    DomainConfig {
                        name: "document".to_string(),
                        description: "Document lifecycle and version control".to_string(),
                        module_path: "cim-domain-document".to_string(),
                        enabled: true,
                        dependencies: vec![],
                    },
                    DomainConfig {
                        name: "policy".to_string(),
                        description: "Business rule enforcement".to_string(),
                        module_path: "cim-domain-policy".to_string(),
                        enabled: true,
                        dependencies: vec![],
                    },
                ],
                relationships: vec![
                    DomainRelationship {
                        source: "document".to_string(),
                        target: "workflow".to_string(),
                        relationship_type: "triggers".to_string(),
                        bidirectional: false,
                    },
                    DomainRelationship {
                        source: "agent".to_string(),
                        target: "graph".to_string(),
                        relationship_type: "analyzes".to_string(),
                        bidirectional: false,
                    },
                ],
            },
            cache: Some(CacheConfig {
                redis_url: Some("redis://localhost:6379".to_string()),
                default_ttl: 3600,
                max_memory_items: 10000,
            }),
        }
    }
}

/// Load configuration from file or create default
pub async fn load_or_create(path: &str) -> Result<AlchemistConfig> {
    let path = Path::new(path);
    
    if path.exists() {
        let content = fs::read_to_string(path).await?;
        let config: AlchemistConfig = toml::from_str(&content)?;
        Ok(config)
    } else {
        // Create default config
        let config = AlchemistConfig::default();
        
        // Add some example AI models
        let mut ai_models = HashMap::new();
        
        ai_models.insert("gpt-4".to_string(), AiModelConfig {
            provider: "openai".to_string(),
            endpoint: Some("https://api.openai.com/v1".to_string()),
            api_key_env: Some("OPENAI_API_KEY".to_string()),
            model_name: "gpt-4-turbo-preview".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            timeout_seconds: Some(30),
            rate_limit: None,
            fallback_model: None,
            params: HashMap::new(),
        });
        
        ai_models.insert("claude-3".to_string(), AiModelConfig {
            provider: "anthropic".to_string(),
            endpoint: Some("https://api.anthropic.com/v1".to_string()),
            api_key_env: Some("ANTHROPIC_API_KEY".to_string()),
            model_name: "claude-3-opus-20240229".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            timeout_seconds: Some(30),
            rate_limit: None,
            fallback_model: Some("gpt-4".to_string()),
            params: HashMap::new(),
        });
        
        ai_models.insert("local-llama".to_string(), AiModelConfig {
            provider: "ollama".to_string(),
            endpoint: Some("http://localhost:11434".to_string()),
            api_key_env: None,
            model_name: "llama2".to_string(),
            max_tokens: Some(2048),
            temperature: Some(0.8),
            timeout_seconds: Some(60),
            rate_limit: None,
            fallback_model: None,
            params: HashMap::new(),
        });
        
        let mut config = config;
        config.ai_models = ai_models;
        config.general.default_ai_model = Some("claude-3".to_string());
        
        // Save default config
        save_config(path, &config).await?;
        
        Ok(config)
    }
}

/// Save configuration to file
pub async fn save_config(path: &Path, config: &AlchemistConfig) -> Result<()> {
    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    
    let content = toml::to_string_pretty(config)?;
    fs::write(path, content).await?;
    
    Ok(())
}