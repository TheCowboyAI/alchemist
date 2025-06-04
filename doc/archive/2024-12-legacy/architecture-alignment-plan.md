# Architecture Alignment Plan

## Overview

This plan addresses the critical misalignment between the current implementation (simple graph editor) and the proposed EventStore-CQRS architecture with NATS JetStream. We need to make a strategic decision about the project direction.

## Current State Assessment

### What We Have (Working)
- **Simple Event-Driven Architecture**: Using Bevy's built-in event system
- **Feature-Complete Graph Editor**: All 6 phases implemented
  - Graph creation and manipulation
  - Node/edge visualization
  - Selection system with multi-select
  - Force-directed layout algorithms
  - Import/export with file dialogs
  - Storage layer using Daggy
- **106/114 Tests Passing**: Good test coverage
- **DDD-Compliant Code**: Proper naming conventions throughout

### What's Proposed (New Architecture)
- **NATS JetStream EventStore**: Distributed event persistence
- **CID-Chained Events**: Cryptographic integrity
- **CQRS Pattern**: Separated read/write models
- **Async/Sync Bridge**: Complex integration layer
- **Component Deduplication**: Memory optimization
- **100K+ Node Performance**: Enterprise scale

## Strategic Options

### Option 1: Preserve Current Architecture (Recommended)

**Approach**: Keep the simple, working architecture and enhance it incrementally.

**Benefits**:
- Preserves all working features
- No breaking changes for users
- Faster time to additional features
- Lower complexity and maintenance burden
- Can still achieve good performance

**Next Steps**:
1. Document current architecture thoroughly
2. Benchmark current performance limits
3. Identify specific bottlenecks
4. Implement targeted optimizations
5. Add features users actually need

### Option 2: Gradual Migration to Event Sourcing

**Approach**: Add event sourcing capabilities alongside current system.

**Benefits**:
- Maintains backward compatibility
- Allows testing new architecture
- Can migrate incrementally
- Lower risk than full rewrite

**Implementation Path**:
1. Add local event store (no NATS initially)
2. Mirror current events to event store
3. Build read models from events
4. Gradually move features to CQRS
5. Add NATS only if distributed system needed

### Option 3: Full Architecture Pivot (Not Recommended)

**Approach**: Implement the full EventStore-CQRS-NATS architecture.

**Risks**:
- Breaks all existing functionality
- 6-8 week implementation timeline
- Complex distributed system overhead
- May not be needed for use case
- High maintenance burden

**Only Consider If**:
- Multiple distributed nodes required
- Cryptographic audit trail needed
- Enterprise-scale performance critical
- Team has distributed systems expertise

## Recommended Action Plan

### Phase 1: Architecture Decision (Week 1)

1. **Performance Benchmarking**
   ```bash
   # Create benchmark suite
   cargo bench --features benchmark

   # Test with increasing node counts
   # 100, 1K, 10K, 100K nodes
   ```

2. **Requirements Validation**
   - Survey actual users for needs
   - Document performance requirements
   - Identify distributed system needs
   - Validate complexity tolerance

3. **Architecture Documentation**
   - Document current architecture
   - Create architecture decision record (ADR)
   - Define migration criteria
   - Set performance targets

### Phase 2: Incremental Improvements (Week 2-3)

If staying with current architecture:

1. **Performance Optimizations**
   - Profile current bottlenecks
   - Implement component pooling
   - Add spatial indexing for large graphs
   - Optimize rendering pipeline

2. **Event System Enhancement**
   - Add event persistence to file
   - Implement undo/redo from events
   - Create event replay for debugging
   - Add event filtering/routing

3. **Storage Improvements**
   - Add graph versioning
   - Implement incremental saves
   - Add compression for large graphs
   - Create backup/restore features

### Phase 3: Optional Event Sourcing (Week 4-6)

If gradual migration chosen:

1. **Local Event Store**
   ```rust
   // Simple file-based event store
   pub struct LocalEventStore {
       events: Vec<DomainEvent>,
       file_path: PathBuf,
   }
   ```

2. **Event Sourcing Adapter**
   - Capture current events
   - Store with timestamps
   - Build projection system
   - Test with small features

3. **CQRS Introduction**
   - Start with one bounded context
   - Build read model
   - Compare performance
   - Evaluate complexity

## Decision Criteria

### Stay with Current Architecture If:
- [ ] Current performance meets needs (< 10K nodes)
- [ ] Single-user or small team usage
- [ ] Simplicity valued over scale
- [ ] No distributed system requirements
- [ ] Fast feature delivery important

### Consider Event Sourcing If:
- [ ] Need audit trail of all changes
- [ ] Multiple users editing same graphs
- [ ] Time-travel/undo requirements
- [ ] Performance issues with current approach
- [ ] Clear path to distributed system

### Require Full NATS Architecture If:
- [ ] Multiple geographic locations
- [ ] Real-time collaboration required
- [ ] 100K+ nodes with sub-10ms queries
- [ ] Cryptographic integrity critical
- [ ] Multiple system integration needed

## Implementation Guidelines

### For Any Path Chosen:

1. **Maintain DDD Principles**
   - Keep clean bounded contexts
   - Use proper event names
   - Maintain ubiquitous language
   - Document domain model

2. **Preserve Test Coverage**
   - Keep tests passing
   - Add tests for new features
   - Benchmark performance changes
   - Test migration paths

3. **Document Decisions**
   - Create ADRs for major choices
   - Update architecture diagrams
   - Maintain migration guides
   - Document performance characteristics

## Risk Mitigation

### Technical Risks
1. **Over-Engineering**: Start simple, add complexity only when needed
2. **Performance Assumptions**: Benchmark before optimizing
3. **Migration Complexity**: Keep backward compatibility
4. **Distributed System Issues**: Avoid unless necessary

### Project Risks
1. **Scope Creep**: Define clear requirements
2. **Architecture Astronauting**: Focus on user needs
3. **Breaking Changes**: Version appropriately
4. **Maintenance Burden**: Consider long-term costs

## Success Metrics

### Short Term (1 month)
- [ ] Architecture decision documented
- [ ] Performance benchmarks completed
- [ ] User requirements validated
- [ ] Implementation plan approved

### Medium Term (3 months)
- [ ] Chosen architecture implemented
- [ ] Performance targets met
- [ ] All tests passing
- [ ] Documentation complete

### Long Term (6 months)
- [ ] System in production use
- [ ] Performance stable
- [ ] User satisfaction high
- [ ] Maintenance sustainable

## Conclusion

The project has a working graph editor that satisfies the original requirements. Any architectural changes should be driven by specific user needs and performance requirements, not theoretical benefits.

**Recommendation**: Stay with the current architecture and optimize incrementally unless there are clear, documented requirements for a distributed event-sourced system.
