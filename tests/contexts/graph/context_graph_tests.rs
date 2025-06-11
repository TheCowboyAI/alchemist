//! Comprehensive tests for ContextGraph functionality
//!
//! Tests all aspects of ContextGraph including:
//! - Creation with different context types
//! - Command execution
//! - Invariant validation
//! - Dependency injection
//! - Edge cases and error handling

use ia::contexts::graph::domain::{
    ContextGraph, ContextType,
};
use ia::contexts::graph::domain::context_graph::{
    InvariantValidator, PositionCalculator,
    DefaultInvariantValidator, DefaultPositionCalculator,
};
use ia::contexts::graph::domain::commands::{AddNode, ConnectNodes};
use ia::contexts::graph::domain::events::NodeAdded;
use ia::shared::types::{GraphId, NodeId, EdgeId, Result, Error};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Test validator that tracks all validation calls
struct TrackingValidator {
    calls: Arc<Mutex<Vec<String>>>,
    delegate: Box<dyn InvariantValidator>,
}

impl TrackingValidator {
    fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
            delegate: Box::new(DefaultInvariantValidator),
        }
    }

    fn get_calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
}

/// Wrapper to allow Arc<TrackingValidator> to implement InvariantValidator
struct SharedTrackingValidator(Arc<TrackingValidator>);

impl SharedTrackingValidator {
    fn new(validator: Arc<TrackingValidator>) -> Self {
        Self(validator)
    }

    fn get_calls(&self) -> Vec<String> {
        self.0.get_calls()
    }
}

impl InvariantValidator for TrackingValidator {
    fn validate_node_addition(&self, graph: &ContextGraph, node_id: &NodeId) -> Result<()> {
        self.calls.lock().unwrap().push(format!("validate_node_addition: {}", node_id));
        self.delegate.validate_node_addition(graph, node_id)
    }

    fn validate_edge_creation(&self, graph: &ContextGraph, source: &NodeId, target: &NodeId) -> Result<()> {
        self.calls.lock().unwrap().push(format!("validate_edge_creation: {} -> {}", source, target));
        self.delegate.validate_edge_creation(graph, source, target)
    }

    fn validate_context_root(&self, graph: &ContextGraph) -> Result<()> {
        self.calls.lock().unwrap().push("validate_context_root".to_string());
        self.delegate.validate_context_root(graph)
    }
}

impl InvariantValidator for SharedTrackingValidator {
    fn validate_node_addition(&self, graph: &ContextGraph, node_id: &NodeId) -> Result<()> {
        self.0.validate_node_addition(graph, node_id)
    }

    fn validate_edge_creation(&self, graph: &ContextGraph, source: &NodeId, target: &NodeId) -> Result<()> {
        self.0.validate_edge_creation(graph, source, target)
    }

    fn validate_context_root(&self, graph: &ContextGraph) -> Result<()> {
        self.0.validate_context_root(graph)
    }
}



/// Test position calculator that returns predictable positions
struct TestPositionCalculator {
    counter: Arc<Mutex<i32>>,
}

impl TestPositionCalculator {
    fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
        }
    }
}

impl PositionCalculator for TestPositionCalculator {
    fn calculate_position(&self, _graph: &ContextGraph, _node_id: &NodeId) -> Result<(f32, f32, f32)> {
        let mut counter = self.counter.lock().unwrap();
        *counter += 1;
        Ok(((*counter as f32) * 100.0, 0.0, 0.0))
    }
}

#[test]
fn test_context_graph_creation_all_types() {
    // Test BoundedContext
    let graph = ContextGraph::new(
        GraphId::new(),
        "Sales Context".to_string(),
        ContextType::BoundedContext {
            name: "Sales".to_string(),
            domain: "E-commerce".to_string(),
        },
        NodeId::new(),
        Box::new(DefaultInvariantValidator),
        Box::new(DefaultPositionCalculator),
    ).unwrap();

    assert_eq!(graph.version(), 0);
    match graph.context_type() {
        ContextType::BoundedContext { name, domain } => {
            assert_eq!(name, "Sales");
            assert_eq!(domain, "E-commerce");
        }
        _ => panic!("Wrong context type"),
    }

    // Test AggregateContext
    let graph = ContextGraph::new(
        GraphId::new(),
        "Order Aggregate".to_string(),
        ContextType::AggregateContext {
            name: "Order".to_string(),
            aggregate_type: "Order".to_string(),
        },
        NodeId::new(),
        Box::new(DefaultInvariantValidator),
        Box::new(DefaultPositionCalculator),
    ).unwrap();

    match graph.context_type() {
        ContextType::AggregateContext { name, aggregate_type } => {
            assert_eq!(name, "Order");
            assert_eq!(aggregate_type, "Order");
        }
        _ => panic!("Wrong context type"),
    }

    // Test ModuleContext
    let graph = ContextGraph::new(
        GraphId::new(),
        "Payment Module".to_string(),
        ContextType::ModuleContext {
            name: "Payment".to_string(),
            purpose: "Handle payment processing".to_string(),
        },
        NodeId::new(),
        Box::new(DefaultInvariantValidator),
        Box::new(DefaultPositionCalculator),
    ).unwrap();

    match graph.context_type() {
        ContextType::ModuleContext { name, purpose } => {
            assert_eq!(name, "Payment");
            assert_eq!(purpose, "Handle payment processing");
        }
        _ => panic!("Wrong context type"),
    }

    // Test ServiceContext
    let graph = ContextGraph::new(
        GraphId::new(),
        "Email Service".to_string(),
        ContextType::ServiceContext {
            name: "EmailService".to_string(),
            capability: "Send transactional emails".to_string(),
        },
        NodeId::new(),
        Box::new(DefaultInvariantValidator),
        Box::new(DefaultPositionCalculator),
    ).unwrap();

    match graph.context_type() {
        ContextType::ServiceContext { name, capability } => {
            assert_eq!(name, "EmailService");
            assert_eq!(capability, "Send transactional emails");
        }
        _ => panic!("Wrong context type"),
    }
}

#[test]
fn test_node_operations() {
    let graph_id = GraphId::new();
    let root_id = NodeId::new();

    let mut graph = ContextGraph::new(
        graph_id,
        "Test Graph".to_string(),
        ContextType::BoundedContext {
            name: "Test".to_string(),
            domain: "Testing".to_string(),
        },
        root_id,
        Box::new(DefaultInvariantValidator),
        Box::new(DefaultPositionCalculator),
    ).unwrap();

    // Test adding nodes
    let node1 = NodeId::new();
    let node2 = NodeId::new();
    let node3 = NodeId::new();

    // Add first node
    let command = AddNode {
        graph_id,
        node_id: node1,
        node_type: "Aggregate".to_string(),
        metadata: HashMap::from([
            ("description".to_string(), serde_json::json!("Order aggregate")),
        ]),
    };

    let events = graph.handle_command(command).unwrap();
    assert_eq!(events.len(), 1);
    // The graph starts with a root node, so we expect 2 nodes after adding one
    assert_eq!(graph.node_count(), 2);
    assert!(graph.has_node(&node1));

    // Add second node with different type
    let command = AddNode {
        graph_id,
        node_id: node2,
        node_type: "Service".to_string(),
        metadata: HashMap::from([
            ("capability".to_string(), serde_json::json!("Payment processing")),
        ]),
    };

    let events = graph.handle_command(command).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(graph.node_count(), 2);
    assert!(graph.has_node(&node2));

    // Add third node with custom type
    let command = AddNode {
        graph_id,
        node_id: node3,
        node_type: "CustomProcessor".to_string(),
        metadata: HashMap::new(),
    };

    let events = graph.handle_command(command).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(graph.node_count(), 3);
    assert!(graph.has_node(&node3));

    // Test node doesn't exist
    assert!(!graph.has_node(&NodeId::new()));
}

#[test]
fn test_edge_operations() {
    let graph_id = GraphId::new();
    let root_id = NodeId::new();

    let mut graph = ContextGraph::new(
        graph_id,
        "Test Graph".to_string(),
        ContextType::AggregateContext {
            name: "TestAggregate".to_string(),
            aggregate_type: "Test".to_string(),
        },
        root_id,
        Box::new(DefaultInvariantValidator),
        Box::new(DefaultPositionCalculator),
    ).unwrap();

    // Add nodes first
    let node1 = NodeId::new();
    let node2 = NodeId::new();
    let node3 = NodeId::new();

    let command = AddNode {
        graph_id,
        node_id: node1,
        node_type: "Entity".to_string(),
        metadata: HashMap::new(),
    };
    graph.handle_command(command).unwrap();

    let command = AddNode {
        graph_id,
        node_id: node2,
        node_type: "Value".to_string(),
        metadata: HashMap::new(),
    };
    graph.handle_command(command).unwrap();

    let command = AddNode {
        graph_id,
        node_id: node3,
        node_type: "Service".to_string(),
        metadata: HashMap::new(),
    };
    graph.handle_command(command).unwrap();

    // Test different edge types
    let edge1 = EdgeId::new();
    let command = ConnectNodes {
        graph_id,
        edge_id: edge1,
        source: node1,
        target: node2,
        edge_type: "Contains".to_string(),
        metadata: HashMap::from([
            ("cardinality".to_string(), serde_json::json!("1:n")),
        ]),
    };

    let events = graph.handle_command(command).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(graph.edge_count(), 1);

    // Add another edge with different type
    let edge2 = EdgeId::new();
    let command = ConnectNodes {
        graph_id,
        edge_id: edge2,
        source: node1,
        target: node3,
        edge_type: "DependsOn".to_string(),
        metadata: HashMap::new(),
    };

    let events = graph.handle_command(command).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(graph.edge_count(), 2);

    // Add custom edge type
    let edge3 = EdgeId::new();
    let command = ConnectNodes {
        graph_id,
        edge_id: edge3,
        source: node2,
        target: node3,
        edge_type: "ValidatedBy".to_string(),
        metadata: HashMap::new(),
    };

    let events = graph.handle_command(command).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(graph.edge_count(), 3);
}

#[test]
fn test_validation_errors() {
    let graph_id = GraphId::new();
    let root_id = NodeId::new();

    let mut graph = ContextGraph::new(
        graph_id,
        "Test Graph".to_string(),
        ContextType::BoundedContext {
            name: "Test".to_string(),
            domain: "Testing".to_string(),
        },
        root_id,
        Box::new(DefaultInvariantValidator),
        Box::new(DefaultPositionCalculator),
    ).unwrap();

    // Test connecting non-existent nodes
    let edge_id = EdgeId::new();
    let command = ConnectNodes {
        graph_id,
        edge_id,
        source: NodeId::new(),
        target: NodeId::new(),
        edge_type: "TestEdge".to_string(),
        metadata: HashMap::new(),
    };

    let result = graph.handle_command(command);
    assert!(result.is_err());

    // Add one node and try to connect to non-existent
    let node1 = NodeId::new();
    let command = AddNode {
        graph_id,
        node_id: node1,
        node_type: "TestNode".to_string(),
        metadata: HashMap::new(),
    };
    graph.handle_command(command).unwrap();

    let command = ConnectNodes {
        graph_id,
        edge_id,
        source: node1,
        target: NodeId::new(),
        edge_type: "TestEdge".to_string(),
        metadata: HashMap::new(),
    };

    let result = graph.handle_command(command);
    assert!(result.is_err());
}

#[test]
fn test_command_graph_id_validation() {
    let graph_id = GraphId::new();
    let wrong_graph_id = GraphId::new();
    let root_id = NodeId::new();

    let mut graph = ContextGraph::new(
        graph_id,
        "Test Graph".to_string(),
        ContextType::ModuleContext {
            name: "TestModule".to_string(),
            purpose: "Testing".to_string(),
        },
        root_id,
        Box::new(DefaultInvariantValidator),
        Box::new(DefaultPositionCalculator),
    ).unwrap();

        // Test AddNode with wrong graph ID
    let command = AddNode {
        graph_id: wrong_graph_id,
        node_id: NodeId::new(),
        node_type: "TestNode".to_string(),
        metadata: HashMap::new(),
    };

    let result = graph.handle_command(command);
    assert!(result.is_err());

        // Test ConnectNodes with wrong graph ID
    let command = ConnectNodes {
        graph_id: wrong_graph_id,
        edge_id: EdgeId::new(),
        source: NodeId::new(),
        target: NodeId::new(),
        edge_type: "TestEdge".to_string(),
        metadata: HashMap::new(),
    };

    let result = graph.handle_command(command);
    assert!(result.is_err());
}

#[test]
fn test_dependency_injection_tracking() {
    let graph_id = GraphId::new();
    let root_id = NodeId::new();

    let validator = Arc::new(TrackingValidator::new());
    let shared_validator = SharedTrackingValidator::new(validator.clone());

    let mut graph = ContextGraph::new(
        graph_id,
        "Test Graph".to_string(),
        ContextType::ServiceContext {
            name: "TestService".to_string(),
            capability: "Testing".to_string(),
        },
        root_id,
        Box::new(shared_validator),
        Box::new(TestPositionCalculator::new()),
    ).unwrap();

    // Check that context root was validated
    let calls = validator.get_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], "validate_context_root");

    // Add a node and check validation was called
    let node1 = NodeId::new();
    let command = AddNode {
        graph_id,
        node_id: node1,
        node_type: "TestNode".to_string(),
        metadata: HashMap::new(),
    };
    graph.handle_command(command).unwrap();

    let calls = validator.get_calls();
    assert_eq!(calls.len(), 2);
    assert!(calls[1].starts_with("validate_node_addition:"));

    // Add another node and connect them
    let node2 = NodeId::new();
    let command = AddNode {
        graph_id,
        node_id: node2,
        node_type: "TestNode2".to_string(),
        metadata: HashMap::new(),
    };
    graph.handle_command(command).unwrap();

    let edge_id = EdgeId::new();
    let command = ConnectNodes {
        graph_id,
        edge_id,
        source: node1,
        target: node2,
        edge_type: "TestEdge".to_string(),
        metadata: HashMap::new(),
    };

    graph.handle_command(command).unwrap();

    let calls = validator.get_calls();
    assert_eq!(calls.len(), 4);
    assert!(calls[3].starts_with("validate_edge_creation:"));
}

#[test]
fn test_custom_validator_rejection() {
    struct RejectAllValidator;

    impl InvariantValidator for RejectAllValidator {
        fn validate_node_addition(&self, _: &ContextGraph, _: &NodeId) -> Result<()> {
            Err(Error::InvariantViolation("No nodes allowed in this context".to_string()))
        }

        fn validate_edge_creation(&self, _: &ContextGraph, _: &NodeId, _: &NodeId) -> Result<()> {
            Err(Error::InvariantViolation("No edges allowed in this context".to_string()))
        }

        fn validate_context_root(&self, _: &ContextGraph) -> Result<()> {
            Ok(()) // Allow creation
        }
    }

    let graph_id = GraphId::new();
    let root_id = NodeId::new();

    let mut graph = ContextGraph::new(
        graph_id,
        "Restricted Graph".to_string(),
        ContextType::BoundedContext {
            name: "Restricted".to_string(),
            domain: "Security".to_string(),
        },
        root_id,
        Box::new(RejectAllValidator),
        Box::new(DefaultPositionCalculator),
    ).unwrap();

    // Try to add a node - should fail
    let command = AddNode {
        graph_id,
        node_id: NodeId::new(),
        node_type: "TestNode".to_string(),
        metadata: HashMap::new(),
    };

    let result = graph.handle_command(command);
    assert!(result.is_err());
}

#[test]
fn test_version_increments() {
    let graph_id = GraphId::new();
    let root_id = NodeId::new();

    let mut graph = ContextGraph::new(
        graph_id,
        "Test Graph".to_string(),
        ContextType::BoundedContext {
            name: "Test".to_string(),
            domain: "Testing".to_string(),
        },
        root_id,
        Box::new(DefaultInvariantValidator),
        Box::new(DefaultPositionCalculator),
    ).unwrap();

    assert_eq!(graph.version(), 0);

    // Apply an event (this would normally come from event sourcing)
    let event = Box::new(NodeAdded {
        graph_id,
        node_id: NodeId::new(),
        node_type: "Test".to_string(),
        metadata: HashMap::new(),
        event_metadata: ia::shared::events::EventMetadata::new(),
    });

    graph.apply_event(event.as_ref()).unwrap();
    assert_eq!(graph.version(), 1);

    // Apply another event
    graph.apply_event(event.as_ref()).unwrap();
    assert_eq!(graph.version(), 2);
}

#[test]
fn test_complex_graph_scenario() {
    let graph_id = GraphId::new();
    let root_id = NodeId::new();

    let mut graph = ContextGraph::new(
        graph_id,
        "E-commerce System".to_string(),
        ContextType::BoundedContext {
            name: "Ordering".to_string(),
            domain: "E-commerce".to_string(),
        },
        root_id,
        Box::new(DefaultInvariantValidator),
        Box::new(DefaultPositionCalculator),
    ).unwrap();

    // Build a realistic domain model
    let order_aggregate = NodeId::new();
    let order_item = NodeId::new();
    let customer = NodeId::new();
    let payment_service = NodeId::new();
    let inventory_service = NodeId::new();

    // Add all nodes
    let command = AddNode {
        graph_id,
        node_id: order_aggregate,
        node_type: "Aggregate".to_string(),
        metadata: HashMap::from([
            ("name".to_string(), serde_json::json!("Order")),
            ("description".to_string(), serde_json::json!("Order aggregate root")),
        ]),
    };
    graph.handle_command(command).unwrap();

    let command = AddNode {
        graph_id,
        node_id: order_item,
        node_type: "Entity".to_string(),
        metadata: HashMap::from([
            ("name".to_string(), serde_json::json!("OrderItem")),
            ("cardinality".to_string(), serde_json::json!("1..*")),
        ]),
    };
    graph.handle_command(command).unwrap();

    let command = AddNode {
        graph_id,
        node_id: customer,
        node_type: "EntityReference".to_string(),
        metadata: HashMap::from([
            ("name".to_string(), serde_json::json!("Customer")),
            ("bounded_context".to_string(), serde_json::json!("CustomerManagement")),
        ]),
    };
    graph.handle_command(command).unwrap();

    let command = AddNode {
        graph_id,
        node_id: payment_service,
        node_type: "Service".to_string(),
        metadata: HashMap::from([
            ("name".to_string(), serde_json::json!("PaymentService")),
            ("capability".to_string(), serde_json::json!("Process payments")),
        ]),
    };
    graph.handle_command(command).unwrap();

    let command = AddNode {
        graph_id,
        node_id: inventory_service,
        node_type: "Service".to_string(),
        metadata: HashMap::from([
            ("name".to_string(), serde_json::json!("InventoryService")),
            ("capability".to_string(), serde_json::json!("Check stock availability")),
        ]),
    };
    graph.handle_command(command).unwrap();

    // Connect them with appropriate relationships
    let command = ConnectNodes {
        graph_id,
        edge_id: EdgeId::new(),
        source: order_aggregate,
        target: order_item,
        edge_type: "Contains".to_string(),
        metadata: HashMap::from([
            ("multiplicity".to_string(), serde_json::json!("1..*")),
        ]),
    };
    graph.handle_command(command).unwrap();

    let command = ConnectNodes {
        graph_id,
        edge_id: EdgeId::new(),
        source: order_aggregate,
        target: customer,
        edge_type: "References".to_string(),
        metadata: HashMap::from([
            ("required".to_string(), serde_json::json!(true)),
        ]),
    };
    graph.handle_command(command).unwrap();

    let command = ConnectNodes {
        graph_id,
        edge_id: EdgeId::new(),
        source: order_aggregate,
        target: payment_service,
        edge_type: "DependsOn".to_string(),
        metadata: HashMap::new(),
    };
    graph.handle_command(command).unwrap();

    let command = ConnectNodes {
        graph_id,
        edge_id: EdgeId::new(),
        source: order_aggregate,
        target: inventory_service,
        edge_type: "DependsOn".to_string(),
        metadata: HashMap::new(),
    };
    graph.handle_command(command).unwrap();

    // Verify the graph structure
    assert_eq!(graph.node_count(), 5);
    assert_eq!(graph.edge_count(), 4);
    assert!(graph.has_node(&order_aggregate));
    assert!(graph.has_node(&order_item));
    assert!(graph.has_node(&customer));
    assert!(graph.has_node(&payment_service));
    assert!(graph.has_node(&inventory_service));
}

// Mermaid diagram showing test coverage
// ```mermaid
// graph TD
//     subgraph "ContextGraph Test Coverage"
//         A[Creation Tests]
//         A --> A1[All Context Types]
//         A --> A2[Dependency Injection]
//         A --> A3[Version Tracking]
//
//         B[Node Operations]
//         B --> B1[Add Nodes]
//         B --> B2[Node Types]
//         B --> B3[Node Metadata]
//         B --> B4[Node Existence]
//
//         C[Edge Operations]
//         C --> C1[Connect Nodes]
//         C --> C2[Edge Types]
//         C --> C3[Edge Metadata]
//         C --> C4[Edge Validation]
//
//         D[Validation Tests]
//         D --> D1[Node Validation]
//         D --> D2[Edge Validation]
//         D --> D3[Custom Validators]
//         D --> D4[Error Handling]
//
//         E[Command Tests]
//         E --> E1[AddNode Command]
//         E --> E2[ConnectNodes Command]
//         E --> E3[Graph ID Validation]
//         E --> E4[Event Generation]
//
//         F[Complex Scenarios]
//         F --> F1[E-commerce Model]
//         F --> F2[Multiple Node Types]
//         F --> F3[Multiple Edge Types]
//         F --> F4[Realistic Relationships]
//     end
// ```
