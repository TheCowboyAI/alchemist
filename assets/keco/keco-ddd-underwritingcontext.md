# KÉCŌ Capital - Underwriting Context

This document details the Domain-Driven Design (DDD) elements of the Underwriting Context, which handles loan evaluation, risk assessment, and approval decisions.

## Context Overview

The Underwriting Context is responsible for:
- Evaluating loan applications
- Assessing borrower risk and property value
- Making approval decisions
- Setting loan conditions
- Issuing commitment letter agreements (CLAs)
- Processing clear to close (CTC) authorizations

## Aggregate: UnderwritingFileAggregate

```mermaid
classDiagram
    class UnderwritingFileAggregate {
        <<Aggregate Root>>
        +UnderwritingFileId id
        +DealFileId dealFileId
        +BorrowerFileId borrowerFileId
        +UnderwritingStatus status
        +DateTime submissionDate
        +DateTime decisionDueDate
        +List~VerificationResult~ verifications
        +StartUnderwriting()
        +RecordVerification(VerificationResult)
        +MakeDecision(Decision)
        +SetConditions(List~Condition~)
        +IssueCLA()
        +IssueCTC()
    }
    
    class VerificationResult {
        <<Value Object>>
        +VerificationType type
        +Boolean passed
        +String notes
        +DateTime verificationDate
        +String verifiedBy
    }
    
    class Decision {
        <<Value Object>>
        +DecisionType type
        +String justification
        +DateTime decisionDate
        +String decisionMaker
    }
    
    class Condition {
        +ConditionId id
        +String description
        +ConditionType type
        +ConditionStatus status
        +DateTime dueDate
        +Satisfy(String evidence)
        +Waive(String justification)
    }
    
    class CommitmentLetterAgreement {
        +CLAId id
        +DateTime issueDate
        +DateTime expirationDate
        +LoanTerms terms
        +List~Condition~ conditions
        +CLAStatus status
        +Issue()
        +Sign()
        +Expire()
    }
    
    class ClearToClose {
        +CTCId id
        +DateTime issueDate
        +CTCStatus status
        +String approvedBy
        +Issue()
        +Approve()
    }
    
    UnderwritingFileAggregate "1" --> "1..*" VerificationResult : contains
    UnderwritingFileAggregate "1" --> "1" Decision : makes
    UnderwritingFileAggregate "1" --> "0..*" Condition : sets
    UnderwritingFileAggregate "1" --> "0..1" CommitmentLetterAgreement : issues
    UnderwritingFileAggregate "1" --> "0..1" ClearToClose : issues
```

## Aggregate: RiskAssessmentAggregate

```mermaid
classDiagram
    class RiskAssessmentAggregate {
        <<Aggregate Root>>
        +RiskAssessmentId id
        +UnderwritingFileId underwritingFileId
        +DateTime assessmentDate
        +RiskLevel overallRisk
        +CreateAssessment()
        +EvaluateBorrowerRisk()
        +EvaluatePropertyRisk()
        +EvaluateLoanStructureRisk()
        +CalculateOverallRisk()
        +GenerateRecommendation()
    }
    
    class BorrowerRiskFactor {
        <<Value Object>>
        +RiskFactorType type
        +RiskLevel level
        +String description
        +Evaluate()
    }
    
    class PropertyRiskFactor {
        <<Value Object>>
        +RiskFactorType type
        +RiskLevel level
        +String description
        +Evaluate()
    }
    
    class LoanStructureRiskFactor {
        <<Value Object>>
        +RiskFactorType type
        +RiskLevel level
        +String description
        +Evaluate()
    }
    
    class RiskRecommendation {
        <<Value Object>>
        +RecommendationType type
        +String rationale
        +List~String~ suggestedConditions
        +Generate()
    }
    
    RiskAssessmentAggregate "1" --> "1..*" BorrowerRiskFactor : evaluates
    RiskAssessmentAggregate "1" --> "1..*" PropertyRiskFactor : evaluates
    RiskAssessmentAggregate "1" --> "1..*" LoanStructureRiskFactor : evaluates
    RiskAssessmentAggregate "1" --> "1" RiskRecommendation : generates
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
    
    class UnderwritingStartedEvent {
        +UnderwritingFileId underwritingFileId
        +DateTime startDate
    }
    
    class VerificationCompletedEvent {
        +UnderwritingFileId underwritingFileId
        +VerificationType type
        +Boolean passed
    }
    
    class RiskAssessmentCompletedEvent {
        +RiskAssessmentId riskAssessmentId
        +RiskLevel overallRisk
        +RecommendationType recommendation
    }
    
    class UnderwritingDecisionMadeEvent {
        +UnderwritingFileId underwritingFileId
        +DecisionType decision
        +DateTime decisionDate
    }
    
    class ConditionsSetEvent {
        +UnderwritingFileId underwritingFileId
        +List~ConditionId~ conditionIds
    }
    
    class ConditionSatisfiedEvent {
        +ConditionId conditionId
        +DateTime satisfactionDate
    }
    
    class CLAIssuedEvent {
        +CLAId claId
        +UnderwritingFileId underwritingFileId
        +DateTime issueDate
    }
    
    class CLASignedEvent {
        +CLAId claId
        +DateTime signDate
    }
    
    class CTCIssuedEvent {
        +CTCId ctcId
        +UnderwritingFileId underwritingFileId
        +DateTime issueDate
    }
    
    DomainEvent <|-- UnderwritingStartedEvent
    DomainEvent <|-- VerificationCompletedEvent
    DomainEvent <|-- RiskAssessmentCompletedEvent
    DomainEvent <|-- UnderwritingDecisionMadeEvent
    DomainEvent <|-- ConditionsSetEvent
    DomainEvent <|-- ConditionSatisfiedEvent
    DomainEvent <|-- CLAIssuedEvent
    DomainEvent <|-- CLASignedEvent
    DomainEvent <|-- CTCIssuedEvent
```

## Entity Relationships

```mermaid
flowchart TD
    UWF[UnderwritingFile]
    RA[RiskAssessment]
    Dec[Decision]
    Cond[Conditions]
    CLA[CommitmentLetterAgreement]
    CTC[ClearToClose]
    BF[BorrowerFile]
    DF[DealFile]
    
    BF -->|submitted to| UWF
    DF -->|submitted to| UWF
    UWF -->|creates| RA
    RA -->|influences| Dec
    UWF -->|makes| Dec
    Dec -->|determines| Cond
    UWF -->|issues| CLA
    CLA -->|when signed leads to| CTC
    UWF -->|issues| CTC
    
    classDef entity fill:#f5f5f5,stroke:#333,stroke-width:1px
    class UWF,RA,Dec,Cond,CLA,CTC,BF,DF entity
```

## Policy Specifications

```mermaid
classDiagram
    class UnderwritingPolicySpecification {
        <<interface>>
        +Boolean isSatisfiedBy(UnderwritingFile)
    }
    
    class TrackRecordSpecification {
        +Int minimumPropertiesRequired
        +Int minimumMonthsExperience
        +Boolean isSatisfiedBy(TrackRecord)
    }
    
    class EntityDocumentSpecification {
        +List~String~ requiredDocumentTypes
        +Boolean isSatisfiedBy(EntityDocuments)
    }
    
    class LoanToValueSpecification {
        +Decimal maximumLTV
        +Boolean isSatisfiedBy(LoanToValue)
    }
    
    class CreditScoreSpecification {
        +Int minimumCreditScore
        +Boolean isSatisfiedBy(CreditScore)
    }
    
    class PropertyTypeSpecification {
        +List~PropertyType~ allowedPropertyTypes
        +Boolean isSatisfiedBy(PropertyType)
    }
    
    UnderwritingPolicySpecification <|-- TrackRecordSpecification
    UnderwritingPolicySpecification <|-- EntityDocumentSpecification
    UnderwritingPolicySpecification <|-- LoanToValueSpecification
    UnderwritingPolicySpecification <|-- CreditScoreSpecification
    UnderwritingPolicySpecification <|-- PropertyTypeSpecification
```

## Repositories

- **UnderwritingFileRepository** - Manages persistence of underwriting files
- **RiskAssessmentRepository** - Manages persistence of risk assessments
- **ConditionRepository** - Tracks loan conditions and their status
- **PolicyRepository** - Provides access to current underwriting policies

## Domain Services

- **UnderwritingService** - Orchestrates the overall underwriting process
- **PolicyEvaluationService** - Evaluates loan applications against policy specifications
- **RiskAssessmentService** - Performs risk analysis of borrower and property
- **ConditionManagementService** - Manages conditions and their satisfaction
- **DocumentVerificationService** - Verifies documents meet underwriting standards

## Integration with Other Contexts

- Receives completed files from Loan Origination Context
- Publishes approval decisions and CLAs to the Loan Origination Context
- Sends CTC authorizations to the Closing Context
- Interacts with Document Context for document verification
- Sends notifications about conditions to borrowers through Notification Context 