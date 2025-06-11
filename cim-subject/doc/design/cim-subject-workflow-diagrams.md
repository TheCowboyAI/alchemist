# CIM Subject Module - Private Mortgage Lending Workflow Diagrams

## Overview

This document provides visual representations of the private mortgage lending workflows described in the CIM Subject user stories using Mermaid diagrams.

## Loan Application Processing

### Multi-Broker Application Routing

```mermaid
graph TD
    A[Loan Application] -->|lending.premierbrokers.northeast.application.submitted.new.v1| B{Broker Tier Check}

    B -->|VIP Broker| C[Priority Queue]
    B -->|Standard Broker| D[Normal Queue]
    B -->|New Broker| E[Enhanced Validation]

    C --> F{Jurisdiction Check}
    D --> F
    E --> F

    F -->|NY/NJ| G[Northeast Processor]
    F -->|FL/GA| H[Southeast Processor]
    F -->|CA/WA| I[West Coast Processor]

    G --> J[Compliance Rules NY]
    H --> K[Compliance Rules FL]
    I --> L[Compliance Rules CA]

    style C fill:#9f9,stroke:#333,stroke-width:2px
    style E fill:#ff9,stroke:#333,stroke-width:2px
```

### Document Collection and Validation Flow

```mermaid
sequenceDiagram
    participant Broker
    participant Platform
    participant OCR as OCR Service
    participant Validator
    participant Underwriter

    Broker->>Platform: lending.broker.income.document.paystub.received.v1
    Platform->>OCR: ocr-requested
    OCR->>OCR: Extract data
    OCR->>Validator: Extracted values

    Validator->>Validator: Check requirements
    Note over Validator: - Date < 30 days<br/>- Employer matches<br/>- Income sufficient

    alt Validation Passed
        Validator->>Platform: document.approved
        Platform->>Underwriter: Ready for review
    else Validation Failed
        Validator->>Platform: document.rejected
        Platform->>Broker: Request updated document
    end
```

### Borrower Identity Verification

```mermaid
graph TB
    A[Identity Verification Request] -->|lending.application.verification.identity.person.requested.v1| B[Verification Router]

    B --> C{Parallel Checks}

    C -->|Credit Bureau| D[Experian]
    C -->|Credit Bureau| E[Equifax]
    C -->|Public Records| F[LexisNexis]
    C -->|Sanctions| G[OFAC Check]
    C -->|Internal| H[Previous History]

    D --> I{Results Aggregator}
    E --> I
    F --> I
    G --> I
    H --> I

    I -->|All Match| J[Identity Confirmed]
    I -->|Discrepancies| K[Manual Review]
    I -->|Red Flags| L[Fraud Alert]

    style J fill:#9f9,stroke:#333,stroke-width:2px
    style K fill:#ff9,stroke:#333,stroke-width:2px
    style L fill:#f99,stroke:#333,stroke-width:2px
```

## Property Valuation and Title Processing

### Appraisal Coordination Workflow

```mermaid
stateDiagram-v2
    [*] --> AppraisalOrdered

    AppraisalOrdered --> LocationAnalysis

    state LocationAnalysis {
        [*] --> DetermineRegion
        DetermineRegion --> DeterminePropertyType
        DeterminePropertyType --> SelectAMC
        SelectAMC --> [*]
    }

    LocationAnalysis --> AMCAssignment

    state AMCAssignment {
        [*] --> CheckAvailability
        CheckAvailability --> AssignAppraiser
        AssignAppraiser --> ScheduleVisit
        ScheduleVisit --> [*]
    }

    AMCAssignment --> AppraisalInProgress
    AppraisalInProgress --> AppraisalCompleted

    AppraisalCompleted --> ValueReview

    state ValueReview {
        [*] --> CompareWithAVM
        CompareWithAVM --> VarianceCheck
        VarianceCheck --> ReviewDecision
        ReviewDecision --> [*]
    }

    ValueReview --> Approved: Within 10% of AVM
    ValueReview --> SecondReview: Variance > 10%
    ValueReview --> Rejected: Major Issues

    Approved --> [*]
    SecondReview --> ChiefAppraiserReview
    ChiefAppraiserReview --> [*]
    Rejected --> [*]
```

### Title Search Process

```mermaid
graph LR
    A[Title Order] -->|lending.property.title.search.initiated.v1| B{State Router}

    B -->|NY| C[First American NY]
    B -->|FL| D[Stewart Title FL]
    B -->|CA| E[Chicago Title CA]

    C --> F[Parallel Searches]
    D --> F
    E --> F

    F --> G[Owner Search]
    F --> H[Lien Search]
    F --> I[Tax Search]
    F --> J[Judgment Search]

    G --> K{Title Assembly}
    H --> K
    I --> K
    J --> K

    K -->|Clear| L[Title Insurance Quote]
    K -->|Defects Found| M[Curative Process]

    M --> N[Resolution Workflow]
    N --> L

    style L fill:#9f9,stroke:#333,stroke-width:2px
    style M fill:#ff9,stroke:#333,stroke-width:2px
```

## Rate Shopping and Pricing

### Multi-Lender Rate Shopping

```mermaid
graph TD
    A[Rate Request] -->|lending.rates.quote.standard.requested.v1| B{Loan Profile Analysis}

    B -->|Non-QM| C[Non-QM Lenders]
    B -->|Jumbo| D[Jumbo Specialists]
    B -->|Bridge| E[Bridge Lenders]
    B -->|Fix & Flip| F[Hard Money Lenders]

    C --> G[Velocity Mortgage]
    C --> H[Citadel Funding]

    D --> I[Private Bank A]
    D --> J[Private Bank B]

    E --> K[Bridge Capital]
    E --> L[Quick Fund LLC]

    F --> M[Hard Money Co]
    F --> N[Fast Capital]

    G -->|3.5% + 2pts| O{Rate Aggregator}
    H -->|3.75% + 1.5pts| O
    I -->|3.25% + 1pt| O
    J -->|3.4% + 0.5pts| O

    O --> P[Normalized Comparison]
    P --> Q[Best Rate Selection]

    style Q fill:#9f9,stroke:#333,stroke-width:2px
```

### Dynamic Pricing Workflow

```mermaid
sequenceDiagram
    participant Base as Base Rate Engine
    participant Credit as Credit Risk
    participant Property as Property Risk
    participant Market as Market Data
    participant Final as Final Pricing

    Base->>Credit: lending.pricing.loan.base-rate.calculated.v1

    Credit->>Credit: Analyze credit score
    Note over Credit: 740+ : -0.25%<br/>680-739: +0%<br/>620-679: +0.5%<br/><620: +1.0%

    Credit->>Property: credit-scored

    Property->>Property: Analyze LTV & Type
    Note over Property: <65% LTV: -0.125%<br/>65-75%: +0%<br/>75-80%: +0.25%<br/>>80%: +0.75%

    Property->>Market: property-scored

    Market->>Market: Check conditions
    Note over Market: Competitor rates<br/>Volume targets<br/>Risk appetite

    Market->>Final: market-adjusted

    Final->>Final: Calculate final rate
    Note over Final: Base + Adjustments<br/>+ Margin<br/>= Final Rate

    Final-->>Base: lending.pricing.loan.final-rate.determined.v1
```

## Underwriting and Decisions

### Automated Underwriting Flow

```mermaid
graph TB
    A[Application Complete] -->|lending.underwriting.application.analysis.started.v1| B[Auto-Underwriting Engine]

    B --> C{Income Analysis}
    B --> D{Asset Verification}
    B --> E{Credit Analysis}
    B --> F{Property Analysis}

    C -->|DTI < 43%| G[Pass]
    C -->|DTI 43-50%| H[Review]
    C -->|DTI > 50%| I[Fail]

    D -->|Verified| G
    D -->|Questionable| H
    D -->|Insufficient| I

    E -->|Score > 700| G
    E -->|Score 620-700| H
    E -->|Score < 620| I

    F -->|LTV < 75%| G
    F -->|LTV 75-80%| H
    F -->|LTV > 80%| I

    G --> J{Decision Matrix}
    H --> J
    I --> J

    J -->|All Pass| K[Auto-Approve]
    J -->|Any Fail| L[Auto-Decline]
    J -->|Mixed| M[Manual Review]

    K --> N[Generate Approval]
    L --> O[Generate Denial]
    M --> P[Route to Underwriter]

    style K fill:#9f9,stroke:#333,stroke-width:2px
    style L fill:#f99,stroke:#333,stroke-width:2px
    style M fill:#ff9,stroke:#333,stroke-width:2px
```

### Exception Handling Hierarchy

```mermaid
graph TD
    A[Exception Identified] -->|lending.underwriting.exception.identified.v1| B{Exception Type}

    B -->|Minor| C[Junior Underwriter]
    B -->|Standard| D[Senior Underwriter]
    B -->|Complex| E[Chief Underwriter]
    B -->|Policy| F[Credit Committee]

    C -->|Can Override| G[Document & Approve]
    C -->|Cannot Override| D

    D -->|Loan < $1M| H[Review & Decision]
    D -->|Loan > $1M| E

    E -->|Within Guidelines| I[Final Decision]
    E -->|Outside Guidelines| F

    F --> J[Committee Review]
    J --> K[Vote Required]

    G --> L[Update Rules Engine]
    H --> L
    I --> L
    K --> L

    style G fill:#9f9,stroke:#333,stroke-width:2px
    style L fill:#9ff,stroke:#333,stroke-width:2px
```

## Closing Coordination

### Closing Workflow Orchestration

```mermaid
stateDiagram-v2
    [*] --> ClosingScheduled

    ClosingScheduled --> ParallelPreparation

    state ParallelPreparation {
        [*] --> DocumentGeneration
        [*] --> TitleClearing
        [*] --> FundingSetup
        [*] --> BorrowerNotification

        DocumentGeneration --> DocsReady
        TitleClearing --> TitleCleared
        FundingSetup --> FundsReady
        BorrowerNotification --> BorrowerConfirmed

        DocsReady --> [*]
        TitleCleared --> [*]
        FundsReady --> [*]
        BorrowerConfirmed --> [*]
    }

    ParallelPreparation --> PreClosingReview

    PreClosingReview --> ClosingDay

    state ClosingDay {
        [*] --> SigningAppointment
        SigningAppointment --> DocumentsSigned
        DocumentsSigned --> FundsReleased
        FundsReleased --> RecordingInitiated
        RecordingInitiated --> [*]
    }

    ClosingDay --> PostClosing

    PostClosing --> [*]
```

### Post-Closing Quality Control

```mermaid
graph LR
    A[Loan Closed] -->|lending.qc.loan.review.initiated.v1| B{QC Sampling}

    B -->|10% Random| C[Standard Review]
    B -->|High Risk| D[Full Review]
    B -->|New Broker| E[Enhanced Review]

    C --> F[Document Audit]
    D --> F
    E --> F

    F --> G[Check Signatures]
    F --> H[Verify Data]
    F --> I[Compliance Check]
    F --> J[Recording Status]

    G --> K{Issue Detection}
    H --> K
    I --> K
    J --> K

    K -->|Clean| L[Pass QC]
    K -->|Minor Issues| M[Quick Fix]
    K -->|Major Issues| N[Escalation]

    M --> O[Remediation]
    O --> L

    N --> P[Management Review]
    P --> Q[Corrective Action]

    style L fill:#9f9,stroke:#333,stroke-width:2px
    style M fill:#ff9,stroke:#333,stroke-width:2px
    style N fill:#f99,stroke:#333,stroke-width:2px
```

## Subject Pattern Examples

### Document Type Routing

```mermaid
graph TD
    A[Document Received] --> B{Parse Subject}

    B --> C[lending.*.income.document.paystub.*]
    B --> D[lending.*.asset.document.bank-statement.*]
    B --> E[lending.*.property.document.appraisal.*]
    B --> F[lending.*.identity.document.drivers-license.*]

    C --> G[Income Verification Queue]
    D --> H[Asset Verification Queue]
    E --> I[Property Valuation Queue]
    F --> J[Identity Verification Queue]

    G --> K[Specialized OCR]
    H --> L[Bank API Validation]
    I --> M[AVM Comparison]
    J --> N[DMV Verification]

    style K fill:#9ff,stroke:#333,stroke-width:2px
    style L fill:#9ff,stroke:#333,stroke-width:2px
    style M fill:#9ff,stroke:#333,stroke-width:2px
    style N fill:#9ff,stroke:#333,stroke-width:2px
```

### Permission-Based Access Control

```mermaid
graph TD
    A[User Request] --> B{Permission Check}

    B --> C{Role Check}

    C -->|Broker| D[lending.{broker_id}.*.application.*]
    C -->|Underwriter| E[lending.*.*.underwriting.*]
    C -->|Processor| F[lending.*.*.processing.*]
    C -->|Admin| G[lending.*.*.*.>]

    D --> H{Attribute Check}
    E --> H
    F --> H
    G --> H

    H -->|broker_id matches| I[Allow]
    H -->|loan_amount < limit| I
    H -->|region authorized| I
    H -->|Otherwise| J[Deny]

    I --> K[Process Request]
    J --> L[Access Denied]

    style I fill:#9f9,stroke:#333,stroke-width:2px
    style J fill:#f99,stroke:#333,stroke-width:2px
```

## Summary

These diagrams illustrate how CIM Subject algebra enables sophisticated private mortgage lending workflows:

1. **Multi-Party Coordination** - Brokers, lenders, AMCs, title companies collaborate through subject routing
2. **Document Intelligence** - OCR, validation, and automated processing through workflow composition
3. **Risk-Based Routing** - Loans route based on characteristics and risk profiles
4. **Regulatory Compliance** - State-specific rules and audit trails maintained
5. **Automated Decisions** - Complex underwriting logic with human oversight when needed

The visual representations demonstrate how subject-based routing creates a flexible, compliant, and efficient mortgage lending platform.
