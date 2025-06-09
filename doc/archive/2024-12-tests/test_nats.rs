//! Simple NATS connectivity test

use anyhow::Result;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Testing NATS connectivity...");

    // Try to connect to NATS
    match async_nats::connect("nats://localhost:4222").await {
        Ok(client) => {
            println!("✓ Successfully connected to NATS at localhost:4222");

            // Try to get JetStream context
            match async_nats::jetstream::new(client.clone()) {
                jetstream => {
                    println!("✓ JetStream context created");

                    // Try to create a test object store
                    match jetstream.create_object_store(async_nats::jetstream::object_store::Config {
                        bucket: "test-connectivity".to_string(),
                        description: Some("Test connectivity bucket".to_string()),
                        ..Default::default()
                    }).await {
                        Ok(_) => {
                            println!("✓ Created test object store");
                            // Clean up
                            let _ = jetstream.delete_object_store("test-connectivity").await;
                        }
                        Err(e) => {
                            println!("✗ Failed to create object store: {e}");
                        }
                    }
                }
            }

            // Test basic pub/sub
            let mut sub = client.subscribe("test.subject").await?;
            client.publish("test.subject", "Hello NATS!".into()).await?;

            match tokio::time::timeout(
                std::time::Duration::from_secs(1),
                sub.next()
            ).await {
                Ok(Some(msg)) => {
                    let payload = String::from_utf8_lossy(&msg.payload);
                    println!("✓ Pub/Sub working: received '{payload}'");
                }
                Ok(None) => {
                    println!("✗ No message received");
                }
                Err(_) => {
                    println!("✗ Timeout waiting for message");
                }
            }

            println!("\nNATS connectivity test completed successfully!");
        }
        Err(e) => {
            eprintln!("✗ Failed to connect to NATS: {e}");
            eprintln!("\nMake sure NATS is running:");
            eprintln!("  nats-server -js");
            std::process::exit(1);
        }
    }

    Ok(())
}
