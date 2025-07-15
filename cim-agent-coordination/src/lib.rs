//! Multi-agent coordination service for the Composable Information Machine
//!
//! This crate provides the foundation for multi-agent collaboration, including:
//! - Agent registration and discovery
//! - Task assignment and delegation
//! - Distributed consensus mechanisms
//! - Agent presence and heartbeat tracking

pub mod registry;
pub mod coordinator;
pub mod discovery;
pub mod presence;
pub mod task;
pub mod consensus;

pub use registry::{AgentRegistry, AgentCapabilities, AgentId};
pub use coordinator::{TaskCoordinator, Assignment, AssignmentStatus, CoordinationError};
pub use discovery::{AgentDiscovery, DiscoveryEvent};
pub use presence::{AgentPresence, PresenceStatus};
pub use task::{CoordinationTask, TaskId, TaskStatus, TaskPriority};
pub use consensus::{ConsensusProtocol, Proposal, Vote, ConsensusResult};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        AgentRegistry, AgentCapabilities, AgentId,
        TaskCoordinator, Assignment, AssignmentStatus,
        CoordinationTask, TaskId, TaskStatus,
        ConsensusProtocol, Proposal, Vote, ConsensusResult,
    };
}
