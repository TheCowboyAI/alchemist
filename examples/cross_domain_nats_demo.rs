//! NATS-based Cross-Domain Integration Demo
//!
//! This demo shows how different domains communicate via NATS messaging.
//! It sets up mock Person and Location domain services that respond to queries.
//!
//! ```mermaid
//! sequenceDiagram
//!     participant Org as Organization Domain
//!     participant NATS as NATS Server
//!     participant Person as Person Domain
//!     participant Location as Location Domain
//!     
//!     Org->>NATS: Request person details
//!     NATS->>Person: Forward request
//!     Person->>NATS: Return person details
//!     NATS->>Org: Forward response
//!     
//!     Org->>NATS: Request location details
//!     NATS->>Location: Forward request
//!     Location->>NATS: Return location details
//!     NATS->>Org: Forward response
//! ```

use async_nats;
use chrono::Utc;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use cim_domain_organization::{
    cross_domain::{
        CrossDomainIntegrationService, CrossDomainResolver, LocationDetails, PersonDetails,
        location_integration::{
            CombinedCrossDomainResolver, GetLocationDetailsBatchRequest,
            GetLocationDetailsBatchResponse, GetLocationDetailsRequest, GetLocationDetailsResponse,
            NatsLocationResolver,
        },
        person_integration::{
            GetPersonDetailsBatchRequest, GetPersonDetailsBatchResponse, GetPersonDetailsRequest,
            GetPersonDetailsResponse, NatsPersonResolver,
        },
    },
    projections::views::{MemberView, OrganizationView},
};

/// Mock Person Domain Service
async fn run_person_domain_service(
    nats_client: Arc<async_nats::Client>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Person Domain Service...");

    // Subscribe to person queries
    let mut subscriber = nats_client.subscribe("people.person.query.v1").await?;
    let mut batch_subscriber = nats_client
        .subscribe("people.person.query-batch.v1")
        .await?;

    // Handle single queries
    let client_clone = nats_client.clone();
    tokio::spawn(async move {
        while let Some(msg) = subscriber.next().await {
            if let Ok(request) = serde_json::from_slice::<GetPersonDetailsRequest>(&msg.payload) {
                println!(
                    "Person Domain: Received query for person {}",
                    request.person_id
                );

                // Mock response
                let response = GetPersonDetailsResponse {
                    person: Some(PersonDetails {
                        person_id: request.person_id,
                        full_name: format!("Person {}", request.person_id),
                        email: Some(format!("person.{}@company.com", request.person_id)),
                        title: Some("Employee".to_string()),
                    }),
                };

                let payload = serde_json::to_vec(&response).unwrap();
                if let Some(reply) = msg.reply {
                    let _ = client_clone.publish(reply, payload.into()).await;
                }
            }
        }
    });

    // Handle batch queries
    tokio::spawn(async move {
        while let Some(msg) = batch_subscriber.next().await {
            if let Ok(request) =
                serde_json::from_slice::<GetPersonDetailsBatchRequest>(&msg.payload)
            {
                println!(
                    "Person Domain: Received batch query for {} persons",
                    request.person_ids.len()
                );

                // Mock response
                let mut persons = HashMap::new();
                for person_id in request.person_ids {
                    persons.insert(
                        person_id,
                        PersonDetails {
                            person_id,
                            full_name: format!("Person {}", person_id),
                            email: Some(format!("person.{}@company.com", person_id)),
                            title: Some("Employee".to_string()),
                        },
                    );
                }

                let response = GetPersonDetailsBatchResponse { persons };
                let payload = serde_json::to_vec(&response).unwrap();
                if let Some(reply) = msg.reply {
                    let _ = nats_client.publish(reply, payload.into()).await;
                }
            }
        }
    });

    Ok(())
}

/// Mock Location Domain Service
async fn run_location_domain_service(
    nats_client: Arc<async_nats::Client>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Location Domain Service...");

    // Subscribe to location queries
    let mut subscriber = nats_client.subscribe("locations.location.query.v1").await?;
    let mut batch_subscriber = nats_client
        .subscribe("locations.location.query-batch.v1")
        .await?;

    // Handle single queries
    let client_clone = nats_client.clone();
    tokio::spawn(async move {
        while let Some(msg) = subscriber.next().await {
            if let Ok(request) = serde_json::from_slice::<GetLocationDetailsRequest>(&msg.payload) {
                println!(
                    "Location Domain: Received query for location {}",
                    request.location_id
                );

                // Mock response
                let response = GetLocationDetailsResponse {
                    location: Some(LocationDetails {
                        location_id: request.location_id,
                        name: "Main Office".to_string(),
                        address: "123 Business Ave".to_string(),
                        city: "Tech City".to_string(),
                        country: "Innovation Land".to_string(),
                    }),
                };

                let payload = serde_json::to_vec(&response).unwrap();
                if let Some(reply) = msg.reply {
                    let _ = client_clone.publish(reply, payload.into()).await;
                }
            }
        }
    });

    // Handle batch queries
    tokio::spawn(async move {
        while let Some(msg) = batch_subscriber.next().await {
            if let Ok(request) =
                serde_json::from_slice::<GetLocationDetailsBatchRequest>(&msg.payload)
            {
                println!(
                    "Location Domain: Received batch query for {} locations",
                    request.location_ids.len()
                );

                // Mock response
                let mut locations = HashMap::new();
                for (i, location_id) in request.location_ids.into_iter().enumerate() {
                    locations.insert(
                        location_id,
                        LocationDetails {
                            location_id,
                            name: format!("Office {}", i + 1),
                            address: format!("{} Street", i + 1),
                            city: "Tech City".to_string(),
                            country: "Innovation Land".to_string(),
                        },
                    );
                }

                let response = GetLocationDetailsBatchResponse { locations };
                let payload = serde_json::to_vec(&response).unwrap();
                if let Some(reply) = msg.reply {
                    let _ = nats_client.publish(reply, payload.into()).await;
                }
            }
        }
    });

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== NATS-based Cross-Domain Integration Demo ===\n");

    // Connect to NATS
    println!("Connecting to NATS server...");
    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let client = Arc::new(async_nats::connect(&nats_url).await?);
    println!("Connected to NATS at {}\n", nats_url);

    // Start mock domain services
    run_person_domain_service(client.clone()).await?;
    run_location_domain_service(client.clone()).await?;

    // Give services time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Create NATS-based resolvers
    let person_resolver = Arc::new(
        NatsPersonResolver::new(client.clone()).with_timeout(std::time::Duration::from_secs(2)),
    );
    let location_resolver = Arc::new(
        NatsLocationResolver::new(client.clone()).with_timeout(std::time::Duration::from_secs(2)),
    );

    // Create combined resolver
    let resolver = Arc::new(CombinedCrossDomainResolver::new(
        person_resolver as Arc<dyn CrossDomainResolver>,
        location_resolver as Arc<dyn CrossDomainResolver>,
    ));

    let service = CrossDomainIntegrationService::new(resolver.clone());

    // Test single person resolution
    println!("1. Testing single person resolution via NATS...");
    let person_id = Uuid::new_v4();
    match resolver.get_person_details(person_id).await {
        Ok(Some(details)) => {
            println!(
                "   ✓ Resolved person: {} ({})",
                details.full_name, details.person_id
            );
        }
        Ok(None) => println!("   ✗ Person not found"),
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    // Test single location resolution
    println!("2. Testing single location resolution via NATS...");
    let location_id = Uuid::new_v4();
    match resolver.get_location_details(location_id).await {
        Ok(Some(details)) => {
            println!(
                "   ✓ Resolved location: {} in {}",
                details.name, details.city
            );
        }
        Ok(None) => println!("   ✗ Location not found"),
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    // Test batch resolution
    println!("3. Testing batch person resolution via NATS...");
    let person_ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();
    match resolver.get_person_details_batch(person_ids.clone()).await {
        Ok(persons) => {
            println!("   ✓ Resolved {} persons:", persons.len());
            for (id, details) in persons.iter() {
                println!(
                    "     • {} - {}",
                    details.full_name,
                    details.email.as_ref().unwrap()
                );
            }
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    // Test organization enrichment
    println!("4. Testing organization enrichment with NATS data...");
    let org_id = Uuid::new_v4();
    let mut members = vec![
        MemberView {
            person_id: person_ids[0],
            organization_id: org_id,
            person_name: "Unknown".to_string(),
            role: "Manager".to_string(),
            joined_date: Utc::now() - chrono::Duration::days(365),
            tenure_days: 365,
        },
        MemberView {
            person_id: person_ids[1],
            organization_id: org_id,
            person_name: "Unknown".to_string(),
            role: "Developer".to_string(),
            joined_date: Utc::now() - chrono::Duration::days(180),
            tenure_days: 180,
        },
    ];

    service.enrich_with_person_names(&mut members).await?;
    println!("   ✓ Enriched member names:");
    for member in &members {
        println!("     • {} - {}", member.person_name, member.role);
    }
    println!();

    // Test event publishing
    println!("5. Publishing domain events...");
    let subject = "organizations.organization.member-added.v1";
    let event = serde_json::json!({
        "organization_id": org_id,
        "person_id": person_ids[0],
        "role": "Manager",
        "timestamp": Utc::now(),
    });

    client
        .publish(subject, serde_json::to_vec(&event)?.into())
        .await?;
    println!("   ✓ Published member-added event to {}", subject);
    println!();

    println!("=== Demo Complete ===");
    println!("\nThis demo showed:");
    println!("• NATS request-reply for cross-domain queries");
    println!("• Batch resolution for efficiency");
    println!("• Resilient error handling (timeouts return None)");
    println!("• Event publishing for domain notifications");
    println!("• Mock domain services responding to queries");

    Ok(())
}
