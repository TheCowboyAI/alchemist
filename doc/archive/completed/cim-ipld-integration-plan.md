# CIM-IPLD Integration Plan for Information Alchemist

## Overview
This plan outlines the integration of the cim-ipld library into Information Alchemist, implementing IA-specific content types and updating the event store to use CID chains.

## Phase 1: Define IA-Specific Content Types

### 1.1 Create Domain Content Types Module
- Location: `/src/domain/content_types/`
- Define GraphContent, NodeContent, EdgeContent implementing TypedContent trait
- Assign custom codec ranges:
  - GraphContent: 0x300100
  - NodeContent: 0x300101
  - EdgeContent: 0x300102
  - ConceptualSpaceContent: 0x300103
  - WorkflowContent: 0x300104

### 1.2 Implement TypedContent for Domain Events
- Wrap existing domain events with CID support
- Create EventContent wrapper implementing TypedContent
- Maintain backward compatibility with existing event structures

### 1.3 Create Content Registry
- Central registry for all IA-specific content types
- Integration with cim-ipld CodecRegistry
- Factory methods for content creation

## Phase 2: Integrate NATS Object Store

### 2.1 Create Object Store Infrastructure
- Location: `/src/infrastructure/object_store/`
- Implement NatsObjectStore wrapper around NATS Object Store
- Configure buckets for different content types:
  - `cim.ia.events` - Event content
  - `cim.ia.graphs` - Graph structures
  - `cim.ia.media` - Media content
  - `cim.ia.documents` - Document content

### 2.2 Implement Content Storage Service
- Store content by CID in NATS Object Store
- Retrieve content by CID with type safety
- Handle content versioning and deduplication

### 2.3 Create Object Store Bridge
- Bridge between domain layer and NATS Object Store
- Async/sync conversion similar to EventBridge
- Caching layer for frequently accessed content

## Phase 3: Update Event Store with CID Chains

### 3.1 Migrate to ChainedContent
- Update DomainEvent to implement ChainedContent
- Add previous_cid tracking to events
- Implement chain validation on event append

### 3.2 Update DistributedEventStore
- Store events with CID as primary identifier
- Maintain aggregate ID index for queries
- Add chain verification on retrieval

### 3.3 Create Event Chain Service
- Validate event chains for tampering
- Rebuild state from event chain
- Support branching and merging chains

## Phase 4: Implement Custom Codecs

### 4.1 Graph Structure Codec
- Efficient serialization for graph structures
- Support for conceptual space coordinates
- Preserve relationships and metadata

### 4.2 Workflow Codec
- Serialize workflow definitions
- Include execution state and history
- Support for workflow templates

### 4.3 Media Content Codec
- Handle images, videos, audio
- Extract and store metadata
- Support for thumbnails and previews

## Phase 5: Testing and Validation

### 5.1 Unit Tests
- Test each content type implementation
- Verify CID generation determinism
- Test chain validation logic

### 5.2 Integration Tests
- Test NATS Object Store integration
- Verify event chain persistence
- Test content retrieval by CID

### 5.3 Performance Tests
- Benchmark CID generation
- Test object store throughput
- Measure chain validation performance

## Phase 6: Documentation and Examples

### 6.1 API Documentation
- Document all content types
- Usage examples for each codec
- Integration patterns

### 6.2 Migration Guide
- Steps to migrate existing data
- Backward compatibility notes
- Troubleshooting guide

## Implementation Order

1. **Week 1**: Content Types and TypedContent implementations
2. **Week 1-2**: NATS Object Store integration
3. **Week 2**: Event Store CID chain migration
4. **Week 2-3**: Custom codec implementations
5. **Week 3**: Testing and validation
6. **Week 3-4**: Documentation and polish

## Success Criteria

- [ ] All domain events have CID identifiers
- [ ] Event chains are cryptographically verifiable
- [ ] Content is deduplicated in object store
- [ ] Performance meets or exceeds current implementation
- [ ] Full test coverage for new functionality
- [ ] Documentation complete and examples working

## Dependencies

- cim-ipld library (already available)
- NATS with JetStream and Object Store enabled
- async-nats with object store support
- Additional crates: blake3, multihash, cid

## Risks and Mitigations

1. **Risk**: Performance impact of CID generation
   - **Mitigation**: Implement caching, use efficient hashing

2. **Risk**: Storage overhead from content addressing
   - **Mitigation**: Deduplication, compression, retention policies

3. **Risk**: Complexity of chain validation
   - **Mitigation**: Clear abstractions, comprehensive testing

4. **Risk**: Migration of existing data
   - **Mitigation**: Backward compatibility layer, gradual migration
