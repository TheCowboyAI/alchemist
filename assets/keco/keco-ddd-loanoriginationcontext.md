# KÉCŌ Capital - Loan Origination Context

This document details the Domain-Driven Design (DDD) elements of the Loan Origination Context, which handles the initial loan application process, borrower file creation, and document collection.

## Context Overview

The Loan Origination Context is responsible for:
- Capturing borrower information
- Creating and maintaining deal files
- Managing the document collection process
- Generating initial term outlines (ITOs)
- Handling the submission of complete files for underwriting

## Aggregate: BorrowerAggregate

```mermaid
classDiagram
    class BorrowerAggregate {
        <<Aggregate Root>>
        +BorrowerFileId id
        +String guarantorName
        +Contact contact
        +Address homeAddress
        +String ssn
        +CreateBorrowerFile()
        +SubmitTrackRecord()
        +SubmitCreditAuthorization()
        +SubmitDeclarations()
        +VerifyEntityDocuments()
        +SubmitLoanApplication()
    }
    
    class BorrowerFile {
        +BorrowerFileId id
        +BorrowerFileStatus status
        +DateTime creationDate
        +List~Document~ documents
        +AddDocument(Document)
        +VerifyDocument(DocumentId)
        +SubmitCompletedFile()
    }
    
    class TrackRecord {
        <<Value Object>>
        +List~Property~ previousProperties
        +DateTime startDate
        +DateTime endDate
        +Validate()
    }
    
    class Contact {
        <<Value Object>>
        +String email
        +String phoneNumber
        +Validate()
    }
    
    class Address {
        <<Value Object>>
        +String street
        +String city
        +String state
        +String zipCode
        +Validate()
    }
    
    BorrowerAggregate "1" --> "1" BorrowerFile : contains
    BorrowerAggregate "1" --> "1" TrackRecord : has
    BorrowerAggregate "1" --> "1" Contact : has
    BorrowerAggregate "1" --> "1" Address : has
```

## Aggregate: DealFileAggregate

```mermaid
classDiagram
    class DealFileAggregate {
        <<Aggregate Root>>
        +DealFileId id
        +PropertyAddress propertyAddress
        +Money purchasePrice
        +Money rehabAmount
        +Money arv
        +DateTime closingDate
        +LoanType loanType
        +DealFileStatus status
        +CreateDealFile()
        +LinkBorrowerFile(BorrowerFileId)
        +SelectLoanType(LoanType)
        +SubmitPropertyDetails()
        +RequestInitialTermOutline()
    }
    
    class PropertyAddress {
        <<Value Object>>
        +String street
        +String city
        +String state
        +String zipCode
        +Validate()
    }
    
    class Money {
        <<Value Object>>
        +Decimal amount
        +Currency currency
        +Validate()
    }
    
    class InitialTermOutline {
        +ItoId id
        +List~LoanOption~ options
        +DateTime generationDate
        +DateTime expirationDate
        +GenerateOptions()
        +Select(LoanOptionId)
    }
    
    class LoanOption {
        <<Value Object>>
        +LoanOptionId id
        +Decimal interestRate
        +Int termMonths
        +Money loanAmount
        +Decimal ltv
        +List~String~ requirements
    }
    
    class DocumentRequest {
        +List~RequiredDocument~ requiredDocuments
        +DateTime requestDate
        +DateTime dueDate
        +TrackStatus()
    }
    
    DealFileAggregate "1" --> "1" PropertyAddress : for
    DealFileAggregate "1" --> "3" Money : has
    DealFileAggregate "1" --> "1" InitialTermOutline : generates
    DealFileAggregate "1" --> "1" DocumentRequest : creates
    InitialTermOutline "1" --> "1..*" LoanOption : contains
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
    
    class BorrowerFileCreatedEvent {
        +BorrowerFileId borrowerFileId
        +String guarantorName
    }
    
    class DealFileCreatedEvent {
        +DealFileId dealFileId
        +PropertyAddress propertyAddress
        +Money purchasePrice
    }
    
    class DocumentUploadedEvent {
        +DocumentId documentId
        +String documentType
        +String fileName
        +DateTime uploadDate
    }
    
    class InitialTermOutlineGeneratedEvent {
        +ItoId itoId
        +DealFileId dealFileId
        +DateTime generationDate
    }
    
    class LoanOptionSelectedEvent {
        +LoanOptionId loanOptionId
        +ItoId itoId
        +DealFileId dealFileId
    }
    
    class DocumentRequestCreatedEvent {
        +List~String~ requiredDocumentTypes
        +DealFileId dealFileId
        +DateTime dueDate
    }
    
    class FileSubmittedToUnderwritingEvent {
        +DealFileId dealFileId
        +BorrowerFileId borrowerFileId
        +DateTime submissionDate
    }
    
    DomainEvent <|-- BorrowerFileCreatedEvent
    DomainEvent <|-- DealFileCreatedEvent
    DomainEvent <|-- DocumentUploadedEvent
    DomainEvent <|-- InitialTermOutlineGeneratedEvent
    DomainEvent <|-- LoanOptionSelectedEvent
    DomainEvent <|-- DocumentRequestCreatedEvent
    DomainEvent <|-- FileSubmittedToUnderwritingEvent
```

## Entity Relationships

```mermaid
flowchart TD
    B[Borrower]
    BF[BorrowerFile]
    D[DealFile]
    ITO[InitialTermOutline]
    DOC[Document]
    DR[DocumentRequest]
    
    B -->|creates| BF
    B -->|submits| D
    D -->|linked to| BF
    D -->|generates| ITO
    D -->|issues| DR
    BF -->|contains| DOC
    DR -->|requires| DOC
    
    classDef entity fill:#f5f5f5,stroke:#333,stroke-width:1px
    class B,BF,D,ITO,DOC,DR entity
```

## Repositories

- **BorrowerRepository** - Manages persistence of Borrower aggregates
- **DealFileRepository** - Manages persistence of DealFile aggregates
- **DocumentRepository** - Provides access to document metadata and content

## Domain Services

- **LoanOriginationService** - Orchestrates the overall loan origination process
- **InitialTermOutlineService** - Generates loan options based on borrower and property information
- **DocumentVerificationService** - Verifies uploaded documents meet requirements
- **FileSubmissionService** - Validates and prepares complete files for underwriting

## Integration with Other Contexts

- Publishes events to Document Context for document verification
- Submits completed files to Underwriting Context for evaluation
- Receives notifications from Timer Service for deadlines 