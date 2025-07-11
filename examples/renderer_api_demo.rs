//! Renderer API demonstration
//! 
//! This example shows how the renderer API enables communication
//! between UI components using event-based patterns.

use alchemist::{
    renderer_api::{RendererApi, RendererCommand, DialogCommand},
    renderer_nats_bridge::{RendererNatsBridge, ComponentType, NatsRendererEvent},
};
use anyhow::Result;
use tracing::info;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "alchemist=info");
    }
    tracing_subscriber::fmt::init();
    
    println!("🔌 Alchemist Renderer API Demo");
    println!("=====================================");
    println!();
    println!("This demo showcases the renderer API that enables");
    println!("event-based communication between UI components.");
    println!();
    
    // Try NATS connection first
    let nats_url = std::env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());
    
    match async_nats::connect(&nats_url).await {
        Ok(client) => {
            println!("✅ Connected to NATS at {}", nats_url);
            println!();
            run_nats_demo(client).await?;
        }
        Err(e) => {
            println!("⚠️  Could not connect to NATS: {}", e);
            println!("Running local demo instead...");
            println!();
            run_local_demo().await?;
        }
    }
    
    Ok(())
}

async fn run_nats_demo(client: async_nats::Client) -> Result<()> {
    println!("=== NATS-Connected Demo ===");
    println!();
    
    // Create renderer bridge
    let mut bridge = RendererNatsBridge::new(client.clone()).await?;
    bridge.start().await?;
    
    // Register multiple components
    println!("📋 Registering UI components...");
    
    let dashboard_rx = bridge.register_component(
        "dashboard-demo".to_string(),
        ComponentType::Dashboard,
    ).await;
    
    let dialog_rx = bridge.register_component(
        "dialog-demo".to_string(),
        ComponentType::Dialog,
    ).await;
    
    let monitor_rx = bridge.register_component(
        "monitor-demo".to_string(),
        ComponentType::Monitor,
    ).await;
    
    println!("✅ Registered 3 components");
    println!();
    
    // Spawn handlers for each component
    tokio::spawn(async move {
        let mut rx = dashboard_rx;
        while let Some(event) = rx.recv().await {
            info!("Dashboard received: {:?}", event);
        }
    });
    
    tokio::spawn(async move {
        let mut rx = dialog_rx;
        while let Some(event) = rx.recv().await {
            info!("Dialog received: {:?}", event);
        }
    });
    
    tokio::spawn(async move {
        let mut rx = monitor_rx;
        while let Some(event) = rx.recv().await {
            info!("Monitor received: {:?}", event);
        }
    });
    
    // Demonstrate various events
    println!("📨 Sending events...");
    println!();
    
    // Send dialog message
    println!("1. Sending dialog message");
    bridge.send_dialog_message(
        "conv-123".to_string(),
        "user".to_string(),
        "Hello from the renderer API demo!".to_string(),
    ).await?;
    
    sleep(Duration::from_millis(500)).await;
    
    // Update dashboard
    println!("2. Updating dashboard data");
    let dashboard_data = alchemist::dashboard::DashboardData {
        event_count: 42,
        active_domains: vec!["workflow".to_string(), "document".to_string()],
        recent_events: vec![
            alchemist::dashboard::EventInfo {
                timestamp: chrono::Utc::now(),
                domain: "demo".to_string(),
                event_type: "test.event".to_string(),
                description: "Demo event from renderer API".to_string(),
            }
        ],
        system_status: alchemist::dashboard::SystemStatus {
            connected: true,
            latency_ms: 15,
            queue_depth: 3,
        },
    };
    bridge.update_dashboard(dashboard_data).await?;
    
    sleep(Duration::from_millis(500)).await;
    
    // Simulate streaming response
    println!("3. Simulating AI streaming response");
    let tokens = ["This ", "is ", "a ", "streaming ", "response ", "demo."];
    for token in &tokens {
        bridge.publish_event(
            "cim.dialog.event",
            &NatsRendererEvent::DialogStreaming {
                conversation_id: "conv-123".to_string(),
                token: token.to_string(),
            },
        ).await?;
        sleep(Duration::from_millis(100)).await;
    }
    
    bridge.publish_event(
        "cim.dialog.event",
        &NatsRendererEvent::DialogComplete {
            conversation_id: "conv-123".to_string(),
        },
    ).await?;
    
    println!();
    println!("✅ Events sent successfully");
    println!();
    
    // Keep running for a bit to show events
    sleep(Duration::from_secs(2)).await;
    
    Ok(())
}

async fn run_local_demo() -> Result<()> {
    println!("=== Local Renderer API Demo ===");
    println!();
    
    // Create renderer API
    let api = RendererApi::new();
    
    // Register renderers
    println!("📋 Registering renderers...");
    
    let dialog_rx = api.register_renderer("dialog-1".to_string());
    let dashboard_rx = api.register_renderer("dashboard-1".to_string());
    
    println!("✅ Registered 2 renderers");
    println!();
    
    // Spawn handlers
    tokio::spawn(async move {
        let mut rx = dialog_rx;
        while let Some(cmd) = rx.recv().await {
            info!("Dialog received command: {:?}", cmd);
        }
    });
    
    tokio::spawn(async move {
        let mut rx = dashboard_rx;
        while let Some(cmd) = rx.recv().await {
            info!("Dashboard received command: {:?}", cmd);
        }
    });
    
    // Send commands
    println!("📨 Sending commands...");
    println!();
    
    // Send dialog commands
    println!("1. Adding message to dialog");
    api.send_dialog_command(
        "dialog-1",
        DialogCommand::AddMessage {
            role: "user".to_string(),
            content: "Hello from the renderer API!".to_string(),
        },
    ).await?;
    
    sleep(Duration::from_millis(200)).await;
    
    println!("2. Setting dialog loading state");
    api.send_dialog_command(
        "dialog-1",
        DialogCommand::SetLoading { loading: true },
    ).await?;
    
    sleep(Duration::from_millis(200)).await;
    
    // Stream tokens
    println!("3. Streaming response tokens");
    let tokens = ["This ", "is ", "a ", "local ", "demo."];
    for token in &tokens {
        api.send_dialog_command(
            "dialog-1",
            DialogCommand::StreamToken {
                token: token.to_string(),
            },
        ).await?;
        sleep(Duration::from_millis(100)).await;
    }
    
    api.send_dialog_command(
        "dialog-1",
        DialogCommand::CompleteStream,
    ).await?;
    
    api.send_dialog_command(
        "dialog-1",
        DialogCommand::SetLoading { loading: false },
    ).await?;
    
    sleep(Duration::from_millis(200)).await;
    
    // Update dashboard
    println!("4. Updating dashboard data");
    api.send_command(
        "dashboard-1",
        RendererCommand::UpdateData {
            data: serde_json::json!({
                "event_count": 10,
                "status": "active",
                "message": "Local renderer API demo"
            }),
        },
    ).await?;
    
    println!();
    println!("✅ Commands sent successfully");
    println!();
    
    // Keep running for a bit
    sleep(Duration::from_secs(1)).await;
    
    Ok(())
}