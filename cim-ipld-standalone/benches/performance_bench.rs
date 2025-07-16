//! Performance benchmarks for CIM-IPLD
//!
//! Benchmarks for measuring performance of core operations.
//!
//! Run with: cargo bench
//!
//! ## Benchmark Categories
//!
//! ```mermaid
//! graph TD
//!     A[Performance Benchmarks] --> B[Storage Operations]
//!     A --> C[Chain Operations]
//!     A --> D[Cache Performance]
//!     A --> E[Codec Performance]
//!
//!     B --> B1[Small Content]
//!     B --> B2[Large Content]
//!     B --> B3[Batch Operations]
//!
//!     C --> C1[Chain Creation]
//!     C --> C2[Chain Validation]
//!     C --> C3[Chain Traversal]
//!
//!     D --> D1[Cache Hit]
//!     D --> D2[Cache Miss]
//!     D --> D3[Cache Eviction]
//!
//!     E --> E1[Encode Speed]
//!     E --> E2[Decode Speed]
//!     E --> E3[Compression Ratio]
//! ```

#![feature(test)]
extern crate test;

use cim_ipld::chain::{ChainedContent, ContentChain};
use cim_ipld::codec::CimCodec;
use cim_ipld::object_store::NatsObjectStore;
use cim_ipld::*;
use std::collections::HashMap;
use test::Bencher;
use tokio::runtime::Runtime;

// Test content for benchmarks
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct BenchContent {
    data: Vec<u8>,
    metadata: HashMap<String, String>,
}

impl TypedContent for BenchContent {
    fn content_type() -> ContentType {
        ContentType::new("bench", "content")
    }
}

/// Generate test data of specified size
fn generate_bench_data(size: usize) -> Vec<u8> {
    vec![0xAB; size]
}

/// Create a test runtime for async benchmarks
fn create_runtime() -> Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

// Storage benchmarks

#[bench]
fn bench_store_small_content(b: &mut Bencher) {
    let rt = create_runtime();
    let storage = rt.block_on(async {
        NatsObjectStore::new("nats://localhost:4222", "bench-bucket")
            .await
            .expect("Failed to create storage")
    });

    let content = BenchContent {
        data: generate_bench_data(1024), // 1KB
        metadata: HashMap::new(),
    };

    b.iter(|| rt.block_on(async { test::black_box(storage.put(&content).await.unwrap()) }));
}

#[bench]
fn bench_store_medium_content(b: &mut Bencher) {
    let rt = create_runtime();
    let storage = rt.block_on(async {
        NatsObjectStore::new("nats://localhost:4222", "bench-bucket")
            .await
            .expect("Failed to create storage")
    });

    let content = BenchContent {
        data: generate_bench_data(102_400), // 100KB
        metadata: HashMap::new(),
    };

    b.iter(|| rt.block_on(async { test::black_box(storage.put(&content).await.unwrap()) }));
}

#[bench]
fn bench_store_large_content(b: &mut Bencher) {
    let rt = create_runtime();
    let storage = rt.block_on(async {
        NatsObjectStore::new("nats://localhost:4222", "bench-bucket")
            .await
            .expect("Failed to create storage")
    });

    let content = BenchContent {
        data: generate_bench_data(10_485_760), // 10MB
        metadata: HashMap::new(),
    };

    b.iter(|| rt.block_on(async { test::black_box(storage.put(&content).await.unwrap()) }));
}

#[bench]
fn bench_retrieve_with_cache_hit(b: &mut Bencher) {
    let rt = create_runtime();
    let storage = rt.block_on(async {
        NatsObjectStore::new("nats://localhost:4222", "bench-bucket")
            .await
            .expect("Failed to create storage")
    });

    // Pre-store content
    let content = BenchContent {
        data: generate_bench_data(10_240), // 10KB
        metadata: HashMap::new(),
    };

    let cid = rt.block_on(async { storage.put(&content).await.unwrap() });

    // Warm up cache
    rt.block_on(async {
        let _: BenchContent = storage.get(&cid).await.unwrap();
    });

    b.iter(|| {
        rt.block_on(async {
            let retrieved: BenchContent = test::black_box(storage.get(&cid).await.unwrap());
            retrieved
        })
    });
}

#[bench]
fn bench_retrieve_cache_miss(b: &mut Bencher) {
    let rt = create_runtime();
    let storage = rt.block_on(async {
        NatsObjectStore::new("nats://localhost:4222", "bench-bucket")
            .await
            .expect("Failed to create storage")
    });

    // Pre-store many items to ensure cache eviction
    let mut cids = Vec::new();
    for i in 0..1000 {
        let content = BenchContent {
            data: generate_bench_data(1024),
            metadata: vec![("index".to_string(), i.to_string())]
                .into_iter()
                .collect(),
        };

        let cid = rt.block_on(async { storage.put(&content).await.unwrap() });
        cids.push(cid);
    }

    // Benchmark retrieval of items likely evicted from cache
    let mut index = 0;
    b.iter(|| {
        rt.block_on(async {
            let cid = &cids[index % cids.len()];
            let retrieved: BenchContent = test::black_box(storage.get(cid).await.unwrap());
            index += 1;
            retrieved
        })
    });
}

// Chain benchmarks

#[bench]
fn bench_chain_append(b: &mut Bencher) {
    let mut chain = ContentChain::<BenchContent>::new();
    let content = BenchContent {
        data: generate_bench_data(1024),
        metadata: HashMap::new(),
    };

    b.iter(|| {
        let chained = ChainedContent {
            content: content.clone(),
            previous_cid: chain.head(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        test::black_box(chain.append(chained).unwrap())
    });
}

#[bench]
fn bench_chain_validation_100_items(b: &mut Bencher) {
    let mut chain = ContentChain::<BenchContent>::new();

    // Build chain with 100 items
    for i in 0..100 {
        let content = BenchContent {
            data: generate_bench_data(1024),
            metadata: vec![("index".to_string(), i.to_string())]
                .into_iter()
                .collect(),
        };

        let chained = ChainedContent {
            content,
            previous_cid: chain.head(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        chain.append(chained).unwrap();
    }

    b.iter(|| test::black_box(chain.validate().unwrap()));
}

#[bench]
fn bench_chain_validation_1000_items(b: &mut Bencher) {
    let mut chain = ContentChain::<BenchContent>::new();

    // Build chain with 1000 items
    for i in 0..1000 {
        let content = BenchContent {
            data: generate_bench_data(256), // Smaller to speed up creation
            metadata: vec![("index".to_string(), i.to_string())]
                .into_iter()
                .collect(),
        };

        let chained = ChainedContent {
            content,
            previous_cid: chain.head(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        chain.append(chained).unwrap();
    }

    b.iter(|| test::black_box(chain.validate().unwrap()));
}

#[bench]
fn bench_chain_traversal(b: &mut Bencher) {
    let mut chain = ContentChain::<BenchContent>::new();

    // Build chain with 100 items
    for i in 0..100 {
        let content = BenchContent {
            data: generate_bench_data(1024),
            metadata: vec![("index".to_string(), i.to_string())]
                .into_iter()
                .collect(),
        };

        let chained = ChainedContent {
            content,
            previous_cid: chain.head(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        chain.append(chained).unwrap();
    }

    b.iter(|| {
        let mut count = 0;
        for item in chain.iter() {
            test::black_box(&item.content);
            count += 1;
        }
        count
    });
}

// Codec benchmarks

#[bench]
fn bench_encode_small_json(b: &mut Bencher) {
    let codec = CimCodec::default();
    let content = BenchContent {
        data: generate_bench_data(1024),
        metadata: vec![
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ]
        .into_iter()
        .collect(),
    };

    b.iter(|| test::black_box(codec.encode(&content).unwrap()));
}

#[bench]
fn bench_decode_small_json(b: &mut Bencher) {
    let codec = CimCodec::default();
    let content = BenchContent {
        data: generate_bench_data(1024),
        metadata: vec![
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ]
        .into_iter()
        .collect(),
    };

    let encoded = codec.encode(&content).unwrap();

    b.iter(|| {
        let decoded: BenchContent = test::black_box(codec.decode(&encoded).unwrap());
        decoded
    });
}

#[bench]
fn bench_encode_with_compression(b: &mut Bencher) {
    let codec = CimCodec::with_compression(true);

    // Create highly compressible content
    let mut metadata = HashMap::new();
    for i in 0..100 {
        metadata.insert(format!("key{i}"), "repeated_value".to_string());
    }

    let content = BenchContent {
        data: vec![0xFF; 10_240], // 10KB of repeated data
        metadata,
    };

    b.iter(|| test::black_box(codec.encode(&content).unwrap()));
}

#[bench]
fn bench_decode_with_compression(b: &mut Bencher) {
    let codec = CimCodec::with_compression(true);

    // Create highly compressible content
    let mut metadata = HashMap::new();
    for i in 0..100 {
        metadata.insert(format!("key{i}"), "repeated_value".to_string());
    }

    let content = BenchContent {
        data: vec![0xFF; 10_240], // 10KB of repeated data
        metadata,
    };

    let encoded = codec.encode(&content).unwrap();

    b.iter(|| {
        let decoded: BenchContent = test::black_box(codec.decode(&encoded).unwrap());
        decoded
    });
}

// Batch operation benchmarks

#[bench]
fn bench_batch_store_10_items(b: &mut Bencher) {
    let rt = create_runtime();
    let storage = rt.block_on(async {
        NatsObjectStore::new("nats://localhost:4222", "bench-bucket")
            .await
            .expect("Failed to create storage")
    });

    let contents: Vec<BenchContent> = (0..10)
        .map(|i| BenchContent {
            data: generate_bench_data(1024),
            metadata: vec![("index".to_string(), i.to_string())]
                .into_iter()
                .collect(),
        })
        .collect();

    b.iter(|| {
        rt.block_on(async {
            let mut cids = Vec::new();
            for content in &contents {
                cids.push(storage.put(content).await.unwrap());
            }
            test::black_box(cids)
        })
    });
}

#[bench]
fn bench_batch_retrieve_10_items(b: &mut Bencher) {
    let rt = create_runtime();
    let storage = rt.block_on(async {
        NatsObjectStore::new("nats://localhost:4222", "bench-bucket")
            .await
            .expect("Failed to create storage")
    });

    // Pre-store items
    let cids: Vec<Cid> = rt.block_on(async {
        let mut cids = Vec::new();
        for i in 0..10 {
            let content = BenchContent {
                data: generate_bench_data(1024),
                metadata: vec![("index".to_string(), i.to_string())]
                    .into_iter()
                    .collect(),
            };
            cids.push(storage.put(&content).await.unwrap());
        }
        cids
    });

    b.iter(|| {
        rt.block_on(async {
            let mut contents = Vec::new();
            for cid in &cids {
                let content: BenchContent = storage.get(cid).await.unwrap();
                contents.push(content);
            }
            test::black_box(contents)
        })
    });
}

// CID calculation benchmarks

#[bench]
fn bench_cid_calculation_small(b: &mut Bencher) {
    let data = generate_bench_data(1024);

    b.iter(|| test::black_box(cim_ipld::calculate_cid(&data)));
}

#[bench]
fn bench_cid_calculation_large(b: &mut Bencher) {
    let data = generate_bench_data(1_048_576); // 1MB

    b.iter(|| test::black_box(cim_ipld::calculate_cid(&data)));
}
