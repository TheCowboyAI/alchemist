# CIM Submodule Dependency Hierarchy

## Overview

This document establishes the dependency hierarchy for all 27 CIM submodules to ensure proper build and test order. Modules are organized in tiers, where each tier depends only on modules from lower tiers.

## Dependency Tiers

### Tier 0: Foundation (No CIM Dependencies)
These modules have no dependencies on other CIM modules and must be built/tested first.

1. **cim-ipld** - IPLD/CID storage foundation
2. **cim-keys** - Cryptographic key management
3. **cim-subject** - Subject/identity primitives
4. **cim-bridge** - Async/sync bridge utilities
5. **cim-component** - Base component definitions
6. **cim-contextgraph** - Context graph primitives

### Tier 1: Core Infrastructure
These modules depend only on Tier 0 modules.

7. **cim-domain** - Base domain infrastructure
   - Dependencies: `cim-subject`, `cim-component`, `cim-ipld`

### Tier 2: Extended Infrastructure
These modules depend on Tier 0-1 modules.

8. **cim-infrastructure** - NATS and messaging infrastructure
   - Dependencies: `cim-domain`

### Tier 3: Base Domains
These modules implement core domain logic and depend on Tiers 0-2.

9. **cim-domain-workflow** - Workflow domain
   - Dependencies: `cim-domain`

10. **cim-domain-person** - Person domain
    - Dependencies: `cim-domain`, `cim-subject`

11. **cim-domain-organization** - Organization domain
    - Dependencies: `cim-domain`, `cim-subject`

12. **cim-domain-policy** - Policy domain
    - Dependencies: `cim-domain`, `cim-subject`

13. **cim-domain-location** - Location domain
    - Dependencies: `cim-domain`, `cim-subject`, `cim-infrastructure`

14. **cim-domain-nix** - Nix configuration domain
    - Dependencies: `cim-domain`, `cim-infrastructure`

15. **cim-domain-graph** - Graph domain
    - Dependencies: `cim-domain`, `cim-infrastructure`

### Tier 4: Advanced Domains
These modules depend on multiple domains or advanced infrastructure.

16. **cim-domain-conceptualspaces** - Conceptual spaces domain
    - Dependencies: `cim-component`, `cim-domain`, `cim-ipld`

17. **cim-domain-agent** - Agent domain
    - Dependencies: `cim-domain`, `cim-subject`

18. **cim-domain-document** - Document domain
    - Dependencies: `cim-domain`, `cim-infrastructure`, `cim-ipld`

19. **cim-domain-identity** - Identity domain
    - Dependencies: `cim-domain`, `cim-component`, `cim-domain-conceptualspaces`

20. **cim-domain-dialog** - Dialog domain
    - Dependencies: `cim-domain`, `cim-domain-conceptualspaces`

### Tier 5: Cross-Domain Integration
These modules integrate multiple domains.

21. **cim-domain-git** - Git integration domain
    - Dependencies: `cim-domain`, `cim-infrastructure`, `cim-ipld`, `cim-subject`, 
      `cim-domain-graph`, `cim-domain-document`, `cim-domain-agent`

22. **cim-conceptgraph** - Concept graph integration
    - Dependencies: `cim-contextgraph`, `cim-domain`

23. **cim-ipld-graph** - IPLD graph storage
    - Dependencies: `cim-contextgraph`, `cim-ipld`

24. **cim-domain-bevy** - Bevy ECS integration
    - Dependencies: `cim-contextgraph`, `cim-domain`

### Tier 6: Composition Layer
These modules compose multiple domains and integrations.

25. **cim-workflow-graph** - Workflow graph visualization
    - Dependencies: `cim-contextgraph`, `cim-domain`, `cim-domain-workflow`

26. **cim-compose** - Domain composition framework
    - Dependencies: `cim-domain`, `cim-domain-document`, `cim-domain-graph`, 
      `cim-domain-person`, `cim-domain-workflow`, `cim-domain-location`, 
      `cim-domain-agent`, `cim-domain-organization`, `cim-domain-conceptualspaces`

### Tier 7: Application Layer
Top-level applications that use all other modules.

27. **cim-agent-alchemist** - AI agent application
    - Dependencies: `cim-domain-agent`, `cim-domain-dialog`, `cim-domain-identity`, 
      `cim-domain-graph`, `cim-domain-conceptualspaces`, `cim-domain-workflow`, 
      `cim-infrastructure`, `cim-bridge`

## Testing Order

Based on the dependency hierarchy, modules should be tested in the following order:

### Phase 1: Foundation (Tier 0)
1. cim-ipld
2. cim-keys
3. cim-subject
4. cim-bridge
5. cim-component
6. cim-contextgraph

### Phase 2: Core Infrastructure (Tiers 1-2)
7. cim-domain
8. cim-infrastructure

### Phase 3: Base Domains (Tier 3)
9. cim-domain-workflow
10. cim-domain-person
11. cim-domain-organization
12. cim-domain-policy
13. cim-domain-location
14. cim-domain-nix
15. cim-domain-graph

### Phase 4: Advanced Domains (Tier 4)
16. cim-domain-conceptualspaces
17. cim-domain-agent
18. cim-domain-document
19. cim-domain-identity
20. cim-domain-dialog

### Phase 5: Integration (Tier 5)
21. cim-domain-git
22. cim-conceptgraph
23. cim-ipld-graph
24. cim-domain-bevy

### Phase 6: Composition (Tier 6)
25. cim-workflow-graph
26. cim-compose

### Phase 7: Applications (Tier 7)
27. cim-agent-alchemist

## Implementation Strategy

1. **Start with Tier 0**: These modules have no CIM dependencies and can be tested independently
2. **Fix failing tests as we go**: Each module's tests should pass before moving to dependent modules
3. **Validate event flows**: Ensure each module properly publishes events to NATS
4. **Document issues**: Track any discovered issues for later resolution
5. **Update progress.json**: Mark each module as tests are completed

## Dependency Visualization

```mermaid
graph TD
    %% Tier 0
    ipld[cim-ipld]
    keys[cim-keys]
    subject[cim-subject]
    bridge[cim-bridge]
    component[cim-component]
    contextgraph[cim-contextgraph]
    
    %% Tier 1
    domain[cim-domain]
    
    %% Tier 2
    infrastructure[cim-infrastructure]
    
    %% Tier 3
    workflow[cim-domain-workflow]
    person[cim-domain-person]
    organization[cim-domain-organization]
    policy[cim-domain-policy]
    location[cim-domain-location]
    nix[cim-domain-nix]
    graph[cim-domain-graph]
    
    %% Tier 4
    conceptualspaces[cim-domain-conceptualspaces]
    agent[cim-domain-agent]
    document[cim-domain-document]
    identity[cim-domain-identity]
    dialog[cim-domain-dialog]
    
    %% Tier 5
    git[cim-domain-git]
    conceptgraph[cim-conceptgraph]
    ipldgraph[cim-ipld-graph]
    bevy[cim-domain-bevy]
    
    %% Tier 6
    workflowgraph[cim-workflow-graph]
    compose[cim-compose]
    
    %% Tier 7
    alchemist[cim-agent-alchemist]
    
    %% Dependencies
    domain --> subject
    domain --> component
    domain --> ipld
    
    infrastructure --> domain
    
    workflow --> domain
    person --> domain
    person --> subject
    organization --> domain
    organization --> subject
    policy --> domain
    policy --> subject
    location --> domain
    location --> subject
    location --> infrastructure
    nix --> domain
    nix --> infrastructure
    graph --> domain
    graph --> infrastructure
    
    conceptualspaces --> component
    conceptualspaces --> domain
    conceptualspaces --> ipld
    agent --> domain
    agent --> subject
    document --> domain
    document --> infrastructure
    document --> ipld
    identity --> domain
    identity --> component
    identity --> conceptualspaces
    dialog --> domain
    dialog --> conceptualspaces
    
    git --> domain
    git --> infrastructure
    git --> ipld
    git --> subject
    git --> graph
    git --> document
    git --> agent
    
    conceptgraph --> contextgraph
    conceptgraph --> domain
    ipldgraph --> contextgraph
    ipldgraph --> ipld
    bevy --> contextgraph
    bevy --> domain
    
    workflowgraph --> contextgraph
    workflowgraph --> domain
    workflowgraph --> workflow
    
    compose --> domain
    compose --> document
    compose --> graph
    compose --> person
    compose --> workflow
    compose --> location
    compose --> agent
    compose --> organization
    compose --> conceptualspaces
    
    alchemist --> agent
    alchemist --> dialog
    alchemist --> identity
    alchemist --> graph
    alchemist --> conceptualspaces
    alchemist --> workflow
    alchemist --> infrastructure
    alchemist --> bridge
```

## Notes

- Some modules may have circular dependencies that need to be resolved
- Optional dependencies in cim-compose are treated as required for testing purposes
- The main application module is not included as it depends on all submodules
- Test fixtures and utilities should be shared where possible to avoid duplication 