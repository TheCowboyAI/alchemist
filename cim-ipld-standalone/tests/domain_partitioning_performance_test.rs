//! Performance tests for domain partitioning

use cim_ipld::object_store::{ContentDomain, PartitionStrategy, PatternMatcher};
use std::time::Instant;

/// Generate a large document for performance testing
fn generate_large_document(size_kb: usize) -> String {
    let base_content = "This is a test document for performance testing. ";
    let repeat_count = (size_kb * 1024) / base_content.len();
    base_content.repeat(repeat_count)
}

#[test]
fn test_pattern_matching_performance() {
    let strategy = PartitionStrategy::default();

    // Test with increasingly large documents
    let sizes = vec![1, 10, 100, 500]; // KB

    for size in sizes {
        let content = generate_large_document(size);
        let invoice_content = format!("Invoice Number: TEST-001\n{content}\nTotal Due: $1000");

        let start = Instant::now();
        let domain = strategy.determine_domain(
            Some("large_invoice.pdf"),
            Some("application/pdf"),
            Some(&invoice_content),
            None,
        );
        let duration = start.elapsed();

        assert_eq!(domain, ContentDomain::Invoices);
        println!("Pattern matching for {size}KB document: {:?}", duration);

        // Performance should be reasonable even for large documents
        assert!(
            duration.as_millis() < 100,
            "Pattern matching took too long for {}KB document",
            size
        );
    }
}

#[test]
fn test_batch_classification_performance() {
    let strategy = PartitionStrategy::default();

    // Generate 1000 different documents
    let document_count = 1000;
    let mut documents = Vec::new();

    for i in 0..document_count {
        let content = match i % 5 {
            0 => format!("Invoice Number: INV-{i}\nTotal Due: ${i * 100}"),
            1 => format!("Patient: Patient {i}\nDiagnosis: Condition {i}"),
            2 => format!("This contract between Party {i} and Party {i + 1}"),
            3 => format!("Research paper {i} with methodology and results"),
            _ => format!("Generic document number {i}"),
        };
        documents.push((format!("doc_{i}.pdf"), content));
    }

    let start = Instant::now();
    let mut classifications = Vec::new();

    for (filename, content) in &documents {
        let domain =
            strategy.determine_domain(Some(filename), Some("application/pdf"), Some(content), None);
        classifications.push(domain);
    }

    let duration = start.elapsed();
    let avg_time_per_doc = duration.as_micros() as f64 / document_count as f64;

    println!("Classified {document_count} documents in {:?}", duration);
    println!(
        "Average time per document: {:.2} microseconds",
        avg_time_per_doc
    );

    // Should be able to classify many documents quickly
    assert!(
        avg_time_per_doc < 1000.0, // Less than 1ms per document
        "Classification is too slow: {:.2} microseconds per document",
        avg_time_per_doc
    );
}

#[test]
fn test_custom_pattern_performance() {
    let mut strategy = PartitionStrategy::default();

    // Add many custom patterns
    for i in 0..50 {
        strategy.add_pattern_matcher(PatternMatcher {
            name: format!("custom_pattern_{i}"),
            keywords: vec![
                format!("keyword_a_{i}"),
                format!("keyword_b_{i}"),
                format!("keyword_c_{i}"),
            ],
            domain: ContentDomain::Documents,
            priority: 50 + i,
        });
    }

    // Test classification with many patterns
    let test_content = "This document contains keyword_a_25 and keyword_b_25 multiple times";

    let start = Instant::now();
    let iterations = 10000;

    for _ in 0..iterations {
        let _domain = strategy.determine_domain(Some("test.txt"), None, Some(test_content), None);
    }

    let duration = start.elapsed();
    let avg_time = duration.as_micros() as f64 / iterations as f64;

    println!(
        "Classification with 50+ patterns: {:.2} microseconds per operation",
        avg_time
    );

    // Should still be fast even with many patterns
    assert!(
        avg_time < 100.0,
        "Classification with many patterns is too slow: {:.2} microseconds",
        avg_time
    );
}

#[test]
fn test_mime_and_extension_lookup_performance() {
    let strategy = PartitionStrategy::default();

    // Test file extension lookup performance
    let extensions = vec![
        "pdf", "doc", "docx", "txt", "md", "jpg", "png", "mp3", "mp4", "rs", "py", "js",
    ];

    let start = Instant::now();
    let iterations = 100000;

    for i in 0..iterations {
        let ext = extensions[i % extensions.len()];
        let _domain = strategy.determine_domain(Some(&format!("file.{ext}")), None, None, None);
    }

    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / iterations as f64;

    println!(
        "Extension lookup performance: {:.2} nanoseconds per lookup",
        avg_time
    );

    // Should be very fast for simple lookups
    assert!(
        avg_time < 1000.0, // Less than 1 microsecond
        "Extension lookup is too slow: {:.2} nanoseconds",
        avg_time
    );
}

#[test]
fn test_concurrent_classification() {
    use std::sync::Arc;
    use std::thread;

    let strategy = Arc::new(PartitionStrategy::default());
    let thread_count = 4;
    let docs_per_thread = 250;

    let start = Instant::now();
    let mut handles = vec![];

    for thread_id in 0..thread_count {
        let strategy_clone = Arc::clone(&strategy);
        let handle = thread::spawn(move || {
            let mut results = Vec::new();

            for i in 0..docs_per_thread {
                let content =
                    format!("Thread {thread_id} document {i}: Invoice Number: INV-{thread_id}-{i}");

                let domain = strategy_clone.determine_domain(
                    Some(&format!("doc_{thread_id}_{i}.pdf")),
                    Some("application/pdf"),
                    Some(&content),
                    None,
                );

                results.push(domain);
            }

            results
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    let mut total_classifications = 0;
    for handle in handles {
        let results = handle.join().unwrap();
        total_classifications += results.len();
    }

    let duration = start.elapsed();
    let total_docs = thread_count * docs_per_thread;

    println!(
        "Concurrent classification: {total_docs} documents across {thread_count} threads in {:?}",
        duration
    );
    println!(
        "Throughput: {:.0} documents/second",
        total_docs as f64 / duration.as_secs_f64()
    );

    assert_eq!(total_classifications, total_docs);
}

#[test]
fn test_memory_efficiency() {
    let strategy = PartitionStrategy::default();

    // Create many classification results to check memory usage
    let mut results = Vec::new();

    for i in 0..10000 {
        let domain = strategy.determine_domain(
            Some(&format!("file_{i}.pdf")),
            Some("application/pdf"),
            Some("Invoice content"),
            None,
        );
        results.push(domain);
    }

    // The ContentDomain enum should be small and efficient
    let size_per_domain = std::mem::size_of::<ContentDomain>();
    println!("Size of ContentDomain enum: {size_per_domain} bytes");

    assert!(
        size_per_domain <= 2, // Should be a small enum
        "ContentDomain enum is too large: {} bytes",
        size_per_domain
    );

    // Check that all results are as expected
    assert!(results.iter().all(|&d| d == ContentDomain::Invoices));
}

#[test]
fn test_worst_case_pattern_matching() {
    let strategy = PartitionStrategy::default();

    // Create content that partially matches many patterns
    let ambiguous_content = r#"
        This document contains many keywords that could match different patterns.
        
        Words like: contract, invoice, patient, diagnosis, research, methodology,
        #hashtag, @mention, configuration, settings, license, permit, financial,
        statement, prescription, agreement, terms, conditions, payment, due, tax,
        medical, record, social, media, share, like, comment, viral, trending.
        
        But the primary focus is on financial transactions and billing.
        Invoice Number: AMB-001
        Total Due: $5000
        Payment Terms: Net 30
    "#;

    let start = Instant::now();
    let iterations = 1000;

    for _ in 0..iterations {
        let domain = strategy.determine_domain(
            Some("ambiguous.pdf"),
            Some("application/pdf"),
            Some(ambiguous_content),
            None,
        );

        // Should still classify correctly based on pattern matching
        assert_eq!(domain, ContentDomain::Invoices);
    }

    let duration = start.elapsed();
    let avg_time = duration.as_micros() as f64 / iterations as f64;

    println!(
        "Worst-case pattern matching: {:.2} microseconds per classification",
        avg_time
    );

    // Even worst case should be reasonably fast
    assert!(
        avg_time < 500.0,
        "Worst-case pattern matching is too slow: {:.2} microseconds",
        avg_time
    );
}
