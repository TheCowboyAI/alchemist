# Acceptance Tests for CIM Graph Editor and Workflow Manager

## Overview

This document defines acceptance tests for the Composable Information Machine (CIM) presentation layer, focusing on graph manipulation, DDD workflow modeling, visualization morphisms, and AI-driven software development capabilities.

## Vision Alignment

The CIM system enables users to:
- **Load** existing graphs from various sources (NATS, files, AI-generated)
- **Display** graphs with multiple visualization modes and conceptual mappings
- **Create** new graphs representing business domains and workflows
- **Model** DDD components and their relationships
- **Combine** multiple graphs through composition and morphisms
- **Apply** transformations and visualizations for different perspectives
- **Generate** software from domain models using AI agents

## Graph Loading and Persistence Tests

### Test: Load Graph from Event Store
```rust
#[tokio::test]
async fn test_load_graph_from_event_store() {
    // Given: A graph exists in the event store
    let graph_id = GraphId::new();
    let event_store = create_test_event_store().await;

    // Create test events
    let events = vec![
        DomainEvent::GraphCreated {
            id: graph_id,
            metadata: GraphMetadata {
                name: "Order Processing Workflow".to_string(),
                domain: "e-commerce".to_string(),
                tags: vec!["workflow", "order-management"],
            },
        },
        DomainEvent::NodeAdded {
            graph_id,
            node_id: NodeId::new(),
            node_type: NodeType::DomainEvent("OrderPlaced"),
            position: Position3D::new(0.0, 0.0, 0.0),
        },
    ];

    event_store.append_events(graph_id, events).await.unwrap();

    // When: Loading the graph
    let loaded_graph = GraphLoader::load_from_event_store(
        &event_store,
        graph_id,
    ).await.unwrap();

    // Then: Graph should be reconstructed correctly
    assert_eq!(loaded_graph.metadata.name, "Order Processing Workflow");
    assert_eq!(loaded_graph.nodes.len(), 1);
    assert!(loaded_graph.metadata.tags.contains(&"workflow".to_string()));
}
```

### Test: Import Graph from DDD Model File
```rust
#[test]
fn test_import_ddd_model_from_file() {
    // Given: A DDD model definition file
    let ddd_model = r#"
    context: OrderManagement
    aggregates:
      - name: Order
        commands:
          - PlaceOrder
          - CancelOrder
        events:
          - OrderPlaced
          - OrderCancelled
    "#;

    // When: Importing the model
    let graph = DddModelImporter::import_from_yaml(ddd_model).unwrap();

    // Then: Graph should represent the domain model
    assert_eq!(graph.metadata.domain, "OrderManagement");
    assert!(graph.find_node_by_type(NodeType::Aggregate("Order")).is_some());
    assert!(graph.find_node_by_type(NodeType::Command("PlaceOrder")).is_some());
    assert!(graph.find_edge_by_type(EdgeType::Handles).is_some());
}
```

## Graph Display and Visualization Tests

### Test: Multiple Visualization Modes
```rust
#[test]
fn test_visualization_mode_switching() {
    // Given: A graph with different node types
    let mut app = create_test_app();
    let graph = create_sample_ddd_graph();

    // When: Switching between visualization modes
    app.world.send_event(VisualizationCommand::SetMode(
        VisualizationMode::DomainFlow
    ));
    app.update();

    // Then: Domain events should be highlighted
    let highlighted = app.world.query::<&Highlighted>()
        .iter(&app.world)
        .count();
    assert!(highlighted > 0);

    // When: Switching to conceptual space view
    app.world.send_event(VisualizationCommand::SetMode(
        VisualizationMode::ConceptualSpace
    ));
    app.update();

    // Then: Nodes should be repositioned based on semantic similarity
    let positions = app.world.query::<(&GraphNode, &Transform)>()
        .iter(&app.world)
        .map(|(node, transform)| (node.node_id, transform.translation))
        .collect::<HashMap<_, _>>();

    // Verify semantic clustering
    assert_semantic_clustering(&positions, &graph);
}
```

### Test: Layered Graph Visualization
```rust
#[test]
fn test_layered_ddd_visualization() {
    // Given: A DDD graph with multiple layers
    let mut app = create_test_app();

    // Create a multi-layered DDD structure
    let layers = vec![
        Layer::Presentation,
        Layer::Application,
        Layer::Domain,
        Layer::Infrastructure,
    ];

    // When: Applying layered layout
    app.world.send_event(LayoutCommand::ApplyLayered {
        layers: layers.clone(),
        spacing: 5.0,
    });
    app.update();

    // Then: Nodes should be positioned by layer
    for (node, transform) in app.world.query::<(&GraphNode, &Transform)>().iter(&app.world) {
        let expected_y = match node.layer {
            Layer::Presentation => 15.0,
            Layer::Application => 10.0,
            Layer::Domain => 5.0,
            Layer::Infrastructure => 0.0,
        };
        assert!((transform.translation.y - expected_y).abs() < 0.1);
    }
}
```

## Graph Creation and Modeling Tests

### Test: Interactive DDD Component Creation
```rust
#[test]
fn test_create_ddd_components_interactively() {
    // Given: An empty graph canvas
    let mut app = create_test_app();

    // When: Creating an aggregate through UI
    app.world.send_event(CreateNodeCommand {
        node_type: NodeType::Aggregate("Customer"),
        position: Position3D::new(0.0, 0.0, 0.0),
        metadata: hashmap! {
            "bounded_context" => "CustomerManagement",
            "description" => "Customer aggregate root",
        },
    });
    app.update();

    // Then: Aggregate node should exist
    let aggregates = app.world.query::<&AggregateComponent>()
        .iter(&app.world)
        .count();
    assert_eq!(aggregates, 1);

    // When: Adding a command to the aggregate
    app.world.send_event(CreateNodeCommand {
        node_type: NodeType::Command("RegisterCustomer"),
        position: Position3D::new(5.0, 0.0, 0.0),
        metadata: hashmap! {
            "aggregate" => "Customer",
        },
    });

    // And: Connecting them
    app.world.send_event(ConnectNodesCommand {
        source_type: NodeType::Command("RegisterCustomer"),
        target_type: NodeType::Aggregate("Customer"),
        edge_type: EdgeType::HandledBy,
    });
    app.update();

    // Then: Command should be connected to aggregate
    let edges = app.world.query::<&GraphEdge>()
        .iter(&app.world)
        .filter(|edge| edge.edge_type == EdgeType::HandledBy)
        .count();
    assert_eq!(edges, 1);
}
```

### Test: Workflow Pattern Templates
```rust
#[test]
fn test_apply_workflow_pattern_template() {
    // Given: A graph and a saga pattern template
    let mut app = create_test_app();

    // When: Applying a saga pattern
    app.world.send_event(ApplyPatternCommand {
        pattern: WorkflowPattern::Saga {
            name: "OrderFulfillment",
            steps: vec![
                "ValidateOrder",
                "ReserveInventory",
                "ProcessPayment",
                "ShipOrder",
            ],
            compensations: vec![
                "CancelOrder",
                "ReleaseInventory",
                "RefundPayment",
                "CancelShipment",
            ],
        },
    });
    app.update();

    // Then: Saga structure should be created
    let saga_nodes = app.world.query::<&WorkflowNode>()
        .iter(&app.world)
        .filter(|node| node.workflow_type == WorkflowType::Saga)
        .count();
    assert_eq!(saga_nodes, 9); // 4 steps + 4 compensations + 1 coordinator

    // And: Compensation edges should exist
    let compensation_edges = app.world.query::<&GraphEdge>()
        .iter(&app.world)
        .filter(|edge| edge.edge_type == EdgeType::Compensates)
        .count();
    assert_eq!(compensation_edges, 4);
}
```

## Graph Combination and Morphism Tests

### Test: Compose Multiple Bounded Contexts
```rust
#[test]
fn test_compose_bounded_contexts() {
    // Given: Two separate bounded context graphs
    let order_context = create_order_management_graph();
    let inventory_context = create_inventory_management_graph();

    // When: Composing them with context mapping
    let composed = GraphComposer::compose(vec![
        (order_context, "OrderManagement"),
        (inventory_context, "InventoryManagement"),
    ])
    .with_context_map(|composer| {
        composer
            .add_shared_kernel("Product")
            .add_upstream_downstream(
                "InventoryManagement",
                "OrderManagement",
                IntegrationType::PublishedLanguage,
            )
    })
    .build();

    // Then: Composed graph should maintain context boundaries
    assert_eq!(composed.bounded_contexts.len(), 2);

    // And: Integration points should be explicit
    let integration_edges = composed.edges.iter()
        .filter(|e| e.edge_type == EdgeType::IntegratesWith)
        .count();
    assert!(integration_edges > 0);

    // And: Shared kernel should be identified
    let shared_nodes = composed.nodes.iter()
        .filter(|n| n.contexts.len() > 1)
        .count();
    assert!(shared_nodes > 0);
}
```

### Test: Apply Graph Morphisms
```rust
#[test]
fn test_apply_graph_morphisms() {
    // Given: A detailed implementation graph
    let implementation_graph = create_implementation_graph();

    // When: Applying abstraction morphism
    let abstract_graph = GraphMorphism::AbstractToPattern
        .apply(&implementation_graph)
        .unwrap();

    // Then: Abstract graph should have fewer nodes
    assert!(abstract_graph.nodes.len() < implementation_graph.nodes.len());

    // And: Pattern relationships should be preserved
    assert!(abstract_graph.find_pattern(Pattern::Repository).is_some());
    assert!(abstract_graph.find_pattern(Pattern::Factory).is_some());

    // When: Applying projection morphism
    let domain_only = GraphMorphism::ProjectToDomainLayer
        .apply(&implementation_graph)
        .unwrap();

    // Then: Only domain components should remain
    for node in &domain_only.nodes {
        assert_eq!(node.layer, Layer::Domain);
    }
}
```

## AI-Driven Development Tests

### Test: Generate Code from Domain Model
```rust
#[tokio::test]
async fn test_generate_code_from_domain_model() {
    // Given: A complete domain model graph
    let domain_graph = create_order_aggregate_graph();

    // When: AI agent generates code
    let code_generator = AiCodeGenerator::new()
        .with_language(Language::Rust)
        .with_patterns(vec![Pattern::EventSourcing, Pattern::CQRS]);

    let generated_code = code_generator
        .generate_from_graph(&domain_graph)
        .await
        .unwrap();

    // Then: Generated code should include all components
    assert!(generated_code.contains("struct Order"));
    assert!(generated_code.contains("enum OrderCommand"));
    assert!(generated_code.contains("enum OrderEvent"));
    assert!(generated_code.contains("impl CommandHandler for Order"));

    // And: Code should follow DDD patterns
    assert!(generated_code.contains("fn handle_command"));
    assert!(generated_code.contains("fn apply_event"));
}
```

### Test: AI Workflow Optimization
```rust
#[tokio::test]
async fn test_ai_workflow_optimization() {
    // Given: A workflow graph with inefficiencies
    let workflow = create_inefficient_workflow();

    // When: AI analyzes and optimizes
    let optimizer = AiWorkflowOptimizer::new()
        .with_goals(vec![
            OptimizationGoal::MinimizeLatency,
            OptimizationGoal::MaximizeThroughput,
        ]);

    let optimized = optimizer
        .optimize(&workflow)
        .await
        .unwrap();

    // Then: Optimized workflow should be more efficient
    assert!(optimized.parallel_paths() > workflow.parallel_paths());
    assert!(optimized.critical_path_length() < workflow.critical_path_length());

    // And: Business constraints should be preserved
    assert_eq!(
        optimized.business_rules_count(),
        workflow.business_rules_count()
    );
}
```

## Fitness Functions for Production Readiness

### Test: Large Graph Performance
```rust
#[test]
fn fitness_large_graph_rendering() {
    // Given: A large enterprise domain model
    let mut app = create_test_app();
    let large_graph = create_enterprise_graph(
        1000, // nodes
        5000, // edges
    );

    // When: Loading and rendering
    let start = Instant::now();
    app.world.send_event(LoadGraphCommand { graph: large_graph });

    // Measure frame time
    for _ in 0..60 { // 60 frames
        app.update();
    }
    let elapsed = start.elapsed();

    // Then: Should maintain 60 FPS
    let avg_frame_time = elapsed.as_millis() / 60;
    assert!(
        avg_frame_time < 16, // 16ms = 60 FPS
        "Frame time {}ms exceeds 16ms target",
        avg_frame_time
    );
}
```

### Test: Conceptual Space Accuracy
```rust
#[test]
fn fitness_conceptual_space_accuracy() {
    // Given: A graph with known semantic relationships
    let test_cases = vec![
        ("OrderService", "OrderRepository", 0.9), // High similarity
        ("OrderService", "UserInterface", 0.2),   // Low similarity
        ("PaymentGateway", "PaymentService", 0.8), // Related
    ];

    // When: Computing conceptual distances
    let space = ConceptualSpace::from_graph(&create_test_graph());

    // Then: Distances should match expectations
    for (node1, node2, expected_similarity) in test_cases {
        let actual = space.similarity(node1, node2);
        assert!(
            (actual - expected_similarity).abs() < 0.1,
            "{} <-> {} similarity {} differs from expected {}",
            node1, node2, actual, expected_similarity
        );
    }
}
```

### Test: AI Code Generation Quality
```rust
#[tokio::test]
async fn fitness_ai_code_quality() {
    // Given: Various domain models
    let test_models = vec![
        create_simple_crud_model(),
        create_event_sourced_model(),
        create_saga_workflow_model(),
    ];

    for model in test_models {
        // When: Generating code
        let generated = AiCodeGenerator::new()
            .generate_from_graph(&model)
            .await
            .unwrap();

        // Then: Code should compile
        let compile_result = compile_rust_code(&generated);
        assert!(
            compile_result.is_ok(),
            "Generated code failed to compile: {:?}",
            compile_result.err()
        );

        // And: Code should pass linting
        let lint_result = lint_rust_code(&generated);
        assert!(
            lint_result.warnings.is_empty(),
            "Generated code has lint warnings: {:?}",
            lint_result.warnings
        );
    }
}
```

## Test Execution Strategy

### Unit Tests (Milliseconds)
- Graph operations
- Morphism applications
- Layout algorithms
- Pattern matching

### Integration Tests (Seconds)
- Event store operations
- NATS communication
- File import/export
- Multi-graph composition

### AI Tests (Minutes)
- Code generation
- Workflow optimization
- Semantic analysis
- Pattern recognition

### Visual Tests (Manual)
- Rendering quality
- Animation smoothness
- Interaction responsiveness
- Layout aesthetics

## Success Criteria

1. **Graph Operations**: All CRUD operations < 10ms
2. **Visualization**: 60 FPS with 1000+ nodes
3. **AI Generation**: Valid code in < 30 seconds
4. **Semantic Accuracy**: 90%+ similarity matching
5. **Workflow Optimization**: 20%+ efficiency gains
6. **Memory Usage**: < 1GB for 10K node graphs
7. **Event Processing**: < 1ms per event
8. **Conceptual Mapping**: < 100ms for full graph
