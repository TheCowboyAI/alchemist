//! Cross-Domain Integration Demo
//!
//! This demo shows how different domains can communicate and integrate
//! through the established cross-domain patterns.
//!
//! ```mermaid
//! graph LR
//!     Person[Person Domain] -->|PersonDetails| Org[Organization Domain]
//!     Location[Location Domain] -->|LocationDetails| Org
//!     Org -->|Enriched Views| UI[User Interface]
//! ```

use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use cim_domain_organization::{
    cross_domain::{
        CrossDomainIntegrationService, CrossDomainResolver, InMemoryCrossDomainResolver,
        LocationDetails, PersonDetails,
    },
    projections::views::{MemberView, OrganizationView},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cross-Domain Integration Demo ===\n");

    // Create in-memory resolver for demo
    let resolver = Arc::new(InMemoryCrossDomainResolver::new());
    let service = CrossDomainIntegrationService::new(resolver.clone());

    // Simulate Person Domain data
    println!("1. Setting up Person Domain data...");
    let ceo_id = Uuid::new_v4();
    let cto_id = Uuid::new_v4();
    let eng1_id = Uuid::new_v4();
    let eng2_id = Uuid::new_v4();

    resolver
        .add_person(PersonDetails {
            person_id: ceo_id,
            full_name: "Sarah Johnson".to_string(),
            email: Some("sarah.johnson@techcorp.com".to_string()),
            title: Some("Chief Executive Officer".to_string()),
        })
        .await;

    resolver
        .add_person(PersonDetails {
            person_id: cto_id,
            full_name: "Michael Chen".to_string(),
            email: Some("michael.chen@techcorp.com".to_string()),
            title: Some("Chief Technology Officer".to_string()),
        })
        .await;

    resolver
        .add_person(PersonDetails {
            person_id: eng1_id,
            full_name: "Emily Rodriguez".to_string(),
            email: Some("emily.rodriguez@techcorp.com".to_string()),
            title: Some("Senior Software Engineer".to_string()),
        })
        .await;

    resolver
        .add_person(PersonDetails {
            person_id: eng2_id,
            full_name: "David Kim".to_string(),
            email: Some("david.kim@techcorp.com".to_string()),
            title: Some("Software Engineer".to_string()),
        })
        .await;

    println!("   ✓ Added 4 persons to Person Domain\n");

    // Simulate Location Domain data
    println!("2. Setting up Location Domain data...");
    let hq_location_id = Uuid::new_v4();
    let dev_office_id = Uuid::new_v4();

    resolver
        .add_location(LocationDetails {
            location_id: hq_location_id,
            name: "Tech Corp Headquarters".to_string(),
            address: "100 Innovation Drive".to_string(),
            city: "San Francisco".to_string(),
            country: "USA".to_string(),
        })
        .await;

    resolver
        .add_location(LocationDetails {
            location_id: dev_office_id,
            name: "Development Center".to_string(),
            address: "200 Code Street".to_string(),
            city: "Austin".to_string(),
            country: "USA".to_string(),
        })
        .await;

    println!("   ✓ Added 2 locations to Location Domain\n");

    // Create Organization with members
    println!("3. Creating Organization with members...");
    let org_id = Uuid::new_v4();

    let mut members = vec![
        MemberView {
            person_id: ceo_id,
            organization_id: org_id,
            person_name: "Unknown".to_string(), // Will be enriched
            role: "CEO".to_string(),
            joined_date: Utc::now() - chrono::Duration::days(1825), // 5 years
            tenure_days: 1825,
        },
        MemberView {
            person_id: cto_id,
            organization_id: org_id,
            person_name: "Unknown".to_string(),
            role: "CTO".to_string(),
            joined_date: Utc::now() - chrono::Duration::days(1460), // 4 years
            tenure_days: 1460,
        },
        MemberView {
            person_id: eng1_id,
            organization_id: org_id,
            person_name: "Unknown".to_string(),
            role: "Senior Engineer".to_string(),
            joined_date: Utc::now() - chrono::Duration::days(730), // 2 years
            tenure_days: 730,
        },
        MemberView {
            person_id: eng2_id,
            organization_id: org_id,
            person_name: "Unknown".to_string(),
            role: "Engineer".to_string(),
            joined_date: Utc::now() - chrono::Duration::days(365), // 1 year
            tenure_days: 365,
        },
    ];

    println!("   ✓ Created organization with {} members\n", members.len());

    // Demonstrate cross-domain enrichment
    println!("4. Enriching organization data with cross-domain information...");

    // Enrich member names from Person Domain
    println!("   - Resolving member names from Person Domain...");
    service.enrich_with_person_names(&mut members).await?;

    println!("   ✓ Member names resolved:");
    for member in &members {
        println!("     • {} - {}", member.person_name, member.role);
    }
    println!();

    // Create organization view
    let mut org = OrganizationView {
        id: org_id,
        name: "Tech Corp International".to_string(),
        category: "Technology".to_string(),
        size: members.len(),
        headquarters_location: Some(hq_location_id),
        founded_date: Some(chrono::NaiveDate::from_ymd_opt(2018, 1, 1).unwrap()),
        member_count: members.len(),
        average_tenure_days: Some(
            members.iter().map(|m| m.tenure_days).sum::<i64>() as f64 / members.len() as f64,
        ),
        primary_location_name: None, // Will be enriched
    };

    // Enrich location name from Location Domain
    println!("   - Resolving headquarters location from Location Domain...");
    service
        .enrich_with_location_name(&mut org, hq_location_id)
        .await?;

    println!(
        "   ✓ Location resolved: {}",
        org.primary_location_name.as_ref().unwrap()
    );
    println!();

    // Display final enriched organization view
    println!("5. Final Enriched Organization View:");
    println!("   ╔════════════════════════════════════════════════════════════╗");
    println!("   ║ Organization: {:44} ║", org.name);
    println!("   ╠════════════════════════════════════════════════════════════╣");
    println!("   ║ Category: {:48} ║", org.category);
    println!(
        "   ║ Founded: {:49} ║",
        org.founded_date.unwrap().format("%B %d, %Y")
    );
    println!("   ║ Size: {:52} employees ║", org.size);
    println!(
        "   ║ Average Tenure: {:42} days ║",
        org.average_tenure_days.unwrap() as i64
    );
    println!(
        "   ║ Headquarters: {:44} ║",
        org.primary_location_name.as_ref().unwrap()
    );
    println!("   ╠════════════════════════════════════════════════════════════╣");
    println!("   ║ Team Members:                                              ║");
    for member in &members {
        println!("   ║   • {:20} - {:30} ║", member.person_name, member.role);
    }
    println!("   ╚════════════════════════════════════════════════════════════╝");
    println!();

    // Demonstrate batch resolution
    println!("6. Testing batch resolution capabilities...");
    let person_ids: Vec<Uuid> = members.iter().map(|m| m.person_id).collect();
    let batch_results = resolver.get_person_details_batch(person_ids).await?;

    println!("   ✓ Batch resolved {} person records", batch_results.len());
    println!();

    // Show statistics
    println!("7. Cross-Domain Statistics:");
    println!("   - Total persons in system: 4");
    println!("   - Total locations in system: 2");
    println!(
        "   - Average member tenure: {:.1} years",
        org.average_tenure_days.unwrap() / 365.0
    );
    println!(
        "   - Organization age: {:.1} years",
        (Utc::now().naive_utc().date() - org.founded_date.unwrap()).num_days() as f64 / 365.0
    );
    println!();

    println!("=== Demo Complete ===");
    println!("\nThis demo showed how:");
    println!("• Person Domain provides employee information");
    println!("• Location Domain provides office information");
    println!("• Organization Domain combines data from multiple domains");
    println!("• Cross-domain resolvers enable seamless integration");
    println!("• Views can be enriched with data from other domains");

    Ok(())
}
