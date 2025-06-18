use ia::domain::conceptual_graph::{
    Action, BusinessRule, ComparisonOperator, ConceptId, Condition, FactReference, FactSet,
    FactValue, LogicalOperator, NotificationSeverity, RuleContext, RuleId, RuleType,
};
use std::collections::HashMap;

fn main() {
    println!("=== RuleContext Demo ===\n");

    // Create concepts for an e-commerce domain
    let order_concept = ConceptId::new();
    let customer_concept = ConceptId::new();
    let product_concept = ConceptId::new();
    let inventory_concept = ConceptId::new();

    println!("Created e-commerce concepts:");
    println!("- Order: {:?}", order_concept);
    println!("- Customer: {:?}", customer_concept);
    println!("- Product: {:?}", product_concept);
    println!("- Inventory: {:?}", inventory_concept);

    // Example 1: Validation Rules
    println!("\n1. Validation Rules - Order Processing");

    let mut order_context = RuleContext::new("Order Processing Rules".to_string(), order_concept);

    // Rule: Order total must be positive
    let positive_total_rule = BusinessRule {
        id: RuleId::new(),
        name: "Positive Order Total".to_string(),
        description: "Order total must be greater than zero".to_string(),
        rule_type: RuleType::Validation,
        conditions: vec![Condition::Comparison {
            left: FactReference::Fact {
                fact_type: "order_total".to_string(),
            },
            operator: ComparisonOperator::GreaterThan,
            right: FactReference::Literal {
                value: FactValue::Number(0.0),
            },
        }],
        actions: vec![],
        priority: 100,
        enabled: true,
        notification_severity: NotificationSeverity::Error,
    };

    order_context.add_rule(positive_total_rule.clone()).unwrap();

    // Rule: Customer must be verified
    let customer_verified_rule = BusinessRule {
        id: RuleId::new(),
        name: "Customer Verification".to_string(),
        description: "Customer must be verified before placing orders".to_string(),
        rule_type: RuleType::Validation,
        conditions: vec![Condition::FactExists {
            fact_type: "customer_verified".to_string(),
            expected_value: Some(FactValue::Boolean(true)),
        }],
        actions: vec![Action::Notify {
            message: "Unverified customer attempted to place order".to_string(),
            severity: NotificationSeverity::Warning,
        }],
        priority: 90,
        enabled: true,
        notification_severity: NotificationSeverity::Error,
    };

    order_context.add_rule(customer_verified_rule).unwrap();

    // Test validation
    let mut facts = FactSet::new();
    facts.add_fact(
        order_concept,
        "order_total".to_string(),
        FactValue::Number(150.0),
    );
    facts.add_fact(
        order_concept,
        "customer_verified".to_string(),
        FactValue::Boolean(true),
    );

    let compliance = order_context.check_compliance(order_concept, &facts);
    println!("Order compliance check: {:?}", compliance.compliant);
    println!("Satisfied rules: {}", compliance.satisfied_rules.len());

    // Example 2: Business Policy Rules
    println!("\n2. Business Policy Rules - Discount Application");

    // Rule: Apply discount for orders over $100
    let discount_rule = BusinessRule {
        id: RuleId::new(),
        name: "Bulk Order Discount".to_string(),
        description: "Apply 10% discount for orders over $100".to_string(),
        rule_type: RuleType::Policy,
        conditions: vec![Condition::Comparison {
            left: FactReference::Fact {
                fact_type: "order_total".to_string(),
            },
            operator: ComparisonOperator::GreaterThan,
            right: FactReference::Literal {
                value: FactValue::Number(100.0),
            },
        }],
        actions: vec![Action::AssertFact {
            fact_type: "discount_percentage".to_string(),
            value: FactValue::Number(10.0),
        }],
        priority: 50,
        enabled: true,
        notification_severity: NotificationSeverity::Error,
    };

    order_context.add_rule(discount_rule).unwrap();

    // Example 3: Constraint Rules
    println!("\n3. Constraint Rules - Inventory Management");

    let mut inventory_context =
        RuleContext::new("Inventory Management Rules".to_string(), inventory_concept);

    // Rule: Cannot sell more than available stock
    let stock_constraint_rule = BusinessRule {
        id: RuleId::new(),
        name: "Stock Availability Constraint".to_string(),
        description: "Order quantity cannot exceed available stock".to_string(),
        rule_type: RuleType::Constraint,
        conditions: vec![Condition::Comparison {
            left: FactReference::Fact {
                fact_type: "order_quantity".to_string(),
            },
            operator: ComparisonOperator::LessThanOrEqual,
            right: FactReference::Fact {
                fact_type: "available_stock".to_string(),
            },
        }],
        actions: vec![Action::Notify {
            message: "Insufficient stock for order".to_string(),
            severity: NotificationSeverity::Error,
        }],
        priority: 100,
        enabled: true,
        notification_severity: NotificationSeverity::Error,
    };

    inventory_context.add_rule(stock_constraint_rule).unwrap();

    // Test constraint
    let mut inventory_facts = FactSet::new();
    inventory_facts.add_fact(
        inventory_concept,
        "order_quantity".to_string(),
        FactValue::Number(5.0),
    );
    inventory_facts.add_fact(
        inventory_concept,
        "available_stock".to_string(),
        FactValue::Number(10.0),
    );

    let inventory_compliance =
        inventory_context.check_compliance(inventory_concept, &inventory_facts);
    println!(
        "Inventory constraint satisfied: {:?}",
        inventory_compliance.compliant
    );

    // Example 4: Complex Logical Rules
    println!("\n4. Complex Logical Rules - Fraud Detection");

    let mut fraud_context = RuleContext::new("Fraud Detection Rules".to_string(), order_concept);

    // Rule: Flag suspicious orders (high value AND new customer AND rush delivery)
    let fraud_detection_rule = BusinessRule {
        id: RuleId::new(),
        name: "Suspicious Order Detection".to_string(),
        description: "Flag orders that match suspicious patterns".to_string(),
        rule_type: RuleType::Policy,
        conditions: vec![Condition::Logical {
            operator: LogicalOperator::And,
            conditions: vec![
                Condition::Comparison {
                    left: FactReference::Fact {
                        fact_type: "order_total".to_string(),
                    },
                    operator: ComparisonOperator::GreaterThan,
                    right: FactReference::Literal {
                        value: FactValue::Number(500.0),
                    },
                },
                Condition::FactExists {
                    fact_type: "new_customer".to_string(),
                    expected_value: Some(FactValue::Boolean(true)),
                },
                Condition::FactExists {
                    fact_type: "rush_delivery".to_string(),
                    expected_value: Some(FactValue::Boolean(true)),
                },
            ],
        }],
        actions: vec![
            Action::AssertFact {
                fact_type: "fraud_risk".to_string(),
                value: FactValue::Text("HIGH".to_string()),
            },
            Action::Notify {
                message: "High fraud risk order detected".to_string(),
                severity: NotificationSeverity::Critical,
            },
        ],
        priority: 100,
        enabled: true,
        notification_severity: NotificationSeverity::Error,
    };

    fraud_context.add_rule(fraud_detection_rule).unwrap();

    // Test fraud detection
    let mut fraud_facts = FactSet::new();
    fraud_facts.add_fact(
        order_concept,
        "order_total".to_string(),
        FactValue::Number(750.0),
    );
    fraud_facts.add_fact(
        order_concept,
        "new_customer".to_string(),
        FactValue::Boolean(true),
    );
    fraud_facts.add_fact(
        order_concept,
        "rush_delivery".to_string(),
        FactValue::Boolean(true),
    );

    let fraud_evaluation = fraud_context.evaluate(order_concept, &fraud_facts).unwrap();
    println!(
        "Fraud rules triggered: {}",
        fraud_evaluation.triggered_rules.len()
    );

    // Example 5: Impact Analysis
    println!("\n5. Impact Analysis - Price Change");

    // Analyze what rules would be affected by a price change
    let price_change = ia::domain::conceptual_graph::FactChange {
        concept_id: order_concept,
        fact_type: "order_total".to_string(),
        old_value: Some(FactValue::Number(50.0)),
        new_value: Some(FactValue::Number(150.0)),
    };

    let affected_rules = order_context.analyze_impact(&price_change);
    println!("Rules affected by price change: {}", affected_rules.len());

    // Example 6: Rule Priority and Execution Order
    println!("\n6. Rule Priority and Execution Order");

    // Create rules with different priorities
    let high_priority_rule = BusinessRule {
        id: RuleId::new(),
        name: "High Priority Rule".to_string(),
        description: "Executes first".to_string(),
        rule_type: RuleType::Validation,
        conditions: vec![],
        actions: vec![],
        priority: 1000,
        enabled: true,
        notification_severity: NotificationSeverity::Error,
    };

    let low_priority_rule = BusinessRule {
        id: RuleId::new(),
        name: "Low Priority Rule".to_string(),
        description: "Executes last".to_string(),
        rule_type: RuleType::Validation,
        conditions: vec![],
        actions: vec![],
        priority: 10,
        enabled: true,
        notification_severity: NotificationSeverity::Error,
    };

    println!(
        "High priority rule: {} (priority: {})",
        high_priority_rule.name, high_priority_rule.priority
    );
    println!(
        "Low priority rule: {} (priority: {})",
        low_priority_rule.name, low_priority_rule.priority
    );

    println!("\n=== Demo Complete ===");
}
