//! Domain Model Importer Service
//!
//! Imports Domain-Driven Design models from various formats into ConceptGraph

use crate::domain::conceptual_graph::{
    ConceptGraph, ConceptNode, ConceptEdge, ConceptType, ConceptRelationship,
    ConceptualPoint, QualityDimension, DimensionType, NodeId, DistanceMetric,
    EdgeId,
};
use crate::domain::services::graph_import::{ImportedGraph, ImportedNode, ImportedEdge};
use crate::domain::DomainError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Domain model importer service
#[derive(Debug, Clone)]
pub struct DomainModelImporter {
    /// Default quality dimensions for domain concepts
    quality_dimensions: Vec<QualityDimension>,
}

impl Default for DomainModelImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainModelImporter {
    /// Create a new domain model importer with default DDD quality dimensions
    pub fn new() -> Self {
        let quality_dimensions = vec![
            QualityDimension {
                name: "Complexity".to_string(),
                dimension_type: DimensionType::Continuous,
                range: 0.0..10.0,
                metric: DistanceMetric::Euclidean,
                weight: 0.8,
            },
            QualityDimension {
                name: "Coupling".to_string(),
                dimension_type: DimensionType::Continuous,
                range: 0.0..1.0,
                metric: DistanceMetric::Euclidean,
                weight: 0.6,
            },
            QualityDimension {
                name: "Abstraction".to_string(),
                dimension_type: DimensionType::Ordinal,
                range: 0.0..5.0,
                metric: DistanceMetric::Manhattan,
                weight: 1.0,
            },
        ];

        Self { quality_dimensions }
    }

    /// Import a DDD model from an ImportedGraph
    pub fn import_ddd_model(&self, imported: ImportedGraph) -> Result<ConceptGraph, DomainError> {
        let mut graph = ConceptGraph::new("DDD Model");

        // Set quality dimensions
        for dim in &self.quality_dimensions {
            graph = graph.with_dimension(dim.clone());
        }

        // Import nodes as DDD concepts
        let mut node_mapping = HashMap::new();
        let mut node_indices = HashMap::new();

        for imported_node in imported.nodes {
            let concept_type = self.map_to_concept_type(&imported_node.node_type);
            let quality_position = self.calculate_quality_position(&imported_node, &concept_type);

            let node = ConceptNode::Atom {
                id: NodeId::new(),
                concept_type,
                quality_position,
                properties: imported_node.properties.clone(),
            };

            let node_id = node.id();
            let node_idx = graph.add_node(node);

            node_mapping.insert(imported_node.id.clone(), node_id);
            node_indices.insert(imported_node.id, node_idx);
        }

        // Import edges as DDD relationships
        for imported_edge in imported.edges {
            if let (Some(&source_idx), Some(&target_idx)) = (
                node_indices.get(&imported_edge.source),
                node_indices.get(&imported_edge.target),
            ) {
                let relationship = self.map_to_concept_relationship(&imported_edge.edge_type);

                let edge = ConceptEdge {
                    id: EdgeId::new(),
                    relationship,
                    properties: imported_edge.properties,
                };

                graph.add_edge(source_idx, target_idx, edge);
            }
        }

        Ok(graph)
    }

    /// Map imported node types to DDD concept types
    fn map_to_concept_type(&self, node_type: &str) -> ConceptType {
        match node_type.to_lowercase().as_str() {
            "entity" | "aggregate" | "aggregateroot" => ConceptType::Entity,
            "valueobject" | "value" | "vo" => ConceptType::ValueObject,
            "event" | "domainevent" => ConceptType::Event,
            "command" => ConceptType::Command,
            "policy" | "businessrule" => ConceptType::Policy,
            _ => ConceptType::Entity, // Default to Entity
        }
    }

    /// Map imported edge types to DDD concept relationships
    fn map_to_concept_relationship(&self, edge_type: &str) -> ConceptRelationship {
        match edge_type.to_lowercase().as_str() {
            "contains" | "has" | "owns" => ConceptRelationship::PartOf,
            "references" | "uses" | "depends" => ConceptRelationship::DependsOn,
            "extends" | "inherits" | "isa" => ConceptRelationship::IsA,
            "triggers" | "emits" | "publishes" => ConceptRelationship::Triggers,
            "constrains" | "validates" | "enforces" => ConceptRelationship::Constrains,
            _ => ConceptRelationship::DependsOn, // Default to DependsOn
        }
    }

    /// Calculate quality position based on node properties and type
    fn calculate_quality_position(
        &self,
        node: &ImportedNode,
        concept_type: &ConceptType,
    ) -> ConceptualPoint {
        let mut coordinates = vec![];

        // Complexity dimension (0-10)
        let complexity = match concept_type {
            ConceptType::Entity => 5.0,
            ConceptType::ValueObject => 2.0,
            ConceptType::Event => 3.0,
            ConceptType::Command => 3.0,
            ConceptType::Policy => 7.0,
            ConceptType::Aggregate => 8.0,
            _ => 4.0,
        };
        coordinates.push(complexity);

        // Coupling dimension (0-1)
        let coupling = node.properties.get("coupling")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5);
        coordinates.push(coupling);

        // Abstraction dimension (0-5)
        let abstraction = match concept_type {
            ConceptType::Entity | ConceptType::ValueObject => 2.0,
            ConceptType::Aggregate => 3.0,
            ConceptType::Policy => 4.0,
            _ => 1.0,
        };
        coordinates.push(abstraction);

        ConceptualPoint::new(coordinates)
    }

    /// Import a domain model from JSON format
    pub fn import_json(&self, json_content: &str) -> Result<ConceptGraph, DomainError> {
        let model: DomainModel = serde_json::from_str(json_content)
            .map_err(|e| DomainError::ValidationFailed(format!("Invalid JSON: {}", e)))?;

        self.convert_to_concept_graph(model)
    }

    /// Import from PlantUML class diagram
    pub fn import_plantuml(&self, plantuml_content: &str) -> Result<ConceptGraph, DomainError> {
        let model = self.parse_plantuml(plantuml_content)?;
        self.convert_to_concept_graph(model)
    }

    /// Convert a domain model to a concept graph
    fn convert_to_concept_graph(&self, model: DomainModel) -> Result<ConceptGraph, DomainError> {
        let mut graph = ConceptGraph::new(&model.name);

        // Add quality dimensions
        for dim in &self.quality_dimensions {
            graph = graph.with_dimension(dim.clone());
        }

        // Track node indices for edge creation
        let mut node_indices = HashMap::new();

        // Add bounded contexts as subgraphs
        for context in model.bounded_contexts {
            let context_position = self.calculate_context_position(&context);

            // Create aggregates
            for aggregate in context.aggregates {
                let node = self.create_aggregate_node(&aggregate, &context_position);
                let idx = graph.add_node(node.clone());
                node_indices.insert(aggregate.id.clone(), idx);

                // Add entities within aggregate
                for entity in aggregate.entities {
                    let entity_node = self.create_entity_node(&entity, &context_position);
                    let entity_idx = graph.add_node(entity_node.clone());
                    node_indices.insert(entity.id.clone(), entity_idx);

                    // Connect entity to aggregate
                    graph.add_edge(
                        idx,
                        entity_idx,
                        ConceptEdge::new(ConceptRelationship::PartOf),
                    );
                }

                // Add value objects
                for value_object in aggregate.value_objects {
                    let vo_node = self.create_value_object_node(&value_object, &context_position);
                    let vo_idx = graph.add_node(vo_node.clone());
                    node_indices.insert(value_object.id.clone(), vo_idx);

                    // Connect value object to aggregate
                    graph.add_edge(
                        idx,
                        vo_idx,
                        ConceptEdge::new(ConceptRelationship::PartOf),
                    );
                }
            }

            // Add domain services
            for service in context.services {
                let service_node = self.create_service_node(&service, &context_position);
                let service_idx = graph.add_node(service_node.clone());
                node_indices.insert(service.id.clone(), service_idx);

                // Connect service to aggregates it uses
                for aggregate_id in service.uses_aggregates {
                    if let Some(&agg_idx) = node_indices.get(&aggregate_id) {
                        graph.add_edge(
                            service_idx,
                            agg_idx,
                            ConceptEdge::new(ConceptRelationship::DependsOn),
                        );
                    }
                }
            }

            // Add policies
            for policy in context.policies {
                let policy_node = self.create_policy_node(&policy, &context_position);
                let policy_idx = graph.add_node(policy_node.clone());
                node_indices.insert(policy.id.clone(), policy_idx);

                // Connect policy to aggregates it affects
                for aggregate_id in policy.affects_aggregates {
                    if let Some(&agg_idx) = node_indices.get(&aggregate_id) {
                        graph.add_edge(
                            policy_idx,
                            agg_idx,
                            ConceptEdge::new(ConceptRelationship::Constrains),
                        );
                    }
                }
            }

            // Add events
            for event in context.events {
                let event_node = self.create_event_node(&event, &context_position);
                let event_idx = graph.add_node(event_node.clone());
                node_indices.insert(event.id.clone(), event_idx);

                // Connect event to source aggregate
                if let Some(&source_idx) = node_indices.get(&event.source_aggregate) {
                    graph.add_edge(
                        source_idx,
                        event_idx,
                        ConceptEdge::new(ConceptRelationship::Triggers),
                    );
                }
            }
        }

        Ok(graph)
    }

    /// Calculate position in conceptual space based on context properties
    fn calculate_context_position(&self, context: &BoundedContext) -> ConceptualPoint {
        // Base position on context characteristics
        let abstraction = match context.context_type {
            ContextType::Core => 0.8,
            ContextType::Supporting => 0.5,
            ContextType::Generic => 0.3,
        };

        let complexity = (context.aggregates.len() as f64 / 10.0).min(1.0);
        let coupling = 0.3; // Default medium coupling

        ConceptualPoint::new(vec![abstraction, complexity, coupling])
    }

    /// Create an aggregate node
    fn create_aggregate_node(&self, aggregate: &Aggregate, base_position: &ConceptualPoint) -> ConceptNode {
        let position = self.adjust_position(base_position, 0.1, 0.1, 0.0);

        ConceptNode::Atom {
            id: NodeId::new(),
            concept_type: ConceptType::Aggregate,
            quality_position: position,
            properties: {
                let mut props = HashMap::new();
                props.insert("name".to_string(), serde_json::Value::String(aggregate.name.clone()));
                props.insert("id".to_string(), serde_json::Value::String(aggregate.id.clone()));
                props
            },
        }
    }

    /// Create an entity node
    fn create_entity_node(&self, entity: &Entity, base_position: &ConceptualPoint) -> ConceptNode {
        let position = self.adjust_position(base_position, 0.0, 0.0, -0.1);

        ConceptNode::Atom {
            id: NodeId::new(),
            concept_type: ConceptType::Entity,
            quality_position: position,
            properties: {
                let mut props = HashMap::new();
                props.insert("name".to_string(), serde_json::Value::String(entity.name.clone()));
                props.insert("id".to_string(), serde_json::Value::String(entity.id.clone()));
                props
            },
        }
    }

    /// Create a value object node
    fn create_value_object_node(&self, value_object: &ValueObject, base_position: &ConceptualPoint) -> ConceptNode {
        let position = self.adjust_position(base_position, -0.1, -0.2, -0.2);

        ConceptNode::Atom {
            id: NodeId::new(),
            concept_type: ConceptType::ValueObject,
            quality_position: position,
            properties: {
                let mut props = HashMap::new();
                props.insert("name".to_string(), serde_json::Value::String(value_object.name.clone()));
                props.insert("id".to_string(), serde_json::Value::String(value_object.id.clone()));
                props
            },
        }
    }

    /// Create a service node
    fn create_service_node(&self, service: &DomainService, base_position: &ConceptualPoint) -> ConceptNode {
        let position = self.adjust_position(base_position, 0.0, 0.2, 0.1);

        ConceptNode::Atom {
            id: NodeId::new(),
            concept_type: ConceptType::Function,
            quality_position: position,
            properties: {
                let mut props = HashMap::new();
                props.insert("name".to_string(), serde_json::Value::String(service.name.clone()));
                props.insert("id".to_string(), serde_json::Value::String(service.id.clone()));
                props
            },
        }
    }

    /// Create a policy node
    fn create_policy_node(&self, policy: &Policy, base_position: &ConceptualPoint) -> ConceptNode {
        let position = self.adjust_position(base_position, 0.0, 0.1, -0.1);

        ConceptNode::Atom {
            id: NodeId::new(),
            concept_type: ConceptType::Policy,
            quality_position: position,
            properties: {
                let mut props = HashMap::new();
                props.insert("name".to_string(), serde_json::Value::String(policy.name.clone()));
                props.insert("id".to_string(), serde_json::Value::String(policy.id.clone()));
                props.insert("rule".to_string(), serde_json::Value::String(policy.rule.clone()));
                props
            },
        }
    }

    /// Create an event node
    fn create_event_node(&self, event: &DomainEvent, base_position: &ConceptualPoint) -> ConceptNode {
        let position = self.adjust_position(base_position, -0.2, 0.0, -0.2);

        ConceptNode::Atom {
            id: NodeId::new(),
            concept_type: ConceptType::Event,
            quality_position: position,
            properties: {
                let mut props = HashMap::new();
                props.insert("name".to_string(), serde_json::Value::String(event.name.clone()));
                props.insert("id".to_string(), serde_json::Value::String(event.id.clone()));
                props
            },
        }
    }

    /// Adjust position relative to base position
    fn adjust_position(&self, base: &ConceptualPoint, d_abstraction: f64, d_complexity: f64, d_coupling: f64) -> ConceptualPoint {
        let mut coords = base.coordinates.clone();
        if coords.len() >= 3 {
            coords[0] = (coords[0] + d_abstraction).clamp(0.0, 1.0);
            coords[1] = (coords[1] + d_complexity).clamp(0.0, 1.0);
            coords[2] = (coords[2] + d_coupling).clamp(0.0, 1.0);
        }
        ConceptualPoint::new(coords)
    }

    /// Parse PlantUML class diagram
    fn parse_plantuml(&self, content: &str) -> Result<DomainModel, DomainError> {
        // Simple PlantUML parser - in production, use a proper parser
        let mut model = DomainModel {
            name: "Imported from PlantUML".to_string(),
            bounded_contexts: vec![],
        };

        let mut current_context = BoundedContext {
            id: Uuid::new_v4().to_string(),
            name: "Default Context".to_string(),
            context_type: ContextType::Core,
            aggregates: vec![],
            services: vec![],
            policies: vec![],
            events: vec![],
        };

        // Parse lines looking for class definitions
        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("class ") && line.contains("<<Aggregate>>") {
                let name = line.split_whitespace()
                    .nth(1)
                    .unwrap_or("Unknown")
                    .trim_end_matches('{')
                    .to_string();

                current_context.aggregates.push(Aggregate {
                    id: Uuid::new_v4().to_string(),
                    name,
                    entities: vec![],
                    value_objects: vec![],
                });
            }
            // Add more parsing logic for other DDD elements
        }

        model.bounded_contexts.push(current_context);
        Ok(model)
    }
}

/// Domain model representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainModel {
    pub name: String,
    pub bounded_contexts: Vec<BoundedContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundedContext {
    pub id: String,
    pub name: String,
    pub context_type: ContextType,
    pub aggregates: Vec<Aggregate>,
    pub services: Vec<DomainService>,
    pub policies: Vec<Policy>,
    pub events: Vec<DomainEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextType {
    Core,
    Supporting,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aggregate {
    pub id: String,
    pub name: String,
    pub entities: Vec<Entity>,
    pub value_objects: Vec<ValueObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueObject {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainService {
    pub id: String,
    pub name: String,
    pub uses_aggregates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub rule: String,
    pub affects_aggregates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub id: String,
    pub name: String,
    pub source_aggregate: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_ddd_model() {
        let importer = DomainModelImporter::new();

        let imported = ImportedGraph {
            nodes: vec![
                ImportedNode {
                    id: "user".to_string(),
                    node_type: "Entity".to_string(),
                    label: "User".to_string(),
                    position: crate::domain::value_objects::Position3D { x: 0.0, y: 0.0, z: 0.0 },
                    properties: HashMap::new(),
                },
                ImportedNode {
                    id: "email".to_string(),
                    node_type: "ValueObject".to_string(),
                    label: "Email".to_string(),
                    position: crate::domain::value_objects::Position3D { x: 1.0, y: 0.0, z: 0.0 },
                    properties: HashMap::new(),
                },
            ],
            edges: vec![
                ImportedEdge {
                    id: "user-email".to_string(),
                    source: "user".to_string(),
                    target: "email".to_string(),
                    edge_type: "contains".to_string(),
                    properties: HashMap::new(),
                },
            ],
            metadata: HashMap::new(),
        };

        let result = importer.import_ddd_model(imported);
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_domain_model_import() {
        let json = r#"{
            "name": "E-Commerce Domain",
            "bounded_contexts": [{
                "id": "order-context",
                "name": "Order Management",
                "context_type": "Core",
                "aggregates": [{
                    "id": "order-agg",
                    "name": "Order",
                    "entities": [{
                        "id": "order-item",
                        "name": "OrderItem"
                    }],
                    "value_objects": [{
                        "id": "money",
                        "name": "Money"
                    }]
                }],
                "services": [],
                "policies": [{
                    "id": "order-policy",
                    "name": "OrderValidationPolicy",
                    "rule": "Order total must be positive",
                    "affects_aggregates": ["order-agg"]
                }],
                "events": [{
                    "id": "order-placed",
                    "name": "OrderPlaced",
                    "source_aggregate": "order-agg"
                }]
            }]
        }"#;

        let importer = DomainModelImporter::new();
        let graph = importer.import_json(json).unwrap();

        assert_eq!(graph.name, "E-Commerce Domain");
        assert!(graph.node_count() > 0);
    }
}
