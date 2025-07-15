//! Agent registry for tracking available agents and their capabilities

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Unique identifier for an agent
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl AgentId {
    /// Create a new agent ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    /// Create from a string
    pub fn from_str(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Capabilities and metadata for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// Unique identifier for the agent
    pub id: AgentId,
    
    /// Human-readable name for the agent
    pub name: String,
    
    /// List of capabilities this agent provides
    pub capabilities: Vec<String>,
    
    /// Maximum number of concurrent tasks this agent can handle
    pub max_concurrent_tasks: usize,
    
    /// Current number of active tasks
    pub active_tasks: usize,
    
    /// Agent metadata
    pub metadata: serde_json::Value,
    
    /// When the agent was registered
    pub registered_at: DateTime<Utc>,
    
    /// Last heartbeat received
    pub last_heartbeat: DateTime<Utc>,
    
    /// Agent version information
    pub version: String,
    
    /// NATS subject for direct communication with this agent
    pub nats_subject: String,
}

impl AgentCapabilities {
    /// Create new agent capabilities
    pub fn new(
        id: AgentId,
        name: String,
        capabilities: Vec<String>,
        nats_subject: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            capabilities,
            max_concurrent_tasks: 5, // Default
            active_tasks: 0,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            registered_at: now,
            last_heartbeat: now,
            version: "1.0.0".to_string(),
            nats_subject,
        }
    }
    
    /// Check if agent has a specific capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }
    
    /// Check if agent can accept more tasks
    pub fn can_accept_tasks(&self) -> bool {
        self.active_tasks < self.max_concurrent_tasks
    }
    
    /// Calculate agent load percentage
    pub fn load_percentage(&self) -> f32 {
        if self.max_concurrent_tasks == 0 {
            return 0.0;
        }
        (self.active_tasks as f32 / self.max_concurrent_tasks as f32) * 100.0
    }
}

/// Registry for tracking all available agents
pub struct AgentRegistry {
    /// All registered agents
    agents: Arc<DashMap<AgentId, AgentCapabilities>>,
    
    /// Index of capabilities to agents
    capability_index: Arc<DashMap<String, Vec<AgentId>>>,
}

impl AgentRegistry {
    /// Create a new agent registry
    pub fn new() -> Self {
        Self {
            agents: Arc::new(DashMap::new()),
            capability_index: Arc::new(DashMap::new()),
        }
    }
    
    /// Register a new agent
    pub fn register_agent(&self, agent: AgentCapabilities) -> Result<(), RegistryError> {
        let agent_id = agent.id.clone();
        
        // Check if agent already exists
        if self.agents.contains_key(&agent_id) {
            return Err(RegistryError::AgentAlreadyRegistered(agent_id));
        }
        
        // Update capability index
        for capability in &agent.capabilities {
            self.capability_index
                .entry(capability.clone())
                .or_default()
                .push(agent_id.clone());
        }
        
        // Store agent
        self.agents.insert(agent_id, agent);
        
        Ok(())
    }
    
    /// Unregister an agent
    pub fn unregister_agent(&self, agent_id: &AgentId) -> Result<(), RegistryError> {
        // Remove from agents map
        let agent = self.agents.remove(agent_id)
            .ok_or_else(|| RegistryError::AgentNotFound(agent_id.clone()))?;
        
        // Remove from capability index
        for capability in &agent.1.capabilities {
            if let Some(mut agents) = self.capability_index.get_mut(capability) {
                agents.retain(|id| id != agent_id);
            }
        }
        
        Ok(())
    }
    
    /// Update agent heartbeat
    pub fn update_heartbeat(&self, agent_id: &AgentId) -> Result<(), RegistryError> {
        self.agents
            .get_mut(agent_id)
            .ok_or_else(|| RegistryError::AgentNotFound(agent_id.clone()))?
            .last_heartbeat = Utc::now();
        
        Ok(())
    }
    
    /// Update agent task count
    pub fn update_task_count(&self, agent_id: &AgentId, active_tasks: usize) -> Result<(), RegistryError> {
        self.agents
            .get_mut(agent_id)
            .ok_or_else(|| RegistryError::AgentNotFound(agent_id.clone()))?
            .active_tasks = active_tasks;
        
        Ok(())
    }
    
    /// Get agent by ID
    pub fn get_agent(&self, agent_id: &AgentId) -> Option<AgentCapabilities> {
        self.agents.get(agent_id).map(|entry| entry.clone())
    }
    
    /// Find agents with a specific capability
    pub fn find_agents_with_capability(&self, capability: &str) -> Vec<AgentCapabilities> {
        self.capability_index
            .get(capability)
            .map(|agents| {
                agents.iter()
                    .filter_map(|id| self.get_agent(id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Find available agents with a specific capability
    pub fn find_available_agents_with_capability(&self, capability: &str) -> Vec<AgentCapabilities> {
        self.find_agents_with_capability(capability)
            .into_iter()
            .filter(|agent| agent.can_accept_tasks())
            .collect()
    }
    
    /// Get all agents
    pub fn get_all_agents(&self) -> Vec<AgentCapabilities> {
        self.agents.iter().map(|entry| entry.value().clone()).collect()
    }
    
    /// Get agents that haven't sent heartbeat in given duration
    pub fn get_stale_agents(&self, stale_after_seconds: i64) -> Vec<AgentCapabilities> {
        let cutoff = Utc::now() - chrono::Duration::seconds(stale_after_seconds);
        
        self.agents
            .iter()
            .filter(|entry| entry.last_heartbeat < cutoff)
            .map(|entry| entry.value().clone())
            .collect()
    }
    
    /// Remove stale agents
    pub fn cleanup_stale_agents(&self, stale_after_seconds: i64) -> Vec<AgentId> {
        let stale_agents = self.get_stale_agents(stale_after_seconds);
        let mut removed = Vec::new();
        
        for agent in stale_agents {
            if self.unregister_agent(&agent.id).is_ok() {
                removed.push(agent.id);
            }
        }
        
        removed
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur in the agent registry
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Agent {0} is already registered")]
    AgentAlreadyRegistered(AgentId),
    
    #[error("Agent {0} not found")]
    AgentNotFound(AgentId),
    
    #[error("No agents available with capability: {0}")]
    NoAgentsWithCapability(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_registration() {
        let registry = AgentRegistry::new();
        
        let agent = AgentCapabilities::new(
            AgentId::from_str("agent-1"),
            "Test Agent".to_string(),
            vec!["deploy".to_string(), "monitor".to_string()],
            "agent.1".to_string(),
        );
        
        // Register agent
        assert!(registry.register_agent(agent.clone()).is_ok());
        
        // Try to register again
        assert!(matches!(
            registry.register_agent(agent),
            Err(RegistryError::AgentAlreadyRegistered(_))
        ));
    }
    
    #[test]
    fn test_capability_search() {
        let registry = AgentRegistry::new();
        
        // Register agents with different capabilities
        let agent1 = AgentCapabilities::new(
            AgentId::from_str("agent-1"),
            "Deploy Agent".to_string(),
            vec!["deploy".to_string()],
            "agent.1".to_string(),
        );
        
        let agent2 = AgentCapabilities::new(
            AgentId::from_str("agent-2"),
            "Monitor Agent".to_string(),
            vec!["monitor".to_string()],
            "agent.2".to_string(),
        );
        
        let agent3 = AgentCapabilities::new(
            AgentId::from_str("agent-3"),
            "Multi Agent".to_string(),
            vec!["deploy".to_string(), "monitor".to_string()],
            "agent.3".to_string(),
        );
        
        registry.register_agent(agent1).unwrap();
        registry.register_agent(agent2).unwrap();
        registry.register_agent(agent3).unwrap();
        
        // Search for deploy capability
        let deploy_agents = registry.find_agents_with_capability("deploy");
        assert_eq!(deploy_agents.len(), 2);
        
        // Search for monitor capability
        let monitor_agents = registry.find_agents_with_capability("monitor");
        assert_eq!(monitor_agents.len(), 2);
    }
}