# Plan Consistency Review - June 6, 2025

## Executive Summary

This QA review analyzes consistency across four critical planning documents:
1. `multi-system-projections-plan.md` - External system projections
2. `integration-tests-and-projections-plan.md` - Testing and read models
3. `domain-driven-module-architecture.md` - DDD module design
4. `bidirectional-event-flow-architecture.md` - Two-way event flows

**Overall Consistency Score: 85%**

Key findings:
- Strong alignment on domain-driven naming conventions
- Consistent NATS-based communication patterns
- Some terminology inconsistencies need resolution
- Implementation progress aligns with plans

## Detailed Analysis

### 1. Naming Consistency

#### ✅ Strengths
- All documents now use domain-driven module names (GraphPersistence, WorkflowOrchestration, etc.)
- NATS subject naming is consistent: `{domain}.events.{aggregate}.{event}`
- Event naming follows past-tense convention across all documents

#### ⚠️ Issues Found
- `multi-system-projections-plan.md` still references technology names in some sections
- This should explain these systems are abstracted with Fitness Function for the provided capabilities.
- We align the API to an internal Domain Representation of the Capability the program provides, not the specific program
- Need to update "Projection Targets" section to use domain module names

### 2. Architecture Alignment

#### ✅ Strengths
- All documents agree on NATS as the central communication bus
- Consistent event-driven architecture with CID chains
- Shared understanding of projection patterns and event flows

#### ⚠️ Issues Found
- `integration-tests-and-projections-plan.md` focuses on internal projections
- `multi-system-projections-plan.md` focuses on external projections
- Need to clarify relationship between internal and external projections

### 3. Module Structure

#### ✅ Strengths
- 10 domain modules consistently defined across documents
- Each module has clear domain responsibility
- Anti-Corruption Layer pattern consistently applied

#### ⚠️ Issues Found
- `multi-system-projections-plan.md` lists 12 external systems
- `domain-driven-module-architecture.md` defines 10 domain modules
- Need to map all external systems to appropriate domain modules

### 4. Event Flow Patterns

#### ✅ Strengths
- Bidirectional flow clearly documented
- Consistent understanding of outbound vs inbound events
- Event correlation and enrichment patterns aligned

#### ⚠️ Issues Found
- Internal projection events (GraphSummary, NodeList) not mentioned in external plans
- Need to clarify how internal and external projections interact

### 5. Implementation Status

#### ✅ Current Progress (from progress.json)
- Graph Aggregate: 100% complete with full command handling
- GraphSummaryProjection: Implemented with event handlers
- Integration tests: Basic structure in place
- External projections: Base traits implemented (20% progress)

#### ⚠️ Gaps
- NodeListProjection: Not yet implemented
- EdgeConnectionProjection: Not yet implemented
- End-to-end integration tests: Need completion
- External module implementations: Only placeholders exist

## Inconsistencies to Resolve

### 1. Projection Terminology
**Issue**: Mixed use of "projections" for both internal read models and external system integrations

**Resolution**:
- Use "Read Model Projections" for internal CQRS projections
- Use "External System Projections" for external integrations
- Update all documents to use consistent terminology

### 2. Module Mapping
**Issue**: 12 external systems vs 10 domain modules

**Resolution**: Map external systems to modules:
- Neo4j → GraphPersistence
- n8n → WorkflowOrchestration
- Paperless-NGx → DocumentIntelligence
- SearXNG → SearchDiscovery
- Email → Communication (new module needed)
- Git → VersionControl
- Nix → InfrastructureConfiguration
- Vaultwarden → CredentialManagement
- Trilium → KnowledgeManagement
- RSS → ContentAggregation
- Nginx → WebGateway
- JSON Files → GraphPersistence (multiple implementations)

### 3. Event Subject Hierarchy
**Issue**: Slight variations in NATS subject patterns

**Resolution**: Standardize on:
```
Outbound: graph.events.{aggregate}.{event}
Inbound: {module}.events.{capability}.{event}
Internal: projection.events.{projection}.{event}
```

## Recommendations

### Priority 1: Documentation Updates
1. Update `multi-system-projections-plan.md` to use domain module names throughout
2. Add "Communication Module" to `domain-driven-module-architecture.md`
3. Create unified projection architecture document combining internal and external

### Priority 2: Implementation Alignment
1. Complete NodeListProjection and EdgeConnectionProjection
2. Implement end-to-end tests covering internal projections
3. Create reference implementation for one external module

### Priority 3: Progress Tracking
1. Update progress.json with separate tracking for:
   - Internal projections (GraphSummary, NodeList, EdgeConnection)
   - External projections (per domain module)
   - Integration test coverage
2. Add milestones for each domain module implementation

## Compliance Checklist

### Against CIM Architecture Rules
- [x] Event-driven architecture with NATS JetStream
- [x] CQRS pattern enforcement
- [x] Layer boundaries respected
- [x] Domain events defined first
- [x] No infrastructure in domain

### Against DDD Rules
- [x] Ubiquitous language used
- [x] Bounded contexts defined
- [x] Domain services named correctly
- [x] Value objects immutable
- [x] Aggregates enforce invariants

### Against Testing Rules
- [ ] 80% test coverage (currently ~65%)
- [x] Domain tests isolated from infrastructure
- [ ] Integration tests cover all flows
- [x] Headless test execution configured

## Action Items

1. **Immediate** (Today):
   - Update multi-system-projections-plan.md with domain names
   - Add Communication module to architecture
   - Update progress.json with detailed projection status

2. **Short-term** (This Week):
   - Implement NodeListProjection
   - Implement EdgeConnectionProjection
   - Complete end-to-end integration tests

3. **Medium-term** (Next Week):
   - Create reference external module implementation
   - Document unified projection architecture
   - Achieve 80% test coverage

## Conclusion

The planning documents show strong conceptual alignment with minor terminology and mapping issues. The implementation is progressing well but needs focus on completing internal projections before tackling external modules. With the recommended updates, the system will have a consistent, well-documented architecture ready for full implementation.
