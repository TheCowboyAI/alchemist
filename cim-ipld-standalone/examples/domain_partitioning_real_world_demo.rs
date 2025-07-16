//! Real-world demonstration of domain partitioning with various document types

use cim_ipld::object_store::{ContentDomain, PartitionStrategy};
use cim_ipld::content_types::*;
use std::collections::HashMap;

/// Create sample documents for testing
fn create_sample_documents() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        // Financial documents
        (
            "invoice_2024_001.pdf",
            "INVOICE\n\nInvoice Number: INV-2024-001\nDate: January 15, 2024\n\nBill To:\nAcme Corporation\n123 Business St\nNew York, NY 10001\n\nDescription: Professional Services\nAmount: $5,000.00\n\nTotal Due: $5,000.00",
            "Invoice for professional services"
        ),
        (
            "bank_statement_jan.pdf",
            "BANK STATEMENT\n\nAccount: ****1234\nPeriod: January 1-31, 2024\n\nBeginning Balance: $10,000\nDeposits: $5,000\nWithdrawals: $3,000\nEnding Balance: $12,000",
            "Monthly bank statement"
        ),
        
        // Medical documents
        (
            "patient_record.pdf",
            "PATIENT MEDICAL RECORD\n\nPatient: John Doe\nDOB: 01/01/1980\nMRN: 123456\n\nVisit Date: January 15, 2024\nChief Complaint: Annual checkup\n\nDiagnosis: Patient in good health\nTreatment: Continue current medications",
            "Patient medical record"
        ),
        (
            "prescription.pdf",
            "PRESCRIPTION\n\nPatient: Jane Smith\nDate: 01/15/2024\n\nRx: Amoxicillin 500mg\nSig: Take 1 capsule 3 times daily\nQuantity: 30\nRefills: 0\n\nDr. Johnson, MD",
            "Medical prescription"
        ),
        
        // Legal documents
        (
            "service_agreement.pdf",
            "SERVICE AGREEMENT\n\nThis Agreement is entered into between Company A and Company B.\n\nWHEREAS, Company A agrees to provide services...\n\nNOW THEREFORE, the parties agree as follows:\n\n1. SERVICES: As described in Exhibit A\n2. PAYMENT: $10,000 per month",
            "Service agreement contract"
        ),
        (
            "nda.docx",
            "NON-DISCLOSURE AGREEMENT\n\nThis Agreement is made between the parties identified below.\n\nCONFIDENTIAL INFORMATION: Each party agrees to maintain confidentiality...",
            "Non-disclosure agreement"
        ),
        
        // Technical documents
        (
            "api_documentation.md",
            "# API Documentation\n\n## Authentication\n\nAll API requests require Bearer token authentication.\n\n### Example Request\n\n```bash\ncurl -H 'Authorization: Bearer TOKEN' https://api.example.com/v1/users\n```",
            "API documentation"
        ),
        (
            "config.json",
            "{\n  \"name\": \"my-app\",\n  \"version\": \"1.0.0\",\n  \"database\": {\n    \"host\": \"localhost\",\n    \"port\": 5432\n  }\n}",
            "Application configuration"
        ),
        
        // Social media content
        (
            "social_post.txt",
            "Just launched our new product! 🚀 #startup #innovation\n\nCheck it out at https://example.com and RT if you like it!\n\n@techcrunch @producthunt",
            "Social media post"
        ),
        (
            "meme.txt",
            "When you fix a bug in production\n\n*happy dance*\n\nBut it creates 3 new bugs\n\n*sad developer noises*\n\nLOL this is so relatable! #programminghumor #meme",
            "Programming meme"
        ),
        
        // Academic documents
        (
            "research_paper.pdf",
            "Abstract\n\nThis paper presents a novel approach to machine learning optimization.\n\n1. Introduction\n\nRecent advances in ML have shown...\n\n2. Methodology\n\nWe employed a hybrid approach...\n\n3. Results\n\nOur experiments demonstrate a 40% improvement...",
            "Research paper"
        ),
        (
            "lesson_plan.docx",
            "LESSON PLAN: Introduction to Physics\n\nGrade: 10th\nDuration: 45 minutes\n\nObjectives:\n- Understand Newton's Laws\n- Apply concepts to real-world examples\n\nActivities:\n1. Introduction (10 min)\n2. Demonstration (20 min)",
            "Educational lesson plan"
        ),
    ]
}

fn main() {
    println!("=== Domain Partitioning Real-World Demo ===\n");
    
    // Create partition strategy
    let strategy = PartitionStrategy::default();
    
    // Process each document
    let documents = create_sample_documents();
    let mut domain_counts: HashMap<ContentDomain, usize> = HashMap::new();
    
    println!("Processing {documents.len(} documents...\n"));
    
    for (filename, content, description) in documents {
        // Determine domain
        let domain = strategy.determine_domain(
            Some(filename),
            None,
            Some(content),
            None,
        );
        
        // Get bucket
        let bucket = strategy.get_bucket_for_domain(domain);
        
        // Update counts
        *domain_counts.entry(domain).or_insert(0) += 1;
        
        // Display result
        println!("📄 {filename}");
        println!("   Description: {description}");
        println!("   Domain: {:?}", domain);
        println!("   Bucket: {bucket}");
        println!("   Preview: {content.lines(}").next().unwrap_or(""));
        println!();
    }
    
    // Summary
    println!("\n=== Summary ===\n");
    println!("Documents processed by domain:");
    let mut sorted_domains: Vec<_> = domain_counts.into_iter().collect();
    sorted_domains.sort_by_key(|(domain, _)| format!("{:?}", domain));
    
    for (domain, count) in sorted_domains {
        println!("  {:?}: {domain} documents", count);
    }
    
    // Demonstrate custom patterns
    println!("\n=== Custom Pattern Example ===\n");
    
    let mut custom_strategy = PartitionStrategy::default();
    
    // Add custom pattern for company-specific documents
    custom_strategy.add_pattern_matcher(cim_ipld::object_store::PatternMatcher {
        name: "acme_corp_detector".to_string(),
        keywords: vec![
            "acme corporation".to_string(),
            "acme corp".to_string(),
            "internal use only".to_string(),
        ],
        domain: ContentDomain::Private,
        priority: 200, // High priority
    });
    
    let internal_doc = "ACME CORPORATION - INTERNAL USE ONLY\n\nConfidential project details...";
    
    let domain = custom_strategy.determine_domain(
        Some("internal_project.pdf"),
        None,
        Some(internal_doc),
        None,
    );
    
    println!("Internal document detected as: {:?}", domain);
    println!("Routed to bucket: {custom_strategy.get_bucket_for_domain(domain}"));
} 