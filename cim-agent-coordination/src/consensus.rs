//! Consensus mechanisms for multi-agent decision making

pub mod protocol;
pub mod simple_majority;
pub mod qualified_majority;
pub mod weighted_voting;

pub use protocol::{ConsensusProtocol, Proposal, Vote, ConsensusResult, ConsensusError};
pub use simple_majority::SimpleMajorityProtocol;
pub use qualified_majority::QualifiedMajorityProtocol;
pub use weighted_voting::WeightedVotingProtocol;