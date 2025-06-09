//! Context Bridge Demo
//!
//! This example demonstrates how to use ContextBridge to translate concepts
//! between different bounded contexts using various mapping patterns.

use ia::domain::conceptual_graph::{
    ConceptGraph, ConceptId, NodeId,
    ContextBridge, ContextMappingType, TranslationRule,
    ConceptTransformation, TranslationDirection,
};
use ia::domain::conceptual_graph::context_bridge::{
    InterfaceContract, ContractOperation, DataSchema, FieldType,
    PublishedLanguage, PublishedConcept,
    AttributeDefinition, ConceptPattern,
};
use std::collections::HashMap;

fn main() {
    println!("=== Context Bridge Demo ===\n");

    // Create two bounded contexts
    let inventory_context = ConceptGraph::new("Inventory Context");
    let order_context = ConceptGraph::new("Order Context");

    println!("Created Inventory Context: {:?}", inventory_context.id);
    println!("Created Order Context: {:?}\n", order_context.id);

    // Demo 1: Customer-Supplier relationship
    demo_customer_supplier(&inventory_context, &order_context);

    // Demo 2: Anti-Corruption Layer
    demo_anti_corruption_layer(&inventory_context, &order_context);

    // Demo 3: Shared Kernel
    demo_shared_kernel(&inventory_context, &order_context);

    // Demo 4: Published Language
    demo_published_language(&inventory_context, &order_context);
}

fn demo_customer_supplier(inventory: &ConceptGraph, order: &ConceptGraph) {
    println!("=== Customer-Supplier Demo ===");
    println!("Inventory (Supplier) provides product data to Order (Customer)\n");

    let mut bridge = ContextBridge::new(
        inventory.id,
        order.id,
        ContextMappingType::CustomerSupplier {
            upstream: inventory.id,
            downstream: order.id,
        },
    );

    // Add translation rule
    let rule = TranslationRule {
        source_concept: inventory.id,
        target_concept: order.id,
        transformation: ConceptTransformation::AttributeMapping {
            mappings: {
                let mut m = HashMap::new();
                m.insert("sku".to_string(), "product_code".to_string());
                m.insert("quantity_on_hand".to_string(), "available_quantity".to_string());
                m
            },
        },
        source_pattern: ConceptPattern {
            concept_type: Some("Product".to_string()),
            required_attributes: vec!["sku".to_string()],
            metadata_patterns: HashMap::new(),
        },
        target_pattern: ConceptPattern {
            concept_type: Some("OrderItem".to_string()),
            required_attributes: vec!["product_code".to_string()],
            metadata_patterns: HashMap::new(),
        },
        bidirectional: false,
    };
    bridge.add_translation_rule(rule);

    println!("Bridge created with mapping type: {}", bridge.mapping_description());
    println!("Translation rules added for Product -> OrderItem mapping");
    println!();
}

fn demo_anti_corruption_layer(inventory: &ConceptGraph, order: &ConceptGraph) {
    println!("=== Anti-Corruption Layer Demo ===");
    println!("Protecting Order context from external Inventory model\n");

    let _bridge = ContextBridge::new(
        order.id,
        inventory.id,
        ContextMappingType::AntiCorruptionLayer {
            internal_context: order.id,
            external_context: inventory.id,
        },
    );

    println!("ACL prevents external concepts from polluting internal model");
    println!("All translations go through explicit transformation rules");
    println!();
}

fn demo_shared_kernel(inventory: &ConceptGraph, order: &ConceptGraph) {
    println!("=== Shared Kernel Demo ===");
    println!("Shared concepts between Inventory and Order contexts\n");

    // Create a shared concept
    let money_concept = ConceptId::new();

    let _bridge = ContextBridge::new(
        inventory.id,
        order.id,
        ContextMappingType::SharedKernel {
            shared_concepts: vec![money_concept],
        },
    );

    println!("Shared concepts: Money, ProductCode");
    println!("These concepts have the same meaning in both contexts");
    println!();
}

fn demo_published_language(inventory: &ConceptGraph, order: &ConceptGraph) {
    println!("=== Published Language Demo ===");
    println!("Well-defined language for cross-context communication\n");

    let language = PublishedLanguage {
        name: "Product Catalog Language".to_string(),
        version: "1.0".to_string(),
        concepts: vec![
            PublishedConcept {
                name: "CatalogProduct".to_string(),
                attributes: vec![
                    AttributeDefinition {
                        name: "product_id".to_string(),
                        attribute_type: "String".to_string(),
                        required: true,
                        constraints: vec!["unique".to_string()],
                    },
                    AttributeDefinition {
                        name: "name".to_string(),
                        attribute_type: "String".to_string(),
                        required: true,
                        constraints: vec!["non_empty".to_string()],
                    },
                ],
                invariants: vec!["product_id must be unique".to_string()],
            },
        ],
        relationships: vec![],
        constraints: vec![],
    };

    let _bridge = ContextBridge::new(
        inventory.id,
        order.id,
        ContextMappingType::PublishedLanguage {
            publisher: inventory.id,
            language_spec: language,
        },
    );

    println!("Published language defines standard product representation");
    println!("All contexts can use this language for integration");
    println!();
}
