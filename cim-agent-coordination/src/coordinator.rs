//! Task coordination and assignment logic

use crate::{
    registry::{AgentRegistry, AgentId, AgentCapabilities},
    task::{CoordinationTask, TaskId, TaskStatus},
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use async_nats::Client as NatsClient;
use anyhow::Result;

/// Assignment of a task to an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub task_id: TaskId,
    pub agent_id: AgentId,
    pub assigned_at: DateTime<Utc>,
    pub status: AssignmentStatus,
}

/// Status of a task assignment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentStatus {
    /// Task has been assigned but not yet acknowledged
    Pending,
    /// Agent has acknowledged the assignment
    Acknowledged,
    /// Task is being executed
    InProgress,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed(String),
    /// Assignment timed out
    TimedOut,
}

/// Errors that can occur during coordination
#[derive(Debug, thiserror::Error)]
pub enum CoordinationError {
    #[error("No capable agents found for task requiring: {0}")]
    NoCapableAgents(String),
    
    #[error("All capable agents are at capacity")]
    AllAgentsAtCapacity,
    
    #[error("Task {0} not found")]
    TaskNotFound(TaskId),
    
    #[error("Agent {0} not found")]
    AgentNotFound(AgentId),
    
    #[error("Task {0} is already assigned")]
    TaskAlreadyAssigned(TaskId),
    
    #[error("Failed to communicate with agent: {0}")]
    CommunicationError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Strategy for routing tasks to agents
#[async_trait::async_trait]
pub trait RoutingStrategy: Send + Sync {
    /// Select an agent from available agents for a task
    async fn select_agent(
        &self,
        task: &CoordinationTask,
        available_agents: &[AgentCapabilities],
    ) -> Option<AgentId>;
}

/// Round-robin routing strategy
pub struct RoundRobinStrategy {
    last_index: Arc<RwLock<usize>>,
}

impl RoundRobinStrategy {
    pub fn new() -> Self {
        Self {
            last_index: Arc::new(RwLock::new(0)),
        }
    }
}

#[async_trait::async_trait]
impl RoutingStrategy for RoundRobinStrategy {
    async fn select_agent(
        &self,
        _task: &CoordinationTask,
        available_agents: &[AgentCapabilities],
    ) -> Option<AgentId> {
        if available_agents.is_empty() {
            return None;
        }
        
        let mut index = self.last_index.write().await;
        *index = (*index + 1) % available_agents.len();
        Some(available_agents[*index].id.clone())
    }
}

/// Load-based routing strategy
pub struct LoadBasedStrategy;

#[async_trait::async_trait]
impl RoutingStrategy for LoadBasedStrategy {
    async fn select_agent(
        &self,
        _task: &CoordinationTask,
        available_agents: &[AgentCapabilities],
    ) -> Option<AgentId> {
        available_agents
            .iter()
            .min_by_key(|agent| agent.active_tasks)
            .map(|agent| agent.id.clone())
    }
}

/// Capability-score based routing strategy
pub struct CapabilityScoreStrategy;

#[async_trait::async_trait]
impl RoutingStrategy for CapabilityScoreStrategy {
    async fn select_agent(
        &self,
        task: &CoordinationTask,
        available_agents: &[AgentCapabilities],
    ) -> Option<AgentId> {
        available_agents
            .iter()
            .map(|agent| (agent, task.calculate_score(&agent.capabilities)))
            .filter(|(_, score)| *score > 0)
            .max_by_key(|(_, score)| *score)
            .map(|(agent, _)| agent.id.clone())
    }
}

/// Task coordinator for managing task assignments
pub struct TaskCoordinator {
    /// Pending tasks waiting for assignment
    pending_tasks: Arc<RwLock<Vec<CoordinationTask>>>,
    
    /// Active task assignments
    assignments: Arc<DashMap<TaskId, Assignment>>,
    
    /// Agent registry
    registry: Arc<AgentRegistry>,
    
    /// Task routing strategies
    strategies: Vec<Box<dyn RoutingStrategy>>,
    
    /// NATS client for communication
    nats_client: Option<NatsClient>,
}

impl TaskCoordinator {
    /// Create a new task coordinator
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            pending_tasks: Arc::new(RwLock::new(Vec::new())),
            assignments: Arc::new(DashMap::new()),
            registry,
            strategies: vec![
                Box::new(CapabilityScoreStrategy),
                Box::new(LoadBasedStrategy),
                Box::new(RoundRobinStrategy::new()),
            ],
            nats_client: None,
        }
    }
    
    /// Set NATS client for agent communication
    pub fn with_nats_client(mut self, client: NatsClient) -> Self {
        self.nats_client = Some(client);
        self
    }
    
    /// Submit a task for coordination
    pub async fn submit_task(&self, task: CoordinationTask) -> Result<TaskId, CoordinationError> {
        // Validate task
        self.validate_task(&task)?;
        
        let task_id = task.id;
        
        // Try immediate assignment
        match self.try_assign_task(&task).await {
            Ok(agent_id) => {
                // Create assignment
                let assignment = Assignment {
                    task_id,
                    agent_id: agent_id.clone(),
                    assigned_at: Utc::now(),
                    status: AssignmentStatus::Pending,
                };
                
                self.assignments.insert(task_id, assignment);
                
                // Notify agent
                if let Err(e) = self.notify_agent(&agent_id, &task).await {
                    // Rollback assignment on notification failure
                    self.assignments.remove(&task_id);
                    return Err(CoordinationError::CommunicationError(e.to_string()));
                }
                
                Ok(task_id)
            }
            Err(_) => {
                // Queue task for later assignment
                self.pending_tasks.write().await.push(task);
                Ok(task_id)
            }
        }
    }
    
    /// Try to assign a task to an available agent
    async fn try_assign_task(&self, task: &CoordinationTask) -> Result<AgentId, CoordinationError> {
        // Find capable agents
        let capable_agents = self.registry
            .find_available_agents_with_capability(&task.required_capability);
        
        if capable_agents.is_empty() {
            return Err(CoordinationError::NoCapableAgents(task.required_capability.clone()));
        }
        
        // Try each strategy in order
        for strategy in &self.strategies {
            if let Some(agent_id) = strategy.select_agent(task, &capable_agents).await {
                // Update agent task count
                if let Some(mut agent) = self.registry.agents.get_mut(&agent_id) {
                    agent.active_tasks += 1;
                }
                return Ok(agent_id);
            }
        }
        
        Err(CoordinationError::AllAgentsAtCapacity)
    }
    
    /// Validate a task before processing
    fn validate_task(&self, task: &CoordinationTask) -> Result<(), CoordinationError> {
        // Check if task is already assigned
        if self.assignments.contains_key(&task.id) {
            return Err(CoordinationError::TaskAlreadyAssigned(task.id));
        }
        
        // Check if task has expired
        if task.is_expired() {
            return Err(CoordinationError::InternalError("Task has expired".to_string()));
        }
        
        Ok(())
    }
    
    /// Notify an agent about a task assignment
    async fn notify_agent(&self, agent_id: &AgentId, task: &CoordinationTask) -> Result<()> {
        if let Some(client) = &self.nats_client {
            if let Some(agent) = self.registry.get_agent(agent_id) {
                let message = serde_json::json!({
                    "type": "task_assignment",
                    "task": task,
                    "assigned_at": Utc::now(),
                });
                
                client
                    .publish(&agent.nats_subject, serde_json::to_vec(&message)?.into())
                    .await?;
                
                Ok(())
            } else {
                Err(anyhow::anyhow!("Agent not found"))
            }
        } else {
            // In test mode, just succeed
            Ok(())
        }
    }
    
    /// Get task status
    pub fn get_task_status(&self, task_id: &TaskId) -> Option<AssignmentStatus> {
        self.assignments
            .get(task_id)
            .map(|assignment| assignment.status.clone())
    }
    
    /// Update task status
    pub fn update_task_status(&self, task_id: &TaskId, status: AssignmentStatus) -> Result<(), CoordinationError> {
        self.assignments
            .get_mut(task_id)
            .ok_or(CoordinationError::TaskNotFound(*task_id))?
            .status = status;
        
        Ok(())
    }
    
    /// Complete a task
    pub async fn complete_task(&self, task_id: &TaskId) -> Result<(), CoordinationError> {
        let assignment = self.assignments
            .get(task_id)
            .ok_or(CoordinationError::TaskNotFound(*task_id))?;
        
        let agent_id = assignment.agent_id.clone();
        
        // Update assignment status
        self.update_task_status(task_id, AssignmentStatus::Completed)?;
        
        // Decrease agent task count
        if let Some(mut agent) = self.registry.agents.get_mut(&agent_id) {
            agent.active_tasks = agent.active_tasks.saturating_sub(1);
        }
        
        // Try to assign pending tasks
        self.process_pending_tasks().await;
        
        Ok(())
    }
    
    /// Process pending tasks
    async fn process_pending_tasks(&self) {
        let mut pending = self.pending_tasks.write().await;
        let mut remaining = Vec::new();
        
        for task in pending.drain(..) {
            match self.try_assign_task(&task).await {
                Ok(agent_id) => {
                    let assignment = Assignment {
                        task_id: task.id,
                        agent_id: agent_id.clone(),
                        assigned_at: Utc::now(),
                        status: AssignmentStatus::Pending,
                    };
                    
                    self.assignments.insert(task.id, assignment);
                    
                    // Best effort notification
                    let _ = self.notify_agent(&agent_id, &task).await;
                }
                Err(_) => {
                    remaining.push(task);
                }
            }
        }
        
        *pending = remaining;
    }
    
    /// Get all pending tasks
    pub async fn get_pending_tasks(&self) -> Vec<CoordinationTask> {
        self.pending_tasks.read().await.clone()
    }
    
    /// Get all assignments
    pub fn get_all_assignments(&self) -> Vec<Assignment> {
        self.assignments.iter().map(|entry| entry.value().clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::AgentCapabilities;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_task_assignment() {
        let registry = Arc::new(AgentRegistry::new());
        let coordinator = TaskCoordinator::new(registry.clone());
        
        // Register an agent
        let agent = AgentCapabilities::new(
            AgentId::from_str("agent-1"),
            "Test Agent".to_string(),
            vec!["deploy".to_string()],
            "agent.1".to_string(),
        );
        registry.register_agent(agent).unwrap();
        
        // Submit a task
        let task = CoordinationTask::new(
            "deployment".to_string(),
            "deploy".to_string(),
            json!({ "service": "web-api" }),
        );
        
        let task_id = coordinator.submit_task(task).await.unwrap();
        
        // Check assignment
        let status = coordinator.get_task_status(&task_id);
        assert!(matches!(status, Some(AssignmentStatus::Pending)));
    }
    
    #[tokio::test]
    async fn test_no_capable_agents() {
        let registry = Arc::new(AgentRegistry::new());
        let coordinator = TaskCoordinator::new(registry);
        
        // Submit a task with no capable agents
        let task = CoordinationTask::new(
            "deployment".to_string(),
            "deploy".to_string(),
            json!({ "service": "web-api" }),
        );
        
        let result = coordinator.submit_task(task).await;
        assert!(result.is_ok()); // Task should be queued
        
        // Check pending tasks
        let pending = coordinator.get_pending_tasks().await;
        assert_eq!(pending.len(), 1);
    }
}