# Document Domain API Documentation

## Overview

The Document Domain provides comprehensive document lifecycle management within CIM. It supports content-addressed storage, versioning, collaboration, templates, and AI-powered content intelligence with full CQRS/Event Sourcing implementation.

**Status**: ✅ Production Ready  
**Tests**: 65 passing (24 library + 41 integration)  
**Version**: 1.0.0

## Table of Contents

1. [Commands](#commands)
2. [Events](#events)
3. [Queries](#queries)
4. [Value Objects](#value-objects)
5. [Services](#services)
6. [Integration Examples](#integration-examples)
7. [Error Handling](#error-handling)

## Commands

### CreateDocument

Creates a new document with metadata.

```rust
pub struct CreateDocument {
    pub document_id: DocumentId,
    pub document_type: DocumentType,
    pub title: String,
    pub author_id: Uuid,
    pub metadata: HashMap<String, String>,
}
```

**Example:**
```json
{
  "type": "CreateDocument",
  "payload": {
    "document_id": "doc-550e8400-e29b-41d4-a716-446655440000",
    "document_type": "Report",
    "title": "Q4 Financial Report",
    "author_id": "user-123e4567-e89b-12d3-a456-426614174000",
    "metadata": {
      "department": "Finance",
      "fiscal_year": "2024"
    }
  }
}
```

### UpdateContent

Updates document content blocks.

```rust
pub struct UpdateContent {
    pub document_id: DocumentId,
    pub content_blocks: Vec<ContentBlock>,
    pub change_summary: String,
    pub updated_by: Uuid,
}
```

**Example:**
```json
{
  "type": "UpdateContent",
  "payload": {
    "document_id": "doc-550e8400-e29b-41d4-a716-446655440000",
    "content_blocks": [
      {
        "id": "block-001",
        "block_type": "section",
        "title": "Executive Summary",
        "content": "Q4 showed strong growth...",
        "metadata": {
          "format": "markdown"
        }
      }
    ],
    "change_summary": "Added executive summary",
    "updated_by": "user-123e4567-e89b-12d3-a456-426614174000"
  }
}
```

### ShareDocument

Shares a document with access control.

```rust
pub struct ShareDocument {
    pub document_id: DocumentId,
    pub share_with: Uuid,
    pub access_level: AccessLevel,
    pub shared_by: Uuid,
}
```

### ChangeState

Changes document workflow state.

```rust
pub struct ChangeState {
    pub document_id: DocumentId,
    pub new_state: DocumentState,
    pub reason: String,
    pub changed_by: Uuid,
}
```

### ArchiveDocument

Archives a document with retention policy.

```rust
pub struct ArchiveDocument {
    pub document_id: Uuid,
    pub reason: String,
    pub retention_days: Option<u32>,
    pub archived_by: Uuid,
}
```

### LinkDocuments

Creates relationships between documents.

```rust
pub struct LinkDocuments {
    pub source_id: DocumentId,
    pub target_id: DocumentId,
    pub link_type: LinkType,
    pub metadata: HashMap<String, String>,
}
```

### AddComment

Adds a comment to a document.

```rust
pub struct AddComment {
    pub document_id: DocumentId,
    pub comment: Comment,
    pub author_id: Uuid,
}
```

### MergeDocuments

Merges multiple documents.

```rust
pub struct MergeDocuments {
    pub source_ids: Vec<DocumentId>,
    pub target_id: DocumentId,
    pub merge_strategy: MergeStrategy,
    pub conflict_resolution: ConflictResolution,
}
```

### ApplyTemplate

Applies a template to create a document.

```rust
pub struct ApplyTemplate {
    pub template_id: TemplateId,
    pub variables: HashMap<String, String>,
    pub document_id: DocumentId,
    pub created_by: Uuid,
}
```

### ImportDocument

Imports a document from external format.

```rust
pub struct ImportDocument {
    pub content: Vec<u8>,
    pub format: ImportFormat,
    pub options: ImportOptions,
    pub document_id: DocumentId,
}
```

## Events

### DocumentCreated

Emitted when a document is created.

```rust
pub struct DocumentCreated {
    pub document_id: DocumentId,
    pub document_type: DocumentType,
    pub title: String,
    pub author_id: Uuid,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}
```

### ContentUpdated

Emitted when content is updated.

```rust
pub struct ContentUpdated {
    pub document_id: DocumentId,
    pub content_blocks: Vec<ContentBlock>,
    pub change_summary: String,
    pub updated_by: Uuid,
    pub updated_at: DateTime<Utc>,
}
```

### StateChanged

Emitted when workflow state changes.

```rust
pub struct StateChanged {
    pub document_id: DocumentId,
    pub old_state: DocumentState,
    pub new_state: DocumentState,
    pub reason: String,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
}
```

### DocumentShared

Emitted when a document is shared.

```rust
pub struct DocumentShared {
    pub document_id: DocumentId,
    pub shared_with: HashSet<Uuid>,
    pub permissions: Vec<String>,
    pub shared_by: String,
    pub shared_at: DateTime<Utc>,
}
```

## Queries

### GetDocument

Retrieves a document by ID.

```rust
pub struct GetDocument {
    pub document_id: DocumentId,
    pub include_content: bool,
    pub include_metadata: bool,
}
```

**Response:**
```rust
pub struct DocumentView {
    pub document_id: DocumentId,
    pub title: String,
    pub document_type: DocumentType,
    pub state: DocumentState,
    pub author_id: Uuid,
    pub content_blocks: Vec<ContentBlock>,
    pub metadata: HashMap<String, String>,
    pub access_list: HashMap<Uuid, AccessLevel>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### SearchDocuments

Searches documents with filters.

```rust
pub struct SearchDocuments {
    pub query: String,
    pub tags: Vec<String>,
    pub mime_types: Vec<String>,
    pub limit: Option<usize>,
}
```

**Response:**
```rust
pub struct DocumentSearchView {
    pub document_id: DocumentId,
    pub title: String,
    pub snippet: String,
    pub score: f32,
    pub highlights: Vec<(usize, usize)>,
}
```

### GetDocumentHistory

Gets version history.

```rust
pub struct GetDocumentHistory {
    pub document_id: DocumentId,
    pub include_content_changes: bool,
}
```

### FindSimilarDocuments

Finds semantically similar documents.

```rust
pub struct FindSimilarDocuments {
    pub document_id: DocumentId,
    pub threshold: f32,
    pub limit: Option<usize>,
}
```

## Value Objects

### DocumentType

```rust
pub enum DocumentType {
    Text,
    Image,
    Video,
    Audio,
    Pdf,
    Spreadsheet,
    Presentation,
    Archive,
    Note,
    Article,
    Proposal,
    Report,
    Contract,
    Other(String),
}
```

### DocumentState

```rust
pub enum DocumentState {
    Draft,
    InReview,
    Approved,
    Rejected,
    Archived,
}
```

### AccessLevel

```rust
pub enum AccessLevel {
    Read,
    Comment,
    Write,
    Admin,
}
```

### ContentBlock

```rust
pub struct ContentBlock {
    pub id: String,
    pub block_type: String,
    pub title: Option<String>,
    pub content: String,
    pub metadata: HashMap<String, String>,
}
```

### DocumentVersion

```rust
pub struct DocumentVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}
```

### LinkType

```rust
pub enum LinkType {
    References,
    Related,
    Supersedes,
    DerivedFrom,
    PartOf,
}
```

## Services

### TemplateService

Manages document templates with variable substitution.

```rust
// Register a template
let template = DocumentTemplate {
    id: TemplateId::new(),
    name: "Meeting Notes".to_string(),
    content: "# {{title}}\nDate: {{date}}\nAttendees: {{attendees}}",
    required_variables: vec![
        TemplateVariable {
            name: "title".to_string(),
            var_type: VariableType::Text,
            required: true,
            ..Default::default()
        }
    ],
    ..Default::default()
};

template_service.register_template(template)?;

// Apply template
let variables = HashMap::from([
    ("title".to_string(), "Q4 Planning".to_string()),
    ("date".to_string(), "2024-12-15".to_string()),
]);

let content = template_service.apply_template(&template_id, &variables)?;
```

### ImportExportService

Handles document import/export.

```rust
// Import from Markdown
let imported = ImportExportService::import_document(
    markdown_content.as_bytes(),
    &ImportFormat::Markdown,
    &ImportOptions::default(),
)?;

// Export to HTML
let html = ImportExportService::export_document(
    &document,
    &ExportFormat::Html,
    &ExportOptions {
        include_metadata: true,
        include_history: false,
        ..Default::default()
    },
)?;
```

### ContentIntelligenceService

AI-powered content analysis (via integration tests).

- Entity extraction (NER)
- Document summarization
- Keyword extraction
- Sentiment analysis
- Content classification

## Integration Examples

### Document Workflow Integration

```rust
// Create document in draft
let create_doc = CreateDocument {
    document_id: DocumentId::new(),
    document_type: DocumentType::Proposal,
    title: "New Feature Proposal".to_string(),
    author_id: user_id,
    metadata: HashMap::new(),
};

command_bus.send(create_doc).await?;

// Move to review when ready
let change_state = ChangeState {
    document_id,
    new_state: DocumentState::InReview,
    reason: "Ready for review".to_string(),
    changed_by: user_id,
};

command_bus.send(change_state).await?;

// Share with reviewers
let share = ShareDocument {
    document_id,
    share_with: reviewer_id,
    access_level: AccessLevel::Comment,
    shared_by: user_id,
};

command_bus.send(share).await?;
```

### Cross-Domain Integration

```rust
// React to Workflow events
match event {
    WorkflowStepCompleted { step_id, .. } if step_id == "approval" => {
        // Change document state
        let cmd = ChangeState {
            document_id: related_doc_id,
            new_state: DocumentState::Approved,
            reason: "Workflow approval completed".to_string(),
            changed_by: system_user,
        };
        
        command_bus.send(cmd).await?;
    }
}
```

### Content Intelligence Integration

```rust
// Extract entities from document
let entities = content_intelligence_service
    .extract_entities(&document_content, &ExtractionOptions {
        extract_entities: true,
        extract_concepts: true,
        confidence_threshold: 0.7,
        ..Default::default()
    })
    .await?;

// Generate summary
let summary = content_intelligence_service
    .generate_summary(&document_content, SummaryLength::Standard)
    .await?;
```

## Error Handling

### Common Errors

```rust
pub enum DocumentError {
    DocumentNotFound(DocumentId),
    InvalidState {
        current: DocumentState,
        requested: DocumentState,
    },
    AccessDenied {
        user: Uuid,
        document: DocumentId,
        required: AccessLevel,
    },
    VersionConflict {
        expected: DocumentVersion,
        actual: DocumentVersion,
    },
    TemplateError(String),
    ImportError(String),
}
```

### Error Responses

```json
{
  "type": "https://cim.dev/errors/document-not-found",
  "title": "Document Not Found",
  "status": 404,
  "detail": "Document 'doc-550e8400-e29b-41d4-a716-446655440000' does not exist",
  "instance": "/api/documents/doc-550e8400-e29b-41d4-a716-446655440000"
}
```

## Performance Considerations

- **Content-Addressed Storage**: All content stored using CIDs for deduplication
- **Version Control**: Efficient diff-based storage for version history
- **Full-Text Search**: Indexed search with relevance scoring
- **Caching**: Frequently accessed documents cached in memory

## WebSocket Subscriptions

```javascript
const ws = new WebSocket('ws://localhost:8080/api/documents/subscribe');

ws.send(JSON.stringify({
  type: 'SubscribeToDocument',
  document_id: 'doc-550e8400-e29b-41d4-a716-446655440000'
}));

ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  // Handle ContentUpdated, StateChanged, etc.
};
```

## Rate Limits

- **Commands**: 50 requests/minute per user
- **Queries**: 500 requests/minute per user
- **Uploads**: 10MB max file size
- **Batch Operations**: 50 documents per batch

## SDK Examples

### Rust

```rust
use cim_document_client::{DocumentClient, CreateDocument};

let client = DocumentClient::new("https://api.cim.dev");
let doc = client.create_document(CreateDocument {
    title: "My Document".to_string(),
    document_type: DocumentType::Report,
    ..Default::default()
}).await?;
```

### TypeScript

```typescript
import { DocumentClient, DocumentType } from '@cim/document-client';

const client = new DocumentClient({
  baseUrl: 'https://api.cim.dev',
  apiKey: process.env.CIM_API_KEY
});

const doc = await client.createDocument({
  title: 'My Document',
  documentType: DocumentType.Report
});
``` 