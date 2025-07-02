//! Person Network Analysis Demo
//!
//! This demo showcases the professional network analysis capabilities of the Person domain,
//! including shortest path finding, influence scoring, and community detection.

use cim_domain_person::{
    NetworkAnalysisService, PersonId,
    value_objects::{ProfessionalNetworkRelation, ProfessionalRelationType},
};
use std::collections::HashMap;

fn main() {
    println!("=== Person Network Analysis Demo ===\n");

    // Create a network analysis service
    let mut network_service = NetworkAnalysisService::new();

    // Create some people
    let people = create_demo_people();

    // Build professional network
    build_professional_network(&mut network_service, &people);

    // Demo 1: Shortest Path Finding
    println!("1. Shortest Path Demo");
    println!("   Finding path from Alice to Frank");
    demo_shortest_path(&network_service, &people);

    // Demo 2: Influence Analysis
    println!("\n2. Influence Analysis Demo");
    println!("   Calculating influence scores for all people");
    demo_influence_analysis(&network_service, &people);

    // Demo 3: Community Detection
    println!("\n3. Community Detection Demo");
    println!("   Detecting communities in the network");
    demo_community_detection(&network_service);

    // Demo 4: Network Metrics
    println!("\n4. Network Metrics Demo");
    println!("   Calculating overall network statistics");
    demo_network_metrics(&network_service, &people);
}

fn create_demo_people() -> HashMap<&'static str, PersonId> {
    let mut people = HashMap::new();

    // Create a network of professionals
    let names = vec![
        "Alice", "Bob", "Carol", "David", "Eve", "Frank", "Grace", "Henry", "Iris", "Jack", "Kate",
        "Liam",
    ];

    for name in names {
        people.insert(name, PersonId::new());
        println!("   Created person: {}", name);
    }

    people
}

fn build_professional_network(
    service: &mut NetworkAnalysisService,
    people: &HashMap<&'static str, PersonId>,
) {
    // Create a realistic professional network
    let connections = vec![
        // Core team
        (
            "Alice",
            "Bob",
            ProfessionalRelationType::Colleague {
                same_team: true,
                same_department: true,
            },
            0.9,
        ),
        (
            "Alice",
            "Carol",
            ProfessionalRelationType::Colleague {
                same_team: true,
                same_department: true,
            },
            0.8,
        ),
        (
            "Bob",
            "Carol",
            ProfessionalRelationType::Colleague {
                same_team: true,
                same_department: true,
            },
            0.7,
        ),
        // Management chain
        ("Alice", "David", ProfessionalRelationType::Manager, 0.95),
        ("David", "Eve", ProfessionalRelationType::Subordinate, 0.85),
        (
            "David",
            "Frank",
            ProfessionalRelationType::Subordinate,
            0.85,
        ),
        // Cross-team collaboration
        (
            "Carol",
            "Grace",
            ProfessionalRelationType::Colleague {
                same_team: false,
                same_department: false,
            },
            0.7,
        ),
        (
            "Grace",
            "Henry",
            ProfessionalRelationType::Colleague {
                same_team: true,
                same_department: true,
            },
            0.8,
        ),
        ("Henry", "Iris", ProfessionalRelationType::Mentor, 0.9),
        // Extended network
        (
            "Frank",
            "Jack",
            ProfessionalRelationType::Colleague {
                same_team: false,
                same_department: true,
            },
            0.6,
        ),
        (
            "Jack",
            "Kate",
            ProfessionalRelationType::BusinessPartner,
            0.7,
        ),
        (
            "Kate",
            "Liam",
            ProfessionalRelationType::Colleague {
                same_team: true,
                same_department: true,
            },
            0.8,
        ),
        // Bridge connections
        (
            "Eve",
            "Grace",
            ProfessionalRelationType::ProfessionalContact,
            0.6,
        ),
        ("Iris", "Liam", ProfessionalRelationType::Mentor, 0.7),
    ];

    for (from, to, rel_type, strength) in connections {
        let relation = ProfessionalNetworkRelation {
            other_person_id: people[to].as_uuid().clone(), // Extract the UUID from PersonId
            relation_type: rel_type.clone(),
            strength,
            established_date: chrono::Utc::now(),
            last_interaction: Some(chrono::Utc::now()),
            interaction_count: 10,
            mutual_connections: 5,
        };

        service.add_relationship(people[from].as_uuid().clone(), relation);
        println!("   Connected {} -> {} ({:?})", from, to, rel_type);
    }
}

fn demo_shortest_path(service: &NetworkAnalysisService, people: &HashMap<&'static str, PersonId>) {
    let alice = people["Alice"];
    let frank = people["Frank"];

    match service.find_shortest_path(alice.as_uuid().clone(), frank.as_uuid().clone()) {
        Some(path) => {
            println!("   Path found with {} steps:", path.path.len() - 1);
            for (i, person_id) in path.path.iter().enumerate() {
                let name = people
                    .iter()
                    .find(|(_, id)| id.as_uuid() == person_id)
                    .map(|(name, _)| *name)
                    .unwrap_or("Unknown");
                println!("   {}. {}", i + 1, name);
            }
        }
        None => println!("   No path found"),
    }
}

fn demo_influence_analysis(
    service: &NetworkAnalysisService,
    people: &HashMap<&'static str, PersonId>,
) {
    // Calculate influence for each person
    let mut influences: Vec<(&str, f32)> = people
        .iter()
        .map(|(name, id)| {
            let metrics = service.calculate_metrics(id.as_uuid().clone());
            (*name, metrics.influence_score)
        })
        .collect();

    // Sort by influence (descending)
    influences.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("   Top 5 most influential people:");
    for (i, (name, influence)) in influences.iter().take(5).enumerate() {
        println!("   {}. {} - Influence: {:.3}", i + 1, name, influence);
    }
}

fn demo_community_detection(service: &NetworkAnalysisService) {
    let communities = service.detect_communities(2); // min_size = 2
    {
        println!("   Found {} communities:", communities.len());

        for (i, community) in communities.iter().enumerate() {
            println!(
                "   Community {}: {} members",
                i + 1,
                community.members.len()
            );
            // In a real demo, we'd map person IDs back to names
        }
    }
}

fn demo_network_metrics(
    service: &NetworkAnalysisService,
    people: &HashMap<&'static str, PersonId>,
) {
    // Calculate metrics for specific people
    let alice = people["Alice"];
    let grace = people["Grace"];

    {
        let metrics = service.calculate_metrics(alice.as_uuid().clone());
        println!("   Alice's network metrics:");
        println!("   - Direct connections: {}", metrics.direct_connections);
        println!(
            "   - Second-degree connections: {}",
            metrics.second_degree_connections
        );
        println!(
            "   - Clustering coefficient: {:.3}",
            metrics.clustering_coefficient
        );
        println!(
            "   - Betweenness centrality: {:.3}",
            metrics.betweenness_centrality
        );
        println!("   - Influence score: {:.3}", metrics.influence_score);
    }

    // Show Grace as a bridge node
    {
        let metrics = service.calculate_metrics(grace.as_uuid().clone());
        println!("\n   Grace's network metrics (bridge node):");
        println!("   - Direct connections: {}", metrics.direct_connections);
        println!(
            "   - Second-degree connections: {}",
            metrics.second_degree_connections
        );
        println!(
            "   - Clustering coefficient: {:.3}",
            metrics.clustering_coefficient
        );
        println!(
            "   - Betweenness centrality: {:.3}",
            metrics.betweenness_centrality
        );
        println!("   - Influence score: {:.3}", metrics.influence_score);
    }
}
