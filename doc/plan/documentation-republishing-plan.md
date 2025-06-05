# Documentation Republishing Plan

## Overview
This plan outlines the strategy for updating and republishing documentation that has become obsolete due to the migration from legacy architecture to CIM-integrated event sourcing.

## Current State Analysis

### Archived Documentation Requiring Updates
1. **Requirements Overview** (`01-requirements-overview.md`)
   - Update to reflect CIM integration goals
   - Add event sourcing requirements
   - Include conceptual spaces and AI readiness

2. **Domain Model** (`02-domain-model.md`)
   - Apply DDD naming conventions (e.g., `GraphCreated` not `GraphCreatedEvent`)
   - Add event sourcing patterns
   - Include CIM conceptual mapping

3. **Technical Architecture** (`03-technical-architecture.md`)
   - Replace legacy architecture with event-sourced design
   - Add NATS integration details
   - Include Bevy ECS presentation layer
   - Document async/sync bridge patterns

4. **User Stories** (`04-user-stories.md`)
   - Update to reflect event-driven workflows
   - Add CIM-specific user stories
   - Include AI agent interaction scenarios

5. **Non-Functional Requirements** (`05-non-functional-requirements.md`)
   - Add distributed system requirements
   - Include event store performance metrics
   - Document conceptual space requirements

6. **Implementation Phases** (`06-implementation-phases.md`)
   - Replace 12-month roadmap with 8-week CIM integration plan
   - Align with current progress tracking
   - Include dog-fooding milestones

## Republishing Strategy

### Phase 1: Core Architecture Documentation (Week 1)
1. **CIM Integration Overview**
   - Merge design justification with architecture
   - Create unified CIM architecture document
   - Include visual diagrams

2. **Event Sourcing Patterns**
   - Document CQRS implementation
   - Detail event store design
   - Include code examples

3. **Domain Model Reference**
   - Update with current aggregates
   - Document event flows
   - Include conceptual mappings

### Phase 2: Implementation Guides (Week 2)
1. **Developer Guide**
   - Setup instructions with Nix
   - Testing patterns (TDD-DDD)
   - Event handling examples

2. **API Reference**
   - Command/Query interfaces
   - Event schemas
   - NATS subject conventions

3. **Integration Patterns**
   - Bevy-NATS bridge
   - Conceptual space mapping
   - AI agent protocols

### Phase 3: User Documentation (Week 3)
1. **User Guide**
   - Graph editor usage
   - Workflow design patterns
   - Conceptual navigation

2. **Administrator Guide**
   - NATS configuration
   - Event store management
   - Performance tuning

3. **AI Integration Guide**
   - Agent communication
   - Tool interfaces
   - Semantic queries

## Documentation Standards

### Naming Conventions
- Follow DDD ubiquitous language
- No technical suffixes unless domain terms
- Event names in past tense without "Event" suffix

### Structure
```
/doc/publish/
├── architecture/
│   ├── cim-overview.md
│   ├── event-sourcing.md
│   └── system-components.md
├── guides/
│   ├── developer/
│   ├── user/
│   └── admin/
├── reference/
│   ├── api/
│   ├── events/
│   └── domain/
└── tutorials/
    ├── getting-started.md
    ├── workflow-design.md
    └── ai-integration.md
```

### Quality Criteria
- All code examples must compile
- Diagrams must be SVG/PNG with source
- Cross-references must be valid
- Version aligned with progress.json

## Execution Plan

### Week 1 Tasks
- [ ] Audit all archived documentation
- [ ] Create documentation template
- [ ] Update architecture documents
- [ ] Generate system diagrams

### Week 2 Tasks
- [ ] Write developer guide
- [ ] Create API reference
- [ ] Document integration patterns
- [ ] Add code examples

### Week 3 Tasks
- [ ] Complete user guide
- [ ] Write admin documentation
- [ ] Create AI integration guide
- [ ] Final review and publishing

## Success Metrics
- All documentation reflects current CIM architecture
- No references to legacy patterns
- Complete coverage of all system components
- Validated by QA process

## Notes
- Coordinate with progress.json updates
- Ensure alignment with .cursor/rules
- Test all code examples in documentation
- Archive superseded versions appropriately
