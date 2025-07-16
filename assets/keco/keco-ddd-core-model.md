# KÉCŌ Capital - Domain-Driven Design Core Model

This document outlines the core domain objects and their relationships for the KÉCŌ Capital private lending platform based on event storming session data from KECO_v6_04012025.json.

## Ubiquitous Language

The following terms form our common language to be used consistently across all documentation, code, and communication:

| Term | Definition |
|------|------------|
| Borrower | An individual or entity seeking financing from KÉCŌ Capital |
| Deal File | A collection of information related to a specific loan request |
| Borrower File | A collection of information about a specific borrower |
| ITO | Initial Term Outline - preliminary loan terms presented to borrower |
| TPO | Third Party Originator |
| CLA | Commitment Letter Agreement |
| CTC | Clear to Close - authorization to proceed with loan closing |
| SOW | Scope of Work - details of planned property improvements |
| ARV | After Repair Value - estimated property value after improvements |
| AML | Anti-Money Laundering documentation |
| BBP | Borrower Business Purpose documentation |

## Core Domain Overview

```mermaid
flowchart TB
    subgraph Core_Domain["Core Domain"]
        LoanProcessing["Loan Processing"]
        Underwriting["Underwriting"]
        DocumentManagement["Document Management"]
    end
    
    subgraph Supporting_Domains["Supporting Domains"]
        BorrowerManagement["Borrower Management"]
        UserManagement["User Management"]
        NotificationService["Notification Service"]
    end
    
    subgraph Generic_Domains["Generic Domains"]
        Authentication["Authentication"]
        Reporting["Reporting"]
        Auditing["Auditing"]
    end
    
    LoanProcessing --> Underwriting
    LoanProcessing --> DocumentManagement
    Underwriting --> DocumentManagement
    BorrowerManagement --> LoanProcessing
    NotificationService --> LoanProcessing
    NotificationService --> Underwriting
    NotificationService --> BorrowerManagement
    
    style Core_Domain fill:#f9f7ed,stroke:#333,stroke-width:2px
    style Supporting_Domains fill:#edf4f9,stroke:#333,stroke-width:2px
    style Generic_Domains fill:#f3edf9,stroke:#333,stroke-width:2px
```

## Bounded Contexts

```mermaid
flowchart TB
    subgraph LoanOriginationContext["Loan Origination Context"]
        BorrowerEntity["Borrower"]
        DealFileEntity["Deal File"]
        LoanApplicationEntity["Loan Application"]
        DocumentCollectionProcess["Document Collection Process"]
    end
    
    subgraph UnderwritingContext["Underwriting Context"]
        LoanEvaluationProcess["Loan Evaluation Process"]
        RiskAssessmentEntity["Risk Assessment"]
        ApprovalDecisionEntity["Approval Decision"]
        ConditionsEntity["Conditions"]
    end
    
    subgraph ClosingContext["Closing Context"]
        ClosingProcessEntity["Closing Process"]
        DocumentPreparationProcess["Document Preparation"]
        FundingEntity["Funding"]
        RecordingEntity["Recording"]
    end
    
    subgraph DocumentContext["Document Context"]
        DocumentEntity["Document"]
        DocumentVerificationProcess["Document Verification"]
        DocumentStatusEntity["Document Status"]
    end
    
    LoanOriginationContext -.->|submits to| UnderwritingContext
    UnderwritingContext -.->|approves to| ClosingContext
    LoanOriginationContext -.->|uses| DocumentContext
    UnderwritingContext -.->|uses| DocumentContext
    ClosingContext -.->|uses| DocumentContext
    
    style LoanOriginationContext fill:#f5f5f5,stroke:#333,stroke-width:2px
    style UnderwritingContext fill:#f5f5f5,stroke:#333,stroke-width:2px
    style ClosingContext fill:#f5f5f5,stroke:#333,stroke-width:2px
    style DocumentContext fill:#f5f5f5,stroke:#333,stroke-width:2px
```

## Context Map

```mermaid
flowchart TB
    subgraph CM["Context Map"]
        LO["Loan Origination"]
        UW["Underwriting"]
        CL["Closing"]
        DM["Document Management"]
        US["User System"]
        NT["Notification"]
        TM["Timer"]
    end
    
    LO -->|U/S| UW
    UW -->|U/S| CL
    LO -->|U/S| DM
    UW -->|U/S| DM
    CL -->|U/S| DM
    US -->|U/S| LO
    NT -->|Published Events| LO
    NT -->|Published Events| UW
    TM -->|Published Events| LO
    
    style CM fill:#f9f9f9,stroke:#333,stroke-width:2px
    
    classDef contextStyle fill:#fff,stroke:#333,stroke-width:1px
    class LO,UW,CL,DM,US,NT,TM contextStyle

    %% U/S = Upstream/Downstream relationship
```

## Domain Model Overview

In the following sections, we will detail each of the main domain components and their relationships. 