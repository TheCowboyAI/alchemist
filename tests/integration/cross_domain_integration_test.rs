//! Integration tests for cross-domain functionality
//!
//! This test suite verifies that different domains can communicate
//! and integrate properly through the established patterns.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// Import domain modules
use cim_domain_organization::cross_domain::{
    CrossDomainIntegrationService, CrossDomainResolver, InMemoryCrossDomainResolver,
    LocationDetails, PersonDetails,
};
use cim_domain_organization::projections::views::{MemberView, OrganizationView};

/// Test that organization domain can resolve person names
#[tokio::test]
async fn test_organization_person_integration() {
    // Create resolver
    let resolver = Arc::new(InMemoryCrossDomainResolver::new());
    let service = CrossDomainIntegrationService::new(resolver.clone());

    // Add test persons
    let person1_id = Uuid::new_v4();
    let person2_id = Uuid::new_v4();

    resolver
        .add_person(PersonDetails {
            person_id: person1_id,
            full_name: "Alice Johnson".to_string(),
            email: Some("alice@example.com".to_string()),
            title: Some("Software Engineer".to_string()),
        })
        .await;

    resolver
        .add_person(PersonDetails {
            person_id: person2_id,
            full_name: "Bob Smith".to_string(),
            email: Some("bob@example.com".to_string()),
            title: Some("Engineering Manager".to_string()),
        })
        .await;

    // Create member views
    let org_id = Uuid::new_v4();
    let mut members = vec![
        MemberView {
            person_id: person1_id,
            organization_id: org_id,
            person_name: "Unknown".to_string(),
            role: "Software Engineer".to_string(),
            joined_date: chrono::Utc::now(),
            tenure_days: 365,
        },
        MemberView {
            person_id: person2_id,
            organization_id: org_id,
            person_name: "Unknown".to_string(),
            role: "Engineering Manager".to_string(),
            joined_date: chrono::Utc::now() - chrono::Duration::days(730),
            tenure_days: 730,
        },
    ];

    // Enrich with person names
    service
        .enrich_with_person_names(&mut members)
        .await
        .unwrap();

    // Verify names were resolved
    assert_eq!(members[0].person_name, "Alice Johnson");
    assert_eq!(members[1].person_name, "Bob Smith");
}

/// Test that organization domain can resolve location names
#[tokio::test]
async fn test_organization_location_integration() {
    // Create resolver
    let resolver = Arc::new(InMemoryCrossDomainResolver::new());
    let service = CrossDomainIntegrationService::new(resolver.clone());

    // Add test location
    let location_id = Uuid::new_v4();
    resolver
        .add_location(LocationDetails {
            location_id,
            name: "Tech Hub".to_string(),
            address: "123 Innovation Drive".to_string(),
            city: "San Francisco".to_string(),
            country: "USA".to_string(),
        })
        .await;

    // Create organization view
    let mut org = OrganizationView {
        id: Uuid::new_v4(),
        name: "Tech Corp".to_string(),
        category: "Technology".to_string(),
        size: 500,
        headquarters_location: Some(location_id),
        founded_date: Some(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
        member_count: 500,
        average_tenure_days: Some(365.0),
        primary_location_name: None,
    };

    // Enrich with location name
    service
        .enrich_with_location_name(&mut org, location_id)
        .await
        .unwrap();

    // Verify location was resolved
    assert_eq!(
        org.primary_location_name,
        Some("Tech Hub, San Francisco".to_string())
    );
}

/// Test batch resolution of multiple persons
#[tokio::test]
async fn test_batch_person_resolution() {
    let resolver = Arc::new(InMemoryCrossDomainResolver::new());

    // Add multiple persons
    let mut person_ids = Vec::new();
    for i in 0..10 {
        let person_id = Uuid::new_v4();
        person_ids.push(person_id);

        resolver
            .add_person(PersonDetails {
                person_id,
                full_name: format!("Person {}", i),
                email: Some(format!("person{}@example.com", i)),
                title: Some("Employee".to_string()),
            })
            .await;
    }

    // Batch resolve
    let results = resolver
        .get_person_details_batch(person_ids.clone())
        .await
        .unwrap();

    // Verify all were resolved
    assert_eq!(results.len(), 10);
    for (i, person_id) in person_ids.iter().enumerate() {
        let details = results.get(person_id).unwrap();
        assert_eq!(details.full_name, format!("Person {}", i));
    }
}

/// Test that cross-domain integration handles missing data gracefully
#[tokio::test]
async fn test_missing_data_handling() {
    let resolver = Arc::new(InMemoryCrossDomainResolver::new());
    let service = CrossDomainIntegrationService::new(resolver.clone());

    // Create member with non-existent person ID
    let mut members = vec![MemberView {
        person_id: Uuid::new_v4(), // This person doesn't exist
        organization_id: Uuid::new_v4(),
        person_name: "Unknown Person".to_string(),
        role: "Developer".to_string(),
        joined_date: chrono::Utc::now(),
        tenure_days: 100,
    }];

    // Try to enrich - should not panic
    service
        .enrich_with_person_names(&mut members)
        .await
        .unwrap();

    // Name should remain unchanged
    assert_eq!(members[0].person_name, "Unknown Person");
}

/// Test combined resolver that handles both person and location resolution
#[tokio::test]
async fn test_combined_resolver() {
    use cim_domain_organization::cross_domain::location_integration::CombinedCrossDomainResolver;

    // Create separate resolvers
    let person_resolver = Arc::new(InMemoryCrossDomainResolver::new());
    let location_resolver = Arc::new(InMemoryCrossDomainResolver::new());

    // Add test data
    let person_id = Uuid::new_v4();
    person_resolver
        .add_person(PersonDetails {
            person_id,
            full_name: "Charlie Brown".to_string(),
            email: Some("charlie@example.com".to_string()),
            title: Some("CEO".to_string()),
        })
        .await;

    let location_id = Uuid::new_v4();
    location_resolver
        .add_location(LocationDetails {
            location_id,
            name: "Corporate HQ".to_string(),
            address: "1 Main Street".to_string(),
            city: "New York".to_string(),
            country: "USA".to_string(),
        })
        .await;

    // Create combined resolver
    let combined = CombinedCrossDomainResolver::new(
        person_resolver as Arc<dyn CrossDomainResolver>,
        location_resolver as Arc<dyn CrossDomainResolver>,
    );

    // Test person resolution through combined resolver
    let person_details = combined.get_person_details(person_id).await.unwrap();
    assert!(person_details.is_some());
    assert_eq!(person_details.unwrap().full_name, "Charlie Brown");

    // Test location resolution through combined resolver
    let location_details = combined.get_location_details(location_id).await.unwrap();
    assert!(location_details.is_some());
    assert_eq!(location_details.unwrap().name, "Corporate HQ");
}

/// Test organization statistics with cross-domain data
#[tokio::test]
async fn test_organization_statistics_enrichment() {
    let resolver = Arc::new(InMemoryCrossDomainResolver::new());
    let service = CrossDomainIntegrationService::new(resolver.clone());

    // Create organization with multiple members
    let org_id = Uuid::new_v4();
    let mut members = Vec::new();

    // Add members with different tenure
    for i in 0..5 {
        let person_id = Uuid::new_v4();

        resolver
            .add_person(PersonDetails {
                person_id,
                full_name: format!("Employee {}", i),
                email: Some(format!("emp{}@company.com", i)),
                title: Some("Engineer".to_string()),
            })
            .await;

        members.push(MemberView {
            person_id,
            organization_id: org_id,
            person_name: "Unknown".to_string(),
            role: "Engineer".to_string(),
            joined_date: chrono::Utc::now() - chrono::Duration::days(i as i64 * 100),
            tenure_days: i as i64 * 100,
        });
    }

    // Enrich member names
    service
        .enrich_with_person_names(&mut members)
        .await
        .unwrap();

    // Calculate average tenure
    let total_tenure: i64 = members.iter().map(|m| m.tenure_days).sum();
    let average_tenure = total_tenure as f64 / members.len() as f64;

    // Create organization view with statistics
    let org = OrganizationView {
        id: org_id,
        name: "Engineering Co".to_string(),
        category: "Technology".to_string(),
        size: members.len(),
        headquarters_location: None,
        founded_date: None,
        member_count: members.len(),
        average_tenure_days: Some(average_tenure),
        primary_location_name: None,
    };

    // Verify statistics
    assert_eq!(org.member_count, 5);
    assert_eq!(org.average_tenure_days, Some(200.0)); // (0 + 100 + 200 + 300 + 400) / 5
}
