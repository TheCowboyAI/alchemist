# Immediate Actions Plan - Updated Progress

## Overview

Based on the Quality Assurance review and recent progress, this plan outlines immediate actions to complete IPLD integration and address remaining issues.

## Completed Work (Phase 1)

### ✅ CID Chain Implementation
- Implemented ChainedEvent with BLAKE3 hashing
- Created EventChain for managing event sequences
- Added comprehensive validation and tampering detection
- Implemented deterministic CID generation
- Added tests for chain validation and CID determinism

### ✅ Distributed Event Store
- Implemented DistributedEventStore with NATS JetStream
- Created stream configuration with file-based storage
- Added event persistence with acknowledgment tracking
- Implemented event retrieval by aggregate ID
- Added LRU cache for performance optimization

### ✅ Event Bridge Architecture
- Created EventBridge with crossbeam channels
- Implemented bidirectional async/sync communication
- Integrated with main application
- Added comprehensive test suite

### ✅ Dynamic Linking Issue Fixed
- Resolved bevy_dylib symbol lookup errors
- Tests now run reliably via `nix build` or `nix run`
- Nix environment properly configures all dependencies

## Priority 1: Extract CIM-IPLD as Standalone Module

### Rationale
CIM-IPLD functionality should be a reusable library across all CIM implementations, not tied to Information Alchemist specifically.

### Implementation Plan

1. **Create New Repository**
   - GitHub: `thecowboyai/cim-ipld`
   - Dual license: Apache-2.0 OR MIT
   - Set up CI/CD with GitHub Actions

2. **Extract Core Components**
   - Move `ChainedEvent` and `EventChain` to standalone lib
   - Generalize for any `TypedContent` implementation
   - Create extensible codec registry system

3. **Design Extension Points**
   - Base traits for content types
   - Codec registration API
   - Custom content type support

### Tasks
- [ ] Create new GitHub repository
- [ ] Extract and generalize CID chain code
- [ ] Implement base traits and types
- [ ] Create codec registry system
- [ ] Add comprehensive tests
- [ ] Publish initial version
- [ ] Update Information Alchemist to use external dependency

### Timeline
- Week 1: Repository setup and core extraction
- Week 2: Implementation and testing
- Week 3: Integration back into Information Alchemist

## Priority 2: IPLD Integration in Information Alchemist

### Updated Approach
Now using external `cim-ipld` library, focus on Information Alchemist-specific extensions.

### Implementation Plan

#### 2.1 Custom Content Types
```rust
// src/domain/ipld/content_types.rs
use cim_ipld::{TypedContent, ContentType};

#[derive(Serialize, Deserialize)]
pub struct GraphAggregate {
    pub id: GraphId,
    pub metadata: GraphMetadata,
    pub nodes: Vec<NodeId>,
    pub edges: Vec<EdgeId>,
}

impl TypedContent for GraphAggregate {
    const CODEC: u64 = 0x330000; // IA-specific range
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x330000);

    // Implement required methods
}
```

#### 2.2 Domain-Specific Codecs
```rust
// src/domain/ipld/codecs.rs
use cim_ipld::{CimCodec, CodecRegistry};

pub fn register_ia_codecs(registry: &mut CodecRegistry) -> Result<()> {
    registry.register(Arc::new(GraphAggregateCodec))?;
    registry.register(Arc::new(ConceptualSpaceCodec))?;
    registry.register(Arc::new(GameTheoryCodec))?;
    Ok(())
}
```

#### 2.3 Object Store Integration
```rust
// src/infrastructure/object_store/nats_object_store.rs
use cim_ipld::TypedContent;

pub struct NatsObjectStore {
    client: NatsClient,
    bucket: String,
}

impl NatsObjectStore {
    pub async fn put<T: TypedContent>(&self, content: &T) -> Result<Cid> {
        let cid = content.calculate_cid()?;
        let bytes = content.to_bytes()?;

        self.client
            .object_store()
            .put(&self.bucket, &cid.to_string(), bytes)
            .await?;

        Ok(cid)
    }
}
```

### Tasks
- [ ] Wait for cim-ipld library availability
- [ ] Define IA-specific content types
- [ ] Implement custom codecs
- [ ] Integrate NATS Object Store
- [ ] Update event store to use new types

## Priority 3: Implement Domain Tests

### Problem
No pure domain logic tests exist (0% domain test coverage).

### Solution
Create domain tests following TDD principles without Bevy/NATS dependencies.

### Running Tests
```bash
# Build and run all tests
nix build

# Run the application
nix run

# Run tests with coverage (once cargo-llvm-cov is added)
nix develop -c cargo llvm-cov --lib --no-default-features --html
```

### Test Structure
```rust
// src/domain/aggregates/graph/tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        // Given
        let id = GraphId::new();
        let metadata = GraphMetadata::new("Test Graph");

        // When
        let graph = Graph::new(id, metadata);

        // Then
        assert_eq!(graph.version(), 0);
        assert_eq!(graph.node_count(), 0);
    }
}
```

### Tasks
- [ ] Create test modules for each aggregate
- [ ] Write tests for all domain commands
- [ ] Test event application logic
- [ ] Verify business rule enforcement

## Priority 4: Add Test Coverage Metrics

### Problem
Unable to measure test coverage (cargo-tarpaulin not available).

### Solution
1. **Add cargo-llvm-cov to Nix environment**
   ```nix
   # flake.nix
   buildInputs = with pkgs; [
     cargo-llvm-cov
     # ... other dependencies
   ];
   ```

2. **Create coverage script**
   ```bash
   # scripts/coverage.sh
   #!/usr/bin/env bash
   BEVY_HEADLESS=1 nix develop -c cargo llvm-cov --lib --no-default-features --html
   ```

### Tasks
- [ ] Update flake.nix with coverage tools
- [ ] Create coverage generation script
- [ ] Add coverage badge to README
- [ ] Set 80% coverage target

## Execution Timeline

### Week 1 (Immediate)
1. **Day 1-2**: Create cim-ipld repository and structure
2. **Day 3-4**: Extract and generalize existing code
3. **Day 5**: Initial testing and documentation

### Week 2 (Short-term)
1. **Day 1-2**: Complete cim-ipld implementation
2. **Day 3-4**: Integrate back into Information Alchemist
3. **Day 5**: Begin domain tests

### Week 3 (Medium-term)
1. **Day 1-2**: Complete domain tests
2. **Day 3-4**: Set up test coverage metrics
3. **Day 5**: Documentation and polish

### Success Criteria
- [x] All tests pass with `nix build`
- [ ] cim-ipld published as standalone library
- [ ] Information Alchemist using external cim-ipld
- [ ] Domain test coverage > 80%
- [ ] Coverage metrics available in CI

## Risk Mitigation

### Library Extraction Risk
- **Impact**: High - Delays IPLD integration
- **Mitigation**: Keep extraction minimal, enhance iteratively

### Integration Risk
- **Impact**: Medium - Breaking changes during migration
- **Mitigation**: Comprehensive tests before switching

### Coverage Tool Risk
- **Impact**: Low - Delays metrics
- **Mitigation**: Use alternative tools if needed

## Next Steps

After completing these immediate actions:
1. Update progress.json with cim-ipld extraction
2. Complete Phase 1.5 with external library
3. Move to Phase 2 (Domain Model with CIM Extensions)
4. Begin dog-fooding with progress visualization

---

*Plan Updated: 2025-06-07*
*Target Completion: 3 Weeks*
*Phase 1 Status: COMPLETED ✅*
*Phase 1.5 Status: REDESIGNED 🔄*
*Dynamic Linking: RESOLVED ✅*
