//! Core consensus protocol definitions

use crate::registry::AgentId;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a proposal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProposalId(pub Uuid);

impl ProposalId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ProposalId {
    fn default() -> Self {
        Self::new()
    }
}

/// A proposal that agents vote on
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: ProposalId,
    pub title: String,
    pub description: String,
    pub proposed_by: AgentId,
    pub created_at: DateTime<Utc>,
    pub deadline: DateTime<Utc>,
    pub proposal_type: ProposalType,
    pub data: serde_json::Value,
}

/// Types of proposals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    /// Deployment decision
    Deployment,
    /// Configuration change
    Configuration,
    /// Task assignment
    TaskAssignment,
    /// Knowledge validation
    KnowledgeValidation,
    /// Custom proposal type
    Custom(String),
}

/// A vote cast by an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Vote {
    /// Approve the proposal
    Approve,
    /// Reject the proposal
    Reject,
    /// Abstain from voting
    Abstain,
    /// Approve with conditions
    ApproveWithConditions { conditions: Vec<String> },
}

impl Vote {
    /// Check if this is an approval (with or without conditions)
    pub fn is_approval(&self) -> bool {
        matches!(self, Vote::Approve | Vote::ApproveWithConditions { .. })
    }
    
    /// Check if this is a rejection
    pub fn is_rejection(&self) -> bool {
        matches!(self, Vote::Reject)
    }
}

/// Result of a consensus process
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConsensusResult {
    /// Proposal approved
    Approved,
    /// Proposal rejected
    Rejected,
    /// Proposal approved with conditions
    ApprovedWithConditions { conditions: Vec<String> },
    /// No consensus reached
    NoConsensus,
    /// Consensus still pending
    Pending,
    /// Consensus failed due to timeout
    TimedOut,
}

/// Errors that can occur during consensus
#[derive(Debug, thiserror::Error)]
pub enum ConsensusError {
    #[error("Unauthorized voter: {0}")]
    UnauthorizedVoter(AgentId),
    
    #[error("Proposal not found: {0:?}")]
    ProposalNotFound(ProposalId),
    
    #[error("Voting period has ended")]
    VotingPeriodEnded,
    
    #[error("Duplicate vote from agent: {0}")]
    DuplicateVote(AgentId),
    
    #[error("Insufficient participants: {current}/{required}")]
    InsufficientParticipants { current: usize, required: usize },
    
    #[error("Consensus timeout")]
    Timeout,
}

/// Trait for consensus protocols
#[async_trait::async_trait]
pub trait ConsensusProtocol: Send + Sync {
    /// Start the consensus process
    async fn start(&mut self) -> Result<(), ConsensusError>;
    
    /// Cast a vote
    async fn cast_vote(&mut self, agent_id: AgentId, vote: Vote) -> Result<(), ConsensusError>;
    
    /// Get the current result
    async fn get_result(&self) -> Result<ConsensusResult, ConsensusError>;
    
    /// Get all votes cast so far
    async fn get_votes(&self) -> HashMap<AgentId, Vote>;
    
    /// Check if voting is still open
    async fn is_voting_open(&self) -> bool;
    
    /// Get the proposal being voted on
    async fn get_proposal(&self) -> &Proposal;
}

/// Configuration for consensus protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Minimum number of participants required
    pub min_participants: usize,
    
    /// Timeout for the consensus process
    pub timeout_seconds: u64,
    
    /// Whether to allow vote changes
    pub allow_vote_changes: bool,
    
    /// Whether to require all participants to vote
    pub require_all_votes: bool,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            min_participants: 3,
            timeout_seconds: 300, // 5 minutes
            allow_vote_changes: false,
            require_all_votes: false,
        }
    }
}