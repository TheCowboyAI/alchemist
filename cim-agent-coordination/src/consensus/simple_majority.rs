//! Simple majority consensus implementation

use super::protocol::*;
use crate::registry::AgentId;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use chrono::Utc;

/// Simple majority consensus protocol
pub struct SimpleMajorityProtocol {
    proposal: Proposal,
    votes: Arc<RwLock<HashMap<AgentId, Vote>>>,
    participants: Vec<AgentId>,
    config: ConsensusConfig,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl SimpleMajorityProtocol {
    /// Create a new simple majority protocol
    pub fn new(proposal: Proposal, participants: Vec<AgentId>, config: ConsensusConfig) -> Self {
        Self {
            proposal,
            votes: Arc::new(RwLock::new(HashMap::new())),
            participants,
            config,
            started_at: None,
        }
    }
    
    /// Calculate the required votes for majority
    fn required_votes(&self) -> usize {
        self.participants.len() / 2 + 1
    }
    
    /// Check if consensus has been reached
    async fn check_consensus(&self) -> ConsensusResult {
        let votes = self.votes.read().await;
        
        let approve_count = votes.values().filter(|v| v.is_approval()).count();
        let reject_count = votes.values().filter(|v| v.is_rejection()).count();
        let total_votes = votes.len();
        
        let required = self.required_votes();
        
        // Collect conditions from conditional approvals
        let mut all_conditions = Vec::new();
        for vote in votes.values() {
            if let Vote::ApproveWithConditions { conditions } = vote {
                all_conditions.extend(conditions.clone());
            }
        }
        
        if approve_count >= required {
            if all_conditions.is_empty() {
                ConsensusResult::Approved
            } else {
                ConsensusResult::ApprovedWithConditions {
                    conditions: all_conditions,
                }
            }
        } else if reject_count >= required {
            ConsensusResult::Rejected
        } else if self.config.require_all_votes && total_votes < self.participants.len() {
            ConsensusResult::Pending
        } else if total_votes == self.participants.len() {
            // All votes cast but no majority
            ConsensusResult::NoConsensus
        } else {
            ConsensusResult::Pending
        }
    }
}

#[async_trait::async_trait]
impl ConsensusProtocol for SimpleMajorityProtocol {
    async fn start(&mut self) -> Result<(), ConsensusError> {
        if self.participants.len() < self.config.min_participants {
            return Err(ConsensusError::InsufficientParticipants {
                current: self.participants.len(),
                required: self.config.min_participants,
            });
        }
        
        self.started_at = Some(Utc::now());
        Ok(())
    }
    
    async fn cast_vote(&mut self, agent_id: AgentId, vote: Vote) -> Result<(), ConsensusError> {
        // Check if agent is authorized
        if !self.participants.contains(&agent_id) {
            return Err(ConsensusError::UnauthorizedVoter(agent_id));
        }
        
        // Check if voting period has ended
        if !self.is_voting_open().await {
            return Err(ConsensusError::VotingPeriodEnded);
        }
        
        let mut votes = self.votes.write().await;
        
        // Check for duplicate vote
        if !self.config.allow_vote_changes && votes.contains_key(&agent_id) {
            return Err(ConsensusError::DuplicateVote(agent_id));
        }
        
        votes.insert(agent_id, vote);
        Ok(())
    }
    
    async fn get_result(&self) -> Result<ConsensusResult, ConsensusError> {
        Ok(self.check_consensus().await)
    }
    
    async fn get_votes(&self) -> HashMap<AgentId, Vote> {
        self.votes.read().await.clone()
    }
    
    async fn is_voting_open(&self) -> bool {
        if let Some(started_at) = self.started_at {
            let elapsed = Utc::now().signed_duration_since(started_at);
            elapsed.num_seconds() < self.config.timeout_seconds as i64
                && self.proposal.deadline > Utc::now()
        } else {
            false
        }
    }
    
    async fn get_proposal(&self) -> &Proposal {
        &self.proposal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::TaskId;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_simple_majority() {
        let proposal = Proposal {
            id: ProposalId::new(),
            title: "Test Proposal".to_string(),
            description: "A test proposal".to_string(),
            proposed_by: AgentId::from_str("proposer"),
            created_at: Utc::now(),
            deadline: Utc::now() + chrono::Duration::hours(1),
            proposal_type: ProposalType::TaskAssignment,
            data: json!({}),
        };
        
        let participants = vec![
            AgentId::from_str("agent1"),
            AgentId::from_str("agent2"),
            AgentId::from_str("agent3"),
            AgentId::from_str("agent4"),
            AgentId::from_str("agent5"),
        ];
        
        let config = ConsensusConfig::default();
        let mut protocol = SimpleMajorityProtocol::new(proposal, participants, config);
        
        // Start consensus
        protocol.start().await.unwrap();
        
        // Cast votes
        protocol.cast_vote(AgentId::from_str("agent1"), Vote::Approve).await.unwrap();
        protocol.cast_vote(AgentId::from_str("agent2"), Vote::Approve).await.unwrap();
        protocol.cast_vote(AgentId::from_str("agent3"), Vote::Approve).await.unwrap();
        protocol.cast_vote(AgentId::from_str("agent4"), Vote::Reject).await.unwrap();
        
        // Check result
        let result = protocol.get_result().await.unwrap();
        assert_eq!(result, ConsensusResult::Approved);
    }
    
    #[tokio::test]
    async fn test_no_consensus() {
        let proposal = Proposal {
            id: ProposalId::new(),
            title: "Test Proposal".to_string(),
            description: "A test proposal".to_string(),
            proposed_by: AgentId::from_str("proposer"),
            created_at: Utc::now(),
            deadline: Utc::now() + chrono::Duration::hours(1),
            proposal_type: ProposalType::TaskAssignment,
            data: json!({}),
        };
        
        let participants = vec![
            AgentId::from_str("agent1"),
            AgentId::from_str("agent2"),
            AgentId::from_str("agent3"),
            AgentId::from_str("agent4"),
        ];
        
        let config = ConsensusConfig::default();
        let mut protocol = SimpleMajorityProtocol::new(proposal, participants, config);
        
        protocol.start().await.unwrap();
        
        // Split vote
        protocol.cast_vote(AgentId::from_str("agent1"), Vote::Approve).await.unwrap();
        protocol.cast_vote(AgentId::from_str("agent2"), Vote::Approve).await.unwrap();
        protocol.cast_vote(AgentId::from_str("agent3"), Vote::Reject).await.unwrap();
        protocol.cast_vote(AgentId::from_str("agent4"), Vote::Reject).await.unwrap();
        
        let result = protocol.get_result().await.unwrap();
        assert_eq!(result, ConsensusResult::NoConsensus);
    }
}