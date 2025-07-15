//! Agent presence and heartbeat tracking

use crate::registry::AgentId;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::time::Duration;

/// Agent presence status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresenceStatus {
    /// Agent is online and responsive
    Online,
    /// Agent hasn't sent heartbeat recently
    Away,
    /// Agent is offline or unresponsive
    Offline,
}

/// Agent presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPresence {
    pub agent_id: AgentId,
    pub status: PresenceStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub latency_ms: Option<u32>,
}

impl AgentPresence {
    /// Create new presence info
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            status: PresenceStatus::Online,
            last_heartbeat: Utc::now(),
            latency_ms: None,
        }
    }
    
    /// Update heartbeat timestamp
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
        self.status = PresenceStatus::Online;
    }
    
    /// Update status based on time since last heartbeat
    pub fn update_status(&mut self, online_threshold: Duration, away_threshold: Duration) {
        let elapsed = Utc::now().signed_duration_since(self.last_heartbeat);
        
        if elapsed < chrono::Duration::from_std(online_threshold).unwrap() {
            self.status = PresenceStatus::Online;
        } else if elapsed < chrono::Duration::from_std(away_threshold).unwrap() {
            self.status = PresenceStatus::Away;
        } else {
            self.status = PresenceStatus::Offline;
        }
    }
    
    /// Check if agent is considered active
    pub fn is_active(&self) -> bool {
        matches!(self.status, PresenceStatus::Online | PresenceStatus::Away)
    }
}

/// Heartbeat message sent by agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub agent_id: AgentId,
    pub timestamp: DateTime<Utc>,
    pub active_tasks: usize,
    pub cpu_usage: Option<f32>,
    pub memory_usage: Option<f32>,
}

impl Heartbeat {
    /// Create a new heartbeat
    pub fn new(agent_id: AgentId, active_tasks: usize) -> Self {
        Self {
            agent_id,
            timestamp: Utc::now(),
            active_tasks,
            cpu_usage: None,
            memory_usage: None,
        }
    }
}