use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents the AI capabilities of an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AICapabilities {
    /// Unique identifier for this capability set
    pub id: Uuid,

    /// List of analysis capabilities
    pub capabilities: Vec<AnalysisCapability>,

    /// Model parameters for AI operations
    pub model_parameters: ModelParameters,

    /// Provider-specific configuration
    pub provider_config: HashMap<String, serde_json::Value>,
}

/// Types of analysis an agent can perform
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisCapability {
    /// Analyze graph structure and properties
    GraphAnalysis,

    /// Optimize workflows for efficiency
    WorkflowOptimization,

    /// Detect patterns in data or behavior
    PatternDetection,

    /// Analyze semantic meaning and relationships
    SemanticAnalysis,

    /// Suggest transformations for improvement
    TransformationSuggestion,

    /// Custom analysis with specific prompt
    Custom(String),
}

/// Parameters for AI model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParameters {
    /// Temperature for response generation (0.0 - 1.0)
    pub temperature: f32,

    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,

    /// Top-p sampling parameter
    pub top_p: Option<f32>,

    /// Frequency penalty
    pub frequency_penalty: Option<f32>,

    /// Presence penalty
    pub presence_penalty: Option<f32>,

    /// Additional model-specific parameters
    pub additional_params: HashMap<String, serde_json::Value>,
}

impl Default for AICapabilities {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            capabilities: vec![
                AnalysisCapability::GraphAnalysis,
                AnalysisCapability::WorkflowOptimization,
            ],
            model_parameters: ModelParameters::default(),
            provider_config: HashMap::new(),
        }
    }
}

impl Default for ModelParameters {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: Some(2000),
            top_p: Some(0.9),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
            additional_params: HashMap::new(),
        }
    }
}
