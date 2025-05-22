# KÉCŌ Capital - Closing Context

This document details the Domain-Driven Design (DDD) elements of the Closing Context, which handles the loan closing process, document signing, loan recording, and payment processing.

## Context Overview

The Closing Context is responsible for:
- Coordinating the loan closing process
- Managing document preparation for closing
- Scheduling and tracking document signing
- Processing loan recording with local authorities
- Handling fund disbursement
- Verifying payment receipts

## Aggregate: ClosingFileAggregate

```mermaid
classDiagram
    class ClosingFileAggregate {
        <<Aggregate Root>>
        +ClosingFileId id
        +DealFileId dealFileId
        +CTCId ctcId
        +ClosingStatus status
        +DateTime creationDate
        +DateTime scheduledClosingDate
        +Address closingLocation
        +List~ClosingParticipant~ participants
        +CreateClosingFile()
        +ScheduleClosing(DateTime, Address)
        +AddParticipant(ClosingParticipant)
        +UpdateStatus(ClosingStatus)
        +CompleteClosing()
    }
    
    class ClosingParticipant {
        <<Value Object>>
        +ParticipantId id
        +String name
        +ParticipantRole role
        +Contact contact
        +Boolean confirmed
        +DateTime confirmationDate
    }
    
    class ClosingDocument {
        +DocumentId id
        +String documentName
        +DocumentType type
        +DocumentStatus status
        +Boolean isSignatureRequired
        +List~String~ requiredSigners
        +DateTime preparedDate
        +Prepare()
        +MarkSigned(List~String~ signers)
    }
    
    class ClosingInstructions {
        +InstructionsId id
        +String title
        +List~String~ steps
        +DateTime creationDate
        +String createdBy
        +Generate()
        +Update(List~String~ steps)
    }
    
    ClosingFileAggregate "1" --> "1..*" ClosingParticipant : involves
    ClosingFileAggregate "1" --> "1..*" ClosingDocument : includes
    ClosingFileAggregate "1" --> "1" ClosingInstructions : contains
```

## Aggregate: FundingAggregate

```mermaid
classDiagram
    class FundingAggregate {
        <<Aggregate Root>>
        +FundingId id
        +ClosingFileId closingFileId
        +Money loanAmount
        +FundingStatus status
        +DateTime requestDate
        +DateTime fundingDate
        +CreateFundingRequest()
        +ApproveFunding()
        +ProcessFunding()
        +ConfirmFundingComplete()
    }
    
    class FundingInstruction {
        <<Value Object>>
        +BankAccount sourceBankAccount
        +BankAccount destinationBankAccount
        +Money amount
        +String referenceNumber
        +DateTime instructionDate
        +String approvedBy
        +Generate()
        +Validate()
    }
    
    class FundingConfirmation {
        +ConfirmationId id
        +String confirmation
        +DateTime confirmationDate
        +Money confirmedAmount
        +String confirmingPerson
        +Record()
        +Verify()
    }
    
    class Money {
        <<Value Object>>
        +Decimal amount
        +Currency currency
        +Validate()
    }
    
    class BankAccount {
        <<Value Object>>
        +String accountNumber
        +String routingNumber
        +String bankName
        +String accountHolderName
        +AccountType type
        +Validate()
    }
    
    FundingAggregate "1" --> "1" FundingInstruction : follows
    FundingAggregate "1" --> "0..1" FundingConfirmation : has
    FundingInstruction "1" --> "1" Money : transfers
    FundingInstruction "1" --> "2" BankAccount : references
```

## Aggregate: RecordingAggregate

```mermaid
classDiagram
    class RecordingAggregate {
        <<Aggregate Root>>
        +RecordingId id
        +ClosingFileId closingFileId
        +LoanId loanId
        +PropertyAddress propertyAddress
        +RecordingStatus status
        +DateTime submissionDate
        +DateTime recordingDate
        +CreateRecordingRequest()
        +SubmitForRecording()
        +ConfirmRecording(String recordingNumber)
        +ProvideRecordingEvidence()
    }
    
    class PropertyAddress {
        <<Value Object>>
        +String street
        +String city
        +String county
        +String state
        +String zipCode
        +Validate()
    }
    
    class RecordingAuthority {
        <<Value Object>>
        +String authorityName
        +String jurisdiction
        +String address
        +String contactInfo
        +Validate()
    }
    
    class RecordingConfirmation {
        +ConfirmationId id
        +String recordingNumber
        +DateTime officialRecordingDate
        +String recordingOfficer
        +Record()
        +Verify()
    }
    
    RecordingAggregate "1" --> "1" PropertyAddress : for
    RecordingAggregate "1" --> "1" RecordingAuthority : submittedTo
    RecordingAggregate "1" --> "0..1" RecordingConfirmation : confirms
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
    
    class ClosingFileCreatedEvent {
        +ClosingFileId closingFileId
        +DealFileId dealFileId
        +DateTime creationDate
    }
    
    class ClosingScheduledEvent {
        +ClosingFileId closingFileId
        +DateTime scheduledDate
        +Address location
    }
    
    class ClosingDocumentPreparedEvent {
        +DocumentId documentId
        +ClosingFileId closingFileId
        +DocumentType type
        +DateTime preparedDate
    }
    
    class DocumentSignedEvent {
        +DocumentId documentId
        +List~String~ signers
        +DateTime signDate
    }
    
    class ClosingCompletedEvent {
        +ClosingFileId closingFileId
        +DateTime completionDate
    }
    
    class FundingRequestedEvent {
        +FundingId fundingId
        +ClosingFileId closingFileId
        +Money amount
        +DateTime requestDate
    }
    
    class FundingApprovedEvent {
        +FundingId fundingId
        +String approvedBy
        +DateTime approvalDate
    }
    
    class FundingCompletedEvent {
        +FundingId fundingId
        +Money amount
        +DateTime fundingDate
    }
    
    class RecordingRequestedEvent {
        +RecordingId recordingId
        +ClosingFileId closingFileId
        +DateTime requestDate
    }
    
    class RecordingConfirmedEvent {
        +RecordingId recordingId
        +String recordingNumber
        +DateTime recordingDate
    }
    
    DomainEvent <|-- ClosingFileCreatedEvent
    DomainEvent <|-- ClosingScheduledEvent
    DomainEvent <|-- ClosingDocumentPreparedEvent
    DomainEvent <|-- DocumentSignedEvent
    DomainEvent <|-- ClosingCompletedEvent
    DomainEvent <|-- FundingRequestedEvent
    DomainEvent <|-- FundingApprovedEvent
    DomainEvent <|-- FundingCompletedEvent
    DomainEvent <|-- RecordingRequestedEvent
    DomainEvent <|-- RecordingConfirmedEvent
```

## Entity Relationships

```mermaid
flowchart TD
    CTC[ClearToClose]
    CF[ClosingFile]
    CD[ClosingDocuments]
    CP[ClosingParticipant]
    CI[ClosingInstructions]
    
    FR[FundingRequest]
    FI[FundingInstructions]
    FC[FundingConfirmation]
    
    RR[RecordingRequest]
    RA[RecordingAuthority]
    RC[RecordingConfirmation]
    
    CTC -->|initiates| CF
    CF -->|contains| CD
    CF -->|involves| CP
    CF -->|includes| CI
    
    CF -->|generates| FR
    FR -->|follows| FI
    FR -->|receives| FC
    
    CF -->|generates| RR
    RR -->|submits to| RA
    RR -->|receives| RC
    
    classDef entity fill:#f5f5f5,stroke:#333,stroke-width:1px
    class CTC,CF,CD,CP,CI,FR,FI,FC,RR,RA,RC entity
```

## Closing Workflow

```mermaid
stateDiagram-v2
    [*] --> CTC_Received: Underwriting issues CTC
    CTC_Received --> Closing_Scheduled: Schedule signing
    Closing_Scheduled --> Documents_Prepared: Prepare docs
    Documents_Prepared --> Signing_In_Progress: Conduct signing
    Signing_In_Progress --> Closing_Complete: All docs signed
    Closing_Complete --> Funding_Requested: Request funding
    Funding_Requested --> Funding_In_Progress: Process wire
    Funding_In_Progress --> Funded: Wire confirmed
    Funded --> Recording_Submitted: Submit to county
    Recording_Submitted --> Recorded: Receive confirmation
    Recorded --> [*]: Complete process
    
    state fork_state <<fork>>
    Closing_Complete --> fork_state
    fork_state --> Funding_Requested
    fork_state --> Recording_Submitted
```

## Repositories

- **ClosingFileRepository** - Manages persistence of closing files and related entities
- **FundingRepository** - Manages funding requests and confirmations
- **RecordingRepository** - Manages recording requests and confirmations

## Domain Services

- **ClosingService** - Orchestrates the overall closing process
- **DocumentPreparationService** - Prepares legal documents for closing
- **SigningService** - Coordinates document signing process
- **FundingService** - Manages the loan funding process
- **RecordingService** - Handles recording of mortgage documents with county offices

## Integration with Other Contexts

- Receives CTC from Underwriting Context to initiate closing
- Interacts with Document Context for document preparation and storage
- Notifies Loan Origination Context of closing status and completion
- Sends payment notifications to accounting systems
- Coordinates with Title Companies and Attorneys through external interfaces 