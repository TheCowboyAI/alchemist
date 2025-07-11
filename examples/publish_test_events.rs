//! Publish test events to NATS for dashboard testing

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    
    println!("Connecting to NATS at: {}", nats_url);
    let client = async_nats::connect(&nats_url).await?;
    println!("✅ Connected!");
    
    let domains = vec!["workflow", "agent", "document", "policy", "graph"];
    let events = vec!["created", "updated", "deleted", "executed", "validated"];
    
    println!("Publishing test events...");
    println!("Press Ctrl+C to stop");
    
    let mut count = 0;
    loop {
        let domain = &domains[count % domains.len()];
        let event = &events[count % events.len()];
        let subject = format!("cim.{}.events.{}", domain, event);
        
        let payload = serde_json::json!({
            "id": format!("event-{}", count),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "domain": domain,
            "event": event,
            "data": {
                "test": true,
                "count": count
            }
        });
        
        client.publish(&subject, payload.to_string().into()).await?;
        println!("📤 Published: {} (event #{})", subject, count);
        
        count += 1;
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}