# DDD Compliance Reassessment Report

## Overview

This report reassesses our design documents against the updated DDD naming conventions. The rules now consistently enforce pure domain language without technical suffixes.

## Key Rule Changes

1. **Events**: No "Event" suffix - events are named as past-tense facts
2. **Services**: Named as `ServiceContext` (verb phrases revealing intent)
3. **Repositories**: Named as `DomainContext` (plural domain terms)
4. **No technical suffixes** unless part of the domain language

## Current Violations in Design Documents

### 1. Event Naming (Rule #6)

**Current (Violates Rule):**
- `GraphCreatedEvent`
- `NodeAddedEvent`
- `EdgeConnectedEvent`
- `GraphViewChangedEvent`

**Should Be:**
- `GraphCreated`
- `NodeAdded`
- `EdgeConnected`
- `GraphViewChanged`

### 2. Service Naming (Rule #3)

**Current (Violates Rule):**
- `LayoutEngine`
- `GraphAnalyzer`
- `GraphValidator`
- `ConflictResolver`

**Should Be (ServiceContext pattern):**
- `ApplyGraphLayout`
- `AnalyzeGraph`
- `ValidateGraph`
- `ResolveConflicts`

### 3. Repository/Storage Naming (Rule #4)

**Current:**
- `GraphCatalog` ✅ (already compliant - domain term)

**Alternative compliant names:**
- `Graphs` (plural domain context)
- `GraphCollection` (if "Collection" is a domain term)

### 4. Component Naming (Rule #8)

**Current (Too Generic):**
- `Animator`
- `Timeline`
- `GraphFormats`

**Should Be (Intention-Revealing):**
- `ReplayGraphChanges`
- `GraphEventTimeline`
- `ImportExportGraphFormats`

## Document-by-Document Assessment

### graph-event-storming-session.md
- ❌ All events have "Event" suffix
- ❌ Domain components don't follow ServiceContext pattern
- ✅ Aggregates and entities follow rules
- ✅ Value objects are properly named

### graph-aggregate-implementation-roadmap.md
- ❌ Events have "Event" suffix throughout
- ❌ Components like `GraphCreator`, `LayoutEngine` don't follow patterns
- ✅ `GraphCatalog` is acceptable as domain term
- ✅ Aggregates properly named

### graph-aggregate-ddd-compliant.md
- ❌ Ironically, this "DDD-compliant" document violates the new rules
- ❌ All events have "Event" suffix
- ❌ Components don't follow ServiceContext pattern

### graph-domain-final-naming.md
- ❌ States "All events end with 'Event' suffix" - directly contradicts new rules
- ❌ Lists components that don't follow new patterns
- ❌ Needs complete revision

## Recommended Actions

### Immediate Changes Needed

1. **Remove "Event" suffix from all events**
   ```rust
   // Before
   pub struct GraphCreatedEvent { }

   // After
   pub struct GraphCreated { }
   ```

2. **Rename services to ServiceContext pattern**
   ```rust
   // Before
   pub struct LayoutEngine { }

   // After
   pub struct ApplyGraphLayout { }
   ```

3. **Use intention-revealing names**
   ```rust
   // Before
   pub struct Animator { }

   // After
   pub struct ReplayGraphChanges { }
   ```

### Topic Naming (Rule #7)

Our event topics should follow:
- `graphs.created` (plural for collections)
- `node.added` (singular for entity events)
- `edges.connected` (plural for edge collections)

## Summary

With the updated rules being internally consistent, we now have clear direction:
- **No technical suffixes** (including "Event")
- **ServiceContext pattern** for services (verb phrases)
- **DomainContext pattern** for repositories (plural)
- **Intention-revealing names** throughout

All design documents need updating to comply with these clarified rules. The changes will make our codebase more aligned with pure domain language.

## Completion Status

### ✅ Design Consolidation Complete

All design documents have been updated and consolidated:

1. **Removed 9 deprecated documents** that violated the new rules
2. **Created 3 clean documents**:
   - `graph-domain-design.md` - Complete DDD-compliant specification
   - `graph-current-state-analysis.md` - Gap analysis with current code
   - `graph-implementation-roadmap.md` - Clear implementation plan
3. **Added README.md** to explain the design structure

The `/doc/design` folder is now clean, consistent, and fully compliant with the updated DDD rules. Implementation can proceed with confidence using these documents as the authoritative source.
