# Domain-Based Content Partitioning

The CIM-IPLD system provides intelligent domain-based content partitioning that automatically routes content to appropriate storage buckets based on content type, patterns, and metadata.

## Overview

Domain partitioning enables:
- **Automatic content classification** based on file type, content patterns, and metadata
- **Organized storage** with domain-specific buckets (music, documents, contracts, etc.)
- **Efficient retrieval** by querying domain-specific buckets
- **Compliance support** for sensitive content (medical, financial, legal)
- **Extensible patterns** for custom domain detection

## Content Domains

The system supports the following content domains:

### Creative & Media
- **Music**: Audio files (MP3, WAV, FLAC, OGG, etc.)
- **Video**: Video files (MP4, MOV, MKV, WebM, etc.)
- **Images**: Image files (JPEG, PNG, GIF, WebP, etc.)
- **Graphics**: Vector/design files (SVG, AI, PSD, etc.)

### Documents & Office
- **Documents**: Text documents (DOC, DOCX, PDF, TXT, MD)
- **Spreadsheets**: Data files (XLS, XLSX, CSV, ODS)
- **Presentations**: Slide decks (PPT, PPTX, ODP)
- **Reports**: Business reports and analyses

### Legal & Business
- **Contracts**: Legal agreements and contracts
- **Agreements**: Service agreements, NDAs
- **Policies**: Company policies, terms of service
- **Compliance**: Regulatory compliance documents

### Social & Communication
- **SocialMedia**: Social media posts and content
- **Memes**: Viral images and memes
- **Messages**: Chat messages and communications
- **Posts**: Blog posts and articles

### Technical & Development
- **SourceCode**: Programming source files
- **Configuration**: Config files (JSON, YAML, TOML)
- **Documentation**: Technical documentation
- **Schemas**: Data schemas and definitions

### Personal & Private
- **Personal**: Personal documents and files
- **Private**: Private/confidential content
- **Encrypted**: Encrypted content
- **Sensitive**: Sensitive personal information

### Research & Academic
- **Research**: Research documents
- **Papers**: Academic papers and publications
- **Studies**: Research studies and analyses
- **Educational**: Educational materials

### Financial & Accounting
- **Financial**: Financial documents
- **Invoices**: Bills and invoices
- **Receipts**: Purchase receipts
- **Statements**: Bank/financial statements

### Medical & Health
- **Medical**: Medical documents
- **HealthRecords**: Patient health records
- **Prescriptions**: Medical prescriptions
- **LabResults**: Laboratory test results

### Government & Public
- **Government**: Government documents
- **PublicRecords**: Public records
- **Licenses**: Licenses and permits
- **Permits**: Building/business permits

## Detection Methods

Content domain is determined using multiple methods in priority order:

### 1. Metadata Hints (Highest Priority)
```rust
let mut metadata = HashMap::new();
metadata.insert("content_domain", "\"Contracts\"".to_string());

let (cid, domain) = store.put_with_domain(
    &content,
    Some("file.pdf"),
    None,
    None,
    Some(&metadata),
).await?;
```

### 2. Content Pattern Matching
The system analyzes content preview text for domain-specific keywords:

```rust
// Contract detection
keywords: ["contract", "agreement", "terms and conditions", "hereby agree"]

// Invoice detection
keywords: ["invoice", "bill to", "payment due", "total due"]

// Medical detection
keywords: ["patient", "diagnosis", "prescription", "lab results"]
```

### 3. MIME Type Mapping
```rust
"audio/mpeg" → Music
"video/mp4" → Video
"application/pdf" → Documents (unless patterns override)
```

### 4. File Extension Mapping
```rust
".mp3", ".wav", ".flac" → Music
".jpg", ".png", ".gif" → Images
".doc", ".docx", ".pdf" → Documents
".rs", ".py", ".js" → SourceCode
```

## Usage Examples

### Basic Usage

```rust
use cim_ipld::object_store::NatsObjectStore;
use cim_ipld::content_types::*;

// Store with automatic domain detection
let (cid, domain) = store.put_with_domain(
    &pdf_content,
    Some("invoice_2024.pdf"),
    Some("application/pdf"),
    Some("Invoice Number: INV-2024-001"),
    None,
).await?;

println!("Stored in domain: {:?}", domain); // Invoices
println!("Bucket: {}", store.get_bucket_for_domain(domain)); // cim-finance-invoices
```

### Retrieving from Domain Buckets

```rust
// Retrieve from specific domain
let content: PdfDocument = store.get_from_domain(
    &cid,
    ContentDomain::Invoices,
).await?;

// List all content in a domain
let invoices = store.list_domain(ContentDomain::Invoices).await?;
for invoice in invoices {
    println!("Invoice CID: {}", invoice.cid);
}
```

### Custom Pattern Matchers

```rust
use cim_ipld::object_store::PatternMatcher;

// Add custom pattern for recipe detection
store.update_partition_strategy(|strategy| {
    strategy.add_pattern_matcher(PatternMatcher {
        name: "recipe_detector".to_string(),
        keywords: vec![
            "ingredients".to_string(),
            "instructions".to_string(),
            "prep time".to_string(),
            "servings".to_string(),
        ],
        domain: ContentDomain::Personal,
        priority: 80,
    });
}).await;
```

### Custom Mappings

```rust
// Add custom file extension
store.update_partition_strategy(|strategy| {
    strategy.add_extension_mapping(
        "recipe".to_string(),
        ContentDomain::Personal,
    );
});

// Add custom MIME type
store.update_partition_strategy(|strategy| {
    strategy.add_mime_mapping(
        "application/x-recipe".to_string(),
        ContentDomain::Personal,
    );
});
```

## Bucket Structure

Each domain maps to a specific NATS JetStream bucket:

| Domain        | Bucket Name            |
| ------------- | ---------------------- |
| Music         | cim-media-music        |
| Video         | cim-media-video        |
| Images        | cim-media-images       |
| Graphics      | cim-media-graphics     |
| Documents     | cim-docs-general       |
| Spreadsheets  | cim-docs-sheets        |
| Presentations | cim-docs-presentations |
| Contracts     | cim-legal-contracts    |
| Invoices      | cim-finance-invoices   |
| Medical       | cim-health-medical     |
| SourceCode    | cim-tech-code          |
| SocialMedia   | cim-social-media       |
| Memes         | cim-social-memes       |

## Best Practices

### 1. Provide Content Previews
For best detection accuracy, provide content previews when storing:

```rust
let preview = &content_text[..1000.min(content_text.len())];
store.put_with_domain(&content, filename, mime_type, Some(preview), metadata).await?;
```

### 2. Use Metadata Hints
For critical content classification, use metadata hints:

```rust
let mut metadata = HashMap::new();
metadata.insert("content_domain", "\"Medical\"".to_string());
metadata.insert("confidential", "true".to_string());
```

### 3. Configure Custom Patterns
Add domain-specific patterns for your use case:

```rust
// Legal firm might add specific contract types
strategy.add_pattern_matcher(PatternMatcher {
    name: "nda_detector".to_string(),
    keywords: vec!["non-disclosure", "confidentiality", "proprietary information"],
    domain: ContentDomain::Agreements,
    priority: 110,
});
```

### 4. Monitor Domain Distribution
Regularly check content distribution across domains:

```rust
for domain in all_domains {
    let count = store.list_domain(domain).await?.len();
    println!("{:?}: {} objects", domain, count);
}
```

## Security Considerations

1. **Sensitive Content**: Medical, financial, and legal domains should have appropriate access controls
2. **Encryption**: Consider encrypting content in sensitive domains before storage
3. **Audit Trails**: Log all access to sensitive domain buckets
4. **Compliance**: Ensure domain partitioning meets regulatory requirements (HIPAA, GDPR, etc.)

## Performance Tips

1. **Batch Operations**: Process similar content types together
2. **Caching**: Cache partition strategy for frequently accessed content
3. **Indexing**: Create domain-specific indices for faster queries
4. **Monitoring**: Track bucket sizes and adjust retention policies as needed

## Extending the System

To add new domains:

1. Add the domain to the `ContentDomain` enum
2. Add bucket mapping in `init_domain_mappings()`
3. Add relevant extension/MIME mappings
4. Create pattern matchers for content detection
5. Update documentation

Example:
```rust
// In ContentDomain enum
Recipes,

// In init_domain_mappings()
self.domain_mapping.insert(ContentDomain::Recipes, "cim-personal-recipes".to_string());

// In init_extension_mappings()
self.extension_mapping.insert("recipe".to_string(), ContentDomain::Recipes);

// Add pattern matcher
PatternMatcher {
    name: "recipe_detector".to_string(),
    keywords: vec!["ingredients", "instructions", "prep time"],
    domain: ContentDomain::Recipes,
    priority: 75,
}
``` 