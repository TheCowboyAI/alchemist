# CIM Workflow Graph

A powerful workflow graph implementation that bridges the CIM workflow domain with ContextGraph format for visualization and analysis.

## Overview

`cim-workflow-graph` provides an enhanced workflow graph structure that integrates the new CIM workflow domain model with ContextGraph projection capabilities. This enables rich workflow visualization, analysis, and integration with other systems.

## Features

### 🏗️ **Domain-Driven Design Integration**
- Built on top of the CIM workflow domain with proper DDD patterns
- Event-driven architecture with domain events and aggregates
- CQRS pattern with commands and projections

### 📊 **ContextGraph Projection**
- Automatic conversion to ContextGraph JSON format
- DOT format export for Graphviz visualization
- Rich graph statistics and analysis
- Round-trip JSON serialization/deserialization

### 🔄 **Workflow Management**
- Step dependencies and validation
- Circular dependency detection
- Workflow lifecycle management (Draft → Running → Completed)
- Step execution tracking

### 📈 **Analysis & Visualization**
- Graph statistics (nodes, edges, depth, cycles)
- Critical path analysis
- Time estimation and business value analysis
- Export capabilities for external tools

## Quick Start

### Basic Usage

```rust
use cim_workflow_graph::WorkflowGraph;
use cim_domain_workflow::value_objects::StepType;
use std::collections::HashMap;

// Create a new workflow
let mut workflow = WorkflowGraph::new(
    "Document Approval Process".to_string(),
    "Complete document approval workflow".to_string(),
)?;

// Add steps with dependencies
let draft_step = workflow.add_step(
    "Create Draft".to_string(),
    "Author creates the initial document draft".to_string(),
    StepType::Manual,
    HashMap::new(), // configuration
    Vec::new(),     // no dependencies
    Some(120),      // 2 hours
    Some("content-author".to_string()),
)?;

let review_step = workflow.add_step(
    "Technical Review".to_string(),
    "Technical expert reviews document".to_string(),
    StepType::Manual,
    HashMap::new(),
    vec![draft_step], // depends on draft
    Some(60),         // 1 hour
    Some("tech-reviewer".to_string()),
)?;

// Start the workflow
workflow.start(HashMap::new())?;

// Validate the workflow structure
workflow.validate()?;

// Export to JSON
let json = workflow.to_json()?;

// Export to DOT format for Graphviz
let dot = workflow.to_dot();
```

### Advanced Features

```rust
// Get workflow statistics
let stats = workflow.statistics();
println!("Total nodes: {}", stats.total_nodes);
println!("Step nodes: {}", stats.step_nodes);
println!("Max depth: {}", stats.max_depth);
println!("Is cyclic: {}", stats.is_cyclic);

// Find steps by type
let manual_steps = workflow.find_steps_by_type(StepType::Manual);
let automated_steps = workflow.find_steps_by_type(StepType::Automated);

// Get executable steps (ready to run)
let executable = workflow.get_executable_steps();

// Add metadata
workflow.add_tag("approval".to_string());
workflow.set_property("priority".to_string(), serde_json::json!("high"));

// Access underlying graph structure
let step_nodes = workflow.get_step_nodes();
let dependency_edges = workflow.get_dependency_edges();
```

## ContextGraph Format

The ContextGraph projection provides a standardized JSON format for workflow visualization:

```json
{
  "id": "workflow-uuid",
  "name": "Document Approval Process",
  "description": "Complete document approval workflow",
  "metadata": {
    "status": "Running",
    "created_at": "2024-01-01T10:00:00Z",
    "step_count": 5,
    "estimated_duration_minutes": 260
  },
  "nodes": [
    {
      "id": "step-uuid",
      "node_type": "step",
      "value": {
        "name": "Create Draft",
        "step_type": "Manual",
        "status": "Pending",
        "estimated_duration_minutes": 120,
        "assigned_to": "content-author"
      }
    }
  ],
  "edges": [
    {
      "id": "edge-uuid",
      "source": "step1-uuid",
      "target": "step2-uuid",
      "edge_type": "dependency"
    }
  ]
}
```

## Visualization

### Graphviz/DOT Export

Generate visual diagrams using Graphviz:

```bash
# Export DOT format from your application
cargo run --example workflow_graph_example > workflow.dot

# Generate PNG image
dot -Tpng workflow.dot -o workflow.png

# Generate SVG for web
dot -Tsvg workflow.dot -o workflow.svg
```

### Graph Statistics

The workflow graph provides comprehensive statistics:

- **Total nodes**: All nodes including start/end
- **Step nodes**: Actual workflow steps
- **Total edges**: All connections
- **Dependency edges**: Step dependencies
- **Max depth**: Longest dependency chain
- **Is cyclic**: Whether the graph has cycles

## Examples

### Document Approval Workflow

See [`examples/workflow_graph_example.rs`](examples/workflow_graph_example.rs) for a complete document approval workflow demonstration featuring:

- **5 steps**: Draft creation, technical review, editorial review, manager approval, and publishing
- **Parallel execution**: Technical and editorial reviews can run simultaneously
- **Multiple step types**: Manual, automated, and approval steps
- **Rich configuration**: Step-specific configuration and metadata
- **Time analysis**: Critical path calculation and business value analysis

Run the example:

```bash
cargo run --example workflow_graph_example
```

## API Reference

### Core Types

- **`WorkflowGraph`**: Main workflow graph structure
- **`WorkflowGraphMetadata`**: Workflow metadata and properties
- **`ContextGraph`**: Re-exported ContextGraph types for convenience
- **`WorkflowGraphError`**: Error types for workflow operations

### Key Methods

#### Workflow Management
- `new(name, description)` - Create new workflow
- `from_workflow(workflow)` - Create from existing workflow aggregate
- `start(context)` - Start workflow execution
- `complete()` - Mark workflow as completed
- `validate()` - Validate workflow structure

#### Step Management
- `add_step(...)` - Add a new step with dependencies
- `find_steps_by_type(step_type)` - Find steps by type
- `find_steps_by_status(status)` - Find steps by status
- `get_executable_steps()` - Get steps ready to execute

#### Export & Analysis
- `to_json()` - Export to ContextGraph JSON
- `from_json(json)` - Import from ContextGraph JSON
- `to_dot()` - Export to Graphviz DOT format
- `statistics()` - Get graph statistics

#### Metadata
- `add_tag(tag)` - Add metadata tag
- `set_property(key, value)` - Set metadata property
- `get_property(key)` - Get metadata property

## Dependencies

- **`cim-domain-workflow`**: Core workflow domain model
- **`cim-contextgraph`**: ContextGraph format types
- **`petgraph`**: Graph data structures (not directly used in public API)
- **`serde`**: Serialization support
- **`serde_json`**: JSON serialization
- **`chrono`**: Date/time handling

## Testing

Run all tests:

```bash
cargo test
```

Run example tests:

```bash
cargo test --example workflow_graph_example
```

## Integration

The workflow graph integrates seamlessly with:

- **CIM Domain Architecture**: Built on DDD principles
- **Event Sourcing**: Domain events for all state changes
- **ContextGraph Ecosystem**: Standard graph format
- **Visualization Tools**: Graphviz, D3.js, etc.
- **Workflow Engines**: Execution runtime integration

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test`
2. Code follows project conventions
3. New features include tests and documentation
4. Examples demonstrate real-world usage

---

For more information, see the [CIM project documentation](../README.md). 