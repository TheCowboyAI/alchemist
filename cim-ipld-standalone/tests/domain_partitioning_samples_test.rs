//! Tests using generated sample documents for domain partitioning

use cim_ipld::object_store::{ContentDomain, PartitionStrategy};
use cim_ipld::content_types::*;
use std::collections::HashMap;

/// Generate a sample invoice document
fn generate_invoice(invoice_num: &str, customer: &str, amount: f64) -> String {
    format!(
        r#"INVOICE

Company Name: Tech Solutions Inc.
123 Business Ave, Suite 100
San Francisco, CA 94105
Phone: (555) 123-4567
Email: billing@techsolutions.com

Invoice Number: {}
Invoice Date: January 15, 2024
Due Date: February 14, 2024

Bill To:
{}
456 Customer Street
New York, NY 10001

Description                          Quantity    Rate        Amount
------------------------------------------------------------------------
Software Development Services            160      $150.00    $24,000.00
Cloud Infrastructure Setup                1     $5,000.00     $5,000.00
Monthly Support and Maintenance          12       $500.00     $6,000.00
API Integration Services                 40       $175.00     $7,000.00

                                                  Subtotal:   $42,000.00
                                                  Tax (8.5%):  $3,570.00
                                                  Total Due:  ${:.2}

Payment Terms: Net 30
Late fees of 1.5% per month will be applied to overdue accounts.

Please make checks payable to: Tech Solutions Inc.
For wire transfers, please contact accounting@techsolutions.com"#,
        invoice_num, customer, amount
    )
}

/// Generate a medical record
fn generate_medical_record(patient: &str, condition: &str) -> String {
    format!(
        r#"MEDICAL RECORD

Healthcare Provider: City General Hospital
Department: Internal Medicine
Provider: Dr. Sarah Johnson, MD

Patient Information:
Name: {}
Date of Birth: March 15, 1985
Medical Record Number: MRN-789012
Insurance: Blue Cross Blue Shield

Visit Date: January 15, 2024
Visit Type: Follow-up Appointment

Chief Complaint:
Patient presents for follow-up regarding {}

History of Present Illness:
The patient reports improvement in symptoms since last visit. Currently taking prescribed medications
as directed. No adverse reactions noted.

Vital Signs:
- Blood Pressure: 118/76 mmHg
- Heart Rate: 72 bpm
- Respiratory Rate: 16/min
- Temperature: 98.4°F
- O2 Saturation: 98% on room air
- Weight: 165 lbs
- Height: 5'10"

Physical Examination:
General: Alert and oriented x3, in no acute distress
HEENT: Normocephalic, atraumatic
Cardiovascular: Regular rate and rhythm, no murmurs
Respiratory: Clear to auscultation bilaterally
Abdomen: Soft, non-tender, non-distended

Assessment and Plan:
{} - stable on current treatment regimen. Continue current medications.
Follow up in 3 months or sooner if symptoms worsen.

Laboratory Orders:
- Complete Blood Count
- Comprehensive Metabolic Panel
- Lipid Panel

Medications:
Continue current regimen as prescribed.

Next Appointment: April 15, 2024

Electronically signed by: Dr. Sarah Johnson, MD
Date: January 15, 2024"#,
        patient, condition, condition
    )
}

/// Generate a legal contract
fn generate_contract(party_a: &str, party_b: &str, service: &str) -> String {
    format!(
        r#"SERVICE AGREEMENT

This Service Agreement ("Agreement") is entered into as of January 15, 2024 ("Effective Date") by and between:

{} ("Service Provider"), a corporation organized and existing under the laws of Delaware, 
with its principal place of business at 789 Corporate Blvd, Wilmington, DE 19801

AND

{} ("Client"), a corporation organized and existing under the laws of California,
with its principal place of business at 321 Tech Park, San Jose, CA 95110

RECITALS

WHEREAS, Service Provider has expertise in providing {};

WHEREAS, Client desires to engage Service Provider to provide such services;

NOW, THEREFORE, in consideration of the mutual covenants and agreements hereinafter set forth and for other 
good and valuable consideration, the receipt and sufficiency of which are hereby acknowledged, the parties 
agree as follows:

1. SERVICES
   1.1 Scope of Services. Service Provider agrees to provide {} services as more particularly 
       described in Exhibit A attached hereto and incorporated herein by reference ("Services").
   
   1.2 Performance Standards. Service Provider shall perform the Services in a professional and 
       workmanlike manner in accordance with generally recognized industry standards.

2. TERM
   2.1 Initial Term. This Agreement shall commence on the Effective Date and shall continue for a 
       period of twelve (12) months ("Initial Term").
   
   2.2 Renewal. This Agreement shall automatically renew for successive one-year terms unless either 
       party provides written notice of non-renewal at least sixty (60) days prior to the end of the 
       then-current term.

3. COMPENSATION
   3.1 Fees. In consideration for the Services, Client shall pay Service Provider the fees set forth 
       in Exhibit B attached hereto.
   
   3.2 Payment Terms. Invoices shall be due and payable within thirty (30) days of receipt.

4. CONFIDENTIALITY
   Each party acknowledges that it may have access to confidential information of the other party. 
   Each party agrees to maintain the confidentiality of such information.

5. TERMINATION
   Either party may terminate this Agreement upon thirty (30) days written notice to the other party.

6. GOVERNING LAW
   This Agreement shall be governed by the laws of the State of Delaware.

IN WITNESS WHEREOF, the parties have executed this Agreement as of the date first above written.

{}                              {}
By: _____________________      By: _____________________
Name: John Smith               Name: Jane Doe
Title: CEO                     Title: VP of Operations
Date: ___________________      Date: ___________________"#,
        party_a, party_b, service, service, party_a, party_b
    )
}

/// Generate a research paper
fn generate_research_paper(title: &str, author: &str, field: &str) -> String {
    format!(
        r#"{}

{}, Ph.D.
Department of {}
University of Technology

Abstract

This paper presents a comprehensive analysis of recent developments in {}. Through extensive 
experimentation and data analysis, we demonstrate significant improvements in performance metrics 
compared to existing approaches. Our methodology combines theoretical frameworks with practical 
implementations, yielding results that advance the current state of knowledge in this field.

Keywords: {}, research, methodology, experimental results, data analysis

1. Introduction

The field of {} has seen remarkable progress in recent years. However, several challenges remain 
unaddressed. This research aims to bridge these gaps by proposing a novel approach that combines 
established principles with innovative techniques.

Recent studies by Smith et al. (2023) and Johnson (2023) have laid the groundwork for our 
investigation. Building upon their findings, we extend the theoretical framework to encompass 
previously unexplored dimensions of the problem space.

2. Literature Review

2.1 Historical Context
The evolution of {} can be traced back to foundational work in the early 2000s. Key milestones 
include the development of fundamental theorems and the establishment of standard methodologies.

2.2 Current State of Research
Contemporary research focuses on optimization and scalability. Notable contributions include the 
work of Chen et al. (2023) on algorithmic efficiency and Martinez (2023) on practical applications.

3. Methodology

3.1 Experimental Design
We employed a mixed-methods approach combining quantitative analysis with qualitative assessments. 
Our experimental setup consisted of:
- Controlled laboratory conditions
- Standardized testing protocols
- Multiple trial runs for statistical significance

3.2 Data Collection
Data was collected over a six-month period using automated systems and manual verification processes. 
We ensured data integrity through cross-validation and redundancy checks.

4. Results

Our experiments yielded the following key findings:
- 45% improvement in efficiency metrics
- 30% reduction in error rates
- Statistically significant correlation between variables (p < 0.001)

Table 1: Comparative Performance Metrics
[Detailed table data would appear here]

5. Discussion

The results strongly support our hypothesis that the proposed methodology offers substantial 
improvements over existing approaches. The implications extend beyond immediate applications to 
suggest new avenues for future research.

6. Conclusion

This research contributes to the growing body of knowledge in {} by demonstrating the viability 
of our proposed approach. Future work will focus on scaling these findings to broader contexts 
and exploring additional optimization opportunities.

References

[1] Smith, J., Brown, M., & Davis, L. (2023). Foundational Principles in {}. Journal of Advanced Research, 45(3), 123-145.
[2] Johnson, K. (2023). Modern Approaches to {} Optimization. Science Quarterly, 78(2), 234-256.
[3] Chen, W., Liu, H., & Wang, X. (2023). Algorithmic Advances in {}. Technical Computing Review, 12(4), 345-367.
[4] Martinez, R. (2023). Practical Applications of {} Theory. Applied Sciences Journal, 89(1), 456-478.

Acknowledgments

The authors thank the research team at the University of Technology for their valuable contributions 
and insights throughout this project."#,
        title, author, field, field, field, field, field, field, field
    )
}

/// Generate a social media post
fn generate_social_media_post(topic: &str, hashtags: Vec<&str>) -> String {
    let hashtag_str = hashtags.iter().map(|h| format!("#{h}")).collect::<Vec<_>>().join(" ");
    format!(
        r#"🚀 Exciting news about {}! 

Just discovered this amazing insight that I had to share with you all. It's incredible how much 
things have evolved in this space.

Here are my top 3 takeaways:
1️⃣ Innovation is accelerating faster than ever
2️⃣ Community engagement is key to success
3️⃣ The future looks incredibly bright

What are your thoughts on this? Drop a comment below and let's discuss! 👇

Don't forget to like and share if you found this valuable. Your support means the world! ❤️

Follow @techinsights for more updates

{} #innovation #technology #community #trending #viral"#,
        topic, hashtag_str
    )
}

/// Generate a configuration file
fn generate_config_file(app_name: &str) -> String {
    format!(
        r#"{{
  "application": {{
    "name": "{}",
    "version": "2.1.0",
    "environment": "production",
    "debug": false
  }},
  "server": {{
    "host": "0.0.0.0",
    "port": 8080,
    "timeout": 30,
    "max_connections": 1000,
    "ssl": {{
      "enabled": true,
      "cert_path": "/etc/ssl/certs/server.crt",
      "key_path": "/etc/ssl/private/server.key"
    }}
  }},
  "database": {{
    "type": "postgresql",
    "host": "db.example.com",
    "port": 5432,
    "name": "{}_db",
    "user": "app_user",
    "password": "${{DB_PASSWORD}}",
    "pool": {{
      "min": 5,
      "max": 20,
      "idle_timeout": 300
    }}
  }},
  "logging": {{
    "level": "info",
    "format": "json",
    "output": "stdout",
    "file": {{
      "enabled": true,
      "path": "/var/log/{}/app.log",
      "max_size": "100MB",
      "max_age": 7,
      "compress": true
    }}
  }},
  "cache": {{
    "type": "redis",
    "host": "cache.example.com",
    "port": 6379,
    "ttl": 3600,
    "prefix": "{}_cache"
  }},
  "features": {{
    "authentication": true,
    "rate_limiting": true,
    "api_versioning": true,
    "metrics": true
  }}
}}"#,
        app_name, app_name, app_name, app_name
    )
}

#[test]
fn test_generated_invoices() {
    let strategy = PartitionStrategy::default();
    
    let invoices = vec![
        ("INV-2024-001", "ABC Corporation", 45570.00),
        ("INV-2024-002", "XYZ Industries", 12350.75),
        ("INV-2024-003", "Global Tech Ltd", 89999.99),
    ];

    for (num, customer, amount) in invoices {
        let invoice_content = generate_invoice(num, customer, amount);
        
        let domain = strategy.determine_domain(
            Some(&format!("invoice_{num}.pdf")),
            Some("application/pdf"),
            Some(&invoice_content),
            None,
        );
        
        assert_eq!(domain, ContentDomain::Invoices, 
            "Failed to classify invoice {} for {}", num, customer);
    }
}

#[test]
fn test_generated_medical_records() {
    let strategy = PartitionStrategy::default();
    
    let patients = vec![
        ("John Smith", "Type 2 Diabetes"),
        ("Mary Johnson", "Hypertension"),
        ("Robert Brown", "Asthma"),
        ("Patricia Davis", "Chronic Back Pain"),
    ];

    for (patient, condition) in patients {
        let medical_content = generate_medical_record(patient, condition);
        
        let domain = strategy.determine_domain(
            Some(&format!("medical_record_{}.pdf", patient.replace(" ", "_")).to_lowercase()),
            Some("application/pdf"),
            Some(&medical_content),
            None,
        );
        
        assert_eq!(domain, ContentDomain::Medical,
            "Failed to classify medical record for patient {}", patient);
    }
}

#[test]
fn test_generated_contracts() {
    let strategy = PartitionStrategy::default();
    
    let contracts = vec![
        ("TechCorp Solutions", "StartupXYZ Inc", "software development"),
        ("Marketing Masters LLC", "Retail Giant Corp", "digital marketing"),
        ("Cloud Services Pro", "Enterprise Co", "cloud infrastructure management"),
    ];

    for (provider, client, service) in contracts {
        let contract_content = generate_contract(provider, client, service);
        
        let domain = strategy.determine_domain(
            Some(&format!("service_agreement_{provider.replace(" ", "_"}_{}.pdf").to_lowercase(),
                client.replace(" ", "_").to_lowercase())),
            Some("application/pdf"),
            Some(&contract_content),
            None,
        );
        
        assert_eq!(domain, ContentDomain::Contracts,
            "Failed to classify contract between {} and {}", provider, client);
    }
}

#[test]
fn test_generated_research_papers() {
    let strategy = PartitionStrategy::default();
    
    let papers = vec![
        ("Machine Learning Applications in Healthcare", "Dr. Emily Chen", "Computer Science"),
        ("Quantum Computing: A New Paradigm", "Dr. Michael Roberts", "Physics"),
        ("Sustainable Energy Solutions for Urban Areas", "Dr. Sarah Green", "Environmental Engineering"),
    ];

    for (title, author, field) in papers {
        let paper_content = generate_research_paper(title, author, field);
        
        let domain = strategy.determine_domain(
            Some(&format!("{title.replace(" ", "_"}.pdf").replace(":", "").to_lowercase())),
            Some("application/pdf"),
            Some(&paper_content),
            None,
        );
        
        assert_eq!(domain, ContentDomain::Papers,
            "Failed to classify research paper: {}", title);
    }
}

#[test]
fn test_generated_social_media() {
    let strategy = PartitionStrategy::default();
    
    let posts = vec![
        ("AI breakthroughs", vec!["AI", "MachineLearning", "FutureTech"]),
        ("startup journey", vec!["StartupLife", "Entrepreneur", "BusinessGrowth"]),
        ("coding tips", vec!["Programming", "CodingLife", "TechTips"]),
    ];

    for (topic, hashtags) in posts {
        let post_content = generate_social_media_post(topic, hashtags.clone());
        
        let domain = strategy.determine_domain(
            Some(&format!("post_{topic.replace(" ", "_"}.json"))),
            Some("application/json"),
            Some(&post_content),
            None,
        );
        
        assert_eq!(domain, ContentDomain::SocialMedia,
            "Failed to classify social media post about {}", topic);
    }
}

#[test]
fn test_generated_config_files() {
    let strategy = PartitionStrategy::default();
    
    let apps = vec![
        "web-server",
        "api-gateway",
        "microservice-auth",
        "data-processor",
    ];

    for app in apps {
        let config_content = generate_config_file(app);
        
        let domain = strategy.determine_domain(
            Some(&format!("{app}.config.json")),
            Some("application/json"),
            Some(&config_content),
            None,
        );
        
        assert_eq!(domain, ContentDomain::Configuration,
            "Failed to classify configuration file for {}", app);
    }
}

#[test]
fn test_mixed_content_classification() {
    let strategy = PartitionStrategy::default();
    
    // Test a document that contains both medical and invoice elements
    let medical_invoice = r#"MEDICAL BILLING STATEMENT

Patient: Jane Doe
Date of Service: January 10, 2024
Provider: Dr. Smith

Services Rendered:
- Office Visit (99213): $150.00
- Blood Test (80053): $75.00
- EKG (93000): $125.00

Diagnosis: Annual checkup, patient in good health

Invoice Number: MED-2024-567
Total Due: $350.00
Payment Due: February 10, 2024"#;

    // Should classify as Medical because medical keywords have higher priority
    let domain = strategy.determine_domain(
        Some("medical_bill.pdf"),
        Some("application/pdf"),
        Some(medical_invoice),
        None,
    );
    assert_eq!(domain, ContentDomain::Medical);
}

#[test]
fn test_file_extension_fallback() {
    let strategy = PartitionStrategy::default();
    
    // Test with minimal content that doesn't match patterns
    let minimal_content = "Data file version 1.0";
    
    // Should use file extension to determine domain
    let test_cases = vec![
        ("data.csv", ContentDomain::Spreadsheets),
        ("script.py", ContentDomain::SourceCode),
        ("image.jpg", ContentDomain::Images),
        ("audio.mp3", ContentDomain::Music),
        ("doc.pdf", ContentDomain::Documents),
    ];

    for (filename, expected_domain) in test_cases {
        let domain = strategy.determine_domain(
            Some(filename),
            None,
            Some(minimal_content),
            None,
        );
        assert_eq!(domain, expected_domain,
            "Failed to classify {} by extension", filename);
    }
}

#[test]
fn test_metadata_override() {
    let strategy = PartitionStrategy::default();
    
    // Content that would normally be classified as a document
    let generic_content = "This is a generic document with no specific patterns.";
    
    // Use metadata to force classification
    let mut metadata = HashMap::new();
    metadata.insert("content_domain".to_string(), "\"Medical\"".to_string());
    
    let domain = strategy.determine_domain(
        Some("generic.txt"),
        Some("text/plain"),
        Some(generic_content),
        Some(&metadata),
    );
    
    assert_eq!(domain, ContentDomain::Medical,
        "Metadata override should force Medical classification");
}

#[test]
fn test_batch_document_classification() {
    let strategy = PartitionStrategy::default();
    
    // Generate a batch of mixed documents
    let documents = vec![
        (generate_invoice("BATCH-001", "Test Corp", 1000.0), "invoice1.pdf", ContentDomain::Invoices),
        (generate_medical_record("Test Patient", "Checkup"), "medical1.pdf", ContentDomain::Medical),
        (generate_contract("Company A", "Company B", "services"), "contract1.pdf", ContentDomain::Contracts),
        (generate_research_paper("Test Study", "Dr. Test", "Science"), "paper1.pdf", ContentDomain::Papers),
        (generate_social_media_post("test topic", vec!["test"]), "post1.json", ContentDomain::SocialMedia),
        (generate_config_file("test-app"), "config1.json", ContentDomain::Configuration),
    ];

    let mut correct_classifications = 0;
    let total = documents.len();

    for (content, filename, expected_domain) in documents {
        let domain = strategy.determine_domain(
            Some(filename),
            Some("application/pdf"),
            Some(&content),
            None,
        );
        
        if domain == expected_domain {
            correct_classifications += 1;
        } else {
            println!("Misclassified {filename}: expected {:?}, got {:?}", expected_domain, domain);
        }
    }

    assert_eq!(correct_classifications, total,
        "All documents should be correctly classified");
} 