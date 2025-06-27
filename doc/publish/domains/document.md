# Document Domain

## Overview

The Document Domain handles all aspects of document lifecycle management within CIM, including creation, versioning, collaboration, workflow integration, and content processing. It supports various document types from simple text files to complex structured documents with rich metadata.

## Key Concepts

### Document
- **Definition**: A container for content with metadata and lifecycle
- **Types**: Text, PDF, spreadsheet, presentation, image, video
- **Properties**: ID, title, content, metadata, version, status
- **Lifecycle**: Draft → Review → Published → Archived

### Document Version
- **Definition**: A specific iteration of a document's content
- **Properties**: Version number, changes, author, timestamp
- **Operations**: Create, compare, restore, merge
- **Storage**: Content-addressed with deduplication

### Document Metadata
- **Definition**: Structured information about a document
- **Categories**: System (size, type), custom (tags, properties)
- **Searchable**: Full-text and metadata search
- **Extensible**: Domain-specific metadata schemas

### Document Workflow
- **Definition**: Processes associated with document lifecycle
- **Types**: Review, approval, signature, distribution
- **Integration**: Workflow domain for complex processes
- **Tracking**: Complete audit trail of all actions

## Domain Events

### Commands
- `cmd.document.create_document` - Create new document
- `cmd.document.update_content` - Modify document content
- `cmd.document.add_metadata` - Attach metadata
- `cmd.document.start_workflow` - Initiate document workflow
- `cmd.document.sign_document` - Apply digital signature

### Events
- `event.document.document_created` - New document created
- `event.document.version_created` - New version saved
- `event.document.metadata_updated` - Metadata changed
- `event.document.workflow_completed` - Process finished
- `event.document.document_signed` - Signature applied

### Queries
- `query.document.search` - Full-text search
- `query.document.get_versions` - Version history
- `query.document.get_metadata` - Retrieve metadata
- `query.document.find_by_workflow` - Documents in process

## API Reference

### DocumentAggregate
```rust
pub struct DocumentAggregate {
    pub id: DocumentId,
    pub title: String,
    pub document_type: DocumentType,
    pub current_version: Version,
    pub versions: HashMap<Version, DocumentVersion>,
    pub metadata: DocumentMetadata,
    pub status: DocumentStatus,
}
```

### Key Methods
- `create_document()` - Initialize document
- `update_content()` - Create new version
- `add_metadata()` - Attach metadata
- `start_workflow()` - Begin process
- `compare_versions()` - Diff versions

## Document Management

### Creating Documents
```rust
// Create text document
let document = CreateDocument {
    title: "Q4 Financial Report".to_string(),
    document_type: DocumentType::Report,
    initial_content: Content::Text(report_text),
    metadata: DocumentMetadata {
        author: author_id,
        department: "Finance".to_string(),
        tags: vec!["quarterly", "financial", "2024"],
        custom: HashMap::from([
            ("fiscal_year", "2024"),
            ("quarter", "Q4"),
        ]),
    },
};

// Create from template
let from_template = CreateFromTemplate {
    template_id,
    title: "New Contract".to_string(),
    variables: HashMap::from([
        ("client_name", "Acme Corp"),
        ("contract_value", "$50,000"),
        ("start_date", "2024-01-01"),
    ]),
};
```

### Version Control
```rust
// Update document content
let update = UpdateContent {
    document_id,
    new_content: Content::Text(updated_text),
    change_summary: "Updated financial projections".to_string(),
    author: editor_id,
};

// Compare versions
let diff = CompareVersions {
    document_id,
    version_a: Version(1),
    version_b: Version(3),
    diff_type: DiffType::LineDiff,
};

// Restore previous version
let restore = RestoreVersion {
    document_id,
    target_version: Version(2),
    reason: "Reverting incorrect changes".to_string(),
};
```

### Content Processing
```rust
// Extract text from various formats
pub trait ContentExtractor {
    fn extract_text(&self, content: &Content) -> Result<String>;
    fn extract_metadata(&self, content: &Content) -> Result<Metadata>;
    fn extract_structure(&self, content: &Content) -> Result<Structure>;
}

// Process document
let processed = ProcessDocument {
    document_id,
    operations: vec![
        Operation::ExtractText,
        Operation::GenerateThumbnail,
        Operation::DetectLanguage,
        Operation::ExtractEntities,
    ],
};

// Results
let results = ProcessingResults {
    text: "Extracted document text...".to_string(),
    thumbnail: image_data,
    language: "en".to_string(),
    entities: vec![
        Entity::Person("John Doe"),
        Entity::Organization("Acme Corp"),
        Entity::Money("$50,000"),
    ],
};
```

## Document Workflows

### Approval Workflow
```rust
// Start approval process
let approval = StartApprovalWorkflow {
    document_id,
    approvers: vec![
        Approver {
            identity_id: manager_id,
            role: ApproverRole::Primary,
            order: 1,
        },
        Approver {
            identity_id: director_id,
            role: ApproverRole::Final,
            order: 2,
        },
    ],
    deadline: SystemTime::now() + Duration::days(7),
    escalation_policy: EscalationPolicy::NotifyManager,
};

// Approve document
let approve = ApproveDocument {
    document_id,
    workflow_id,
    approver_id,
    comments: Some("Approved with minor suggestions".to_string()),
    conditions: vec![
        "Fix typo on page 3",
        "Update chart on page 7",
    ],
};
```

### Digital Signatures
```rust
// Request signatures
let signature_request = RequestSignatures {
    document_id,
    signers: vec![
        Signer {
            identity_id: party_a_id,
            signature_type: SignatureType::Digital,
            fields: vec!["signature_1", "initial_1"],
        },
        Signer {
            identity_id: party_b_id,
            signature_type: SignatureType::Electronic,
            fields: vec!["signature_2"],
        },
    ],
    completion_deadline: SystemTime::now() + Duration::days(30),
};

// Apply signature
let sign = SignDocument {
    document_id,
    signature_request_id,
    signer_id,
    signature_data: SignatureData {
        signature_image: image_data,
        certificate: certificate_data,
        timestamp: SystemTime::now(),
        ip_address: "192.168.1.1".to_string(),
    },
};
```

## Search and Discovery

### Full-Text Search
```rust
// Search documents
let search = SearchDocuments {
    query: "financial projections 2024".to_string(),
    filters: SearchFilters {
        document_types: vec![DocumentType::Report, DocumentType::Spreadsheet],
        date_range: Some(DateRange {
            start: "2024-01-01".to_string(),
            end: "2024-12-31".to_string(),
        }),
        authors: vec![author_id],
        tags: vec!["financial"],
        status: Some(DocumentStatus::Published),
    },
    options: SearchOptions {
        highlight: true,
        snippet_length: 200,
        max_results: 50,
        sort_by: SortBy::Relevance,
    },
};

// Search results
let results = SearchResults {
    total_hits: 127,
    documents: vec![
        SearchHit {
            document_id,
            title: "Q4 Financial Report".to_string(),
            snippet: "...projected <em>financial</em> growth for <em>2024</em>...",
            relevance_score: 0.95,
            metadata: hit_metadata,
        },
        // ... more results
    ],
    facets: SearchFacets {
        document_types: HashMap::from([
            (DocumentType::Report, 89),
            (DocumentType::Spreadsheet, 38),
        ]),
        tags: HashMap::from([
            ("financial", 127),
            ("quarterly", 76),
        ]),
    },
};
```

### Metadata Queries
```rust
// Query by metadata
let metadata_query = QueryByMetadata {
    filters: vec![
        MetadataFilter::Equals("department", "Finance"),
        MetadataFilter::Contains("tags", "approved"),
        MetadataFilter::GreaterThan("version", "2"),
    ],
    include_content: false,
};

// Aggregate metadata
let aggregation = AggregateMetadata {
    group_by: "department",
    metrics: vec![
        Metric::Count,
        Metric::Sum("file_size"),
        Metric::Average("version"),
    ],
};
```

## Collaboration Features

### Comments and Annotations
```rust
// Add comment to document
let comment = AddComment {
    document_id,
    comment: Comment {
        author_id,
        text: "Please review the budget figures".to_string(),
        context: Some(CommentContext {
            page: 3,
            selection: "Q4 revenue projections",
        }),
        thread_id: None, // New thread
    },
};

// Add annotation
let annotation = AddAnnotation {
    document_id,
    annotation: Annotation {
        type: AnnotationType::Highlight,
        page: 5,
        bounds: Rectangle { x: 100, y: 200, width: 300, height: 50 },
        color: Color::Yellow,
        note: Some("Important figure".to_string()),
    },
};
```

### Sharing and Permissions
```rust
// Share document
let share = ShareDocument {
    document_id,
    recipients: vec![
        Recipient {
            identity_id: user_id,
            permission: Permission::View,
            expiry: Some(SystemTime::now() + Duration::days(30)),
        },
        Recipient {
            identity_id: editor_id,
            permission: Permission::Edit,
            expiry: None,
        },
    ],
    message: Some("Please review and provide feedback".to_string()),
};

// Set access control
let access = SetAccessControl {
    document_id,
    access_control: AccessControl {
        owner: owner_id,
        public_access: PublicAccess::None,
        inherit_from_folder: false,
        explicit_permissions: permissions,
    },
};
```

## Integration Features

### Template Management
```rust
// Create template from document
let template = CreateTemplate {
    source_document_id,
    template_name: "Contract Template".to_string(),
    variables: vec![
        TemplateVariable {
            name: "client_name",
            type: VariableType::Text,
            required: true,
            default: None,
        },
        TemplateVariable {
            name: "contract_value",
            type: VariableType::Currency,
            required: true,
            default: Some("$0.00"),
        },
    ],
    sections: vec![
        TemplateSection {
            name: "terms",
            optional: true,
            repeatable: false,
        },
    ],
};
```

### External Storage Integration
```rust
// Store in external system
impl DocumentStorage for S3Storage {
    async fn store(&self, document: &Document) -> Result<StorageRef> {
        let key = format!("documents/{}/{}", document.id, document.version);
        let result = self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(document.content.as_bytes())
            .send()
            .await?;
        
        Ok(StorageRef {
            storage_type: StorageType::S3,
            location: key,
            etag: result.e_tag,
        })
    }
}
```

## Use Cases

### Contract Management
- Template-based creation
- Approval workflows
- Digital signatures
- Version tracking
- Expiry monitoring

### Knowledge Base
- Document organization
- Full-text search
- Categorization
- Related documents
- Access control

### Compliance Documentation
- Audit trails
- Version history
- Approval tracking
- Retention policies
- Access logs

### Collaborative Editing
- Real-time collaboration
- Comment threads
- Change tracking
- Conflict resolution
- Notification system

## Performance Characteristics

- **Document Capacity**: 10M+ documents
- **Version History**: Unlimited with deduplication
- **Search Speed**: <100ms for full-text search
- **Upload Speed**: 100MB/s with chunking

## Best Practices

1. **Version Control**: Always create new versions for changes
2. **Metadata Standards**: Use consistent metadata schemas
3. **Search Optimization**: Index frequently searched fields
4. **Access Control**: Implement least-privilege access
5. **Retention Policies**: Define and enforce document lifecycle

## Related Domains

- **Workflow Domain**: Document-centric processes
- **Identity Domain**: Author and access control
- **Agent Domain**: AI-powered document processing
- **Policy Domain**: Retention and compliance rules 