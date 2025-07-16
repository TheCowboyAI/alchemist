//! Tests for domain-based content partitioning

use cim_ipld::object_store::{ContentDomain, PartitionStrategy, PatternMatcher};
use std::collections::HashMap;

#[test]
fn test_music_domain_detection() {
    let strategy = PartitionStrategy::default();

    // Test by extension
    let domain = strategy.determine_domain(Some("song.mp3"), None, None, None);
    assert_eq!(domain, ContentDomain::Music);

    // Test by MIME type
    let domain = strategy.determine_domain(None, Some("audio/mpeg"), None, None);
    assert_eq!(domain, ContentDomain::Music);

    // Test various music extensions
    for ext in &["wav", "flac", "ogg", "m4a", "aac"] {
        let filename = format!("audio.{ext}");
        let domain = strategy.determine_domain(Some(&filename), None, None, None);
        assert_eq!(
            domain,
            ContentDomain::Music,
            "Failed for extension: {}",
            ext
        );
    }
}

#[test]
fn test_document_domain_detection() {
    let strategy = PartitionStrategy::default();

    // Test office documents
    assert_eq!(
        strategy.determine_domain(Some("report.docx"), None, None, None),
        ContentDomain::Documents
    );

    assert_eq!(
        strategy.determine_domain(Some("data.xlsx"), None, None, None),
        ContentDomain::Spreadsheets
    );

    assert_eq!(
        strategy.determine_domain(Some("slides.pptx"), None, None, None),
        ContentDomain::Presentations
    );
}

#[test]
fn test_contract_pattern_detection() {
    let strategy = PartitionStrategy::default();

    let domain = strategy.determine_domain(
        Some("document.pdf"),
        Some("application/pdf"),
        Some("This contract is entered into between Party A and Party B, whereby the parties hereby agree to the following terms and conditions"),
        None,
    );
    assert_eq!(domain, ContentDomain::Contracts);
}

#[test]
fn test_invoice_pattern_detection() {
    let strategy = PartitionStrategy::default();

    let domain = strategy.determine_domain(
        Some("inv_2024.pdf"),
        None,
        Some("Invoice Number: INV-2024-001\nBill To: Customer Name\nPayment Due: 30 days\nSubtotal: $1000\nTax: $100\nTotal Due: $1100"),
        None,
    );
    assert_eq!(domain, ContentDomain::Invoices);
}

#[test]
fn test_medical_pattern_detection() {
    let strategy = PartitionStrategy::default();

    let domain = strategy.determine_domain(
        Some("record.pdf"),
        None,
        Some("Patient Name: John Doe\nDiagnosis: Annual checkup\nTreatment: Routine examination\nLab Results: All normal"),
        None,
    );
    assert_eq!(domain, ContentDomain::Medical);
}

#[test]
fn test_social_media_detection() {
    let strategy = PartitionStrategy::default();

    let domain = strategy.determine_domain(
        Some("post.json"),
        None,
        Some("Check out this amazing #sunset! @friend you should see this! Please like and share if you enjoyed it"),
        None,
    );
    assert_eq!(domain, ContentDomain::SocialMedia);
}

#[test]
fn test_meme_detection() {
    let strategy = PartitionStrategy::default();

    let domain = strategy.determine_domain(
        Some("funny.jpg"),
        Some("image/jpeg"),
        Some("LOL this meme is so funny and viral!"),
        None,
    );
    assert_eq!(domain, ContentDomain::Memes);
}

#[test]
fn test_source_code_detection() {
    let strategy = PartitionStrategy::default();

    // Test various programming language extensions
    for (ext, expected) in &[
        ("rs", ContentDomain::SourceCode),
        ("py", ContentDomain::SourceCode),
        ("js", ContentDomain::SourceCode),
        ("go", ContentDomain::SourceCode),
        ("java", ContentDomain::SourceCode),
    ] {
        let filename = format!("main.{ext}");
        let domain = strategy.determine_domain(Some(&filename), None, None, None);
        assert_eq!(domain, *expected, "Failed for extension: {}", ext);
    }
}

#[test]
fn test_configuration_detection() {
    let strategy = PartitionStrategy::default();

    // Test configuration file extensions
    for ext in &["json", "yaml", "toml", "ini", "conf", "xml"] {
        let filename = format!("config.{ext}");
        let domain = strategy.determine_domain(Some(&filename), None, None, None);
        assert_eq!(
            domain,
            ContentDomain::Configuration,
            "Failed for extension: {}",
            ext
        );
    }
}

#[test]
fn test_metadata_hint_priority() {
    let strategy = PartitionStrategy::default();

    let mut metadata = HashMap::new();
    metadata.insert("content_domain".to_string(), "\"Contracts\"".to_string());

    // Metadata should override file extension
    let domain = strategy.determine_domain(
        Some("file.txt"), // Would normally be Documents
        None,
        None,
        Some(&metadata),
    );
    assert_eq!(domain, ContentDomain::Contracts);
}

#[test]
fn test_pattern_priority() {
    let strategy = PartitionStrategy::default();

    // Content with both invoice and contract keywords
    // Invoice has more matches (3) than contract (1), so it wins despite lower priority
    let content =
        "This contract for services includes Invoice Number: INV-001 with payment due in 30 days";

    let domain = strategy.determine_domain(Some("document.pdf"), None, Some(content), None);

    // Invoice has more keyword matches, so it wins
    assert_eq!(domain, ContentDomain::Invoices);

    // Test priority when match counts are equal
    let equal_match_content = "This is a contract with an invoice";
    let domain2 =
        strategy.determine_domain(Some("document.pdf"), None, Some(equal_match_content), None);

    // Both have 1 match, contract has higher priority
    assert_eq!(domain2, ContentDomain::Contracts);
}

#[test]
fn test_bucket_name_mapping() {
    let strategy = PartitionStrategy::default();

    assert_eq!(
        strategy.get_bucket_for_domain(ContentDomain::Music),
        "cim-media-music"
    );
    assert_eq!(
        strategy.get_bucket_for_domain(ContentDomain::Contracts),
        "cim-legal-contracts"
    );
    assert_eq!(
        strategy.get_bucket_for_domain(ContentDomain::Invoices),
        "cim-finance-invoices"
    );
    assert_eq!(
        strategy.get_bucket_for_domain(ContentDomain::Memes),
        "cim-social-memes"
    );
    assert_eq!(
        strategy.get_bucket_for_domain(ContentDomain::Medical),
        "cim-health-medical"
    );
    assert_eq!(
        strategy.get_bucket_for_domain(ContentDomain::SourceCode),
        "cim-tech-code"
    );
}

#[test]
fn test_custom_pattern_matcher() {
    let mut strategy = PartitionStrategy::default();

    // Add custom pattern for detecting research papers
    strategy.add_pattern_matcher(PatternMatcher {
        name: "custom_research".to_string(),
        keywords: vec![
            "hypothesis".to_string(),
            "experiment".to_string(),
            "data analysis".to_string(),
        ],
        domain: ContentDomain::Research,
        priority: 200, // High priority
    });

    let domain = strategy.determine_domain(
        Some("study.pdf"),
        None,
        Some("Our hypothesis is that this experiment will show significant results through data analysis"),
        None,
    );

    assert_eq!(domain, ContentDomain::Research);
}

#[test]
fn test_custom_extension_mapping() {
    let mut strategy = PartitionStrategy::default();

    // Add custom extension
    strategy.add_extension_mapping("recipe".to_string(), ContentDomain::Personal);

    let domain = strategy.determine_domain(Some("chocolate_cake.recipe"), None, None, None);

    assert_eq!(domain, ContentDomain::Personal);
}

#[test]
fn test_custom_mime_mapping() {
    let mut strategy = PartitionStrategy::default();

    // Add custom MIME type
    strategy.add_mime_mapping("application/x-recipe".to_string(), ContentDomain::Personal);

    let domain = strategy.determine_domain(None, Some("application/x-recipe"), None, None);

    assert_eq!(domain, ContentDomain::Personal);
}

#[test]
fn test_default_fallback() {
    let strategy = PartitionStrategy::default();

    // Unknown file type with no patterns
    let domain = strategy.determine_domain(
        Some("unknown.xyz"),
        Some("application/unknown"),
        Some("Random content with no patterns"),
        None,
    );

    // Should default to Documents
    assert_eq!(domain, ContentDomain::Documents);
}
