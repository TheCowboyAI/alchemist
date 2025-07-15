//! Task definitions for agent coordination

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Unique identifier for a coordination task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl TaskId {
    /// Create a new task ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Priority levels for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    /// Lowest priority
    Low = 0,
    /// Normal priority (default)
    Normal = 1,
    /// High priority
    High = 2,
    /// Critical priority - must be handled immediately
    Critical = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Status of a coordination task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is waiting to be assigned
    Pending,
    /// Task has been assigned to an agent
    Assigned { agent_id: String },
    /// Task is currently being executed
    InProgress { 
        agent_id: String,
        started_at: DateTime<Utc>,
    },
    /// Task completed successfully
    Completed {
        agent_id: String,
        completed_at: DateTime<Utc>,
        result: serde_json::Value,
    },
    /// Task failed
    Failed {
        agent_id: String,
        failed_at: DateTime<Utc>,
        error: String,
    },
    /// Task was cancelled
    Cancelled {
        cancelled_at: DateTime<Utc>,
        reason: String,
    },
}

/// A task that can be coordinated between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationTask {
    /// Unique identifier for this task
    pub id: TaskId,
    
    /// Type of task (e.g., "deploy", "analyze", "transform")
    pub task_type: String,
    
    /// Required capability for an agent to handle this task
    pub required_capability: String,
    
    /// Optional additional capabilities that would be beneficial
    pub preferred_capabilities: Vec<String>,
    
    /// Task priority
    pub priority: TaskPriority,
    
    /// Task payload containing task-specific data
    pub payload: serde_json::Value,
    
    /// Metadata about the task
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// When the task was created
    pub created_at: DateTime<Utc>,
    
    /// Optional deadline for task completion
    pub deadline: Option<DateTime<Utc>>,
    
    /// Optional parent task ID for hierarchical tasks
    pub parent_task_id: Option<TaskId>,
    
    /// Optional correlation ID for related tasks
    pub correlation_id: Option<String>,
}

impl CoordinationTask {
    /// Create a new coordination task
    pub fn new(
        task_type: String,
        required_capability: String,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: TaskId::new(),
            task_type,
            required_capability,
            preferred_capabilities: Vec::new(),
            priority: TaskPriority::default(),
            payload,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            deadline: None,
            parent_task_id: None,
            correlation_id: None,
        }
    }
    
    /// Set the task priority
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set the task deadline
    pub fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }
    
    /// Add preferred capabilities
    pub fn with_preferred_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.preferred_capabilities = capabilities;
        self
    }
    
    /// Set parent task ID
    pub fn with_parent(mut self, parent_id: TaskId) -> Self {
        self.parent_task_id = Some(parent_id);
        self
    }
    
    /// Set correlation ID
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Check if task has expired
    pub fn is_expired(&self) -> bool {
        if let Some(deadline) = self.deadline {
            Utc::now() > deadline
        } else {
            false
        }
    }
    
    /// Calculate task score for agent matching
    pub fn calculate_score(&self, agent_capabilities: &[String]) -> u32 {
        let mut score = 0;
        
        // Must have required capability
        if !agent_capabilities.contains(&self.required_capability) {
            return 0;
        }
        
        // Base score for having required capability
        score += 100;
        
        // Additional points for each preferred capability
        for cap in &self.preferred_capabilities {
            if agent_capabilities.contains(cap) {
                score += 10;
            }
        }
        
        // Priority bonus
        score += match self.priority {
            TaskPriority::Low => 0,
            TaskPriority::Normal => 5,
            TaskPriority::High => 20,
            TaskPriority::Critical => 50,
        };
        
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_task_creation() {
        let task = CoordinationTask::new(
            "deploy".to_string(),
            "deployment".to_string(),
            json!({ "service": "web-api" }),
        );
        
        assert_eq!(task.task_type, "deploy");
        assert_eq!(task.required_capability, "deployment");
        assert_eq!(task.priority, TaskPriority::Normal);
    }
    
    #[test]
    fn test_task_scoring() {
        let task = CoordinationTask::new(
            "analyze".to_string(),
            "nlp".to_string(),
            json!({}),
        )
        .with_preferred_capabilities(vec!["ml".to_string(), "gpu".to_string()])
        .with_priority(TaskPriority::High);
        
        // Agent with only required capability
        let score1 = task.calculate_score(&["nlp".to_string()]);
        assert_eq!(score1, 120); // 100 base + 20 priority
        
        // Agent with required + one preferred
        let score2 = task.calculate_score(&["nlp".to_string(), "ml".to_string()]);
        assert_eq!(score2, 130); // 100 base + 20 priority + 10 preferred
        
        // Agent without required capability
        let score3 = task.calculate_score(&["ml".to_string(), "gpu".to_string()]);
        assert_eq!(score3, 0);
    }
}