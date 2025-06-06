# NATS Subject Naming Standard

## Overview

This document defines the standard naming convention for NATS subjects across the CIM architecture.

## Core Principles

1. **Hierarchical Structure**: Use dot notation for hierarchy
2. **Domain-First**: Start with domain context, not technology
3. **Event Semantics**: Clearly indicate event nature
4. **Consistency**: Same pattern everywhere

## Subject Patterns

### Domain Events (Core → Modules)
```
{domain}.events.{aggregate}.{event}
```

Examples:
- `graph.events.structure.node_added`
- `graph.events.structure.edge_connected`
- `graph.events.workflow.execution_requested`
- `graph.events.document.uploaded`

### Module Events (Modules → Core/Other Modules)
```
{module}.events.{capability}.{event}
```

Examples:
- `persistence.events.graph.stored`
- `orchestration.events.workflow.completed`
- `intelligence.events.entities.extracted`
- `search.events.index.updated`

### Command Subjects
```
{domain}.commands.{aggregate}.{command}
```

Examples:
- `graph.commands.structure.add_node`
- `graph.commands.workflow.execute`

### Query Subjects
```
{domain}.queries.{aggregate}.{query}
```

Examples:
- `graph.queries.structure.find_path`
- `graph.queries.summary.get_statistics`

## Wildcard Subscriptions

- `graph.events.>` - All graph events
- `graph.events.structure.>` - All structure-related events
- `*.events.>` - All events from all domains
- `persistence.events.>` - All persistence module events

## Migration from Legacy Patterns

### Old Pattern → New Pattern
- `graph.events.created` → `graph.events.lifecycle.created`
- `graph.events.deleted` → `graph.events.lifecycle.deleted`
- `node.events.added` → `graph.events.structure.node_added`
- `edge.events.connected` → `graph.events.structure.edge_connected`

## Implementation Guidelines

1. **No Technology Names**: Use domain capabilities, not implementation details
2. **Past Tense for Events**: Events describe what happened
3. **Imperative for Commands**: Commands express intent
4. **Question Form for Queries**: Queries ask for information
5. **Lowercase with Underscores**: Use snake_case for multi-word segments

## Examples by Module

### GraphPersistence Module
- Subscribes: `graph.events.structure.>`, `graph.events.lifecycle.>`
- Publishes: `persistence.events.graph.stored`, `persistence.events.graph.loaded`

### WorkflowOrchestration Module
- Subscribes: `graph.events.workflow.>`
- Publishes: `orchestration.events.workflow.started`, `orchestration.events.workflow.completed`

### DocumentIntelligence Module
- Subscribes: `graph.events.document.>`
- Publishes: `intelligence.events.entities.extracted`, `intelligence.events.knowledge.discovered`

## Validation Rules

1. Maximum 5 levels deep
2. Each segment lowercase, alphanumeric + underscore
3. No special characters except dots as separators
4. Total subject length < 255 characters

## Monitoring Subjects

For system monitoring and metrics:
- `system.metrics.{module}.{metric}`
- `system.health.{module}.{status}`
- `system.logs.{module}.{level}`

This standard ensures consistent, predictable, and maintainable event routing across the entire CIM ecosystem.
