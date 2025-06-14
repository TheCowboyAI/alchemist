#!/bin/bash
set -e

echo "Extracting Graph Domain from cim-domain to cim-domain-graph"

# Create the new domain directory structure
echo "Creating cim-domain-graph directory structure..."
mkdir -p cim-domain-graph/{src/{aggregate,commands,events,handlers,projections,queries,value_objects},tests}

# Create Cargo.toml
echo "Creating Cargo.toml..."
cat > cim-domain-graph/Cargo.toml << 'EOF'
[package]
name = "cim-domain-graph"
version = "0.1.0"
edition = "2021"
authors = ["The CowboyAI Team"]
description = "Graph domain for the Composable Information Machine - Core composition layer"
license = "MIT OR Apache-2.0"
repository = "https://github.com/thecowboyai/cim-domain-graph"
keywords = ["graph", "domain", "ddd", "event-sourcing", "cim", "composition"]
categories = ["data-structures", "asynchronous"]

[dependencies]
# Core dependencies
cim-core-domain = { path = "../cim-core-domain" }
cim-infrastructure = { path = "../cim-infrastructure" }

# Async runtime
tokio = { version = "1.41", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# UUID generation
uuid = { version = "1.11", features = ["v4", "serde"] }

# Time handling
chrono = { version = "0.4", features = ["serde"] }

# Collections
indexmap = "2.7"

# Graph library
petgraph = { version = "0.6", features = ["serde-1"] }

[dev-dependencies]
tokio-test = "0.4"
proptest = "1.6"
EOF

# Create README.md
echo "Creating README.md..."
cat > cim-domain-graph/README.md << 'EOF'
# CIM Domain Graph

The graph domain for the Composable Information Machine (CIM). This is the core composition layer that enables other domains to be composed into graphs without creating dependencies.

## Overview

The graph domain provides:
- Graph aggregates (ConceptGraph, DomainGraph)
- Node and edge management
- Graph composition capabilities
- Graph-based workflows
- Conceptual space integration

## Architecture

This domain serves as the composition layer for CIM. Other domains (person, organization, workflow, etc.) can be composed into graphs, but they do not depend on the graph domain. This ensures clean separation of concerns and prevents circular dependencies.

## Features

- **ConceptGraph**: For knowledge representation and conceptual relationships
- **DomainGraph**: For domain model visualization and composition
- **Graph Events**: GraphCreated, NodeAdded, EdgeAdded, etc.
- **Graph Commands**: CreateGraph, AddNode, ConnectNodes, etc.
- **Projections**: GraphSummary, NodeList, etc.

## Usage

```rust
use cim_domain_graph::{ConceptGraph, DomainGraph, GraphCommand};

// Create a new concept graph
let graph = ConceptGraph::new(graph_id, "Knowledge Graph");

// Add nodes from other domains
let person_node = graph.add_node(person_id, NodeType::Person);
let org_node = graph.add_node(org_id, NodeType::Organization);

// Connect nodes
graph.connect_nodes(person_node, org_node, EdgeType::WorksFor);
```

## License

MIT OR Apache-2.0
EOF

# Move graph-related files
echo "Moving graph aggregates..."
cp cim-domain/src/concept_graph.rs cim-domain-graph/src/aggregate/concept_graph.rs
cp cim-domain/src/domain_graph.rs cim-domain-graph/src/aggregate/domain_graph.rs

# Extract graph events from domain_events.rs
echo "Extracting graph events..."
cat > cim-domain-graph/src/events/graph_events.rs << 'EOF'
//! Graph domain events

use cim_core_domain::{DomainEvent, entity::EntityId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

// Re-export identifiers that will be moved here eventually
pub use cim_core_domain::{GraphId, NodeId, EdgeId};

/// Graph created event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphCreated {
    /// The unique identifier of the graph
    pub graph_id: GraphId,
    /// The name of the graph
    pub name: String,
    /// A description of the graph's purpose
    pub description: String,
    /// Additional metadata about the graph
    pub metadata: HashMap<String, serde_json::Value>,
    /// When the graph was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Node added event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAdded {
    /// The graph to which the node was added
    pub graph_id: GraphId,
    /// The unique identifier of the node
    pub node_id: NodeId,
    /// The type of node (e.g., "task", "decision", "gateway")
    pub node_type: String,
    /// Additional metadata about the node
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Node removed event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRemoved {
    /// The graph from which the node was removed
    pub graph_id: GraphId,
    /// The ID of the node that was removed
    pub node_id: NodeId,
}

/// Node updated event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeUpdated {
    /// The graph containing the updated node
    pub graph_id: GraphId,
    /// The ID of the node that was updated
    pub node_id: NodeId,
    /// The updated metadata for the node
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Edge added event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeAdded {
    /// The graph to which the edge was added
    pub graph_id: GraphId,
    /// The unique identifier of the edge
    pub edge_id: EdgeId,
    /// The source node of the edge
    pub source_id: NodeId,
    /// The target node of the edge
    pub target_id: NodeId,
    /// The type of edge (e.g., "sequence", "conditional", "parallel")
    pub edge_type: String,
    /// Additional metadata about the edge
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Edge removed event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRemoved {
    /// The graph from which the edge was removed
    pub graph_id: GraphId,
    /// The ID of the edge that was removed
    pub edge_id: EdgeId,
}

// Implement DomainEvent trait for all events
impl DomainEvent for GraphCreated {
    fn aggregate_id(&self) -> Uuid {
        self.graph_id.into()
    }

    fn event_type(&self) -> &'static str {
        "GraphCreated"
    }

    fn subject(&self) -> String {
        format!("graphs.graph.created.v1")
    }
}

impl DomainEvent for NodeAdded {
    fn aggregate_id(&self) -> Uuid {
        self.graph_id.into()
    }

    fn event_type(&self) -> &'static str {
        "NodeAdded"
    }

    fn subject(&self) -> String {
        format!("graphs.node.added.v1")
    }
}

impl DomainEvent for NodeRemoved {
    fn aggregate_id(&self) -> Uuid {
        self.graph_id.into()
    }

    fn event_type(&self) -> &'static str {
        "NodeRemoved"
    }

    fn subject(&self) -> String {
        format!("graphs.node.removed.v1")
    }
}

impl DomainEvent for NodeUpdated {
    fn aggregate_id(&self) -> Uuid {
        self.graph_id.into()
    }

    fn event_type(&self) -> &'static str {
        "NodeUpdated"
    }

    fn subject(&self) -> String {
        format!("graphs.node.updated.v1")
    }
}

impl DomainEvent for EdgeAdded {
    fn aggregate_id(&self) -> Uuid {
        self.graph_id.into()
    }

    fn event_type(&self) -> &'static str {
        "EdgeAdded"
    }

    fn subject(&self) -> String {
        format!("graphs.edge.added.v1")
    }
}

impl DomainEvent for EdgeRemoved {
    fn aggregate_id(&self) -> Uuid {
        self.graph_id.into()
    }

    fn event_type(&self) -> &'static str {
        "EdgeRemoved"
    }

    fn subject(&self) -> String {
        format!("graphs.edge.removed.v1")
    }
}
EOF

# Create domain events enum
echo "Creating domain events enum..."
cat > cim-domain-graph/src/domain_events.rs << 'EOF'
//! Domain events enum for graph domain

use crate::events::{GraphCreated, NodeAdded, NodeRemoved, NodeUpdated, EdgeAdded, EdgeRemoved};
use cim_core_domain::DomainEvent;
use serde::{Deserialize, Serialize};

/// Enum wrapper for graph domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphDomainEvent {
    /// A new graph was created
    GraphCreated(GraphCreated),
    /// A node was added to a graph
    NodeAdded(NodeAdded),
    /// A node was removed from a graph
    NodeRemoved(NodeRemoved),
    /// A node's metadata was updated
    NodeUpdated(NodeUpdated),
    /// An edge was added between nodes
    EdgeAdded(EdgeAdded),
    /// An edge was removed from the graph
    EdgeRemoved(EdgeRemoved),
}

impl DomainEvent for GraphDomainEvent {
    fn subject(&self) -> String {
        match self {
            Self::GraphCreated(e) => e.subject(),
            Self::NodeAdded(e) => e.subject(),
            Self::NodeRemoved(e) => e.subject(),
            Self::NodeUpdated(e) => e.subject(),
            Self::EdgeAdded(e) => e.subject(),
            Self::EdgeRemoved(e) => e.subject(),
        }
    }

    fn aggregate_id(&self) -> uuid::Uuid {
        match self {
            Self::GraphCreated(e) => e.aggregate_id(),
            Self::NodeAdded(e) => e.aggregate_id(),
            Self::NodeRemoved(e) => e.aggregate_id(),
            Self::NodeUpdated(e) => e.aggregate_id(),
            Self::EdgeAdded(e) => e.aggregate_id(),
            Self::EdgeRemoved(e) => e.aggregate_id(),
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            Self::GraphCreated(e) => e.event_type(),
            Self::NodeAdded(e) => e.event_type(),
            Self::NodeRemoved(e) => e.event_type(),
            Self::NodeUpdated(e) => e.event_type(),
            Self::EdgeAdded(e) => e.event_type(),
            Self::EdgeRemoved(e) => e.event_type(),
        }
    }
}
EOF

# Create module files
echo "Creating module files..."
cat > cim-domain-graph/src/aggregate/mod.rs << 'EOF'
//! Graph aggregates

pub mod concept_graph;
pub mod domain_graph;

pub use concept_graph::*;
pub use domain_graph::*;
EOF

cat > cim-domain-graph/src/events/mod.rs << 'EOF'
//! Graph events

pub mod graph_events;

pub use graph_events::*;
EOF

cat > cim-domain-graph/src/commands/mod.rs << 'EOF'
//! Graph commands

// TODO: Extract graph commands from cim-domain
EOF

cat > cim-domain-graph/src/handlers/mod.rs << 'EOF'
//! Graph command and event handlers

// TODO: Implement graph command handlers
EOF

cat > cim-domain-graph/src/projections/mod.rs << 'EOF'
//! Graph projections

// TODO: Move graph projections from cim-domain
EOF

cat > cim-domain-graph/src/queries/mod.rs << 'EOF'
//! Graph queries

// TODO: Implement graph queries
EOF

cat > cim-domain-graph/src/value_objects/mod.rs << 'EOF'
//! Graph value objects

// TODO: Extract graph value objects
EOF

# Create main lib.rs
echo "Creating lib.rs..."
cat > cim-domain-graph/src/lib.rs << 'EOF'
//! Graph domain for the Composable Information Machine
//!
//! This is the core composition layer that enables other domains to be composed
//! into graphs without creating dependencies. Other domains do not depend on this
//! domain, but can be composed into graphs through it.

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod value_objects;
pub mod domain_events;

// Re-export main types
pub use aggregate::*;
pub use events::*;
pub use domain_events::*;

// Re-export identifiers that will eventually move here
pub use cim_core_domain::{GraphId, NodeId, EdgeId};
EOF

# Create basic tests
echo "Creating tests..."
cat > cim-domain-graph/tests/graph_tests.rs << 'EOF'
//! Integration tests for graph domain

use cim_domain_graph::{ConceptGraph, GraphId};

#[test]
fn test_graph_creation() {
    // TODO: Implement tests after moving graph functionality
    assert!(true);
}
EOF

echo "Graph domain extraction script created successfully!"
echo ""
echo "Next steps:"
echo "1. Run this script to create the cim-domain-graph structure"
echo "2. Update imports in the moved files"
echo "3. Remove graph code from cim-domain"
echo "4. Fix compilation issues"
echo "5. Run tests to ensure everything works"
EOF
