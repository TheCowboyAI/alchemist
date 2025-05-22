# KÉCŌ Capital - Document Context

This document details the Domain-Driven Design (DDD) elements of the Document Context, which manages document uploads, verifications, and storage across the lending platform.

## Context Overview

The Document Context is responsible for:
- Managing document uploads and storage
- Tracking document metadata and status
- Coordinating document verification processes
- Ensuring document compliance with requirements
- Providing document access to other contexts

## Aggregate: DocumentAggregate

```mermaid
classDiagram
    class DocumentAggregate {
        <<Aggregate Root>>
        +DocumentId id
        +String fileName
        +DocumentType type
        +DateTime uploadDate
        +String uploadedBy
        +DocumentStatus status
        +EntityId ownerId
        +FileLocation location
        +Upload(File file)
        +VerifyDocument()
        +UpdateStatus(DocumentStatus)
        +Archive()
    }
    
    class FileMetadata {
        <<Value Object>>
        +String contentType
        +Long sizeInBytes
        +String hash
        +DateTime lastModified
        +Validate()
    }
    
    class DocumentContent {
        +Byte[] content
        +String encoding
        +Retrieve()
        +Store()
    }
    
    class VerificationStatus {
        <<Value Object>>
        +Boolean isVerified
        +String verifiedBy
        +DateTime verificationDate
        +String verificationNotes
        +List~VerificationRule~ appliedRules
    }
    
    class VerificationRule {
        <<Value Object>>
        +RuleType type
        +String description
        +Boolean passed
        +String failureReason
    }
    
    DocumentAggregate "1" --> "1" FileMetadata : has
    DocumentAggregate "1" --> "1" DocumentContent : contains
    DocumentAggregate "1" --> "1" VerificationStatus : has
    VerificationStatus "1" --> "0..*" VerificationRule : applies
```

## Aggregate: DocumentRequirementAggregate

```mermaid
classDiagram
    class DocumentRequirementAggregate {
        <<Aggregate Root>>
        +RequirementId id
        +EntityId requestingEntityId
        +EntityId targetEntityId
        +List~DocumentTypeRequirement~ requiredDocuments
        +DateTime requestDate
        +DateTime dueDate
        +RequirementStatus status
        +CreateRequirement()
        +AddDocumentType(DocumentTypeRequirement)
        +RemoveDocumentType(DocumentTypeId)
        +TrackFulfillment()
        +Complete()
    }
    
    class DocumentTypeRequirement {
        +DocumentTypeId id
        +DocumentType type
        +Boolean isRequired
        +String description
        +List~VerificationCriteria~ criteria
    }
    
    class VerificationCriteria {
        <<Value Object>>
        +CriteriaType type
        +String description
        +String validationRule
    }
    
    class FulfillmentStatus {
        <<Value Object>>
        +DocumentTypeId documentTypeId
        +Boolean isFulfilled
        +DocumentId documentId
        +DateTime fulfillmentDate
    }
    
    DocumentRequirementAggregate "1" --> "1..*" DocumentTypeRequirement : requires
    DocumentRequirementAggregate "1" --> "0..*" FulfillmentStatus : tracks
    DocumentTypeRequirement "1" --> "0..*" VerificationCriteria : specifies
```

## Domain Events

```mermaid
classDiagram
    class DomainEvent {
        <<abstract>>
        +EventId id
        +DateTime timestamp
        +String aggregateId
    }
    
    class DocumentUploadedEvent {
        +DocumentId documentId
        +EntityId ownerId
        +DocumentType type
        +DateTime uploadDate
    }
    
    class DocumentVerificationStartedEvent {
        +DocumentId documentId
        +DateTime startDate
    }
    
    class DocumentVerifiedEvent {
        +DocumentId documentId
        +Boolean passed
        +List~String~ failedRules
        +DateTime verificationDate
    }
    
    class DocumentRequirementCreatedEvent {
        +RequirementId requirementId
        +EntityId targetEntityId
        +List~DocumentTypeId~ requiredDocumentTypes
        +DateTime dueDate
    }
    
    class DocumentRequirementFulfilledEvent {
        +RequirementId requirementId
        +DocumentTypeId documentTypeId
        +DocumentId documentId
        +DateTime fulfillmentDate
    }
    
    class DocumentRequirementCompletedEvent {
        +RequirementId requirementId
        +DateTime completionDate
    }
    
    DomainEvent <|-- DocumentUploadedEvent
    DomainEvent <|-- DocumentVerificationStartedEvent
    DomainEvent <|-- DocumentVerifiedEvent
    DomainEvent <|-- DocumentRequirementCreatedEvent
    DomainEvent <|-- DocumentRequirementFulfilledEvent
    DomainEvent <|-- DocumentRequirementCompletedEvent
```

## Entity Relationships

```mermaid
flowchart TD
    Doc[Document]
    MD[FileMetadata]
    VS[VerificationStatus]
    VR[VerificationRule]
    
    DR[DocumentRequirement]
    DTR[DocumentTypeRequirement]
    VC[VerificationCriteria]
    FS[FulfillmentStatus]
    
    BF[BorrowerFile]
    DF[DealFile]
    
    Doc -->|has| MD
    Doc -->|has| VS
    VS -->|applies| VR
    
    DR -->|requires| DTR
    DR -->|tracks| FS
    DTR -->|specifies| VC
    
    BF -->|requests| DR
    DF -->|requests| DR
    DR -->|fulfilled by| Doc
    
    classDef entity fill:#f5f5f5,stroke:#333,stroke-width:1px
    class Doc,MD,VS,VR,DR,DTR,VC,FS,BF,DF entity
```

## Service Layer: Document Verification Service

```mermaid
classDiagram
    class DocumentVerificationService {
        +VerifyDocument(DocumentId)
        +VerifyAgainstCriteria(Document, List~VerificationCriteria~)
        +HandleFailedVerification(Document, List~String~)
        +RequestManualVerification(Document)
    }
    
    class AutomatedVerifier {
        <<interface>>
        +Boolean verify(Document, VerificationCriteria)
        +String getFailureReason()
    }
    
    class TextExtractorVerifier {
        +Boolean verify(Document, VerificationCriteria)
        +String getFailureReason()
        -String extractText(Document)
        -Boolean matchPattern(String, String)
    }
    
    class ImageVerifier {
        +Boolean verify(Document, VerificationCriteria)
        +String getFailureReason()
        -Boolean validateImageQuality(Document)
        -Boolean detectDocumentType(Document)
    }
    
    class SignatureVerifier {
        +Boolean verify(Document, VerificationCriteria)
        +String getFailureReason()
        -Boolean detectSignature(Document)
        -Boolean validateSignatureDate(Document)
    }
    
    DocumentVerificationService ..> AutomatedVerifier : uses
    AutomatedVerifier <|.. TextExtractorVerifier
    AutomatedVerifier <|.. ImageVerifier
    AutomatedVerifier <|.. SignatureVerifier
```

## Document Type Catalog

```mermaid
classDiagram
    class DocumentType {
        <<enumeration>>
        OperatingAgreement
        ArticlesOfOrganization
        TrackRecord
        PurchaseContract
        AntiMoneyLaundering
        BorrowerBusinessPurpose
        ScopeOfWork
        CostOfGoodsSchedule
        GCLicense
        GCInsurance
        DriversLicense
        Appraisal
        InsurancePolicy
        TitleReport
        W9
        BankStatements
        LoanApplication
        CorporateResolution
    }
```

## Repositories

- **DocumentRepository** - Manages persistence of documents and their content
- **DocumentRequirementRepository** - Manages document requirements and fulfillment
- **VerificationRuleRepository** - Provides access to verification rules and criteria

## Domain Services

- **DocumentService** - Handles document upload, retrieval, and lifecycle management
- **DocumentVerificationService** - Coordinates document verification processes
- **DocumentRequirementService** - Manages document requirements and tracks fulfillment
- **DocumentSearchService** - Provides search capabilities across documents

## Integration with Other Contexts

- Receives document upload requests from Loan Origination Context
- Receives document verification requests from Underwriting Context
- Notifies other contexts of document verification results
- Provides document access and retrieval services to all contexts
- Coordinates with Timer Service for document requirement deadlines 