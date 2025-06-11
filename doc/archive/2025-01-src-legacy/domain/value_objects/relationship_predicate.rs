use serde::{Deserialize, Serialize};

/// Describes how nodes are related to each other in the graph
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelatedBy {
    /// Nodes are similar based on content or position
    Similar,

    /// One node depends on another
    DependsOn,

    /// Nodes are part of the same category
    SameCategory,

    /// One node is derived from another
    DerivedFrom,

    /// Nodes are connected in a workflow
    FlowsTo,

    /// One node references another
    References,

    /// Nodes are alternatives to each other
    Alternative,

    /// One node contains another
    Contains,

    /// Custom relationship with description
    Custom(String),
}
