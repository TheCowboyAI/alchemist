# Plan to Fix Documentation Inconsistencies - June 6, 2025

## Overview

This plan addresses the inconsistencies identified in the QA review of our planning documents. The goal is to achieve 100% consistency across all architecture and planning documentation.

## Immediate Actions (Today - June 6, 2025)

### 1. Update multi-system-projections-plan.md

Replace all technology-specific names with domain module names:

```
Old → New
Neo4j → GraphPersistence Module
n8n → WorkflowOrchestration Module
Paperless-NGx → DocumentIntelligence Module
SearXNG → SearchDiscovery Module
Email (SMTP) → Communication Module
Git → VersionControl Module
Nix → InfrastructureConfiguration Module
Vaultwarden → CredentialManagement Module
Trilium → KnowledgeManagement Module
RSS/Atom → ContentAggregation Module
Nginx → WebGateway Module
JSON Files → GraphPersistence Module (alternate implementation)
```

### 2. Add Communication Module to domain-driven-module-architecture.md

Define the missing Communication module:

```rust
pub trait Communication {
    async fn send_message(&self, message: Message) -> Result<MessageId>;
    async fn receive_messages(&self, channel: Channel) -> Result<Vec<Message>>;
    async fn subscribe_to_channel(&self, channel: Channel) -> Result<Subscription>;
}
```

Implementations:
- `Communication<Email>` - SMTP/IMAP
- `Communication<Matrix>` - Matrix protocol
- `Communication<Slack>` - Slack API

### 3. Update progress.json with Detailed Tracking

Add new nodes for:
- Internal projections progress
  - GraphSummaryProjection (100% complete)
  - NodeListProjection (0% - pending)
  - EdgeConnectionProjection (0% - pending)
- External module implementations (each at 0%)
- Integration test coverage metrics

## Short-term Actions (This Week - by June 13, 2025)

### 1. Create Unified Projection Architecture Document

File: `/doc/design/unified-projection-architecture.md`

Contents:
- Clear distinction between internal and external projections
- How they interact through NATS
- Event flow diagrams
- Implementation patterns

### 2. Standardize NATS Subject Naming

Update all documents to use:
```
graph.events.{aggregate}.{event}     # Core domain events
{module}.events.{capability}.{event}  # External module events
projection.events.{name}.{event}      # Internal projection events
```

### 3. Complete Missing Projections

Implement:
- NodeListProjection
- EdgeConnectionProjection

## Medium-term Actions (Next Week - by June 20, 2025)

### 1. Reference Module Implementation

Choose GraphPersistence<Neo4j> as reference implementation:
- Full ACL implementation
- NATS integration
- Error handling
- Testing patterns

### 2. Update All Planning Documents

Ensure consistent use of:
- "Read Model Projections" for internal CQRS
- "External System Projections" for integrations
- Domain module names throughout

### 3. Create Module Development Guide

Document:
- How to implement a new module
- ACL patterns
- NATS integration patterns
- Testing requirements

## Success Criteria

- [ ] All documents use consistent terminology
- [ ] No technology names in module references
- [ ] Clear distinction between projection types
- [ ] All 11 domain modules documented (including Communication)
- [ ] NATS subject hierarchy standardized
- [ ] Progress tracking includes all components

## Timeline

| Date | Action | Status |
|------|--------|--------|
| June 6, 2025 | Update multi-system-projections-plan.md | Pending |
| June 6, 2025 | Add Communication module | Pending |
| June 6, 2025 | Update progress.json | Pending |
| June 7, 2025 | Create unified projection architecture | Pending |
| June 10, 2025 | Implement NodeListProjection | Pending |
| June 11, 2025 | Implement EdgeConnectionProjection | Pending |
| June 13, 2025 | Complete reference module | Pending |
| June 20, 2025 | All documentation consistent | Pending |

## Verification

After each update:
1. Run consistency check across all documents
2. Verify no technology names remain in module contexts
3. Ensure all cross-references are accurate
4. Update this plan with completion status
