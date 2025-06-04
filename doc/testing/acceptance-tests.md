# Acceptance Tests for Presentation Layer

## Overview

This document defines acceptance tests for the presentation layer user stories, focusing on event-driven architecture, command processing, and Bevy ECS integration.

## Event-Driven Architecture Tests

### Test: Command Event Flow
```rust
#[test]
fn test_command_event_flow() {
    // Given: A Bevy app with GraphEditorPlugin
    let mut app = App::new();
    app.add_plugins(GraphEditorPlugin);

    // When: A command is sent through events
    let graph_id = GraphId::new();
    app.world.send_event(CommandEvent {
        command: Command::Graph(GraphCommand::CreateGraph {
            id: graph_id,
            name: "Test Graph".to_string(),
        }),
    });

    // And: The app updates
    app.update();

    // Then: An EventNotification should be generated
    let events = app.world.resource::<Events<EventNotification>>();
    let mut reader = events.get_reader();
    let notifications: Vec<_> = reader.read(events).collect();

    assert_eq!(notifications.len(), 1);
    match &notifications[0].event {
        DomainEvent::Graph(GraphEvent::GraphCreated { id, .. }) => {
            assert_eq!(*id, graph_id);
        }
        _ => panic!("Expected GraphCreated event"),
    }
}
```

### Test: Multiple Commands Per Frame
```rust
#[test]
fn test_multiple_commands_per_frame() {
    // Given: A Bevy app with command processing
    let mut app = App::new();
    app.add_plugins(GraphEditorPlugin);

    // When: Multiple commands are sent
    let graph_id1 = GraphId::new();
    let graph_id2 = GraphId::new();

    app.world.send_event(CommandEvent {
        command: Command::Graph(GraphCommand::CreateGraph {
            id: graph_id1,
            name: "Graph 1".to_string(),
        }),
    });

    app.world.send_event(CommandEvent {
        command: Command::Graph(GraphCommand::CreateGraph {
            id: graph_id2,
            name: "Graph 2".to_string(),
        }),
    });

    // And: The app updates once
    app.update();

    // Then: Both commands should be processed
    let events = app.world.resource::<Events<EventNotification>>();
    let mut reader = events.get_reader();
    let notifications: Vec<_> = reader.read(events).collect();

    assert_eq!(notifications.len(), 2);
}
```

### Test: Node Command Handler (Pending)
```rust
#[test]
#[ignore = "Node commands not yet implemented"]
fn test_node_command_processing() {
    // Given: A graph exists
    let mut app = App::new();
    app.add_plugins(GraphEditorPlugin);

    let graph_id = GraphId::new();
    let node_id = NodeId::new();

    // When: A node command is sent
    app.world.send_event(CommandEvent {
        command: Command::Node(NodeCommand::AddNode {
            graph_id,
            node_id,
            node_type: NodeType::Concept,
            position: Position3D::new(0.0, 0.0, 0.0),
        }),
    });

    app.update();

    // Then: A NodeAdded event should be generated
    let events = app.world.resource::<Events<EventNotification>>();
    let mut reader = events.get_reader();
    let notifications: Vec<_> = reader.read(events).collect();

    assert_eq!(notifications.len(), 1);
    match &notifications[0].event {
        DomainEvent::Node(NodeEvent::NodeAdded { .. }) => {}
        _ => panic!("Expected NodeAdded event"),
    }
}
```

## Presentation Layer Tests

### Test: Camera Initialization
```rust
#[test]
fn test_camera_initialization() {
    // Given: A Bevy app with GraphEditorPlugin
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(GraphEditorPlugin);

    // When: The app starts up
    app.update();

    // Then: A camera should exist
    let camera_query = app.world.query::<&Camera3d>();
    assert_eq!(camera_query.iter(&app.world).count(), 1);

    // And: The camera should be positioned correctly
    let transform_query = app.world.query::<(&Camera3d, &Transform)>();
    for (_, transform) in transform_query.iter(&app.world) {
        assert_eq!(transform.translation, Vec3::new(0.0, 5.0, 10.0));

        // Verify it's looking at the origin
        let forward = transform.forward();
        let to_origin = Vec3::ZERO - transform.translation;
        let angle = forward.angle_between(to_origin.normalize());
        assert!(angle < 0.01, "Camera should look at origin");
    }
}
```

### Test: Plugin Registration
```rust
#[test]
fn test_plugin_registers_events() {
    // Given: An empty Bevy app
    let mut app = App::new();

    // When: GraphEditorPlugin is added
    app.add_plugins(GraphEditorPlugin);

    // Then: Events should be registered
    assert!(app.world.contains_resource::<Events<CommandEvent>>());
    assert!(app.world.contains_resource::<Events<EventNotification>>());
}
```

### Test: Startup Graph Creation
```rust
#[test]
fn test_startup_graph_creation() {
    // Given: A Bevy app with the main setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(GraphEditorPlugin)
       .add_systems(Startup, setup);

    // When: The app starts
    app.update();

    // Then: A graph creation command should be sent
    let events = app.world.resource::<Events<CommandEvent>>();
    let mut reader = events.get_reader();
    let commands: Vec<_> = reader.read(events).collect();

    assert_eq!(commands.len(), 1);
    match &commands[0].command {
        Command::Graph(GraphCommand::CreateGraph { name, .. }) => {
            assert_eq!(name, "Test Graph");
        }
        _ => panic!("Expected CreateGraph command"),
    }
}
```

## NATS Integration Tests

### Test: NATS Configuration
```rust
#[test]
fn test_nats_configuration() {
    use ia::infrastructure::nats::NatsConfig;

    // Given: Default configuration
    let config = NatsConfig::default();

    // Then: Configuration should have sensible defaults
    assert_eq!(config.url, "nats://localhost:4222");
    assert!(config.jetstream_enabled);
    assert_eq!(config.connection_timeout.as_secs(), 5);
    assert_eq!(config.request_timeout.as_secs(), 10);
}
```

### Test: NATS Health Check (Integration)
```rust
#[tokio::test]
#[ignore = "Requires NATS server running"]
async fn test_nats_health_check_integration() {
    use ia::infrastructure::nats::{NatsClient, NatsConfig};

    // Given: A NATS client
    let config = NatsConfig::default();
    let client = NatsClient::connect(config).await.unwrap();

    // When: Health check is performed
    let health = client.health_check().await.unwrap();

    // Then: Health check should succeed
    assert!(health.is_healthy);
    assert!(health.latency_ms > 0.0);
    assert!(health.latency_ms < 100.0); // Should be fast locally
}
```

## Event-to-ECS Bridge Tests (Pending)

### Test: Domain Event Creates Entity
```rust
#[test]
#[ignore = "ECS entity creation not yet implemented"]
fn test_domain_event_creates_entity() {
    // Given: A Bevy app processing events
    let mut app = App::new();
    app.add_plugins(GraphEditorPlugin);

    let graph_id = GraphId::new();

    // When: A GraphCreated event is received
    app.world.send_event(EventNotification {
        event: DomainEvent::Graph(GraphEvent::GraphCreated {
            id: graph_id,
            metadata: GraphMetadata::new("Test".to_string()),
        }),
    });

    app.update();

    // Then: A graph entity should be created
    let graph_query = app.world.query::<&GraphEntity>();
    assert_eq!(graph_query.iter(&app.world).count(), 1);

    // And: The entity should have the correct ID
    for graph_entity in graph_query.iter(&app.world) {
        assert_eq!(graph_entity.graph_id, graph_id);
    }
}
```

### Test: Node Event Creates Visual
```rust
#[test]
#[ignore = "Visual entity creation not yet implemented"]
fn test_node_event_creates_visual() {
    // Given: A graph exists
    let mut app = App::new();
    app.add_plugins(GraphEditorPlugin);

    let graph_id = GraphId::new();
    let node_id = NodeId::new();

    // When: A NodeAdded event is received
    app.world.send_event(EventNotification {
        event: DomainEvent::Node(NodeEvent::NodeAdded {
            graph_id,
            node_id,
            node_type: NodeType::Concept,
            position: Position3D::new(1.0, 2.0, 3.0),
        }),
    });

    app.update();

    // Then: A visual entity should be created
    let node_query = app.world.query::<(&GraphNode, &Transform)>();
    assert_eq!(node_query.iter(&app.world).count(), 1);

    // And: It should be positioned correctly
    for (node, transform) in node_query.iter(&app.world) {
        assert_eq!(node.node_id, node_id);
        assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
    }
}
```

## Fitness Functions

### Performance: Event Processing Latency
```rust
#[test]
fn fitness_event_processing_latency() {
    use std::time::Instant;

    // Given: A Bevy app with event processing
    let mut app = App::new();
    app.add_plugins(GraphEditorPlugin);

    // When: 1000 commands are sent
    let start = Instant::now();

    for i in 0..1000 {
        app.world.send_event(CommandEvent {
            command: Command::Graph(GraphCommand::CreateGraph {
                id: GraphId::new(),
                name: format!("Graph {}", i),
            }),
        });
    }

    app.update();
    let elapsed = start.elapsed();

    // Then: Processing should complete within 100ms
    assert!(
        elapsed.as_millis() < 100,
        "Event processing took {}ms, expected < 100ms",
        elapsed.as_millis()
    );
}
```

### Memory: Event Buffer Size
```rust
#[test]
fn fitness_event_buffer_memory() {
    // Given: A Bevy app
    let mut app = App::new();
    app.add_plugins(GraphEditorPlugin);

    // When: Many events are sent without processing
    for _ in 0..10_000 {
        app.world.send_event(CommandEvent {
            command: Command::Graph(GraphCommand::CreateGraph {
                id: GraphId::new(),
                name: "Test".to_string(),
            }),
        });
    }

    // Then: The app should handle it gracefully
    // (This tests that we don't run out of memory)
    app.update();

    // Verify events were processed
    let events = app.world.resource::<Events<EventNotification>>();
    assert!(events.len() > 0);
}
```

## Test Execution Plan

### Unit Tests (Fast, No Dependencies)
- Configuration tests
- Event structure tests
- Command handler logic

### Integration Tests (Require Setup)
- NATS connectivity tests
- Full event flow tests
- Plugin integration tests

### Visual Tests (Manual Verification)
- Camera positioning
- Graph rendering
- User interactions

### Performance Tests (Benchmarks)
- Event processing throughput
- Memory usage under load
- Frame rate with large graphs
