use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Priority level for recommendations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority - requires immediate attention
    Critical,
}

/// Impact level of changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Impact {
    /// Low impact - minimal effect
    Low,
    /// Medium impact - moderate effect
    Medium,
    /// High impact - significant effect
    High,
}

/// Effort level required
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffortLevel {
    /// Low effort - minimal work required
    Low,
    /// Medium effort - moderate work required
    Medium,
    /// High effort - significant work required
    High,
}

/// Represents a recommendation from AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Unique identifier for the recommendation
    pub id: Uuid,
    /// Title of the recommendation
    pub title: String,
    /// Detailed description of the recommendation
    pub description: String,
    /// Priority level of the recommendation
    pub priority: Priority,
    /// Expected impact description
    pub expected_impact: String,
    /// Effort level required to implement
    pub effort_level: EffortLevel,
    /// List of actions to implement the recommendation
    pub actions: Vec<RecommendedAction>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Represents a specific action within a recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    /// Unique identifier for the action
    pub id: Uuid,
    /// Type of action to perform
    pub action_type: String,
    /// Target of the action
    pub target: String,
    /// Description of the action
    pub description: String,
    /// Estimated time to complete the action
    pub estimated_duration: std::time::Duration,
    /// Parameters for the action
    pub parameters: HashMap<String, serde_json::Value>,
    /// IDs of actions that must be completed first
    pub dependencies: Vec<Uuid>,
}

/// Result of AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Unique identifier for the analysis
    pub id: Uuid,
    /// Confidence score (0.0 to 1.0)
    pub confidence_score: f32,
    /// Summary of the analysis findings
    pub summary: String,
    /// List of recommendations from the analysis
    pub recommendations: Vec<Recommendation>,
    /// List of insights discovered
    pub insights: Vec<Insight>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// When the analysis was performed
    pub timestamp: std::time::SystemTime,
}

/// Represents an insight from analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    /// Unique identifier for the insight
    pub id: Uuid,
    /// Category of the insight
    pub category: String,
    /// Description of the insight
    pub description: String,
    /// Supporting evidence for the insight
    pub evidence: Vec<String>,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    /// Potential impact of the insight
    pub impact: Impact,
}

impl Default for AnalysisResult {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            confidence_score: 0.0,
            summary: String::new(),
            recommendations: Vec::new(),
            insights: Vec::new(),
            metadata: HashMap::new(),
            timestamp: std::time::SystemTime::now(),
        }
    }
}

impl Default for Recommendation {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            title: String::new(),
            description: String::new(),
            priority: Priority::Medium,
            expected_impact: String::new(),
            effort_level: EffortLevel::Medium,
            actions: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

impl Default for RecommendedAction {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            action_type: String::new(),
            target: String::new(),
            description: String::new(),
            estimated_duration: std::time::Duration::from_secs(3600),
            parameters: HashMap::new(),
            dependencies: Vec::new(),
        }
    }
} 