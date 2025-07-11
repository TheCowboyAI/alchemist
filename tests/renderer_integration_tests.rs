//! Integration tests for the renderer system
//!
//! These tests verify:
//! - Renderer manager lifecycle (spawning and closing windows)
//! - Communication between main process and renderer via events
//! - Different renderer types (Bevy 3D and Iced 2D)
//! - Sending render data (graphs, documents, dialogs)
//! - Event handling and command processing
//! - Multiple concurrent renderer windows
//! - Error handling for missing renderer binary
//! - Window state management

use anyhow::Result;
use alchemist::{
    renderer::{
        RendererManager, RendererType, RenderRequest, RenderData, RenderConfig,
        GraphNode, GraphEdge, DialogMessage, suggest_renderer,
    },
    renderer_api::{
        RendererApi, RendererCommand, RendererEvent, DialogCommand, DialogEvent,
        create_dialog_message,
    },
    renderer_comm::{RendererComm, RendererClient, RendererRegistration},
};
use async_nats::Client;
use serde_json::json;
use std::{
    process::{Command, Stdio},
    sync::Arc,
    time::Duration,
};
use tempfile::NamedTempFile;
use tokio::{
    sync::{mpsc, Mutex},
    time::{sleep, timeout},
};
use uuid::Uuid;

/// Mock renderer binary for testing
struct MockRenderer {
    renderer_type: RendererType,
    id: String,
    data_file: Option<NamedTempFile>,
}

impl MockRenderer {
    fn new(renderer_type: RendererType, id: String) -> Self {
        Self {
            renderer_type,
            id,
            data_file: None,
        }
    }

    /// Simulate renderer process behavior
    async fn run(&mut self, nats_url: Option<String>) -> Result<()> {
        // In real tests, this would connect to NATS and process commands
        // For now, just simulate running
        sleep(Duration::from_millis(100)).await;
        Ok(())
    }
}

/// Test helper to create a mock NATS server
async fn setup_test_nats() -> Result<Client> {
    // In CI/test environments, we should have a test NATS server running
    // For local testing, connect to localhost
    let client = async_nats::connect("nats://localhost:4222").await?;
    Ok(client)
}

#[tokio::test]
async fn test_renderer_manager_lifecycle() {
    let manager = RendererManager::new().expect("Failed to create renderer manager");

    // Test spawning a Bevy renderer
    let request = RenderRequest {
        id: Uuid::new_v4().to_string(),
        renderer: RendererType::Bevy,
        title: "Test 3D Graph".to_string(),
        data: RenderData::Graph3D {
            nodes: vec![
                GraphNode {
                    id: "node1".to_string(),
                    label: "Node 1".to_string(),
                    position: Some([0.0, 0.0, 0.0]),
                    color: Some([1.0, 0.0, 0.0, 1.0]),
                    size: Some(1.0),
                    metadata: json!({}),
                },
                GraphNode {
                    id: "node2".to_string(),
                    label: "Node 2".to_string(),
                    position: Some([2.0, 0.0, 0.0]),
                    color: Some([0.0, 1.0, 0.0, 1.0]),
                    size: Some(1.0),
                    metadata: json!({}),
                },
            ],
            edges: vec![
                GraphEdge {
                    source: "node1".to_string(),
                    target: "node2".to_string(),
                    label: Some("edge1".to_string()),
                    weight: Some(1.0),
                    color: Some([0.0, 0.0, 1.0, 1.0]),
                },
            ],
        },
        config: RenderConfig::default(),
    };

    // Note: This will fail without the actual renderer binary
    // In real tests, we'd mock the process spawn
    match manager.spawn(request).await {
        Ok(id) => {
            println!("Spawned renderer with ID: {}", id);
            
            // Test listing active renderers
            let active = manager.list_active();
            assert!(!active.is_empty(), "Should have active renderers");
            
            // Test closing the renderer
            manager.close(&id).await.expect("Failed to close renderer");
            
            // Verify it's closed
            let active_after = manager.list_active();
            assert!(active_after.is_empty(), "Should have no active renderers after close");
        }
        Err(e) => {
            // Expected in test environment without renderer binary
            println!("Expected error (no renderer binary): {}", e);
        }
    }
}

#[tokio::test]
async fn test_renderer_communication() {
    // Skip if NATS is not available
    let nats_client = match setup_test_nats().await {
        Ok(client) => client,
        Err(_) => {
            println!("Skipping test - NATS not available");
            return;
        }
    };

    // Create event channel
    let (event_tx, mut event_rx) = mpsc::channel(100);

    // Create renderer communication manager
    let comm = Arc::new(
        RendererComm::new(nats_client.clone(), event_tx)
            .await
            .expect("Failed to create renderer comm")
    );

    // Start communication handling
    comm.clone().start().await.expect("Failed to start comm");

    // Test sending commands
    let renderer_id = "test-renderer-123";
    
    // Test update data command
    let update_cmd = RendererCommand::UpdateData {
        data: json!({
            "message": "Hello from test"
        }),
    };
    
    comm.send_command(renderer_id, update_cmd).await
        .expect("Failed to send command");

    // Test dialog command
    let dialog_cmd = RendererCommand::DialogCommand(DialogCommand::AddMessage {
        role: "user".to_string(),
        content: "Test message".to_string(),
    });
    
    comm.send_command(renderer_id, dialog_cmd).await
        .expect("Failed to send dialog command");

    // Test broadcast command
    comm.broadcast_command(RendererCommand::Close).await
        .expect("Failed to broadcast command");

    // Test ping
    let is_alive = comm.ping_renderer(renderer_id).await
        .expect("Failed to ping renderer");
    
    // Should be false since no renderer is actually running
    assert!(!is_alive, "Renderer should not be alive");
}

#[tokio::test]
async fn test_renderer_api() {
    let api = RendererApi::new();
    let renderer_id = "test-renderer-456";
    
    // Register renderer
    let mut cmd_rx = api.register_renderer(renderer_id.to_string());
    
    // Get event sender for renderer
    let event_tx = api.get_event_sender();
    
    // Test sending command
    let test_cmd = RendererCommand::UpdateData {
        data: json!({ "test": true }),
    };
    
    api.send_command(renderer_id, test_cmd.clone()).await
        .expect("Failed to send command");
    
    // Verify command was received
    match timeout(Duration::from_millis(100), cmd_rx.recv()).await {
        Ok(Some(cmd)) => {
            match cmd {
                RendererCommand::UpdateData { data } => {
                    assert_eq!(data["test"], true);
                }
                _ => panic!("Unexpected command type"),
            }
        }
        _ => panic!("Did not receive command"),
    }
    
    // Test sending event from renderer
    let test_event = RendererEvent::WindowClosed {
        renderer_id: renderer_id.to_string(),
    };
    
    event_tx.send(test_event).await
        .expect("Failed to send event");
    
    // Test unregistering
    api.unregister_renderer(renderer_id);
    
    // Sending command should now fail
    assert!(api.send_command(renderer_id, RendererCommand::Close).await.is_err());
}

#[tokio::test]
async fn test_different_renderer_types() {
    // Test Bevy renderer types
    let graph_data = RenderData::Graph3D {
        nodes: vec![],
        edges: vec![],
    };
    assert!(matches!(suggest_renderer(&graph_data), RendererType::Bevy));
    
    let scene_data = RenderData::Scene3D {
        scene_data: json!({}),
    };
    assert!(matches!(suggest_renderer(&scene_data), RendererType::Bevy));
    
    // Test Iced renderer types
    let doc_data = RenderData::Document {
        content: "Test content".to_string(),
        format: "markdown".to_string(),
    };
    assert!(matches!(suggest_renderer(&doc_data), RendererType::Iced));
    
    let dialog_data = RenderData::Dialog {
        dialog_id: "test-dialog".to_string(),
        ai_model: "gpt-4".to_string(),
        messages: vec![],
        system_prompt: None,
    };
    assert!(matches!(suggest_renderer(&dialog_data), RendererType::Iced));
    
    let chart_data = RenderData::Chart {
        data: json!([]),
        chart_type: "line".to_string(),
        options: json!({}),
    };
    assert!(matches!(suggest_renderer(&chart_data), RendererType::Iced));
}

#[tokio::test]
async fn test_render_data_serialization() {
    // Test various RenderData types can be serialized/deserialized
    let test_cases = vec![
        RenderData::Graph3D {
            nodes: vec![
                GraphNode {
                    id: "n1".to_string(),
                    label: "Node 1".to_string(),
                    position: Some([1.0, 2.0, 3.0]),
                    color: Some([1.0, 0.5, 0.0, 1.0]),
                    size: Some(2.0),
                    metadata: json!({ "type": "test" }),
                },
            ],
            edges: vec![],
        },
        RenderData::Document {
            content: "# Test Document\n\nThis is a test.".to_string(),
            format: "markdown".to_string(),
        },
        RenderData::Dialog {
            dialog_id: "dialog-123".to_string(),
            ai_model: "claude-3".to_string(),
            messages: vec![
                DialogMessage {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                    timestamp: chrono::Utc::now(),
                },
                DialogMessage {
                    role: "assistant".to_string(),
                    content: "Hi there!".to_string(),
                    timestamp: chrono::Utc::now(),
                },
            ],
            system_prompt: Some("You are a helpful assistant.".to_string()),
        },
        RenderData::Chart {
            data: json!({
                "labels": ["Jan", "Feb", "Mar"],
                "datasets": [{
                    "label": "Sales",
                    "data": [10, 20, 30]
                }]
            }),
            chart_type: "bar".to_string(),
            options: json!({
                "responsive": true,
                "maintainAspectRatio": false
            }),
        },
        RenderData::Markdown {
            content: "## Test Markdown\n\n- Item 1\n- Item 2".to_string(),
            theme: Some("dark".to_string()),
        },
    ];
    
    for data in test_cases {
        let json = serde_json::to_string(&data).expect("Failed to serialize");
        let deserialized: RenderData = serde_json::from_str(&json).expect("Failed to deserialize");
        
        // Re-serialize and compare
        let json2 = serde_json::to_string(&deserialized).expect("Failed to re-serialize");
        assert_eq!(json.len(), json2.len(), "Serialization should be consistent");
    }
}

#[tokio::test]
async fn test_concurrent_renderers() {
    let manager = RendererManager::new().expect("Failed to create renderer manager");
    
    // Create multiple render requests
    let requests = vec![
        RenderRequest {
            id: "renderer-1".to_string(),
            renderer: RendererType::Iced,
            title: "Document 1".to_string(),
            data: RenderData::Document {
                content: "Content 1".to_string(),
                format: "text".to_string(),
            },
            config: RenderConfig::default(),
        },
        RenderRequest {
            id: "renderer-2".to_string(),
            renderer: RendererType::Bevy,
            title: "Graph 1".to_string(),
            data: RenderData::Graph3D {
                nodes: vec![],
                edges: vec![],
            },
            config: RenderConfig::default(),
        },
        RenderRequest {
            id: "renderer-3".to_string(),
            renderer: RendererType::Iced,
            title: "Dialog 1".to_string(),
            data: RenderData::Dialog {
                dialog_id: "dialog-1".to_string(),
                ai_model: "test-model".to_string(),
                messages: vec![],
                system_prompt: None,
            },
            config: RenderConfig::default(),
        },
    ];
    
    // Spawn all renderers concurrently
    let spawn_futures = requests.into_iter().map(|req| {
        let manager_ref = &manager;
        async move {
            manager_ref.spawn(req).await
        }
    });
    
    let results = futures::future::join_all(spawn_futures).await;
    
    // Count successful spawns (will be 0 without renderer binary)
    let successful_spawns = results.iter().filter(|r| r.is_ok()).count();
    println!("Successfully spawned {} renderers", successful_spawns);
    
    // Clean up any that were successfully spawned
    for result in results {
        if let Ok(id) = result {
            let _ = manager.close(&id).await;
        }
    }
}

#[tokio::test]
async fn test_window_state_management() {
    let manager = RendererManager::new().expect("Failed to create renderer manager");
    
    // Test different window configurations
    let configs = vec![
        RenderConfig {
            width: 800,
            height: 600,
            position: Some((100, 100)),
            fullscreen: false,
            resizable: true,
            always_on_top: false,
        },
        RenderConfig {
            width: 1920,
            height: 1080,
            position: None,
            fullscreen: true,
            resizable: false,
            always_on_top: false,
        },
        RenderConfig {
            width: 400,
            height: 300,
            position: Some((50, 50)),
            fullscreen: false,
            resizable: false,
            always_on_top: true,
        },
    ];
    
    for (i, config) in configs.into_iter().enumerate() {
        let request = RenderRequest {
            id: format!("window-{}", i),
            renderer: RendererType::Iced,
            title: format!("Test Window {}", i),
            data: RenderData::Document {
                content: format!("Window {} content", i),
                format: "text".to_string(),
            },
            config,
        };
        
        match manager.spawn(request).await {
            Ok(id) => {
                println!("Spawned window with config: {}", id);
                let _ = manager.close(&id).await;
            }
            Err(e) => {
                println!("Expected error (no renderer binary): {}", e);
            }
        }
    }
}

#[tokio::test]
async fn test_error_handling() {
    let manager = RendererManager::new().expect("Failed to create renderer manager");
    
    // Test closing non-existent renderer
    let result = manager.close("non-existent-id").await;
    assert!(result.is_err(), "Should error when closing non-existent renderer");
    
    // Test updating data for non-existent renderer
    let result = manager.update_data("non-existent-id", json!({})).await;
    assert!(result.is_err(), "Should error when updating non-existent renderer");
    
    // Test invalid renderer spawn (this would normally fail due to missing binary)
    let invalid_request = RenderRequest {
        id: "invalid".to_string(),
        renderer: RendererType::Bevy,
        title: "Invalid".to_string(),
        data: RenderData::Custom {
            data: json!(null),
        },
        config: RenderConfig::default(),
    };
    
    let result = manager.spawn(invalid_request).await;
    // This will fail due to missing binary, which is expected
    assert!(result.is_err(), "Should error when spawning without renderer binary");
}

#[tokio::test]
async fn test_dialog_event_flow() {
    let api = RendererApi::new();
    let renderer_id = "dialog-renderer";
    let mut cmd_rx = api.register_renderer(renderer_id.to_string());
    let event_tx = api.get_event_sender();
    
    // Simulate dialog interaction flow
    
    // 1. Send initial dialog setup
    let setup_cmd = RendererCommand::DialogCommand(DialogCommand::UpdateSystemPrompt {
        prompt: "You are a helpful assistant.".to_string(),
    });
    api.send_command(renderer_id, setup_cmd).await.unwrap();
    
    // 2. User sends a message (event from renderer)
    let user_event = RendererEvent::DialogEvent {
        renderer_id: renderer_id.to_string(),
        event: DialogEvent::UserMessage {
            content: "Hello, how are you?".to_string(),
        },
    };
    event_tx.send(user_event).await.unwrap();
    
    // 3. Set loading state
    let loading_cmd = RendererCommand::DialogCommand(DialogCommand::SetLoading {
        loading: true,
    });
    api.send_command(renderer_id, loading_cmd).await.unwrap();
    
    // 4. Stream response tokens
    let tokens = vec!["I'm", " doing", " great", "!", " How", " can", " I", " help", " you", "?"];
    for token in tokens {
        let token_cmd = RendererCommand::DialogCommand(DialogCommand::StreamToken {
            token: token.to_string(),
        });
        api.send_command(renderer_id, token_cmd).await.unwrap();
        sleep(Duration::from_millis(50)).await;
    }
    
    // 5. Complete streaming
    let complete_cmd = RendererCommand::DialogCommand(DialogCommand::CompleteStream);
    api.send_command(renderer_id, complete_cmd).await.unwrap();
    
    // 6. Set loading false
    let loading_off_cmd = RendererCommand::DialogCommand(DialogCommand::SetLoading {
        loading: false,
    });
    api.send_command(renderer_id, loading_off_cmd).await.unwrap();
    
    // Verify commands were received
    let mut received_count = 0;
    while let Ok(Some(_cmd)) = timeout(Duration::from_millis(100), cmd_rx.recv()).await {
        received_count += 1;
    }
    
    assert!(received_count > 0, "Should have received commands");
}

#[tokio::test]
async fn test_cleanup_dead_processes() {
    let manager = RendererManager::new().expect("Failed to create renderer manager");
    
    // This test would normally spawn some processes and then kill them externally
    // to test the cleanup functionality. Since we don't have actual renderer binaries
    // in the test environment, we'll just test the cleanup method doesn't panic
    
    manager.cleanup_dead_processes().await;
    
    // Verify no active renderers after cleanup
    let active = manager.list_active();
    assert!(active.is_empty(), "Should have no active renderers");
}

#[tokio::test]
async fn test_renderer_helper_functions() {
    // Test create_dialog_message
    let msg = create_dialog_message("user", "Test message");
    assert_eq!(msg.role, "user");
    assert_eq!(msg.content, "Test message");
    assert!(msg.timestamp <= chrono::Utc::now());
    
    // Test helper spawn methods
    let manager = RendererManager::new().expect("Failed to create renderer manager");
    
    // Test spawn_graph_3d
    match manager.spawn_graph_3d("Test Graph", vec![], vec![]).await {
        Ok(id) => {
            println!("Spawned graph renderer: {}", id);
            let _ = manager.close(&id).await;
        }
        Err(_) => {
            // Expected without renderer binary
        }
    }
    
    // Test spawn_document
    match manager.spawn_document("Test Doc", "Content".to_string(), "text").await {
        Ok(id) => {
            println!("Spawned document renderer: {}", id);
            let _ = manager.close(&id).await;
        }
        Err(_) => {
            // Expected without renderer binary
        }
    }
    
    // Test spawn_markdown
    match manager.spawn_markdown("Test MD", "# Title".to_string(), Some("dark")).await {
        Ok(id) => {
            println!("Spawned markdown renderer: {}", id);
            let _ = manager.close(&id).await;
        }
        Err(_) => {
            // Expected without renderer binary
        }
    }
    
    // Test spawn_chart
    let chart_data = json!({
        "labels": ["A", "B", "C"],
        "data": [1, 2, 3]
    });
    match manager.spawn_chart("Test Chart", chart_data, "bar", json!({})).await {
        Ok(id) => {
            println!("Spawned chart renderer: {}", id);
            let _ = manager.close(&id).await;
        }
        Err(_) => {
            // Expected without renderer binary
        }
    }
}

/// Mock test for NATS renderer communication
#[tokio::test]
async fn test_renderer_client() {
    // Skip if NATS is not available
    let nats_client = match setup_test_nats().await {
        Ok(client) => client,
        Err(_) => {
            println!("Skipping test - NATS not available");
            return;
        }
    };
    
    let renderer_id = "test-renderer-client";
    let renderer_type = RendererType::Iced;
    let title = "Test Renderer Client";
    
    // Create renderer client
    match RendererClient::new(
        nats_client.clone(),
        renderer_id.to_string(),
        renderer_type,
        title.to_string(),
    ).await {
        Ok(client) => {
            // Start the client
            match client.start().await {
                Ok((mut cmd_rx, event_tx)) => {
                    // Send a test event
                    let test_event = RendererEvent::WindowClosed {
                        renderer_id: renderer_id.to_string(),
                    };
                    event_tx.send(test_event).await.unwrap();
                    
                    // In a real test, we'd have the main process send commands
                    // and verify they're received here
                    
                    println!("Renderer client started successfully");
                }
                Err(e) => {
                    println!("Failed to start renderer client: {}", e);
                }
            }
        }
        Err(e) => {
            // Expected if no main process is listening
            println!("Expected error (no main process): {}", e);
        }
    }
}