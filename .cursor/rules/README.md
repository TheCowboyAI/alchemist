# CIM Cursor Rules Guide

## Overview

These rules help Cursor agents understand and work effectively with the Composable Information Machine (CIM) architecture. They provide specific patterns, examples, and constraints that ensure consistent, high-quality code generation.

## Rule Structure

### Core Architecture Rules
- **proven-patterns.mdc** - Validated patterns from 5/8 completed domains (62.5% complete)
- **cim-architecture.mdc** - Foundation patterns and principles
- **event-sourcing-cim.mdc** - Event store and CQRS implementation
- **conceptual-spaces.mdc** - Knowledge representation patterns

### Domain-Specific Rules
- **ddd.mdc** - Domain-Driven Design principles
- **graphs.mdc** - Graph implementation with petgraph and Bevy
- **bevy_ecs.mdc** - Bevy ECS patterns and refactoring
- **bounded-context-refactoring.mdc** - Bounded context separation and hexagonal architecture

### Technical Rules
- **rust.mdc** - Rust and NixOS environment setup
- **nixos.mdc** - NixOS-specific configurations
- **tdd.mdc** - Test-Driven Development requirements
- **bevy-testing.mdc** - Bevy-specific testing patterns

### Process Rules
- **main.mdc** - General project instructions
- **QA.mdc** - Quality assurance procedures

## How to Use These Rules Effectively

### 1. Rule Selection

When working on a feature, identify which rules apply:

```yaml
# For a new graph feature:
- Apply: cim-architecture.mdc, graphs.mdc, event-sourcing-cim.mdc
- Reference: ddd.mdc for domain modeling

# For AI integration:
- Apply: conceptual-spaces.mdc, cim-architecture.mdc
- Reference: event-sourcing-cim.mdc for event patterns

# For testing:
- Apply: tdd.mdc, bevy-testing.mdc
- Reference: rust.mdc for environment setup
```

### 2. Rule Priorities

1. **Always Active**: main.mdc, ddd.mdc, tdd.mdc
2. **Context-Specific**: Apply based on current file/feature
3. **Reference**: Use other rules as needed

### 3. Common Workflows

#### Creating a New Feature

1. Start with domain events (event-sourcing-cim.mdc)
2. Define commands and aggregates (cim-architecture.mdc)
3. Implement handlers (cim-architecture.mdc)
4. Add Bevy visualization (bevy_ecs.mdc, graphs.mdc)
5. Write tests first (tdd.mdc)

#### Adding AI Capabilities

1. Define conceptual space (conceptual-spaces.mdc)
2. Create embedding bridge (conceptual-spaces.mdc)
3. Integrate with events (event-sourcing-cim.mdc)
4. Connect to graph (graphs.mdc)

#### Refactoring Existing Code

1. Check architecture boundaries (cim-architecture.mdc)
2. Ensure event sourcing patterns (event-sourcing-cim.mdc)
3. Validate domain model (ddd.mdc)
4. Update tests (tdd.mdc)

#### Bounded Context Refactoring

1. Follow module structure (bounded-context-refactoring.mdc)
2. Extract foundation modules first (bounded-context-refactoring.mdc)
3. Create contexts with hexagonal architecture (bounded-context-refactoring.mdc)
4. Implement event translators for integration (bounded-context-refactoring.mdc)
5. Verify no circular dependencies (bounded-context-refactoring.mdc)

## Best Practices for Rule Usage

### 1. Combine Rules Intelligently

```markdown
# Good: Combining rules for a complete solution
- Use cim-architecture.mdc for overall structure
- Apply event-sourcing-cim.mdc for state management
- Reference graphs.mdc for visualization
- Follow tdd.mdc for testing

# Bad: Using rules in isolation
- Only following one rule without considering others
```

### 2. Use Rule Examples

Each rule contains concrete code examples. Use them as templates:

```rust
// From cim-architecture.mdc - Creating a new feature
// 1. Start with the event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureActivated {
    pub feature_id: FeatureId,
    pub timestamp: SystemTime,
}

// 2. Then the command
#[derive(Debug, Clone)]
pub struct ActivateFeature {
    pub feature_id: FeatureId,
}

// Follow the pattern exactly
```

### 3. Check Decision Trees

Many rules include decision trees for common scenarios:

```markdown
# From cim-architecture.mdc
When to Create a New Aggregate?
1. Does it have its own lifecycle? → Yes → New Aggregate
2. Does it enforce unique business rules? → Yes → New Aggregate
3. Can it exist independently? → Yes → New Aggregate
4. Otherwise → Add to existing aggregate
```

### 4. Avoid Anti-Patterns

Each rule lists specific anti-patterns to avoid:

```rust
// From event-sourcing-cim.mdc
❌ WRONG: Mutable events
event.timestamp = SystemTime::now(); // Never modify events!

✅ CORRECT: Immutable events
let new_event = Event {
    timestamp: SystemTime::now(),
    ..event
};
```

## Rule Metadata

### Globs
- Specify which files the rule applies to
- Use for automatic context awareness

### alwaysApply
- Set to `true` only for critical rules
- Most rules should be `false` and context-specific

### Description
- Clear, concise explanation of the rule's purpose
- Helps Cursor understand when to apply it

## Troubleshooting

### Rule Not Being Applied?
1. Check the glob patterns match your files
2. Verify alwaysApply setting
3. Ensure rule file has .mdc extension

### Conflicting Rules?
1. More specific rules override general ones
2. Check rule priorities (main.mdc is highest)
3. Use explicit rule references in your query

### Performance Issues?
1. Limit alwaysApply rules
2. Use specific globs
3. Keep rules focused and concise

## Updating Rules

When updating rules:
1. Test with example code first
2. Update related rules for consistency
3. Document changes in this README
4. Verify no conflicts with existing rules

## Rule Development Guidelines

### Creating New Rules
1. Start with a clear purpose
2. Include concrete examples
3. List anti-patterns
4. Add decision criteria
5. Test with Cursor

### Rule Template
```markdown
---
description: [Clear purpose]
globs: [Specific file patterns]
alwaysApply: false
---

# Rule Title

## Purpose
[Why this rule exists]

## Patterns
[Code examples of correct usage]

## Anti-Patterns
[What to avoid]

## Decision Criteria
[When to apply this pattern]

## Integration
[How it works with other rules]
```

## Conclusion

These rules form a comprehensive system for working with CIM. Use them together, follow the examples, and avoid the anti-patterns to ensure high-quality, consistent code that aligns with the CIM architecture.
## PROJECT STATUS UPDATE: 62.5% COMPLETE
NEW RULE: proven-patterns.mdc contains validated patterns from 5/8 completed domains
USE proven-patterns.mdc FOR ALL NEW WORK - Contains validated architecture patterns
