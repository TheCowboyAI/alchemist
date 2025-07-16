//! Demonstration of cross-domain semantic search using category theory

use cim_domain::category::{DomainCategory, IdentityFunctor};
use cim_domain::domain::semantic_analyzer::{SemanticAnalyzer, Concept};
use cim_domain::integration::{EventBridge, CrossDomainSearchEngine, CrossDomainQuery};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cross-Domain Semantic Search Demo ===\n");
    
    // 1. Set up the infrastructure
    println!("1. Setting up search infrastructure...");
    let event_bridge = Arc::new(EventBridge::new(Default::default()));
    let search_engine = CrossDomainSearchEngine::new(
        event_bridge.clone(),
        Default::default(),
    );
    
    // 2. Create and register domains with their semantic analyzers
    println!("2. Creating domains and semantic analyzers...");
    
    // Sales domain
    let sales_domain = DomainCategory::new("Sales".to_string());
    let sales_analyzer = Arc::new(SemanticAnalyzer::new());
    
    // Add sales concepts
    sales_analyzer.add_concept(Concept::new(
        "Order".to_string(),
        vec![0.9, 0.8, 0.7, 0.6, 0.5], // High commerce, high transaction
    )).await?;
    
    sales_analyzer.add_concept(Concept::new(
        "Customer".to_string(),
        vec![0.8, 0.9, 0.6, 0.7, 0.4], // High identity, high relationship
    )).await?;
    
    sales_analyzer.add_concept(Concept::new(
        "Product".to_string(),
        vec![0.7, 0.5, 0.9, 0.6, 0.8], // High physical, high value
    )).await?;
    
    sales_analyzer.add_concept(Concept::new(
        "ShoppingCart".to_string(),
        vec![0.8, 0.7, 0.6, 0.9, 0.5], // High temporal, high collection
    )).await?;
    
    search_engine.register_domain(sales_domain, sales_analyzer).await?;
    
    // Billing domain
    let billing_domain = DomainCategory::new("Billing".to_string());
    let billing_analyzer = Arc::new(SemanticAnalyzer::new());
    
    // Add billing concepts
    billing_analyzer.add_concept(Concept::new(
        "Invoice".to_string(),
        vec![0.85, 0.75, 0.65, 0.55, 0.45], // Similar to Order but more formal
    )).await?;
    
    billing_analyzer.add_concept(Concept::new(
        "Payment".to_string(),
        vec![0.9, 0.6, 0.4, 0.8, 0.7], // High transaction, high financial
    )).await?;
    
    billing_analyzer.add_concept(Concept::new(
        "Account".to_string(),
        vec![0.7, 0.85, 0.5, 0.6, 0.4], // High identity, moderate everything else
    )).await?;
    
    billing_analyzer.add_concept(Concept::new(
        "TaxCalculation".to_string(),
        vec![0.6, 0.4, 0.3, 0.9, 0.8], // High computational, high regulatory
    )).await?;
    
    search_engine.register_domain(billing_domain, billing_analyzer).await?;
    
    // Shipping domain
    let shipping_domain = DomainCategory::new("Shipping".to_string());
    let shipping_analyzer = Arc::new(SemanticAnalyzer::new());
    
    // Add shipping concepts
    shipping_analyzer.add_concept(Concept::new(
        "Shipment".to_string(),
        vec![0.8, 0.5, 0.9, 0.7, 0.6], // High physical, high logistics
    )).await?;
    
    shipping_analyzer.add_concept(Concept::new(
        "DeliveryAddress".to_string(),
        vec![0.5, 0.7, 0.8, 0.4, 0.3], // High location, high identity
    )).await?;
    
    shipping_analyzer.add_concept(Concept::new(
        "TrackingNumber".to_string(),
        vec![0.6, 0.8, 0.4, 0.5, 0.7], // High identity, moderate everything
    )).await?;
    
    shipping_analyzer.add_concept(Concept::new(
        "Carrier".to_string(),
        vec![0.7, 0.9, 0.6, 0.5, 0.4], // High organization, high service
    )).await?;
    
    search_engine.register_domain(shipping_domain, shipping_analyzer).await?;
    
    // Support domain
    let support_domain = DomainCategory::new("Support".to_string());
    let support_analyzer = Arc::new(SemanticAnalyzer::new());
    
    // Add support concepts
    support_analyzer.add_concept(Concept::new(
        "Ticket".to_string(),
        vec![0.7, 0.8, 0.4, 0.6, 0.5], // High identity, high communication
    )).await?;
    
    support_analyzer.add_concept(Concept::new(
        "CustomerIssue".to_string(),
        vec![0.75, 0.85, 0.5, 0.7, 0.4], // Very similar to Customer
    )).await?;
    
    support_analyzer.add_concept(Concept::new(
        "Resolution".to_string(),
        vec![0.6, 0.5, 0.3, 0.8, 0.7], // High outcome, high completion
    )).await?;
    
    search_engine.register_domain(support_domain, support_analyzer).await?;
    
    println!("✓ Registered 4 domains with semantic concepts\n");
    
    // 3. Demonstrate various search scenarios
    println!("3. Performing cross-domain searches...\n");
    
    // Search 1: Find concepts related to "Order"
    println!("Search 1: Finding concepts related to 'Order' across all domains");
    let query1 = CrossDomainQuery {
        query: "Order".to_string(),
        start_domain: Some("Sales".to_string()),
        target_domains: vec![], // Search all domains
        concept_vector: Some(vec![0.9, 0.8, 0.7, 0.6, 0.5]),
        config_overrides: None,
    };
    
    let results1 = search_engine.search(query1).await?;
    print_results(&results1, "Order");
    
    // Search 2: Find concepts related to "Customer"
    println!("\nSearch 2: Finding concepts related to 'Customer' across all domains");
    let query2 = CrossDomainQuery {
        query: "Customer".to_string(),
        start_domain: Some("Sales".to_string()),
        target_domains: vec![], // Search all domains
        concept_vector: Some(vec![0.8, 0.9, 0.6, 0.7, 0.4]),
        config_overrides: None,
    };
    
    let results2 = search_engine.search(query2).await?;
    print_results(&results2, "Customer");
    
    // Search 3: Targeted search in specific domains
    println!("\nSearch 3: Finding financial concepts in Billing and Sales domains only");
    let query3 = CrossDomainQuery {
        query: "Financial Transaction".to_string(),
        start_domain: None,
        target_domains: vec!["Sales".to_string(), "Billing".to_string()],
        concept_vector: Some(vec![0.9, 0.6, 0.4, 0.8, 0.7]), // Payment-like vector
        config_overrides: None,
    };
    
    let results3 = search_engine.search(query3).await?;
    print_results(&results3, "Financial Transaction");
    
    // 4. Show aggregated concepts
    println!("\n4. Aggregated concepts across domains:");
    for (i, result) in [&results1, &results2, &results3].iter().enumerate() {
        if !result.aggregated_concepts.is_empty() {
            println!("\nFrom search {}:", i + 1);
            for concept in &result.aggregated_concepts {
                println!("  - '{}' appears in {} domains with avg similarity {:.2}",
                    concept.name,
                    concept.domains.len(),
                    concept.avg_similarity
                );
            }
        }
    }
    
    println!("\n=== Demo Complete ===");
    println!("\nThis demo showcased:");
    println!("- Cross-domain semantic search using concept vectors");
    println!("- Finding related concepts across multiple domains");
    println!("- Targeted search in specific domains");
    println!("- Concept aggregation across domains");
    println!("- Similarity scoring based on semantic dimensions");
    
    Ok(())
}

fn print_results(results: &cim_domain::integration::cross_domain_search::CrossDomainResult, query: &str) {
    println!("Results for '{}' (searched {} domains in {}ms):",
        query,
        results.metadata.domains_searched.len(),
        results.metadata.duration_ms
    );
    
    for (domain, domain_results) in &results.domain_results {
        if !domain_results.is_empty() {
            println!("  {}:", domain);
            for result in domain_results.iter().take(3) {
                println!("    - {} (similarity: {:.2})",
                    result.concept_name,
                    result.similarity
                );
            }
        }
    }
    
    println!("  Total results: {}", results.metadata.total_results);
}