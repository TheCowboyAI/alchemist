# CIM Vocabulary Graph Guide

## Overview

The `vocabulary-graph.json` file represents the CIM vocabulary as a loadable graph structure. This guide explains the format and how to use it in the Information Alchemist system.

## Graph Structure

### Metadata
```json
{
  "metadata": {
    "name": "CIM Vocabulary Graph",
    "description": "...",
    "version": "1.0.0",
    "created": "2024-12-04"
  }
}
```

### Domains (Subgraphs)
Each domain represents a logical grouping of related vocabulary terms:

- **core-concepts**: Fundamental CIM architecture terms
- **event-sourcing**: Event sourcing and CQRS patterns
- **technical**: Technical infrastructure and patterns
- **graph-domain**: Graph aggregate and related concepts
- **domain-events**: Event types in the system
- **commands**: Command types for state changes
- **infrastructure**: Infrastructure components and services
- **bevy-integration**: Bevy ECS components for visualization
- **knowledge-domain**: Knowledge management concepts
- **organizational**: Organization and user management
- **business**: Business concepts and models
- **governance**: Policies and governance

Each domain includes:
- `id`: Unique identifier
- `name`: Display name
- `description`: Brief description
- `color`: Hex color for visualization

### Nodes (Vocabulary Terms)
Each node represents a vocabulary term with:
- `id`: Unique identifier (kebab-case)
- `label`: Display name
- `domain`: Which domain it belongs to
- `type`: Classification (concept, pattern, component, entity, etc.)
- `definition`: Brief definition

Node types include:
- `concept`: Core conceptual terms
- `pattern`: Design patterns
- `component`: System components
- `entity`: Domain entities
- `aggregate-root`: DDD aggregate roots
- `value-object`: DDD value objects
- `event`: Event types
- `command`: Command types
- `service`: Service components
- `repository`: Repository patterns
- `system`: Bevy systems
- `framework`: Technical frameworks
- `library`: External libraries
- `methodology`: Design methodologies

### Edges (Relationships)
Edges define relationships between vocabulary terms:
- `id`: Unique identifier
- `source`: Source node ID
- `target`: Target node ID
- `relationship`: Type of relationship
- `label`: Display label

Relationship types include:
- `specializes`: Inheritance/specialization (is-a)
- `implements`: Implementation relationship
- `contains`: Composition relationship
- `identified-by`: Identification relationship
- `connects`: Connection relationship
- `processes`: Processing relationship
- `produces`: Production/generation relationship
- `uses`: Usage relationship
- `references`: Reference relationship
- `component-of`: Part-of relationship
- `pattern-of`: Pattern membership
- `built-from`: Construction relationship
- `manages`: Management relationship
- `guided-by`: Guidance relationship

## Loading the Graph

To load this vocabulary graph into Information Alchemist:

```rust
// Example loading code
let vocabulary_json = include_str!("vocabulary-graph.json");
let vocabulary_graph: VocabularyGraph = serde_json::from_str(vocabulary_json)?;

// Create graph
let graph_id = GraphId::new();
let metadata = GraphMetadata {
    name: vocabulary_graph.metadata.name,
    created_at: SystemTime::now(),
    updated_at: SystemTime::now(),
    tags: vec!["vocabulary".to_string(), "reference".to_string()],
};

// Issue CreateGraph command
commands.send(CreateGraph { metadata });

// Add domains as subgraphs
for domain in vocabulary_graph.domains {
    // Create subgraph for each domain
    // Add nodes belonging to that domain
    // Apply domain color for visualization
}

// Add nodes
for node in vocabulary_graph.nodes {
    let content = NodeContent {
        label: node.label,
        node_type: NodeType::Vocabulary,
        properties: hashmap! {
            "domain" => node.domain,
            "type" => node.type,
            "definition" => node.definition,
        },
    };

    commands.send(AddNode {
        graph_id,
        content,
        position: None, // Use layout algorithm
    });
}

// Add edges
for edge in vocabulary_graph.edges {
    commands.send(ConnectNodes {
        graph_id,
        source: node_map[&edge.source],
        target: node_map[&edge.target],
        relationship: EdgeRelationship {
            relationship_type: edge.relationship,
            label: edge.label,
            strength: 1.0,
        },
    });
}
```

## Visualization Suggestions

When visualizing the vocabulary graph:

1. **Domain Clustering**: Group nodes by their domain, using the domain colors
2. **Hierarchical Layout**: Use a hierarchical layout for inheritance relationships
3. **Force-Directed**: Apply force-directed layout within domains
4. **Edge Styling**: Different edge styles for different relationship types
5. **Interactive Exploration**: Click nodes to see definitions and relationships

## Extending the Vocabulary

To add new terms:

1. Determine the appropriate domain
2. Add a node with unique ID and proper type
3. Define relationships to existing terms
4. Update the version number

## Use Cases

1. **Documentation Reference**: Interactive exploration of system concepts
2. **Onboarding**: Help new developers understand the architecture
3. **Design Validation**: Ensure new features align with existing patterns
4. **Knowledge Graph**: Foundation for building domain knowledge graphs

## Graph Statistics

- **Domains**: 12
- **Nodes**: 60+ vocabulary terms
- **Edges**: 48+ relationships
- **Relationship Types**: 15+ different types

This vocabulary graph serves as both documentation and a loadable knowledge structure for the CIM system.
