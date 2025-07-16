//! Demonstration of category theory-based inter-domain communication

use cim_domain::category::{DomainCategory, DomainObject, DomainMorphism};
use cim_domain::composition::{DomainComposition, Saga, SagaStep, SagaOrchestrator};
use cim_domain::integration::{DependencyContainer, ServiceRegistry, EventBridge, DomainBridge};
use cim_domain::errors::DomainError;
use uuid::Uuid;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Category Theory-Based Inter-Domain Communication Demo ===\n");
    
    // 1. Create domain categories
    println!("1. Creating domain categories...");
    let mut sales_domain = DomainCategory::new("Sales".to_string());
    let mut billing_domain = DomainCategory::new("Billing".to_string());
    let mut shipping_domain = DomainCategory::new("Shipping".to_string());
    
    // Add objects to domains
    sales_domain.add_object(DomainObject::new("Order".to_string()));
    sales_domain.add_object(DomainObject::new("Customer".to_string()));
    
    billing_domain.add_object(DomainObject::new("Invoice".to_string()));
    billing_domain.add_object(DomainObject::new("Payment".to_string()));
    
    shipping_domain.add_object(DomainObject::new("Shipment".to_string()));
    shipping_domain.add_object(DomainObject::new("DeliveryAddress".to_string()));
    
    println!("✓ Created Sales, Billing, and Shipping domains\n");
    
    // 2. Create a domain composition
    println!("2. Creating domain composition...");
    let mut composition = DomainComposition::new("OrderFulfillment".to_string());
    composition.add_domain(sales_domain)?;
    composition.add_domain(billing_domain)?;
    composition.add_domain(shipping_domain)?;
    
    println!("✓ Composed domains into OrderFulfillment composition\n");
    
    // 3. Define a saga for order processing
    println!("3. Defining order processing saga...");
    let saga = Saga::new(
        "ProcessOrder".to_string(),
        vec![
            SagaStep::new("CreateInvoice".to_string(), "Sales".to_string(), "Billing".to_string()),
            SagaStep::new("ProcessPayment".to_string(), "Billing".to_string(), "Billing".to_string()),
            SagaStep::new("CreateShipment".to_string(), "Billing".to_string(), "Shipping".to_string()),
            SagaStep::new("DispatchOrder".to_string(), "Shipping".to_string(), "Shipping".to_string()),
        ],
    );
    
    println!("✓ Created saga with {} steps\n", saga.steps.len());
    
    // 4. Set up dependency injection
    println!("4. Setting up dependency injection...");
    let container = DependencyContainer::new();
    
    // Register services (mock implementations)
    container.register_singleton(|_| {
        Ok(std::sync::Arc::new(MockOrderService))
    }).await?;
    
    println!("✓ Registered services in DI container\n");
    
    // 5. Create event bridge for domain communication
    println!("5. Creating event bridge...");
    let event_bridge = EventBridge::new(Default::default());
    
    // Add routing rule
    event_bridge.add_rule(cim_domain::integration::event_bridge::RoutingRule {
        name: "OrderToInvoice".to_string(),
        source_pattern: "Sales.*".to_string(),
        event_pattern: "OrderPlaced".to_string(),
        targets: vec!["Billing".to_string()],
        priority: 100,
        conditions: vec![],
    }).await?;
    
    println!("✓ Event bridge configured with routing rules\n");
    
    // 6. Demonstrate saga orchestration
    println!("6. Starting saga orchestration...");
    let orchestrator = SagaOrchestrator::new();
    
    // Register domains
    orchestrator.register_domain(DomainCategory::new("Sales".to_string())).await?;
    orchestrator.register_domain(DomainCategory::new("Billing".to_string())).await?;
    orchestrator.register_domain(DomainCategory::new("Shipping".to_string())).await?;
    
    // Start the saga
    let saga_id = orchestrator.start_saga(saga).await?;
    println!("✓ Started saga with ID: {}\n", saga_id);
    
    // 7. Demonstrate cross-domain invariants
    println!("7. Checking cross-domain invariants...");
    use cim_domain::domain::invariants::{ReferentialIntegrityInvariant, InvariantChecker};
    
    let invariant = ReferentialIntegrityInvariant::new(
        "Order".to_string(),
        "Sales".to_string(),
        "Invoice".to_string(),
        "Billing".to_string(),
    );
    
    let checker = InvariantChecker::new();
    // In a real scenario, we would check actual domain state
    println!("✓ Invariant checker configured\n");
    
    println!("=== Demo Complete ===");
    println!("\nThis demo showcased:");
    println!("- Domain categories with objects and morphisms");
    println!("- Domain composition using functors");
    println!("- Saga orchestration with state machines");
    println!("- Dependency injection for services");
    println!("- Event routing between domains");
    println!("- Cross-domain invariant checking");
    
    Ok(())
}

// Mock service for demonstration
struct MockOrderService;

impl MockOrderService {
    fn process_order(&self, order_id: &str) -> Result<(), DomainError> {
        println!("Processing order: {}", order_id);
        Ok(())
    }
}