#!/usr/bin/env bash
set -euo pipefail

# Extract Document Domain Script
# This script extracts document-related code from cim-domain into cim-domain-document

echo "=== Extracting Document Domain ==="

# 1. Create the new document domain directory structure
echo "Creating cim-domain-document directory structure..."
mkdir -p cim-domain-document/{src,tests}
mkdir -p cim-domain-document/src/{aggregate,commands,events,handlers,projections,queries,value_objects}

# 2. Create Cargo.toml for document domain
echo "Creating Cargo.toml..."
cat > cim-domain-document/Cargo.toml << 'EOF'
[package]
name = "cim-domain-document"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core dependencies
uuid = { version = "1.11", features = ["v4", "serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2.0"
cid = { version = "0.11", features = ["serde"] }

# CIM dependencies
cim-core-domain = { path = "../cim-core-domain" }
cim-component = { path = "../cim-component" }

[dev-dependencies]
tokio = { version = "1.42", features = ["full"] }
EOF

# 3. Extract document aggregate
echo "Extracting document aggregate..."
cp cim-domain/src/document.rs cim-domain-document/src/aggregate/mod.rs

# 4. Create lib.rs
echo "Creating lib.rs..."
cat > cim-domain-document/src/lib.rs << 'EOF'
//! Document domain module for CIM
//!
//! This module contains the document aggregate and related components for managing
//! business documents with content-addressed storage using CIDs.

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod value_objects;

// Re-export main types
pub use aggregate::{
    Document, DocumentMarker,
    DocumentInfoComponent, ContentAddressComponent, ClassificationComponent,
    OwnershipComponent, LifecycleComponent, AccessControlComponent,
    RelationshipsComponent, ProcessingComponent,
    ConfidentialityLevel, DocumentStatus, RelationType,
    DocumentRelation, ExternalReference, ThumbnailInfo,
    PublicDocumentView, SearchIndexProjection,
};

pub use commands::*;
pub use events::*;
pub use handlers::{DocumentCommandHandler, DocumentEventHandler};
pub use projections::DocumentView;
pub use queries::{SearchDocuments, DocumentQueryHandler};
EOF

# 5. Create commands module
echo "Creating commands module..."
cat > cim-domain-document/src/commands/mod.rs << 'EOF'
//! Document commands

use cim_core_domain::command::Command;
use cim_core_domain::identifiers::EntityId;
use crate::aggregate::{
    DocumentMarker, DocumentInfoComponent, ConfidentialityLevel,
    DocumentStatus, DocumentRelation, ExternalReference,
};
use uuid::Uuid;
use cid::Cid;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Upload a new document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadDocument {
    /// Document's unique ID (generated by caller)
    pub document_id: Uuid,
    /// Document information
    pub info: DocumentInfoComponent,
    /// Content CID from object store
    pub content_cid: Cid,
    /// Whether the document is chunked
    pub is_chunked: bool,
    /// Chunk CIDs if chunked
    pub chunk_cids: Vec<Cid>,
    /// Who is uploading the document
    pub uploaded_by: Uuid,
}

impl Command for UploadDocument {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<DocumentMarker>> {
        Some(EntityId::from_uuid(self.document_id))
    }
}

/// Classify a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifyDocument {
    /// The ID of the document to classify
    pub document_id: Uuid,
    /// Document type
    pub document_type: String,
    /// Business category
    pub category: String,
    /// Subcategories
    pub subcategories: Vec<String>,
    /// Tags
    pub tags: Vec<String>,
    /// Confidentiality level
    pub confidentiality: ConfidentialityLevel,
    /// Who is classifying
    pub classified_by: Uuid,
}

impl Command for ClassifyDocument {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<DocumentMarker>> {
        Some(EntityId::from_uuid(self.document_id))
    }
}

/// Assign document ownership
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignDocumentOwnership {
    /// The ID of the document
    pub document_id: Uuid,
    /// New owner ID
    pub owner_id: Uuid,
    /// Author IDs
    pub authors: Vec<Uuid>,
    /// Department
    pub department: Option<String>,
    /// Project ID
    pub project_id: Option<Uuid>,
    /// Who is assigning ownership
    pub assigned_by: Uuid,
}

impl Command for AssignDocumentOwnership {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<DocumentMarker>> {
        Some(EntityId::from_uuid(self.document_id))
    }
}

/// Set document access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetDocumentAccessControl {
    /// The ID of the document
    pub document_id: Uuid,
    /// Read access list
    pub read_access: Vec<Uuid>,
    /// Write access list
    pub write_access: Vec<Uuid>,
    /// Share access list
    pub share_access: Vec<Uuid>,
    /// Enable audit logging
    pub audit_access: bool,
    /// Who is setting access control
    pub set_by: Uuid,
}

impl Command for SetDocumentAccessControl {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<DocumentMarker>> {
        Some(EntityId::from_uuid(self.document_id))
    }
}

/// Set document status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetDocumentStatus {
    /// The ID of the document
    pub document_id: Uuid,
    /// New status
    pub status: DocumentStatus,
    /// Reason for status change
    pub reason: Option<String>,
    /// Who is changing the status
    pub changed_by: Uuid,
}

impl Command for SetDocumentStatus {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<DocumentMarker>> {
        Some(EntityId::from_uuid(self.document_id))
    }
}

/// Process document (extract text, generate thumbnails, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDocument {
    /// The ID of the document to process
    pub document_id: Uuid,
    /// Extract text
    pub extract_text: bool,
    /// Perform OCR if needed
    pub perform_ocr: bool,
    /// Generate thumbnails
    pub generate_thumbnails: bool,
    /// Index for search
    pub index_document: bool,
    /// Who requested processing
    pub requested_by: Uuid,
}

impl Command for ProcessDocument {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<DocumentMarker>> {
        Some(EntityId::from_uuid(self.document_id))
    }
}

/// Add document relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDocumentRelationship {
    /// The ID of the document
    pub document_id: Uuid,
    /// Related document
    pub related_document: DocumentRelation,
    /// Who is adding the relationship
    pub added_by: Uuid,
}

impl Command for AddDocumentRelationship {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<DocumentMarker>> {
        Some(EntityId::from_uuid(self.document_id))
    }
}

/// Remove document relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveDocumentRelationship {
    /// The ID of the document
    pub document_id: Uuid,
    /// Related document ID to remove
    pub related_document_id: Uuid,
    /// Who is removing the relationship
    pub removed_by: Uuid,
}

impl Command for RemoveDocumentRelationship {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<DocumentMarker>> {
        Some(EntityId::from_uuid(self.document_id))
    }
}

/// Create a new version of a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentVersion {
    /// The ID of the document
    pub document_id: Uuid,
    /// New version number
    pub version_number: String,
    /// New content CID
    pub content_cid: Cid,
    /// Changes description
    pub changes_description: String,
    /// Who is creating the version
    pub created_by: Uuid,
}

impl Command for CreateDocumentVersion {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<DocumentMarker>> {
        Some(EntityId::from_uuid(self.document_id))
    }
}

/// Archive a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveDocument {
    /// The ID of the document to archive
    pub document_id: Uuid,
    /// Reason for archiving
    pub reason: String,
    /// Retention period in days
    pub retention_days: Option<u32>,
    /// Who is archiving
    pub archived_by: Uuid,
}

impl Command for ArchiveDocument {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<DocumentMarker>> {
        Some(EntityId::from_uuid(self.document_id))
    }
}
EOF

# 6. Create events module (placeholder - will be filled by complete script)
echo "Creating events module placeholder..."
cat > cim-domain-document/src/events/mod.rs << 'EOF'
//! Document domain events

// Events will be extracted by the complete script
EOF

# 7. Create handlers module
echo "Creating handlers module..."
cat > cim-domain-document/src/handlers/mod.rs << 'EOF'
//! Document command and event handlers

pub mod command_handler;
pub mod event_handler;

pub use command_handler::DocumentCommandHandler;
pub use event_handler::DocumentEventHandler;
EOF

# 8. Create command handler placeholder
echo "Creating command handler placeholder..."
cat > cim-domain-document/src/handlers/command_handler.rs << 'EOF'
//! Document command handler

use cim_core_domain::command::{CommandHandler, CommandEnvelope};
use cim_core_domain::repository::AggregateRepository;
use crate::Document;

pub struct DocumentCommandHandler<R: AggregateRepository<Document>> {
    repository: R,
}

impl<R: AggregateRepository<Document>> DocumentCommandHandler<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

// Command handler implementations will be added by complete script
EOF

# 9. Create event handler placeholder
echo "Creating event handler placeholder..."
cat > cim-domain-document/src/handlers/event_handler.rs << 'EOF'
//! Document event handler

use cim_core_domain::event::EventHandler;

pub struct DocumentEventHandler;

// Event handler implementations will be added as needed
EOF

# 10. Create projections module
echo "Creating projections module..."
cat > cim-domain-document/src/projections/mod.rs << 'EOF'
//! Document projections

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Document view for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentView {
    /// Document's unique identifier
    pub document_id: Uuid,
    /// Title of the document
    pub title: String,
    /// MIME type of the document
    pub mime_type: String,
    /// Current status of the document
    pub status: String,
    /// Name of the document owner
    pub owner_name: Option<String>,
    /// Size of the document in bytes
    pub size_bytes: u64,
    /// Creation timestamp
    pub created_at: String,
    /// Tags associated with the document
    pub tags: Vec<String>,
}
EOF

# 11. Create queries module
echo "Creating queries module..."
cat > cim-domain-document/src/queries/mod.rs << 'EOF'
//! Document queries

use cim_core_domain::query::Query;
use serde::{Deserialize, Serialize};

/// Query to search documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocuments {
    /// Text search query
    pub query: String,
    /// Filter by tags
    pub tags: Vec<String>,
    /// Filter by MIME types
    pub mime_types: Vec<String>,
    /// Maximum number of results to return
    pub limit: Option<usize>,
}

impl Query for SearchDocuments {}

/// Document query handler
pub struct DocumentQueryHandler;

// Query handler implementations will be added by complete script
EOF

# 12. Create value_objects module
echo "Creating value_objects module..."
cat > cim-domain-document/src/value_objects/mod.rs << 'EOF'
//! Document value objects

// Value objects are defined in the aggregate module
// This module can be used for additional value objects if needed
EOF

# 13. Create basic test
echo "Creating basic test..."
cat > cim-domain-document/tests/document_tests.rs << 'EOF'
use cim_domain_document::*;
use cim_core_domain::identifiers::EntityId;
use cid::Cid;

#[test]
fn test_document_creation() {
    let document_id = EntityId::<DocumentMarker>::new();
    let info = DocumentInfoComponent {
        title: "Test Document".to_string(),
        description: Some("A test document".to_string()),
        mime_type: "text/plain".to_string(),
        filename: Some("test.txt".to_string()),
        size_bytes: 1024,
        language: Some("en".to_string()),
    };

    let content_cid = Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap();

    let document = Document::new(document_id, info.clone(), content_cid);

    assert_eq!(document.id(), document_id);
    assert_eq!(document.version(), 0);
    assert_eq!(document.get_component::<DocumentInfoComponent>().unwrap().title, "Test Document");
}
EOF

# 14. Initialize git repository
echo "Initializing git repository..."
cd cim-domain-document
git init
git add .
git commit -m "Initial commit: Document domain module"

echo "=== Document Domain Extraction Complete ==="
echo "Next steps:"
echo "1. Run extract-document-domain-complete.sh to extract events and handlers"
echo "2. Remove document-related code from cim-domain"
echo "3. Push to GitHub repository"
echo "4. Add as submodule to main project"
