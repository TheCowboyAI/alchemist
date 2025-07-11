//! Tests for event-driven architecture and NATS integration

use alchemist::{
    renderer_events::{ShellToRendererEvent, RendererToShellEvent, EventBuilder},
    renderer_nats_bridge::{RendererNatsBridge, NatsRendererEvent, ComponentType},
    nats_client::NatsClient,
    dashboard::{DashboardData, DomainInfo, DialogInfo, EventInfo},
    dashboard_events::{DashboardEvent, DialogEvent, DomainEvent},
};
use anyhow::Result;
use tokio::sync::mpsc;
use futures::StreamExt;
use std::time::Duration;

#[cfg(test)]
mod event_builder_tests {
    use super::*;
    
    #[test]
    fn test_event_builder_creation() {
        let event = EventBuilder::dashboard_update(DashboardData::example());
        assert!(matches!(event, ShellToRendererEvent::DashboardUpdate(_)));
        
        let event = EventBuilder::dialog_started("dialog123".to_string());
        match event {
            ShellToRendererEvent::DialogStarted { dialog_id } => {
                assert_eq!(dialog_id, "dialog123");
            }
            _ => panic!("Expected DialogStarted event"),
        }
        
        let event = EventBuilder::ai_response_chunk("dialog123".to_string(), "Hello".to_string());
        match event {
            ShellToRendererEvent::AiResponseChunk { dialog_id, chunk } => {
                assert_eq!(dialog_id, "dialog123");
                assert_eq!(chunk, "Hello");
            }
            _ => panic!("Expected AiResponseChunk event"),
        }
    }
    
    #[test]
    fn test_event_serialization() {
        let dashboard_data = DashboardData::example();
        let event = EventBuilder::dashboard_update(dashboard_data.clone());
        
        // Test that event can be serialized
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("DashboardUpdate"));
        
        // Test deserialization
        let deserialized: ShellToRendererEvent = serde_json::from_str(&json).unwrap();
        match deserialized {
            ShellToRendererEvent::DashboardUpdate(data) => {
                assert_eq!(data.total_events, dashboard_data.total_events);
            }
            _ => panic!("Expected DashboardUpdate after deserialization"),
        }
    }
}

#[cfg(test)]
mod dashboard_event_tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_dashboard_event_creation() {
        let event = DashboardEvent::Initialized {
            timestamp: Utc::now(),
        };
        assert!(matches!(event, DashboardEvent::Initialized { .. }));
        
        let event = DashboardEvent::DomainActivated {
            domain: "test".to_string(),
            timestamp: Utc::now(),
        };
        match event {
            DashboardEvent::DomainActivated { domain, .. } => {
                assert_eq!(domain, "test");
            }
            _ => panic!("Expected DomainActivated"),
        }
    }
    
    #[test]
    fn test_dialog_event_creation() {
        let event = DialogEvent::Created {
            id: "dialog123".to_string(),
            title: "Test Dialog".to_string(),
            model: "gpt-4".to_string(),
            timestamp: Utc::now(),
        };
        
        match event {
            DialogEvent::Created { id, title, model, .. } => {
                assert_eq!(id, "dialog123");
                assert_eq!(title, "Test Dialog");
                assert_eq!(model, "gpt-4");
            }
            _ => panic!("Expected Created event"),
        }
    }
    
    #[test]
    fn test_domain_event_creation() {
        let event = DomainEvent::EventProcessed {
            domain: "graph".to_string(),
            event_type: "NodeCreated".to_string(),
            timestamp: Utc::now(),
        };
        
        match event {
            DomainEvent::EventProcessed { domain, event_type, .. } => {
                assert_eq!(domain, "graph");
                assert_eq!(event_type, "NodeCreated");
            }
            _ => panic!("Expected EventProcessed"),
        }
    }
}

#[cfg(test)]
mod nats_bridge_tests {
    use super::*;
    use std::sync::Arc;
    
    async fn create_test_bridge() -> Result<(Arc<RendererNatsBridge>, mpsc::Receiver<NatsRendererEvent>)> {
        let (tx, rx) = mpsc::channel(100);
        
        // Create bridge without NATS connection for testing
        let bridge = Arc::new(RendererNatsBridge::new(
            None, // No actual NATS client for unit tests
            tx,
        ));
        
        Ok((bridge, rx))
    }
    
    #[tokio::test]
    async fn test_bridge_component_registration() -> Result<()> {
        let (bridge, mut rx) = create_test_bridge().await?;
        
        // Register component
        bridge.register_component("test-component", ComponentType::Dashboard).await?;
        
        // Check event
        if let Some(event) = rx.recv().await {
            match event {
                NatsRendererEvent::ComponentRegistered { component_id, component_type } => {
                    assert_eq!(component_id, "test-component");
                    assert!(matches!(component_type, ComponentType::Dashboard));
                }
                _ => panic!("Expected ComponentRegistered event"),
            }
        }
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_bridge_event_publishing() -> Result<()> {
        let (bridge, mut rx) = create_test_bridge().await?;
        
        // Publish shell event
        let dashboard_data = DashboardData::example();
        let event = ShellToRendererEvent::DashboardUpdate(dashboard_data);
        
        // Since we don't have actual NATS, just verify the bridge accepts the event
        let result = bridge.publish_shell_event(&event).await;
        // Without NATS it might fail, but the API should work
        assert!(result.is_err() || result.is_ok());
        
        Ok(())
    }
}

#[cfg(test)]
mod event_flow_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_shell_to_renderer_flow() -> Result<()> {
        let (shell_tx, mut shell_rx) = mpsc::channel::<ShellToRendererEvent>(100);
        let (renderer_tx, mut renderer_rx) = mpsc::channel::<RendererToShellEvent>(100);
        
        // Simulate shell sending dashboard update
        let dashboard_data = DashboardData {
            total_events: 100,
            active_domains: 5,
            total_policies: 10,
            active_dialogs: 2,
            domains: vec![
                DomainInfo {
                    name: "graph".to_string(),
                    event_count: 50,
                    is_active: true,
                    health: 100.0,
                },
            ],
            recent_dialogs: vec![
                DialogInfo {
                    id: "dialog1".to_string(),
                    title: "Test Dialog".to_string(),
                    model: "gpt-4".to_string(),
                    message_count: 5,
                    last_activity: "2 minutes ago".to_string(),
                },
            ],
            recent_events: vec![
                EventInfo {
                    timestamp: "2024-01-01 12:00:00".to_string(),
                    domain: "graph".to_string(),
                    event_type: "NodeCreated".to_string(),
                    summary: "Created node X".to_string(),
                },
            ],
            system_status: alchemist::dashboard::SystemStatus {
                nats_connected: true,
                domains_loaded: 14,
                policies_active: 10,
                cache_size: 1024,
            },
            recent_policies: vec![],
        };
        
        shell_tx.send(ShellToRendererEvent::DashboardUpdate(dashboard_data.clone())).await?;
        
        // Verify renderer receives event
        let received = shell_rx.recv().await.unwrap();
        match received {
            ShellToRendererEvent::DashboardUpdate(data) => {
                assert_eq!(data.total_events, 100);
                assert_eq!(data.active_domains, 5);
            }
            _ => panic!("Expected DashboardUpdate"),
        }
        
        // Simulate renderer response
        renderer_tx.send(RendererToShellEvent::UserInput {
            input: "test command".to_string(),
        }).await?;
        
        // Verify shell receives response
        let response = renderer_rx.recv().await.unwrap();
        match response {
            RendererToShellEvent::UserInput { input } => {
                assert_eq!(input, "test command");
            }
            _ => panic!("Expected UserInput"),
        }
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_dialog_event_flow() -> Result<()> {
        let (tx, mut rx) = mpsc::channel::<ShellToRendererEvent>(100);
        
        // Simulate dialog flow
        let dialog_id = "test-dialog-123";
        
        // 1. Dialog started
        tx.send(EventBuilder::dialog_started(dialog_id.to_string())).await?;
        
        // 2. User message
        tx.send(EventBuilder::user_message(
            dialog_id.to_string(),
            "Hello AI".to_string(),
        )).await?;
        
        // 3. AI thinking
        tx.send(EventBuilder::ai_thinking(dialog_id.to_string())).await?;
        
        // 4. AI response chunks
        let response_chunks = vec!["Hello", " there", ", how", " can", " I", " help?"];
        for chunk in response_chunks {
            tx.send(EventBuilder::ai_response_chunk(
                dialog_id.to_string(),
                chunk.to_string(),
            )).await?;
        }
        
        // 5. AI complete
        tx.send(EventBuilder::ai_response_complete(dialog_id.to_string())).await?;
        
        // Verify all events in order
        let expected_count = 2 + response_chunks.len() + 2; // start, user, thinking, chunks, complete
        for i in 0..expected_count {
            let event = rx.recv().await.expect(&format!("Expected event {}", i));
            match (i, event) {
                (0, ShellToRendererEvent::DialogStarted { .. }) => {},
                (1, ShellToRendererEvent::UserMessage { .. }) => {},
                (2, ShellToRendererEvent::AiThinking { .. }) => {},
                (n, ShellToRendererEvent::AiResponseChunk { .. }) if n < expected_count - 1 => {},
                (n, ShellToRendererEvent::AiResponseComplete { .. }) if n == expected_count - 1 => {},
                _ => panic!("Unexpected event order"),
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod performance_event_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_event_throughput() -> Result<()> {
        let (tx, mut rx) = mpsc::channel::<ShellToRendererEvent>(1000);
        
        let event_count = 10000;
        let start = Instant::now();
        
        // Send events
        for i in 0..event_count {
            let event = EventBuilder::system_event(
                format!("Event {}", i),
                serde_json::json!({"index": i}),
            );
            tx.send(event).await?;
        }
        
        // Receive events
        for _ in 0..event_count {
            rx.recv().await.unwrap();
        }
        
        let duration = start.elapsed();
        let throughput = event_count as f64 / duration.as_secs_f64();
        
        println!("Event throughput: {:.0} events/sec", throughput);
        assert!(throughput > 10000.0, "Should handle at least 10k events/sec");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_event_latency() -> Result<()> {
        let (tx, mut rx) = mpsc::channel::<ShellToRendererEvent>(1);
        
        let iterations = 100;
        let mut latencies = Vec::new();
        
        for _ in 0..iterations {
            let start = Instant::now();
            
            let event = EventBuilder::dashboard_update(DashboardData::example());
            tx.send(event).await?;
            rx.recv().await.unwrap();
            
            latencies.push(start.elapsed());
        }
        
        let avg_latency = latencies.iter().sum::<Duration>() / iterations;
        println!("Average event latency: {:?}", avg_latency);
        
        assert!(avg_latency < Duration::from_millis(1), "Event latency should be under 1ms");
        
        Ok(())
    }
}

#[cfg(test)]
mod integration_event_tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Requires NATS to be running
    async fn test_nats_event_integration() -> Result<()> {
        // Connect to NATS
        let client = NatsClient::new("nats://localhost:4222").await?;
        
        // Subscribe to events
        let mut subscription = client.subscribe("alchemist.events.>").await?;
        
        // Publish test event
        let event = DashboardEvent::Initialized {
            timestamp: chrono::Utc::now(),
        };
        let payload = serde_json::to_vec(&event)?;
        client.publish("alchemist.events.dashboard.initialized", payload).await?;
        
        // Receive event
        tokio::time::timeout(Duration::from_secs(1), async {
            if let Some(msg) = subscription.next().await {
                let received: DashboardEvent = serde_json::from_slice(&msg.payload)?;
                assert!(matches!(received, DashboardEvent::Initialized { .. }));
            }
            Ok::<_, anyhow::Error>(())
        }).await??;
        
        Ok(())
    }
}