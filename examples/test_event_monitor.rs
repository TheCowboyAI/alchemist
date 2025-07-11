//! Test script for event monitoring system

use anyhow::Result;
use async_nats;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Event Monitor Test Script");
    println!("========================");
    
    // Connect to NATS
    let nats_url = "nats://localhost:4222";
    println!("Connecting to NATS at {}...", nats_url);
    
    let client = match async_nats::connect(nats_url).await {
        Ok(c) => {
            println!("Connected to NATS successfully!");
            c
        }
        Err(e) => {
            eprintln!("Failed to connect to NATS: {}", e);
            eprintln!("Make sure NATS is running (docker-compose up -d nats)");
            return Err(e.into());
        }
    };
    
    // Send various test events
    println!("\nSending test events...");
    
    // Workflow events
    for i in 1..=5 {
        let event = json!({
            "id": format!("workflow-{}", i),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "severity": if i % 2 == 0 { "info" } else { "warning" },
            "correlation_id": format!("corr-{}", i / 2),
            "payload": {
                "workflow_id": format!("wf-{}", i),
                "status": if i % 2 == 0 { "started" } else { "completed" },
                "step": i,
            }
        });
        
        client.publish(
            format!("alchemist.workflow.{}", if i % 2 == 0 { "started" } else { "completed" }),
            event.to_string().into()
        ).await?;
        
        println!("  Sent workflow event {}", i);
        sleep(Duration::from_millis(100)).await;
    }
    
    // AI events
    for i in 1..=3 {
        let event = json!({
            "id": format!("ai-{}", i),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "severity": "info",
            "correlation_id": format!("ai-corr-{}", i),
            "payload": {
                "model": "gpt-4",
                "action": if i == 1 { "request" } else if i == 2 { "response" } else { "error" },
                "tokens": i * 100,
            }
        });
        
        client.publish(
            format!("alchemist.ai.{}", if i == 1 { "request" } else if i == 2 { "response" } else { "error" }),
            event.to_string().into()
        ).await?;
        
        println!("  Sent AI event {}", i);
        sleep(Duration::from_millis(200)).await;
    }
    
    // Error events
    for i in 1..=2 {
        let event = json!({
            "id": format!("error-{}", i),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "severity": if i == 1 { "error" } else { "critical" },
            "payload": {
                "error": format!("Test error {}", i),
                "component": if i == 1 { "policy" } else { "deployment" },
                "stack_trace": "at test_event_monitor.rs:123",
            }
        });
        
        client.publish(
            format!("alchemist.{}.error", if i == 1 { "policy" } else { "deployment" }),
            event.to_string().into()
        ).await?;
        
        println!("  Sent error event {}", i);
        sleep(Duration::from_millis(300)).await;
    }
    
    println!("\nAll test events sent!");
    println!("\nTo view events, run:");
    println!("  alchemist event list");
    println!("  alchemist event stats");
    println!("  alchemist event query \"severity:error OR severity:critical\"");
    
    Ok(())
}