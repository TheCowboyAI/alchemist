//! Integration tests for domain partitioning with real document content

use cim_ipld::content_types::*;
use cim_ipld::object_store::{ContentDomain, NatsObjectStore, PartitionStrategy, PatternMatcher};
use cim_ipld::TypedContent;
use std::collections::HashMap;

/// Test helper to create document metadata
fn create_metadata(title: &str, author: &str, keywords: Vec<&str>) -> DocumentMetadata {
    DocumentMetadata {
        title: Some(title.to_string()),
        author: Some(author.to_string()),
        created_at: Some(1705276800), // January 15, 2024 timestamp
        modified_at: Some(1705276800),
        tags: keywords.iter().map(|s| s.to_string()).collect(),
        language: Some("en".to_string()),
    }
}

#[test]
fn test_legal_document_variations() {
    let strategy = PartitionStrategy::default();

    // Test 1: Service Agreement
    let service_agreement = "SERVICE AGREEMENT

This Service Agreement (this 'Agreement') is entered into as of January 15, 2024 (the 'Effective Date') 
between ABC Corporation, a Delaware corporation ('Service Provider'), and XYZ Inc., a California 
corporation ('Client').

WHEREAS, Service Provider desires to provide certain services to Client; and
WHEREAS, Client desires to obtain such services from Service Provider;

NOW, THEREFORE, in consideration of the mutual covenants and agreements hereinafter set forth and 
for other good and valuable consideration, the receipt and sufficiency of which are hereby acknowledged, 
the parties agree as follows:

1. SERVICES. Service Provider shall provide the services described in Exhibit A attached hereto.
2. TERM. This Agreement shall commence on the Effective Date and continue for one (1) year.
3. PAYMENT. Client shall pay Service Provider the fees set forth in Exhibit B.";

    let domain = strategy.determine_domain(
        Some("service_agreement_2024.pdf"),
        Some("application/pdf"),
        Some(service_agreement),
        None,
    );
    assert_eq!(domain, ContentDomain::Contracts);

    // Test 2: Non-Disclosure Agreement
    let nda = "MUTUAL NON-DISCLOSURE AGREEMENT

This Mutual Non-Disclosure Agreement ('Agreement') is entered into between the parties identified below.

The parties hereby agree to the following terms and conditions:

1. CONFIDENTIAL INFORMATION. Each party may disclose certain confidential and proprietary information.
2. OBLIGATIONS. Each party agrees to maintain the confidentiality of the other party's information.
3. EXCEPTIONS. The obligations set forth in Section 2 shall not apply to information that is publicly available.";

    let domain = strategy.determine_domain(
        Some("mutual_nda.docx"),
        Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
        Some(nda),
        None,
    );
    assert_eq!(domain, ContentDomain::Agreements);

    // Test 3: Terms of Service
    let tos = "TERMS OF SERVICE

Last updated: January 15, 2024

Please read these Terms of Service ('Terms', 'Terms of Service') carefully before using our service.

Your access to and use of the Service is conditioned on your acceptance of and compliance with these Terms.

By accessing or using the Service you agree to be bound by these Terms. If you disagree with any part of 
these terms then you may not access the Service.";

    let domain = strategy.determine_domain(
        Some("terms_of_service.html"),
        Some("text/html"),
        Some(tos),
        None,
    );
    assert_eq!(domain, ContentDomain::Agreements);

    // Test 4: Company Policy
    let policy = "REMOTE WORK POLICY

Effective Date: January 1, 2024

PURPOSE: This policy establishes guidelines for employees working remotely.

SCOPE: This policy applies to all full-time employees of the company.

POLICY GUIDELINES:
1. Employees must maintain regular working hours
2. Company equipment must be used securely
3. Regular communication with team members is required";

    let domain = strategy.determine_domain(
        Some("remote_work_policy.pdf"),
        Some("application/pdf"),
        Some(policy),
        None,
    );
    assert_eq!(domain, ContentDomain::Policies);
}

#[test]
fn test_financial_document_variations() {
    let strategy = PartitionStrategy::default();

    // Test 1: Detailed Invoice
    let invoice = "INVOICE

Invoice Number: INV-2024-00123
Invoice Date: January 15, 2024
Due Date: February 14, 2024

Bill To:
Customer Corporation
123 Business Street
New York, NY 10001

Description                     Quantity    Unit Price    Amount
Professional Services              40         $150.00    $6,000.00
Software License                    1       $2,000.00    $2,000.00
Support and Maintenance            12         $100.00    $1,200.00

                                           Subtotal:    $9,200.00
                                           Tax (8%):      $736.00
                                           Total Due:   $9,936.00

Payment Terms: Net 30
Please remit payment to: Bank of America, Account #123456789";

    let domain = strategy.determine_domain(
        Some("invoice_january_2024.pdf"),
        Some("application/pdf"),
        Some(invoice),
        None,
    );
    assert_eq!(domain, ContentDomain::Invoices);

    // Test 2: Receipt
    let receipt = "RECEIPT

Store: TechMart Electronics
Date: 01/15/2024
Transaction #: 78901234

Items Purchased:
- Laptop Computer               $1,299.99
- Wireless Mouse                  $49.99
- USB-C Cable                     $19.99

Subtotal:                      $1,369.97
Sales Tax:                       $109.60
Total:                         $1,479.57

Payment Method: Credit Card ****1234
Thank you for your purchase!";

    let domain = strategy.determine_domain(
        Some("purchase_receipt.txt"),
        Some("text/plain"),
        Some(receipt),
        None,
    );
    assert_eq!(domain, ContentDomain::Receipts);

    // Test 3: Bank Statement
    let statement = "MONTHLY STATEMENT

Account Number: ****5678
Statement Period: December 1, 2023 - December 31, 2023

Beginning Balance: $5,234.56

Deposits:
12/05  Direct Deposit - Salary      $3,500.00
12/20  Transfer from Savings          $500.00

Withdrawals:
12/07  Electric Company              -$125.43
12/10  Grocery Store                 -$234.56
12/15  Mortgage Payment            -$1,500.00

Ending Balance: $7,374.57";

    let domain = strategy.determine_domain(
        Some("bank_statement_dec_2023.pdf"),
        Some("application/pdf"),
        Some(statement),
        None,
    );
    assert_eq!(domain, ContentDomain::Statements);

    // Test 4: Financial Report (should be Financial, not specific subcategory)
    let report = "QUARTERLY FINANCIAL REPORT

Q4 2023 Financial Summary

Revenue: $2.5M (up 15% YoY)
Operating Expenses: $1.8M
Net Income: $700K

Key Metrics:
- Gross Margin: 28%
- Operating Margin: 12%
- Cash Flow: $500K positive";

    let domain = strategy.determine_domain(
        Some("q4_financial_report.pdf"),
        Some("application/pdf"),
        Some(report),
        None,
    );
    assert_eq!(domain, ContentDomain::Financial);
}

#[test]
fn test_medical_document_variations() {
    let strategy = PartitionStrategy::default();

    // Test 1: Patient Medical Record
    let medical_record = "PATIENT MEDICAL RECORD

Patient Name: John Doe
Date of Birth: 01/15/1980
Medical Record Number: MRN-123456

Visit Date: January 15, 2024
Chief Complaint: Annual physical examination

Vital Signs:
- Blood Pressure: 120/80
- Heart Rate: 72 bpm
- Temperature: 98.6°F

Diagnosis: Patient is in good health. No significant findings.

Treatment Plan: Continue current medications. Follow up in one year.

Physician: Dr. Jane Smith, MD";

    let domain = strategy.determine_domain(
        Some("patient_record_doe_john.pdf"),
        Some("application/pdf"),
        Some(medical_record),
        None,
    );
    assert_eq!(domain, ContentDomain::Medical);

    // Test 2: Prescription
    let prescription = "PRESCRIPTION

Patient: Jane Smith
Date: 01/15/2024

Rx: Amoxicillin 500mg
Sig: Take 1 capsule by mouth three times daily for 10 days
Quantity: #30
Refills: 0

Prescriber: Dr. Robert Johnson, MD
DEA: BJ1234567
License: 12345

Pharmacy Use Only:
Date Filled: ___________
RPh: _________________";

    let domain = strategy.determine_domain(
        Some("prescription_smith_jane.pdf"),
        Some("application/pdf"),
        Some(prescription),
        None,
    );
    assert_eq!(domain, ContentDomain::Prescriptions);

    // Test 3: Lab Results
    let lab_results = "LABORATORY RESULTS

Patient: Michael Brown
Collection Date: 01/14/2024
Report Date: 01/15/2024

Complete Blood Count (CBC):
- WBC: 7.2 K/uL (Normal: 4.5-11.0)
- RBC: 4.8 M/uL (Normal: 4.5-5.5)
- Hemoglobin: 14.5 g/dL (Normal: 13.5-17.5)
- Hematocrit: 42% (Normal: 40-50)

Basic Metabolic Panel:
- Glucose: 95 mg/dL (Normal: 70-100)
- Creatinine: 1.0 mg/dL (Normal: 0.7-1.3)

All results within normal limits.";

    let domain = strategy.determine_domain(
        Some("lab_results_brown_michael.pdf"),
        Some("application/pdf"),
        Some(lab_results),
        None,
    );
    assert_eq!(domain, ContentDomain::LabResults);

    // Test 4: Health Insurance Card (should be HealthRecords)
    let insurance = "HEALTH INSURANCE CARD

Member Name: Sarah Johnson
Member ID: HIX123456789
Group Number: GRP-5678
Plan: PPO Gold

Primary Care Physician: Dr. Emily Chen
Effective Date: 01/01/2024

Emergency: Call 911
Member Services: 1-800-123-4567";

    let domain = strategy.determine_domain(
        Some("insurance_card.pdf"),
        Some("application/pdf"),
        Some(insurance),
        None,
    );
    assert_eq!(domain, ContentDomain::HealthRecords);
}

#[test]
fn test_social_media_content() {
    let strategy = PartitionStrategy::default();

    // Test 1: Twitter/X-style post
    let tweet = "Just launched our new product! 🚀 #startup #innovation #tech @techcrunch 

Check it out at https://example.com and don't forget to retweet if you like it! 

What do you think? Drop a comment below 👇";

    let domain = strategy.determine_domain(Some("social_post.json"), None, Some(tweet), None);
    assert_eq!(domain, ContentDomain::SocialMedia);

    // Test 2: Meme content
    let meme_text = "When you fix a bug in production

*happy dance*

But it creates 3 new bugs

*sad developer noises*

LOL this is so relatable! Tag a developer friend who needs to see this funny meme 😂

#programminghumor #meme #viral #trending";

    let domain = strategy.determine_domain(Some("dev_meme.txt"), None, Some(meme_text), None);
    assert_eq!(domain, ContentDomain::Memes);

    // Test 3: Blog post (should be Posts)
    let blog_post = "10 Tips for Better Code Reviews

Published on January 15, 2024 by Tech Blog

Code reviews are an essential part of the development process. Here are our top tips:

1. Keep pull requests small
2. Write clear descriptions
3. Be constructive with feedback

Share this post if you found it helpful!";

    let domain = strategy.determine_domain(
        Some("blog_post.md"),
        Some("text/markdown"),
        Some(blog_post),
        None,
    );
    assert_eq!(domain, ContentDomain::Posts);

    // Test 4: Direct message (should be Messages)
    let message = "Hey Sarah,

Just wanted to follow up on our conversation yesterday. Can you send me the project files?

Thanks,
John

Sent from my iPhone";

    let domain = strategy.determine_domain(Some("message.txt"), None, Some(message), None);
    assert_eq!(domain, ContentDomain::Messages);
}

#[test]
fn test_technical_content() {
    let strategy = PartitionStrategy::default();

    // Test 1: Source code
    let rust_code = r#"use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("key", "value");
    
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_example() {
        assert_eq!(2 + 2, 4);
    }
}"#;

    let domain =
        strategy.determine_domain(Some("main.rs"), Some("text/x-rust"), Some(rust_code), None);
    assert_eq!(domain, ContentDomain::SourceCode);

    // Test 2: Configuration file
    let config = r#"{
    "name": "my-app",
    "version": "1.0.0",
    "dependencies": {
        "express": "^4.18.0",
        "mongoose": "^6.0.0"
    },
    "scripts": {
        "start": "node index.js",
        "test": "jest"
    }
}"#;

    let domain = strategy.determine_domain(
        Some("package.json"),
        Some("application/json"),
        Some(config),
        None,
    );
    assert_eq!(domain, ContentDomain::Configuration);

    // Test 3: Technical documentation
    let tech_doc = "# API Documentation

## Authentication

All API requests require authentication using Bearer tokens.

### Example Request

```bash
curl -H 'Authorization: Bearer YOUR_TOKEN' https://api.example.com/v1/users
```

### Response Format

All responses are returned in JSON format with the following structure:

```json
{
  \"status\": \"success\",
  \"data\": {}
}
```";

    let domain = strategy.determine_domain(
        Some("api_docs.md"),
        Some("text/markdown"),
        Some(tech_doc),
        None,
    );
    assert_eq!(domain, ContentDomain::Documentation);

    // Test 4: Database schema
    let schema = r#"CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);"#;

    let domain = strategy.determine_domain(
        Some("schema.sql"),
        Some("application/sql"),
        Some(schema),
        None,
    );
    assert_eq!(domain, ContentDomain::Schemas);
}

#[test]
fn test_academic_content() {
    let strategy = PartitionStrategy::default();

    // Test 1: Research paper
    let paper = "Abstract

This paper presents a novel approach to machine learning optimization using quantum computing principles. 
We demonstrate that our methodology achieves a 40% improvement in training time.

1. Introduction

Recent advances in quantum computing have opened new possibilities for machine learning applications.

2. Methodology

We employed a hybrid quantum-classical approach using the following experimental setup...

3. Results

Our experiments show significant improvements across all benchmark datasets.

4. Conclusion

This research demonstrates the potential of quantum-enhanced machine learning.

References

[1] Smith, J. et al. (2023). Quantum Computing Fundamentals. Nature.
[2] Johnson, M. (2023). Machine Learning Optimization. Science.";

    let domain = strategy.determine_domain(
        Some("quantum_ml_paper.pdf"),
        Some("application/pdf"),
        Some(paper),
        None,
    );
    assert_eq!(domain, ContentDomain::Papers);

    // Test 2: Research proposal
    let proposal = "RESEARCH PROPOSAL

Title: Investigating the Effects of Climate Change on Marine Ecosystems

Principal Investigator: Dr. Sarah Martinez

Hypothesis: Rising ocean temperatures significantly impact coral reef biodiversity.

Research Objectives:
1. Measure temperature variations in key reef locations
2. Document species diversity changes over time
3. Develop predictive models for ecosystem changes

Expected Outcomes:
This research will provide critical data for conservation efforts.";

    let domain =
        strategy.determine_domain(Some("research_proposal.docx"), None, Some(proposal), None);
    assert_eq!(domain, ContentDomain::Research);

    // Test 3: Educational material
    let educational = "LESSON PLAN: Introduction to Photosynthesis

Grade Level: 7th Grade
Duration: 45 minutes

Learning Objectives:
- Students will understand the basic process of photosynthesis
- Students will identify the inputs and outputs of photosynthesis

Materials Needed:
- Whiteboard and markers
- Plant samples
- Microscopes

Activities:
1. Introduction (10 min): Discuss why plants are green
2. Main Lesson (20 min): Explain photosynthesis equation
3. Lab Activity (10 min): Observe plant cells under microscope
4. Wrap-up (5 min): Review key concepts

Assessment: Quiz on Friday";

    let domain = strategy.determine_domain(
        Some("photosynthesis_lesson.pdf"),
        None,
        Some(educational),
        None,
    );
    assert_eq!(domain, ContentDomain::Educational);

    // Test 4: Case study
    let case_study = "CASE STUDY: Digital Transformation at Global Corp

Executive Summary

This study examines how Global Corp successfully transformed its operations through digital initiatives.

Background:
Global Corp faced declining market share and needed to modernize its systems.

Methodology:
We conducted interviews with 50 employees and analyzed 3 years of performance data.

Findings:
- 35% increase in operational efficiency
- 50% reduction in processing time
- $2M annual cost savings

Conclusions:
Digital transformation requires strong leadership and employee buy-in.";

    let domain = strategy.determine_domain(
        Some("digital_transformation_case.pdf"),
        None,
        Some(case_study),
        None,
    );
    assert_eq!(domain, ContentDomain::Studies);
}

#[test]
fn test_government_documents() {
    let strategy = PartitionStrategy::default();

    // Test 1: Government notice
    let notice = "OFFICIAL GOVERNMENT NOTICE

Department of Transportation
Notice ID: DOT-2024-001

PUBLIC NOTICE OF ROAD CONSTRUCTION

The Department of Transportation hereby notifies all residents that Main Street will be under 
construction from February 1, 2024 to March 15, 2024.

Affected Areas:
- Main Street between 1st Avenue and 5th Avenue
- All cross streets in the construction zone

For more information, contact the DOT at 1-800-ROADWORK";

    let domain = strategy.determine_domain(Some("public_notice.pdf"), None, Some(notice), None);
    assert_eq!(domain, ContentDomain::Government);

    // Test 2: Business license
    let license = "BUSINESS LICENSE

License Number: BL-2024-12345
Issue Date: January 15, 2024
Expiration Date: January 14, 2025

Business Name: ABC Coffee Shop
Business Address: 123 Main Street, Anytown, ST 12345
Business Type: Food Service Establishment

This license permits the above-named business to operate in accordance with city ordinances.

Issued by: City Clerk's Office
Authorized Signature: _________________";

    let domain = strategy.determine_domain(Some("business_license.pdf"), None, Some(license), None);
    assert_eq!(domain, ContentDomain::Licenses);

    // Test 3: Building permit
    let permit = "BUILDING PERMIT

Permit Number: BP-2024-00789
Property Address: 456 Oak Street

Permit Type: Residential Addition
Work Description: Add 200 sq ft bedroom and bathroom

Contractor: XYZ Construction LLC
License #: C-123456

Approved Plans: See attached architectural drawings
Inspection Required: Yes

This permit is valid for 180 days from issue date.";

    let domain = strategy.determine_domain(Some("building_permit.pdf"), None, Some(permit), None);
    assert_eq!(domain, ContentDomain::Permits);

    // Test 4: Public record
    let public_record = "PUBLIC RECORD REQUEST RESPONSE

Request ID: PRR-2024-567
Requester: Jane Citizen
Date: January 15, 2024

Documents Provided:
1. City Council Meeting Minutes - December 2023
2. Budget Report FY2023
3. Zoning Map - District 5

Total Pages: 127
Fee: $12.70 (copying costs)

These records are provided in accordance with the Freedom of Information Act.";

    let domain = strategy.determine_domain(
        Some("public_records_response.pdf"),
        None,
        Some(public_record),
        None,
    );
    assert_eq!(domain, ContentDomain::PublicRecords);
}

#[test]
fn test_edge_cases_and_ambiguous_content() {
    let strategy = PartitionStrategy::default();

    // Test 1: Document with multiple domain keywords
    let mixed_content = "MEDICAL INVOICE

Patient: John Doe
Date of Service: January 10, 2024

Services Rendered:
- Annual Physical Examination: $200.00
- Blood Test Panel: $150.00
- EKG: $100.00

Diagnosis: Routine checkup, patient in good health

Total Due: $450.00
Payment Due Date: February 10, 2024

Please remit payment to: Medical Associates LLC";

    // Should classify as Medical due to medical keywords despite invoice format
    let domain =
        strategy.determine_domain(Some("medical_invoice.pdf"), None, Some(mixed_content), None);
    assert_eq!(domain, ContentDomain::Medical);

    // Test 2: Generic document with no clear patterns
    let generic = "Meeting Notes

Date: January 15, 2024
Attendees: Team Members

Topics Discussed:
- Project timeline
- Budget review
- Next steps

Action Items:
- Follow up by Friday
- Send updated report";

    // Should default to Documents
    let domain = strategy.determine_domain(Some("meeting_notes.txt"), None, Some(generic), None);
    assert_eq!(domain, ContentDomain::Documents);

    // Test 3: Very short content
    let short_content = "Thanks for your help!";

    // Should use file extension
    let domain = strategy.determine_domain(Some("thank_you.txt"), None, Some(short_content), None);
    assert_eq!(domain, ContentDomain::Documents);

    // Test 4: Empty content
    let domain =
        strategy.determine_domain(Some("empty.pdf"), Some("application/pdf"), Some(""), None);
    assert_eq!(domain, ContentDomain::Documents);
}

#[test]
fn test_custom_patterns_override() {
    let mut strategy = PartitionStrategy::default();

    // Add custom pattern for company-specific documents
    strategy.add_pattern_matcher(PatternMatcher {
        name: "company_contract".to_string(),
        keywords: vec![
            "acme corp".to_string(),
            "proprietary".to_string(),
            "confidential agreement".to_string(),
        ],
        domain: ContentDomain::Contracts,
        priority: 150, // Higher than default contract priority
    });

    let company_doc = "ACME CORP CONFIDENTIAL AGREEMENT

This document contains proprietary information of ACME Corp.

Project Details:
- Code Name: Phoenix
- Budget: $500,000
- Timeline: 6 months

All information herein is strictly confidential.";

    let domain = strategy.determine_domain(Some("acme_project.pdf"), None, Some(company_doc), None);
    assert_eq!(domain, ContentDomain::Contracts);
}

#[tokio::test]
async fn test_domain_partitioning_with_real_storage() {
    // This test requires NATS to be running
    // Skip if NATS is not available
    let client = match async_nats::connect("nats://localhost:4222").await {
        Ok(client) => client,
        Err(_) => {
            println!("Skipping test - NATS not available");
            return;
        }
    };

    let jetstream = async_nats::jetstream::new(client);
    let store = NatsObjectStore::new(jetstream.clone(), 1024).await.unwrap();

    // Test storing different types of content
    let test_cases = vec![
        (
            "invoice_test.pdf",
            "Invoice Number: TEST-001\nBill To: Test Customer\nTotal Due: $1,000",
            ContentDomain::Invoices,
        ),
        (
            "contract_test.pdf",
            "This contract is entered into between Party A and Party B",
            ContentDomain::Contracts,
        ),
        (
            "medical_test.pdf",
            "Patient Name: Test Patient\nDiagnosis: Test condition",
            ContentDomain::Medical,
        ),
        (
            "code_test.rs",
            "fn main() { println!(\"Hello, world!\"); }",
            ContentDomain::SourceCode,
        ),
    ];

    for (filename, content, expected_domain) in test_cases {
        let pdf = PdfDocument {
            data: content.as_bytes().to_vec(),
            metadata: create_metadata("Test Document", "Test Author", vec!["test"]),
        };

        let (cid, domain) = store
            .put_with_domain(
                &pdf,
                Some(filename),
                Some("application/pdf"),
                Some(content),
                None,
            )
            .await
            .unwrap();

        assert_eq!(domain, expected_domain);

        // Verify we can retrieve from the correct domain
        let retrieved: PdfDocument = store.get_from_domain(&cid, domain).await.unwrap();
        assert_eq!(retrieved.data, pdf.data);
    }

    // Test listing by domain
    let invoices = store.list_domain(ContentDomain::Invoices).await.unwrap();
    assert!(invoices.len() >= 1);

    let contracts = store.list_domain(ContentDomain::Contracts).await.unwrap();
    assert!(contracts.len() >= 1);
}
