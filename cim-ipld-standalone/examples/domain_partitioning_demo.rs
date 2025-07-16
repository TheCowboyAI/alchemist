//! Example demonstrating domain-based content partitioning
//!
//! This example shows how content is automatically routed to appropriate
//! storage buckets based on domain detection.

use cim_ipld::content_types::*;
use cim_ipld::object_store::{NatsObjectStore, ContentDomain, PatternMatcher};
use cim_ipld::TypedContent;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("CIM-IPLD Domain Partitioning Demo");
    println!("=================================\n");

    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222").await?;
    let jetstream = async_nats::jetstream::new(client);

    // Create object store with domain partitioning
    let store = NatsObjectStore::new(jetstream.clone(), 1024).await?;

    // Example 1: Music file
    println!("1. Storing music file:");
    let music_content = Mp3Audio {
        data: vec![0xFF, 0xFB, 0x90, 0x00], // MP3 header
        metadata: AudioMetadata {
            duration_seconds: Some(180.0),
            bitrate: Some(320),
            sample_rate: Some(44100),
            channels: Some(2),
            codec: Some("MP3".to_string()),
            artist: Some("CIM Band".to_string()),
            album: Some("Domain Sounds".to_string()),
            title: Some("Partition Beat".to_string()),
        },
    };

    let (music_cid, music_domain) = store.put_with_domain(
        &music_content,
        Some("partition_beat.mp3"),
        Some("audio/mpeg"),
        None,
        None,
    ).await?;

    println!("  - CID: {music_cid}");
    println!("  - Domain: {:?}", music_domain);
    println!("  - Bucket: cim-media-music\n");

    // Example 2: Contract document
    println!("2. Storing contract document:");
    let contract_content = PdfDocument {
        data: b"%PDF-1.4\nThis contract is entered into between Party A and Party B...".to_vec(),
        metadata: DocumentMetadata {
            title: Some("Service Agreement".to_string()),
            author: Some("Legal Department".to_string()),
            created: Some("2024-01-15".to_string()),
            modified: Some("2024-01-15".to_string()),
            pages: Some(5),
            language: Some("en".to_string()),
            keywords: Some(vec!["contract".to_string(), "agreement".to_string()]),
        },
    };

    let (contract_cid, contract_domain) = store.put_with_domain(
        &contract_content,
        Some("service_agreement.pdf"),
        Some("application/pdf"),
        Some("This contract is entered into between Party A and Party B"),
        None,
    ).await?;

    println!("  - CID: {contract_cid}");
    println!("  - Domain: {:?}", contract_domain);
    println!("  - Bucket: cim-legal-contracts\n");

    // Example 3: Invoice
    println!("3. Storing invoice:");
    let invoice_metadata = {
        let mut meta = HashMap::new();
        meta.insert("document_type".to_string(), "invoice".to_string());
        meta.insert("invoice_number".to_string(), "INV-2024-001".to_string());
        meta
    };

    let invoice_content = PdfDocument {
        data: b"%PDF-1.4\nInvoice Number: INV-2024-001\nBill To: Customer Corp\nTotal Due: $5,000".to_vec(),
        metadata: DocumentMetadata {
            title: Some("Invoice INV-2024-001".to_string()),
            author: Some("Billing System".to_string()),
            created: Some("2024-01-15".to_string()),
            modified: Some("2024-01-15".to_string()),
            pages: Some(2),
            language: Some("en".to_string()),
            keywords: Some(vec!["invoice".to_string(), "billing".to_string()]),
        },
    };

    let (invoice_cid, invoice_domain) = store.put_with_domain(
        &invoice_content,
        Some("invoice_2024_001.pdf"),
        Some("application/pdf"),
        Some("Invoice Number: INV-2024-001\nBill To: Customer Corp"),
        Some(&invoice_metadata),
    ).await?;

    println!("  - CID: {invoice_cid}");
    println!("  - Domain: {:?}", invoice_domain);
    println!("  - Bucket: cim-finance-invoices\n");

    // Example 4: Social media meme
    println!("4. Storing social media meme:");
    let meme_content = JpegImage {
        data: vec![0xFF, 0xD8, 0xFF, 0xE0], // JPEG header
        metadata: ImageMetadata {
            width: Some(800),
            height: Some(600),
            format: Some("JPEG".to_string()),
            color_space: Some("RGB".to_string()),
            has_alpha: Some(false),
            dpi: Some(72),
            camera_make: None,
            camera_model: None,
            taken_date: None,
            gps_location: None,
        },
    };

    let meme_metadata = {
        let mut meta = HashMap::new();
        meta.insert("tags".to_string(), "#funny #meme #viral".to_string());
        meta.insert("source".to_string(), "social_media".to_string());
        meta
    };

    let (meme_cid, meme_domain) = store.put_with_domain(
        &meme_content,
        Some("funny_cat_meme.jpg"),
        Some("image/jpeg"),
        Some("LOL this is so funny! #meme #viral"),
        Some(&meme_metadata),
    ).await?;

    println!("  - CID: {meme_cid}");
    println!("  - Domain: {:?}", meme_domain);
    println!("  - Bucket: cim-social-memes\n");

    // Example 5: Medical record
    println!("5. Storing medical record:");
    let medical_content = PdfDocument {
        data: b"%PDF-1.4\nPatient: John Doe\nDiagnosis: Annual checkup\nLab Results: Normal".to_vec(),
        metadata: DocumentMetadata {
            title: Some("Medical Record - John Doe".to_string()),
            author: Some("Dr. Smith".to_string()),
            created: Some("2024-01-15".to_string()),
            modified: Some("2024-01-15".to_string()),
            pages: Some(3),
            language: Some("en".to_string()),
            keywords: Some(vec!["medical".to_string(), "patient".to_string(), "diagnosis".to_string()]),
        },
    };

    let medical_metadata = {
        let mut meta = HashMap::new();
        meta.insert("document_type".to_string(), "medical_record".to_string());
        meta.insert("patient_id".to_string(), "P12345".to_string());
        meta.insert("confidential".to_string(), "true".to_string());
        meta
    };

    let (medical_cid, medical_domain) = store.put_with_domain(
        &medical_content,
        Some("patient_record_p12345.pdf"),
        Some("application/pdf"),
        Some("Patient: John Doe\nDiagnosis: Annual checkup"),
        Some(&medical_metadata),
    ).await?;

    println!("  - CID: {medical_cid}");
    println!("  - Domain: {:?}", medical_domain);
    println!("  - Bucket: cim-health-medical\n");

    // Example 6: Source code
    println!("6. Storing source code:");
    let code_content = TextDocument {
        data: b"fn main() {\n    println!(\"Hello, CIM!\");\n}".to_vec(),
        metadata: DocumentMetadata {
            title: Some("main.rs".to_string()),
            author: Some("Developer".to_string()),
            created: Some("2024-01-15".to_string()),
            modified: Some("2024-01-15".to_string()),
            pages: None,
            language: Some("rust".to_string()),
            keywords: Some(vec!["rust".to_string(), "code".to_string()]),
        },
    };

    let (code_cid, code_domain) = store.put_with_domain(
        &code_content,
        Some("main.rs"),
        Some("text/x-rust"),
        None,
        None,
    ).await?;

    println!("  - CID: {code_cid}");
    println!("  - Domain: {:?}", code_domain);
    println!("  - Bucket: cim-tech-code\n");

    // Example 7: Custom pattern matcher
    println!("7. Adding custom pattern matcher for research papers:");
    
    store.update_partition_strategy(|strategy| {
        strategy.add_pattern_matcher(PatternMatcher {
            name: "research_paper_detector".to_string(),
            keywords: vec![
                "abstract".to_string(),
                "introduction".to_string(),
                "methodology".to_string(),
                "results".to_string(),
                "conclusion".to_string(),
                "references".to_string(),
            ],
            domain: ContentDomain::Papers,
            priority: 85,
        });
    }).await;

    let research_content = PdfDocument {
        data: b"%PDF-1.4\nAbstract: This paper presents...\nIntroduction: In recent years...\nMethodology: We used...".to_vec(),
        metadata: DocumentMetadata {
            title: Some("Novel Approach to Domain Partitioning".to_string()),
            author: Some("Research Team".to_string()),
            created: Some("2024-01-15".to_string()),
            modified: Some("2024-01-15".to_string()),
            pages: Some(12),
            language: Some("en".to_string()),
            keywords: Some(vec!["research".to_string(), "domain".to_string(), "partitioning".to_string()]),
        },
    };

    let (research_cid, research_domain) = store.put_with_domain(
        &research_content,
        Some("domain_partitioning_paper.pdf"),
        Some("application/pdf"),
        Some("Abstract: This paper presents a novel approach to domain partitioning"),
        None,
    ).await?;

    println!("  - CID: {research_cid}");
    println!("  - Domain: {:?}", research_domain);
    println!("  - Bucket: cim-academic-papers\n");

    // List contents of different domain buckets
    println!("8. Listing domain bucket contents:");
    
    let music_objects = store.list_domain(ContentDomain::Music).await?;
    println!("  - Music bucket ({music_objects.len(} objects)"));
    
    let contract_objects = store.list_domain(ContentDomain::Contracts).await?;
    println!("  - Contracts bucket ({contract_objects.len(} objects)"));
    
    let invoice_objects = store.list_domain(ContentDomain::Invoices).await?;
    println!("  - Invoices bucket ({invoice_objects.len(} objects)"));
    
    let meme_objects = store.list_domain(ContentDomain::Memes).await?;
    println!("  - Memes bucket ({meme_objects.len(} objects)"));
    
    let medical_objects = store.list_domain(ContentDomain::Medical).await?;
    println!("  - Medical bucket ({medical_objects.len(} objects)"));
    
    let code_objects = store.list_domain(ContentDomain::SourceCode).await?;
    println!("  - Source Code bucket ({code_objects.len(} objects)"));
    
    let paper_objects = store.list_domain(ContentDomain::Papers).await?;
    println!("  - Papers bucket ({paper_objects.len(} objects)"));

    // Retrieve content from domain buckets
    println!("\n9. Retrieving content from domain buckets:");
    
    let retrieved_music: Mp3Audio = store.get_from_domain(&music_cid, ContentDomain::Music).await?;
    println!("  - Retrieved music: {retrieved_music.metadata.title.as_ref(} by {}").unwrap(),
        retrieved_music.metadata.artist.as_ref().unwrap()
    );
    
    let retrieved_contract: PdfDocument = store.get_from_domain(&contract_cid, ContentDomain::Contracts).await?;
    println!("  - Retrieved contract: {retrieved_contract.metadata.title.as_ref(}").unwrap()
    );

    println!("\nDomain partitioning demo completed successfully!");
    println!("\nContent has been automatically routed to appropriate buckets based on:");
    println!("  - File extensions");
    println!("  - MIME types");
    println!("  - Content patterns");
    println!("  - Metadata hints");

    Ok(())
} 