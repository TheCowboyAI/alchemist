//! Value objects for the application

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// AI capabilities and model parameters
pub mod ai_capabilities;
/// Analysis results and recommendations
pub mod analysis_result;

pub use ai_capabilities::{AICapabilities, AnalysisCapability, ModelParameters};
pub use analysis_result::{
    AnalysisResult, Recommendation, RecommendedAction, 
    Insight, Impact, Priority, EffortLevel
};

// Note: The types below are defined in this module, so they're automatically available
// when importing from value_objects

// Identity-related value objects
/// Identity type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IdentityType {
    /// Person identity
    Person,
    /// Organization identity
    Organization,
    /// System identity
    System,
}

/// Verification level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub enum VerificationLevel {
    /// Not verified
    None = 0,
    /// Basic verification
    Basic = 1,
    /// Advanced verification
    Advanced = 2,
    /// Full verification
    Full = 3,
}

/// Verification method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationMethod {
    /// Email verification
    Email,
    /// Phone verification
    Phone,
    /// Document verification
    Document,
    /// Biometric verification
    Biometric,
}

/// Relationship type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationshipType {
    /// Parent-child relationship
    ParentChild,
    /// Sibling relationship
    Sibling,
    /// Employment relationship
    Employment,
    /// Partnership relationship
    Partnership,
    /// Custom relationship
    Custom(String),
}

/// Workflow type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowType {
    /// Approval workflow
    Approval,
    /// Review workflow
    Review,
    /// Processing workflow
    Processing,
}

/// Workflow status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Pending status
    Pending,
    /// In progress
    InProgress,
    /// Completed
    Completed,
    /// Failed
    Failed,
}

/// Projection type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectionType {
    /// Summary projection
    Summary,
    /// Detail projection
    Detail,
    /// Analytics projection
    Analytics,
}

/// Projection context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionContext {
    /// Context ID
    pub id: String,
    /// Context name
    pub name: String,
}

/// Relationship ID
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipId(pub Uuid);

/// Agent constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConstraint {
    /// Constraint name
    pub name: String,
    /// Constraint value
    pub value: String,
}

/// Agent type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentType {
    /// AI agent
    AI,
    /// Human agent
    Human,
    /// System agent
    System,
}

/// Authentication method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    /// API key authentication
    ApiKey,
    /// OAuth authentication
    OAuth,
    /// JWT authentication
    JWT,
}

/// Workflow ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowId(pub Uuid);

/// Node ID for graph nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

/// Edge ID for graph edges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub Uuid);

/// Graph ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphId(pub Uuid);

/// Node type enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    /// Process node
    Process,
    /// Decision node
    Decision,
    /// Start node
    Start,
    /// End node
    End,
    /// Data node
    Data,
    /// Custom node type
    Custom(String),
}

/// Edge relationship type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EdgeRelationship {
    /// Simple connection
    Connects,
    /// Data flow
    DataFlow,
    /// Control flow
    ControlFlow,
    /// Dependency
    DependsOn,
    /// Custom relationship
    Custom(String),
}

/// 3D position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position3D {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
    /// Z coordinate
    pub z: f32,
} 