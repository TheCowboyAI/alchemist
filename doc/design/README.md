# Graph Domain Design Documentation

## Overview

This folder contains the complete design documentation for the Information Alchemist Graph system, following strict Domain-Driven Design (DDD) principles.

## Document Structure

### 1. [graph-domain-design.md](./graph-domain-design.md)
**The Complete Design Specification**

This is the authoritative design document that defines:
- Domain model (aggregates, entities, value objects)
- Domain events (without technical suffixes)
- Domain components (following ServiceContext pattern)
- Bounded contexts and their relationships
- Implementation architecture

All implementation must follow this design.

### 2. [graph-current-state-analysis.md](./graph-current-state-analysis.md)
**Gap Analysis**

Analyzes the current implementation against the target design:
- What exists today
- What needs to be built
- Migration requirements
- Risk assessment

Use this to understand what work remains.

### 3. [graph-implementation-roadmap.md](./graph-implementation-roadmap.md)
**Implementation Plan**

Provides the step-by-step roadmap:
- 8-week implementation schedule
- Sprint-by-sprint deliverables
- Technical details
- Success metrics

Follow this for systematic implementation.

## Key Design Principles

1. **Pure Domain Language**: No technical suffixes (Repository, Service, Handler, etc.)
2. **Event Naming**: Events are past-tense facts without "Event" suffix
3. **Service Pattern**: Services use verb phrases (e.g., `CreateGraph`, `ApplyGraphLayout`)
4. **Storage Pattern**: Storage uses plural domain terms (e.g., `Graphs`)
5. **Clear Intent**: All names reveal their business purpose

## Quick Reference

### Events
- ❌ `GraphCreatedEvent`
- ✅ `GraphCreated`

### Services
- ❌ `LayoutEngine`
- ✅ `ApplyGraphLayout`

### Storage
- ❌ `GraphRepository`
- ✅ `Graphs`

## Getting Started

1. Read `graph-domain-design.md` to understand the target architecture
2. Review `graph-current-state-analysis.md` to see what needs to be done
3. Follow `graph-implementation-roadmap.md` for implementation steps

## Compliance

All code must comply with the DDD rules defined in `.cursor/rules/ddd.mdc`. The design documents in this folder are fully compliant with those rules.
