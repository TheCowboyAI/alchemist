# Lazy CID Evaluation Pattern

## Overview
CID (Content Identifier) calculation is computationally expensive. We should only calculate CIDs when they're actually needed, and then cache the result for the lifetime of the immutable content.

## Core Principles

1. **Lazy Evaluation** - Don't calculate CIDs until requested
2. **Memoization** - Cache calculated CIDs for reuse
3. **Invalidation** - Clear cache when content changes
4. **Immutability** - Content must be immutable while CID is cached

## Implementation Pattern

### Lazy CID Wrapper
```rust
use std::sync::OnceLock;
use cid::Cid;

#[derive(Debug, Clone)]
pub struct LazyCid<T: TypedContent> {
    content: T,
    cid_cache: OnceLock<Cid>,
}

impl<T: TypedContent> LazyCid<T> {
    pub fn new(content: T) -> Self {
        Self {
            content,
            cid_cache: OnceLock::new(),
        }
    }

    /// Get the CID, calculating it if necessary
    pub fn cid(&self) -> &Cid {
        self.cid_cache.get_or_init(|| {
            // Expensive calculation happens only once
            self.content.calculate_cid()
                .expect("CID calculation should not fail")
        })
    }

    /// Get the content
    pub fn content(&self) -> &T {
        &self.content
    }

    /// Replace content (invalidates CID cache)
    pub fn replace_content(self, new_content: T) -> Self {
        Self::new(new_content)
    }
}
```

### ContentGraph with Lazy CIDs
```rust
pub struct ContentGraph {
    // Identity
    pub id: GraphId,

    // Lazy CID for current state
    state_cid: LazyCid<GraphState>,

    // Previous CID (already calculated, so just store it)
    pub previous_cid: Option<Cid>,

    // Content nodes with lazy CIDs
    pub nodes: HashMap<NodeId, ContentNode>,
    pub edges: HashMap<EdgeId, ContentEdge>,

    // Cached calculations
    metrics_cache: OnceLock<BusinessMetrics>,
    patterns_cache: OnceLock<Vec<DetectedPattern>>,
}

pub struct ContentNode {
    pub id: NodeId,
    // Lazy CID for node content
    content: LazyCid<NodeContent>,
    pub position: Position3D,
    pub conceptual_coordinates: Vec<f64>,
}

impl ContentNode {
    pub fn content_cid(&self) -> &Cid {
        self.content.cid()
    }

    pub fn content(&self) -> &NodeContent {
        self.content.content()
    }
}
```

### Event Handling with Lazy CIDs
```rust
impl ContentGraph {
    pub fn handle_command(&mut self, cmd: ContentCommand) -> Result<Vec<DomainEvent>> {
        match cmd {
            ContentCommand::AddContent { content, position } => {
                let node_id = NodeId::new();

                // Create lazy CID wrapper
                let lazy_content = LazyCid::new(content);

                // CID is NOT calculated here
                let node = ContentNode {
                    id: node_id,
                    content: lazy_content,
                    position,
                    conceptual_coordinates: vec![],
                };

                self.nodes.insert(node_id, node);

                // Only calculate CID if event needs it
                let event = ContentAdded {
                    graph_id: self.id,
                    node_id,
                    // CID calculated here only if needed for event
                    content_cid: if self.should_include_cid_in_event() {
                        Some(node.content_cid().clone())
                    } else {
                        None
                    },
                    position,
                };

                Ok(vec![DomainEvent::ContentAdded(event)])
            }
        }
    }
}
```

### Memoized Calculations
```rust
impl ContentGraph {
    /// Get business metrics (calculated once and cached)
    pub fn metrics(&self) -> &BusinessMetrics {
        self.metrics_cache.get_or_init(|| {
            self.calculate_business_metrics()
        })
    }

    /// Get detected patterns (calculated once and cached)
    pub fn patterns(&self) -> &Vec<DetectedPattern> {
        self.patterns_cache.get_or_init(|| {
            self.detect_patterns()
        })
    }

    /// Invalidate caches when graph changes
    fn invalidate_caches(&mut self) {
        // Can't invalidate OnceLock, so we need a different approach
        // Option 1: Use RefCell<Option<T>>
        // Option 2: Create new graph instance
        // Option 3: Use versioning
    }
}
```

### Alternative: Versioned Memoization
```rust
use std::cell::RefCell;

pub struct VersionedCache<T> {
    version: u64,
    cache: RefCell<Option<(u64, T)>>,
}

impl<T> VersionedCache<T> {
    pub fn new() -> Self {
        Self {
            version: 0,
            cache: RefCell::new(None),
        }
    }

    pub fn get_or_compute<F>(&self, f: F) -> T
    where
        F: FnOnce() -> T,
        T: Clone,
    {
        let mut cache = self.cache.borrow_mut();

        match &*cache {
            Some((cached_version, value)) if *cached_version == self.version => {
                value.clone()
            }
            _ => {
                let value = f();
                *cache = Some((self.version, value.clone()));
                value
            }
        }
    }

    pub fn invalidate(&mut self) {
        self.version += 1;
    }
}
```

## Performance Benefits

1. **Deferred Computation** - CIDs only calculated when needed
2. **Single Calculation** - Each CID calculated exactly once
3. **Memory Efficient** - No unnecessary CID storage
4. **Fast Access** - Cached CIDs returned immediately

## Usage Patterns

### Pattern 1: Batch Operations
```rust
// Add multiple nodes without calculating CIDs
for content in contents {
    graph.add_content(content)?;
}

// CIDs calculated only when persisting
let cids: Vec<Cid> = graph.nodes.values()
    .map(|node| node.content_cid().clone())
    .collect();
```

### Pattern 2: Selective CID Usage
```rust
// Only calculate CID for specific nodes
if node.requires_verification() {
    let cid = node.content_cid();
    verify_cid(cid)?;
}
```

### Pattern 3: Event Sourcing
```rust
// Only include CID in events when necessary
let event = match self.event_detail_level {
    DetailLevel::Minimal => ContentAdded {
        node_id,
        content_cid: None, // No CID calculation
    },
    DetailLevel::Full => ContentAdded {
        node_id,
        content_cid: Some(node.content_cid().clone()), // CID calculated here
    },
};
```

## Testing Considerations

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_lazy_cid_calculation() {
        let content = NodeContent::new("test");
        let lazy = LazyCid::new(content);

        // CID not calculated yet
        assert!(!lazy.cid_cache.is_initialized());

        // First access calculates CID
        let cid1 = lazy.cid();
        assert!(lazy.cid_cache.is_initialized());

        // Second access returns cached value
        let cid2 = lazy.cid();
        assert_eq!(cid1, cid2);
    }
}
```

## Integration with IPLD Store

```rust
impl ObjectStore {
    /// Store content and return CID (lazy evaluation)
    pub async fn put_lazy<T: TypedContent>(&self, lazy: &LazyCid<T>) -> Result<Cid> {
        // CID calculated here if not already cached
        let cid = lazy.cid().clone();

        // Store content
        let bytes = lazy.content().to_bytes()?;
        self.store_bytes(&cid, bytes).await?;

        Ok(cid)
    }
}
```

## Best Practices

1. **Immutability** - Never modify content after creating LazyCid
2. **Clear Lifecycle** - Know when CIDs are needed
3. **Batch Operations** - Group CID calculations when possible
4. **Profile Performance** - Measure CID calculation impact
5. **Document Intent** - Make lazy evaluation explicit in APIs
