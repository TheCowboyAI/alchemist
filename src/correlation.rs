use std::fmt::{self, Display};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// Re-export from cim-ipld for CID support
use cim_ipld::Cid;

/// Wrapper for CID that implements Serialize/Deserialize
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SerializableCid(pub Cid);

impl Serialize for SerializableCid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize as string
        self.0.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SerializableCid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize from string
        let s = String::deserialize(deserializer)?;
        let cid = s.parse::<Cid>()
            .map_err(serde::de::Error::custom)?;
        Ok(SerializableCid(cid))
    }
}

impl Display for SerializableCid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Errors that can occur in correlation/causation operations
#[derive(Debug, Error)]
pub enum CorrelationError {
    /// Attempted to create a message without proper correlation
    #[error("Messages must have correlation ID")]
    MissingCorrelation,
    
    /// Attempted to create a caused message without causation
    #[error("Caused messages must have causation ID")]
    MissingCausation,
    
    /// Detected a cycle in the causation chain
    #[error("Cycle detected in causation chain")]
    CyclicCausation,
    
    /// Invalid message identity configuration
    #[error("Invalid message identity: {0}")]
    InvalidIdentity(String),
}

/// Result type for correlation operations
pub type Result<T> = std::result::Result<T, CorrelationError>;

/// Type of identifier used in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdType {
    /// UUID for commands and queries
    Uuid(Uuid),
    /// Content-addressed ID for events
    Cid(SerializableCid),
}

impl Display for IdType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdType::Uuid(uuid) => write!(f, "{uuid}"),
            IdType::Cid(cid) => write!(f, "{cid}"),
        }
    }
}

/// Unique identifier for correlating related messages
///
/// For the first message in a correlation chain, this is a self-reference.
/// All subsequent messages in the chain share the same correlation ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(pub IdType);

impl Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "correlation:{}", self.0)
    }
}

/// Identifies what caused this message to be created
///
/// This MUST reference an existing message that has already been processed.
/// Only messages that are caused by other messages have a causation ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CausationId(pub IdType);

impl Display for CausationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "causation:{}", self.0)
    }
}

/// Message identity containing correlation and causation information
///
/// This is the core structure that every message in the system must contain.
/// It enables tracking of message relationships and causal chains.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageIdentity {
    /// Unique identifier for this message
    pub message_id: IdType,
    
    /// Groups related messages together
    pub correlation_id: CorrelationId,
    
    /// Identifies what caused this message
    pub causation_id: CausationId,
}

impl CorrelationId {
    /// Create a new correlation ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(IdType::Uuid(uuid))
    }
    
    /// Create a correlation ID from a CID
    pub fn from_cid(cid: Cid) -> Self {
        Self(IdType::Cid(SerializableCid(cid)))
    }
    
    /// Get the inner ID type
    pub fn inner(&self) -> &IdType {
        &self.0
    }
}

impl CausationId {
    /// Create a new causation ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(IdType::Uuid(uuid))
    }
    
    /// Create a causation ID from a CID
    pub fn from_cid(cid: Cid) -> Self {
        Self(IdType::Cid(SerializableCid(cid)))
    }
    
    /// Get the inner ID type
    pub fn inner(&self) -> &IdType {
        &self.0
    }
}

impl MessageIdentity {
    /// Create a root message identity (self-correlated)
    ///
    /// Used for messages that originate from user actions or external triggers.
    /// For root messages: MessageId = CorrelationId = CausationId
    pub fn root(message_id: IdType) -> Self {
        Self {
            correlation_id: CorrelationId(message_id.clone()),
            causation_id: CausationId(message_id.clone()),
            message_id,
        }
    }
    
    /// Create a caused message identity
    ///
    /// Used for messages that are caused by other messages.
    /// Inherits correlation from parent, sets causation to parent's ID.
    pub fn caused_by(
        message_id: IdType,
        parent_correlation: CorrelationId,
        parent_id: IdType,
    ) -> Self {
        Self {
            message_id,
            correlation_id: parent_correlation,
            causation_id: CausationId(parent_id),
        }
    }
    
    /// Check if this is a root message (self-correlated)
    pub fn is_root(&self) -> bool {
        self.message_id == self.correlation_id.0 && self.message_id == self.causation_id.0
    }
    
    /// Convert to NATS headers
    pub fn to_nats_headers(&self) -> Vec<(&'static str, String)> {
        vec![
            ("X-Message-ID", self.message_id.to_string()),
            ("X-Correlation-ID", self.correlation_id.0.to_string()),
            ("X-Causation-ID", self.causation_id.0.to_string()),
        ]
    }
}

/// Factory for creating messages with proper correlation/causation
///
/// This is the primary interface for creating messages in the system.
/// It ensures that all messages follow the correlation algebra rules.
pub struct MessageFactory;

impl MessageFactory {
    /// Create a root command (starts new correlation chain)
    pub fn create_root_command(command_id: Uuid) -> MessageIdentity {
        MessageIdentity::root(IdType::Uuid(command_id))
    }
    
    /// Create a root query (starts new correlation chain)
    pub fn create_root_query(query_id: Uuid) -> MessageIdentity {
        MessageIdentity::root(IdType::Uuid(query_id))
    }
    
    /// Create a root event (starts new correlation chain)
    pub fn create_root_event(event_cid: Cid) -> MessageIdentity {
        MessageIdentity::root(IdType::Cid(SerializableCid(event_cid)))
    }
    
    /// Create a command caused by another command
    pub fn command_from_command(
        command_id: Uuid,
        parent_identity: &MessageIdentity,
    ) -> MessageIdentity {
        MessageIdentity::caused_by(
            IdType::Uuid(command_id),
            parent_identity.correlation_id.clone(),
            parent_identity.message_id.clone(),
        )
    }
    
    /// Create a command caused by a query
    pub fn command_from_query(
        command_id: Uuid,
        parent_identity: &MessageIdentity,
    ) -> MessageIdentity {
        MessageIdentity::caused_by(
            IdType::Uuid(command_id),
            parent_identity.correlation_id.clone(),
            parent_identity.message_id.clone(),
        )
    }
    
    /// Create a command caused by an event
    pub fn command_from_event(
        command_id: Uuid,
        parent_identity: &MessageIdentity,
    ) -> MessageIdentity {
        MessageIdentity::caused_by(
            IdType::Uuid(command_id),
            parent_identity.correlation_id.clone(),
            parent_identity.message_id.clone(),
        )
    }
    
    /// Create a query caused by a command
    pub fn query_from_command(
        query_id: Uuid,
        parent_identity: &MessageIdentity,
    ) -> MessageIdentity {
        MessageIdentity::caused_by(
            IdType::Uuid(query_id),
            parent_identity.correlation_id.clone(),
            parent_identity.message_id.clone(),
        )
    }
    
    /// Create a query caused by another query
    pub fn query_from_query(
        query_id: Uuid,
        parent_identity: &MessageIdentity,
    ) -> MessageIdentity {
        MessageIdentity::caused_by(
            IdType::Uuid(query_id),
            parent_identity.correlation_id.clone(),
            parent_identity.message_id.clone(),
        )
    }
    
    /// Create a query caused by an event
    pub fn query_from_event(
        query_id: Uuid,
        parent_identity: &MessageIdentity,
    ) -> MessageIdentity {
        MessageIdentity::caused_by(
            IdType::Uuid(query_id),
            parent_identity.correlation_id.clone(),
            parent_identity.message_id.clone(),
        )
    }
    
    /// Create an event caused by a command
    pub fn event_from_command(
        event_cid: Cid,
        parent_identity: &MessageIdentity,
    ) -> MessageIdentity {
        MessageIdentity::caused_by(
            IdType::Cid(SerializableCid(event_cid)),
            parent_identity.correlation_id.clone(),
            parent_identity.message_id.clone(),
        )
    }
    
    /// Create an event caused by a query
    pub fn event_from_query(
        event_cid: Cid,
        parent_identity: &MessageIdentity,
    ) -> MessageIdentity {
        MessageIdentity::caused_by(
            IdType::Cid(SerializableCid(event_cid)),
            parent_identity.correlation_id.clone(),
            parent_identity.message_id.clone(),
        )
    }
    
    /// Create an event caused by another event
    pub fn event_from_event(
        event_cid: Cid,
        parent_identity: &MessageIdentity,
    ) -> MessageIdentity {
        MessageIdentity::caused_by(
            IdType::Cid(SerializableCid(event_cid)),
            parent_identity.correlation_id.clone(),
            parent_identity.message_id.clone(),
        )
    }
}

/// Validator for correlation chains
pub struct CorrelationValidator {
    /// Maximum depth for causation chains to prevent infinite loops
    pub max_chain_depth: usize,
}

impl Default for CorrelationValidator {
    fn default() -> Self {
        Self {
            max_chain_depth: 100,
        }
    }
}

impl CorrelationValidator {
    /// Validate a message identity
    pub fn validate(&self, identity: &MessageIdentity) -> Result<()> {
        // Root messages must have self-correlation
        if identity.is_root() {
            return Ok(());
        }
        
        // Non-root messages must have different message ID and causation ID
        match (&identity.message_id, &identity.causation_id.0) {
            (IdType::Uuid(msg), IdType::Uuid(caus)) if msg == caus => {
                return Err(CorrelationError::InvalidIdentity(
                    "Non-root message cannot be self-caused".to_string()
                ));
            }
            (IdType::Cid(msg), IdType::Cid(caus)) if msg.0 == caus.0 => {
                return Err(CorrelationError::InvalidIdentity(
                    "Non-root message cannot be self-caused".to_string()
                ));
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Check for cycles in a causation chain
    pub fn check_cycles(&self, chain: &[MessageIdentity]) -> Result<()> {
        if chain.len() > self.max_chain_depth {
            return Err(CorrelationError::CyclicCausation);
        }
        
        // Build a map of message IDs to check for cycles
        let mut seen = std::collections::HashSet::new();
        
        for identity in chain {
            if !seen.insert(&identity.message_id) {
                return Err(CorrelationError::CyclicCausation);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_root_message_identity() {
        let command_id = Uuid::new_v4();
        let identity = MessageFactory::create_root_command(command_id);
        
        assert!(identity.is_root());
        assert_eq!(identity.message_id, IdType::Uuid(command_id));
        assert_eq!(identity.correlation_id.0, IdType::Uuid(command_id));
        assert_eq!(identity.causation_id.0, IdType::Uuid(command_id));
    }
    
    #[test]
    fn test_caused_message_identity() {
        // Create root command
        let root_id = Uuid::new_v4();
        let root_identity = MessageFactory::create_root_command(root_id);
        
        // Create command caused by root
        let caused_id = Uuid::new_v4();
        let caused_identity = MessageFactory::command_from_command(caused_id, &root_identity);
        
        assert!(!caused_identity.is_root());
        assert_eq!(caused_identity.message_id, IdType::Uuid(caused_id));
        assert_eq!(caused_identity.correlation_id, root_identity.correlation_id);
        assert_eq!(caused_identity.causation_id.0, root_identity.message_id);
    }
    
    #[test]
    fn test_nats_headers() {
        let command_id = Uuid::new_v4();
        let identity = MessageFactory::create_root_command(command_id);
        let headers = identity.to_nats_headers();
        
        assert_eq!(headers.len(), 3);
        assert_eq!(headers[0].0, "X-Message-ID");
        assert_eq!(headers[1].0, "X-Correlation-ID");
        assert_eq!(headers[2].0, "X-Causation-ID");
    }
    
    #[test]
    fn test_correlation_validator() {
        let validator = CorrelationValidator::default();
        
        // Valid root message
        let root_id = Uuid::new_v4();
        let root_identity = MessageFactory::create_root_command(root_id);
        assert!(validator.validate(&root_identity).is_ok());
        
        // Valid caused message
        let caused_id = Uuid::new_v4();
        let caused_identity = MessageFactory::command_from_command(caused_id, &root_identity);
        assert!(validator.validate(&caused_identity).is_ok());
    }
} 